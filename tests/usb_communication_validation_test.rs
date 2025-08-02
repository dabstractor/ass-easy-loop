//! USB Communication Validation Tests
//! 
//! This module implements comprehensive integration tests for USB HID communication validation.
//! It validates bidirectional data transfer, message integrity checking, transmission error detection,
//! and configurable message count and timing parameters.
//! 
//! Requirements: 9.4, 9.5

#![cfg(test)]

use ass_easy_loop::{
    TestCommandProcessor, TestType, TestStatus, TestParameters, TestResult,
    TestMeasurements, ResourceUsageStats, PerformanceMetrics, TestProcessorStatistics,
    TestExecutionError, TestParameterError,
    CommandReport, TestResponse, ErrorCode
};
use ass_easy_loop::test_processor::{
    UsbCommunicationTestParameters, UsbCommunicationStatistics, TimingMeasurement
};
use ass_easy_loop::command::parsing::{CommandReport as CmdReport, TestCommand};
use ass_easy_loop::logging::{LogLevel, LogMessage, LogQueue, LogReport};
use heapless::Vec;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Mock USB HID device for testing bidirectional communication
#[derive(Debug, Clone)]
pub struct MockUsbHidDevice {
    pub device_id: u16,
    pub is_connected: bool,
    pub transmitted_messages: Arc<Mutex<Vec<UsbTestMessage>>>,
    pub received_messages: Arc<Mutex<Vec<UsbTestMessage>>>,
    pub transmission_errors: Arc<Mutex<u32>>,
    pub reception_errors: Arc<Mutex<u32>>,
    pub message_id_counter: Arc<Mutex<u32>>,
    pub round_trip_times: Arc<Mutex<HashMap<u32, Instant>>>,
    pub error_injection_enabled: bool,
    pub error_injection_rate: u8,
}

/// Test message structure for USB communication validation
#[derive(Debug, Clone, PartialEq)]
pub struct UsbTestMessage {
    pub message_id: u32,
    pub timestamp_ms: u32,
    pub data: Vec<u8>,
    pub checksum: u32,
    pub is_outbound: bool,
}

impl UsbTestMessage {
    /// Create a new USB test message with integrity checking
    pub fn new(message_id: u32, timestamp_ms: u32, data: &[u8], is_outbound: bool) -> Self {
        let mut message_data = Vec::new();
        message_data.extend_from_slice(data);
        
        // Calculate checksum (XOR of message ID and all data bytes)
        let mut checksum = message_id;
        for &byte in data {
            checksum ^= byte as u32;
        }
        
        Self {
            message_id,
            timestamp_ms,
            data: message_data,
            checksum,
            is_outbound,
        }
    }

    /// Serialize message to bytes with integrity checking
    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized = Vec::new();
        
        // Message ID (4 bytes)
        serialized.extend_from_slice(&self.message_id.to_le_bytes());
        
        // Timestamp (4 bytes)
        serialized.extend_from_slice(&self.timestamp_ms.to_le_bytes());
        
        // Data length (2 bytes)
        serialized.extend_from_slice(&(self.data.len() as u16).to_le_bytes());
        
        // Data
        serialized.extend_from_slice(&self.data);
        
        // Checksum (4 bytes)
        serialized.extend_from_slice(&self.checksum.to_le_bytes());
        
        serialized
    }

    /// Deserialize message from bytes with integrity validation
    pub fn deserialize(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 14 {
            return Err("Message too short");
        }

        let message_id = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let timestamp_ms = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let data_len = u16::from_le_bytes([data[8], data[9]]) as usize;
        
        if data.len() < 14 + data_len {
            return Err("Incomplete message data");
        }

        let message_data = data[10..10 + data_len].to_vec();
        let checksum = u32::from_le_bytes([
            data[10 + data_len],
            data[11 + data_len],
            data[12 + data_len],
            data[13 + data_len],
        ]);

        // Validate checksum
        let mut calculated_checksum = message_id;
        for &byte in &message_data {
            calculated_checksum ^= byte as u32;
        }

        if calculated_checksum != checksum {
            return Err("Checksum validation failed");
        }

        Ok(Self {
            message_id,
            timestamp_ms,
            data: message_data,
            checksum,
            is_outbound: false, // Will be set by receiver
        })
    }

    /// Inject error into message for testing error detection
    pub fn inject_error(&mut self, error_type: ErrorType) {
        match error_type {
            ErrorType::CorruptChecksum => {
                self.checksum ^= 0xDEADBEEF; // Corrupt checksum
            }
            ErrorType::CorruptData => {
                if !self.data.is_empty() {
                    self.data[0] ^= 0xFF; // Corrupt first data byte
                }
            }
            ErrorType::TruncateMessage => {
                if self.data.len() > 1 {
                    self.data.truncate(self.data.len() / 2); // Truncate data
                }
            }
        }
    }
}

/// Error types for testing error detection
#[derive(Debug, Clone, Copy)]
pub enum ErrorType {
    CorruptChecksum,
    CorruptData,
    TruncateMessage,
}

impl MockUsbHidDevice {
    /// Create a new mock USB HID device
    pub fn new(device_id: u16) -> Self {
        Self {
            device_id,
            is_connected: false,
            transmitted_messages: Arc::new(Mutex::new(Vec::new())),
            received_messages: Arc::new(Mutex::new(Vec::new())),
            transmission_errors: Arc::new(Mutex::new(0)),
            reception_errors: Arc::new(Mutex::new(0)),
            message_id_counter: Arc::new(Mutex::new(0)),
            round_trip_times: Arc::new(Mutex::new(HashMap::new())),
            error_injection_enabled: false,
            error_injection_rate: 0,
        }
    }

    /// Connect the device
    pub fn connect(&mut self) {
        self.is_connected = true;
    }

    /// Disconnect the device
    pub fn disconnect(&mut self) {
        self.is_connected = false;
    }

    /// Enable error injection for testing
    pub fn enable_error_injection(&mut self, rate_percent: u8) {
        self.error_injection_enabled = true;
        self.error_injection_rate = rate_percent;
    }

    /// Send a message (device to host)
    pub fn send_message(&mut self, data: &[u8]) -> Result<u32, &'static str> {
        if !self.is_connected {
            *self.transmission_errors.lock().unwrap() += 1;
            return Err("Device not connected");
        }

        let message_id = {
            let mut counter = self.message_id_counter.lock().unwrap();
            *counter += 1;
            *counter
        };

        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u32;

        let mut message = UsbTestMessage::new(message_id, timestamp_ms, data, true);

        // Inject errors if enabled
        if self.error_injection_enabled && self.should_inject_error() {
            message.inject_error(ErrorType::CorruptChecksum);
        }

        // Record send time for round-trip calculation
        self.round_trip_times.lock().unwrap().insert(message_id, Instant::now());

        // Store transmitted message
        self.transmitted_messages.lock().unwrap().push(message);

        Ok(message_id)
    }

    /// Receive a message (host to device)
    pub fn receive_message(&mut self, serialized_data: &[u8]) -> Result<UsbTestMessage, &'static str> {
        if !self.is_connected {
            *self.reception_errors.lock().unwrap() += 1;
            return Err("Device not connected");
        }

        // Inject errors if enabled
        let mut data = serialized_data.to_vec();
        if self.error_injection_enabled && self.should_inject_error() {
            if !data.is_empty() {
                data[0] ^= 0xFF; // Corrupt first byte
            }
        }

        match UsbTestMessage::deserialize(&data) {
            Ok(mut message) => {
                message.is_outbound = false;
                
                // Calculate round-trip time if this is a response
                if let Some(send_time) = self.round_trip_times.lock().unwrap().remove(&message.message_id) {
                    let rtt = send_time.elapsed();
                    // Store RTT for statistics (simplified)
                }

                self.received_messages.lock().unwrap().push(message.clone());
                Ok(message)
            }
            Err(error) => {
                *self.reception_errors.lock().unwrap() += 1;
                Err(error)
            }
        }
    }

    /// Check if error should be injected based on rate
    fn should_inject_error(&self) -> bool {
        if self.error_injection_rate == 0 {
            return false;
        }
        
        // Simple random number generation for testing
        let random_value = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() % 100) as u8;
        
        random_value < self.error_injection_rate
    }

    /// Get transmission statistics
    pub fn get_transmission_stats(&self) -> (u32, u32, u32, u32) {
        let transmitted_count = self.transmitted_messages.lock().unwrap().len() as u32;
        let received_count = self.received_messages.lock().unwrap().len() as u32;
        let transmission_errors = *self.transmission_errors.lock().unwrap();
        let reception_errors = *self.reception_errors.lock().unwrap();
        
        (transmitted_count, received_count, transmission_errors, reception_errors)
    }

    /// Clear all messages and statistics
    pub fn clear_stats(&mut self) {
        self.transmitted_messages.lock().unwrap().clear();
        self.received_messages.lock().unwrap().clear();
        *self.transmission_errors.lock().unwrap() = 0;
        *self.reception_errors.lock().unwrap() = 0;
        self.round_trip_times.lock().unwrap().clear();
    }
}

// ============================================================================
// USB Communication Test Parameter Validation Tests
// ============================================================================

#[test]
fn test_usb_communication_parameters_validation() {
    // Test valid parameters
    let valid_params = UsbCommunicationTestParameters {
        message_count: 100,
        message_interval_ms: 10,
        message_size_bytes: 32,
        timeout_per_message_ms: 1000,
        enable_integrity_checking: true,
        enable_error_injection: false,
        error_injection_rate_percent: 0,
        bidirectional_test: true,
        concurrent_messages: 2,
    };
    assert!(valid_params.validate().is_ok());

    // Test invalid message count (zero)
    let mut invalid_params = valid_params;
    invalid_params.message_count = 0;
    assert_eq!(invalid_params.validate(), Err(TestParameterError::InvalidResourceLimits));

    // Test invalid message count (too high)
    invalid_params = valid_params;
    invalid_params.message_count = 20_000;
    assert_eq!(invalid_params.validate(), Err(TestParameterError::InvalidResourceLimits));

    // Test invalid message size (zero)
    invalid_params = valid_params;
    invalid_params.message_size_bytes = 0;
    assert_eq!(invalid_params.validate(), Err(TestParameterError::PayloadTooLarge));

    // Test invalid message size (too large)
    invalid_params = valid_params;
    invalid_params.message_size_bytes = 128;
    assert_eq!(invalid_params.validate(), Err(TestParameterError::PayloadTooLarge));

    // Test invalid timeout (zero)
    invalid_params = valid_params;
    invalid_params.timeout_per_message_ms = 0;
    assert_eq!(invalid_params.validate(), Err(TestParameterError::InvalidDuration));

    // Test invalid error injection rate (over 100%)
    invalid_params = valid_params;
    invalid_params.error_injection_rate_percent = 150;
    assert_eq!(invalid_params.validate(), Err(TestParameterError::InvalidResourceLimits));

    // Test invalid concurrent messages (zero)
    invalid_params = valid_params;
    invalid_params.concurrent_messages = 0;
    assert_eq!(invalid_params.validate(), Err(TestParameterError::InvalidResourceLimits));
}

#[test]
fn test_usb_communication_parameters_serialization() {
    let params = UsbCommunicationTestParameters {
        message_count: 500,
        message_interval_ms: 20,
        message_size_bytes: 48,
        timeout_per_message_ms: 2000,
        enable_integrity_checking: true,
        enable_error_injection: true,
        error_injection_rate_percent: 5,
        bidirectional_test: true,
        concurrent_messages: 3,
    };

    // Serialize parameters
    let serialized = params.serialize();
    assert!(serialized.len() >= 17); // Minimum expected size

    // Deserialize and verify
    let deserialized = UsbCommunicationTestParameters::from_payload(&serialized).unwrap();
    assert_eq!(deserialized.message_count, params.message_count);
    assert_eq!(deserialized.message_interval_ms, params.message_interval_ms);
    assert_eq!(deserialized.message_size_bytes, params.message_size_bytes);
    assert_eq!(deserialized.timeout_per_message_ms, params.timeout_per_message_ms);
    assert_eq!(deserialized.enable_integrity_checking, params.enable_integrity_checking);
    assert_eq!(deserialized.enable_error_injection, params.enable_error_injection);
    assert_eq!(deserialized.error_injection_rate_percent, params.error_injection_rate_percent);
    assert_eq!(deserialized.bidirectional_test, params.bidirectional_test);
    assert_eq!(deserialized.concurrent_messages, params.concurrent_messages);
}

// ============================================================================
// Message Integrity Checking Tests
// ============================================================================

#[test]
fn test_message_integrity_validation() {
    let test_data = b"Test message data for integrity checking";
    let message = UsbTestMessage::new(12345, 67890, test_data, true);

    // Test successful serialization and deserialization
    let serialized = message.serialize();
    let deserialized = UsbTestMessage::deserialize(&serialized).unwrap();

    assert_eq!(deserialized.message_id, message.message_id);
    assert_eq!(deserialized.timestamp_ms, message.timestamp_ms);
    assert_eq!(deserialized.data, message.data);
    assert_eq!(deserialized.checksum, message.checksum);
}

#[test]
fn test_message_integrity_corruption_detection() {
    let test_data = b"Test message for corruption detection";
    let mut message = UsbTestMessage::new(54321, 98765, test_data, true);

    // Test checksum corruption detection
    message.inject_error(ErrorType::CorruptChecksum);
    let serialized = message.serialize();
    let result = UsbTestMessage::deserialize(&serialized);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Checksum validation failed");
}

#[test]
fn test_message_truncation_detection() {
    let test_data = b"Test message for truncation detection";
    let message = UsbTestMessage::new(11111, 22222, test_data, true);
    let mut serialized = message.serialize();

    // Truncate the serialized message
    serialized.truncate(serialized.len() / 2);
    
    let result = UsbTestMessage::deserialize(&serialized);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Incomplete message data");
}

// ============================================================================
// Bidirectional Communication Tests
// ============================================================================

#[test]
fn test_bidirectional_communication_success() {
    let mut device = MockUsbHidDevice::new(0x1234);
    device.connect();

    // Test device-to-host communication
    let outbound_data = b"Device to host message";
    let message_id = device.send_message(outbound_data).unwrap();
    assert!(message_id > 0);

    // Test host-to-device communication
    let inbound_message = UsbTestMessage::new(message_id + 1, 33333, b"Host to device response", false);
    let serialized_response = inbound_message.serialize();
    let received = device.receive_message(&serialized_response).unwrap();

    assert_eq!(received.message_id, inbound_message.message_id);
    assert_eq!(received.data, inbound_message.data);
    assert!(!received.is_outbound);

    // Verify statistics
    let (tx_count, rx_count, tx_errors, rx_errors) = device.get_transmission_stats();
    assert_eq!(tx_count, 1);
    assert_eq!(rx_count, 1);
    assert_eq!(tx_errors, 0);
    assert_eq!(rx_errors, 0);
}

#[test]
fn test_bidirectional_communication_with_disconnection() {
    let mut device = MockUsbHidDevice::new(0x5678);
    device.connect();

    // Send message successfully
    let message_id = device.send_message(b"Before disconnect").unwrap();
    assert!(message_id > 0);

    // Disconnect device
    device.disconnect();

    // Attempt to send message while disconnected
    let result = device.send_message(b"During disconnect");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Device not connected");

    // Attempt to receive message while disconnected
    let test_message = UsbTestMessage::new(99999, 44444, b"Test", false);
    let serialized = test_message.serialize();
    let result = device.receive_message(&serialized);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Device not connected");

    // Verify error counts
    let (tx_count, rx_count, tx_errors, rx_errors) = device.get_transmission_stats();
    assert_eq!(tx_count, 1); // Only successful transmission
    assert_eq!(rx_count, 0); // No successful receptions
    assert_eq!(tx_errors, 1); // One transmission error
    assert_eq!(rx_errors, 1); // One reception error
}

// ============================================================================
// Error Detection and Recovery Tests
// ============================================================================

#[test]
fn test_error_injection_and_detection() {
    let mut device = MockUsbHidDevice::new(0x9ABC);
    device.connect();
    device.enable_error_injection(100); // 100% error rate for testing

    let mut successful_transmissions = 0;
    let mut failed_transmissions = 0;

    // Attempt multiple transmissions with error injection
    for i in 0..10 {
        let test_data = format!("Test message {}", i);
        match device.send_message(test_data.as_bytes()) {
            Ok(_) => successful_transmissions += 1,
            Err(_) => failed_transmissions += 1,
        }
    }

    // With 100% error injection, we should still have successful transmissions
    // (errors are injected in message content, not transmission mechanism)
    assert!(successful_transmissions > 0);

    // Test error detection in received messages
    let mut successful_receptions = 0;
    let mut failed_receptions = 0;

    for i in 0..10 {
        let test_message = UsbTestMessage::new(i, 55555, b"Test reception", false);
        let serialized = test_message.serialize();
        
        match device.receive_message(&serialized) {
            Ok(_) => successful_receptions += 1,
            Err(_) => failed_receptions += 1,
        }
    }

    // With error injection, we should have some failed receptions
    assert!(failed_receptions > 0);
    
    let (_, _, tx_errors, rx_errors) = device.get_transmission_stats();
    assert!(rx_errors > 0); // Should have reception errors due to corruption
}

#[test]
fn test_error_recovery_after_corruption() {
    let mut device = MockUsbHidDevice::new(0xDEF0);
    device.connect();

    // Send corrupted message
    let mut corrupted_message = UsbTestMessage::new(77777, 88888, b"Corrupted message", false);
    corrupted_message.inject_error(ErrorType::CorruptData);
    let serialized_corrupted = corrupted_message.serialize();
    
    let result = device.receive_message(&serialized_corrupted);
    assert!(result.is_err());

    // Send valid message after corruption
    let valid_message = UsbTestMessage::new(99999, 11111, b"Valid message", false);
    let serialized_valid = valid_message.serialize();
    
    let result = device.receive_message(&serialized_valid);
    assert!(result.is_ok());
    
    let received = result.unwrap();
    assert_eq!(received.message_id, valid_message.message_id);
    assert_eq!(received.data, valid_message.data);

    // Verify device recovered and can continue normal operation
    let (_, rx_count, _, rx_errors) = device.get_transmission_stats();
    assert_eq!(rx_count, 1); // One successful reception
    assert_eq!(rx_errors, 1); // One reception error
}

// ============================================================================
// Performance and Timing Tests
// ============================================================================

#[test]
fn test_message_throughput_performance() {
    let mut device = MockUsbHidDevice::new(0x1111);
    device.connect();

    let message_count = 1000;
    let test_data = b"Performance test message data";
    
    let start_time = Instant::now();
    
    // Send messages as fast as possible
    let mut successful_count = 0;
    for i in 0..message_count {
        if device.send_message(test_data).is_ok() {
            successful_count += 1;
        }
        
        // Small delay to simulate realistic timing
        thread::sleep(Duration::from_micros(100));
    }
    
    let elapsed = start_time.elapsed();
    let throughput = successful_count as f64 / elapsed.as_secs_f64();
    
    // Verify reasonable throughput (should be > 1000 messages/sec)
    assert!(throughput > 1000.0, "Throughput too low: {:.2} msg/sec", throughput);
    assert_eq!(successful_count, message_count);
    
    let (tx_count, _, tx_errors, _) = device.get_transmission_stats();
    assert_eq!(tx_count, message_count);
    assert_eq!(tx_errors, 0);
}

#[test]
fn test_round_trip_time_measurement() {
    let mut device = MockUsbHidDevice::new(0x2222);
    device.connect();

    let mut round_trip_times = Vec::new();
    
    // Measure round-trip times for multiple messages
    for i in 0..50 {
        let send_time = Instant::now();
        
        // Send message
        let message_id = device.send_message(b"RTT test message").unwrap();
        
        // Simulate processing delay
        thread::sleep(Duration::from_micros(500));
        
        // Create and send response
        let response = UsbTestMessage::new(message_id, 66666, b"Response", false);
        let serialized_response = response.serialize();
        device.receive_message(&serialized_response).unwrap();
        
        let rtt = send_time.elapsed();
        round_trip_times.push(rtt);
    }
    
    // Calculate statistics
    let total_rtt: Duration = round_trip_times.iter().sum();
    let avg_rtt = total_rtt / round_trip_times.len() as u32;
    let min_rtt = *round_trip_times.iter().min().unwrap();
    let max_rtt = *round_trip_times.iter().max().unwrap();
    
    // Verify reasonable timing characteristics
    assert!(avg_rtt.as_micros() > 500, "Average RTT too low: {:?}", avg_rtt);
    assert!(avg_rtt.as_micros() < 10000, "Average RTT too high: {:?}", avg_rtt);
    assert!(max_rtt.as_micros() < avg_rtt.as_micros() * 3, "Max RTT too high compared to average");
    
    println!("RTT Statistics: avg={:?}, min={:?}, max={:?}", avg_rtt, min_rtt, max_rtt);
}

// ============================================================================
// Integration Tests with Test Processor
// ============================================================================

#[test]
fn test_usb_communication_test_integration() {
    let mut processor = TestCommandProcessor::new();
    
    // Create USB communication test parameters
    let params = UsbCommunicationTestParameters {
        message_count: 50,
        message_interval_ms: 20,
        message_size_bytes: 32,
        timeout_per_message_ms: 1000,
        enable_integrity_checking: true,
        enable_error_injection: false,
        error_injection_rate_percent: 0,
        bidirectional_test: true,
        concurrent_messages: 2,
    };
    
    let test_id = 42;
    let timestamp_ms = 12345;
    
    // Execute USB communication test
    let result = processor.execute_usb_communication_test(test_id, params, timestamp_ms);
    assert!(result.is_ok());
    
    // Verify test is active
    assert!(processor.has_active_test());
    let active_test_type = processor.get_active_test_type();
    assert_eq!(active_test_type, Some(TestType::UsbCommunicationTest));
    
    // Simulate processing some messages
    for i in 0..10 {
        let message_data = format!("Test message {}", i);
        let result = processor.process_usb_communication_message(
            i,
            message_data.as_bytes(),
            i % 2 == 0, // Alternate between outbound and inbound
            timestamp_ms + i * 20,
        );
        assert!(result.is_ok());
    }
    
    // Get current statistics
    let stats = processor.get_usb_communication_statistics();
    assert!(stats.is_some());
    let stats = stats.unwrap();
    assert!(stats.messages_sent > 0);
    
    // Complete the test
    let final_stats = processor.complete_usb_communication_test(timestamp_ms + 2000);
    assert!(final_stats.is_ok());
    let final_stats = final_stats.unwrap();
    
    // Verify final statistics
    assert!(final_stats.test_duration_ms > 0);
    assert!(final_stats.success_rate_percent > 0.0);
    assert_eq!(final_stats.bidirectional_success, true);
}

#[test]
fn test_usb_communication_test_with_errors() {
    let mut processor = TestCommandProcessor::new();
    
    // Create test parameters with error injection
    let params = UsbCommunicationTestParameters {
        message_count: 20,
        message_interval_ms: 10,
        message_size_bytes: 24,
        timeout_per_message_ms: 500,
        enable_integrity_checking: true,
        enable_error_injection: true,
        error_injection_rate_percent: 20, // 20% error rate
        bidirectional_test: true,
        concurrent_messages: 1,
    };
    
    let test_id = 99;
    let timestamp_ms = 54321;
    
    // Execute test
    processor.execute_usb_communication_test(test_id, params, timestamp_ms).unwrap();
    
    // Process messages with some that will fail integrity checking
    let mut successful_messages = 0;
    let mut failed_messages = 0;
    
    for i in 0..20 {
        let message_data = format!("Error test message {}", i);
        let result = processor.process_usb_communication_message(
            i,
            message_data.as_bytes(),
            true,
            timestamp_ms + i * 10,
        );
        
        if result.is_ok() {
            successful_messages += 1;
        } else {
            failed_messages += 1;
        }
    }
    
    // Should have some failures due to error injection
    assert!(failed_messages > 0);
    assert!(successful_messages > 0);
    
    // Complete test and verify error statistics
    let final_stats = processor.complete_usb_communication_test(timestamp_ms + 1000).unwrap();
    assert!(final_stats.error_rate_percent > 0.0);
    assert!(final_stats.success_rate_percent < 100.0);
    assert!(final_stats.integrity_check_failures > 0);
}

// ============================================================================
// Concurrent Communication Tests
// ============================================================================

#[test]
fn test_concurrent_usb_communication() {
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    let device = Arc::new(Mutex::new(MockUsbHidDevice::new(0x3333)));
    device.lock().unwrap().connect();
    
    let mut handles = vec![];
    let message_count_per_thread = 25;
    let thread_count = 4;
    
    // Spawn multiple threads for concurrent communication
    for thread_id in 0..thread_count {
        let device_clone = Arc::clone(&device);
        let handle = thread::spawn(move || {
            let mut successful_ops = 0;
            let mut failed_ops = 0;
            
            for i in 0..message_count_per_thread {
                let message_data = format!("Thread {} message {}", thread_id, i);
                
                // Send message
                match device_clone.lock().unwrap().send_message(message_data.as_bytes()) {
                    Ok(message_id) => {
                        successful_ops += 1;
                        
                        // Send response
                        let response = UsbTestMessage::new(
                            message_id + 1000,
                            77777,
                            b"Concurrent response",
                            false
                        );
                        let serialized = response.serialize();
                        
                        match device_clone.lock().unwrap().receive_message(&serialized) {
                            Ok(_) => successful_ops += 1,
                            Err(_) => failed_ops += 1,
                        }
                    }
                    Err(_) => failed_ops += 1,
                }
                
                // Small delay to allow other threads to interleave
                thread::sleep(Duration::from_micros(100));
            }
            
            (successful_ops, failed_ops)
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    let mut total_successful = 0;
    let mut total_failed = 0;
    
    for handle in handles {
        let (successful, failed) = handle.join().unwrap();
        total_successful += successful;
        total_failed += failed;
    }
    
    // Verify concurrent operations completed successfully
    let expected_total_ops = thread_count * message_count_per_thread * 2; // Send + receive per message
    assert_eq!(total_successful + total_failed, expected_total_ops);
    assert!(total_successful > total_failed, "Too many failed operations in concurrent test");
    
    // Verify device statistics
    let (tx_count, rx_count, tx_errors, rx_errors) = device.lock().unwrap().get_transmission_stats();
    assert_eq!(tx_count + rx_count, total_successful as u32);
    assert_eq!(tx_errors + rx_errors, total_failed as u32);
}

// ============================================================================
// Test Summary and Statistics Validation
// ============================================================================

#[test]
fn test_usb_communication_statistics_calculation() {
    let mut stats = UsbCommunicationStatistics::new();
    
    // Set test data
    stats.test_duration_ms = 5000; // 5 seconds
    stats.messages_sent = 100;
    stats.messages_received = 95;
    stats.messages_acknowledged = 90;
    stats.transmission_errors = 2;
    stats.reception_errors = 3;
    stats.timeout_errors = 1;
    stats.integrity_check_failures = 4;
    
    // Add some round-trip time measurements
    stats.add_round_trip_time(1000); // 1ms
    stats.add_round_trip_time(1500); // 1.5ms
    stats.add_round_trip_time(800);  // 0.8ms
    stats.add_round_trip_time(2000); // 2ms
    
    // Calculate derived statistics
    stats.calculate_derived_stats();
    
    // Verify calculations
    let total_operations = stats.messages_sent + stats.messages_received; // 195
    let total_errors = stats.transmission_errors + stats.reception_errors + 
                      stats.timeout_errors + stats.integrity_check_failures; // 10
    let expected_error_rate = (total_errors as f32 / total_operations as f32) * 100.0; // ~5.13%
    let expected_success_rate = 100.0 - expected_error_rate; // ~94.87%
    let expected_throughput = total_operations as f32 / 5.0; // 39 messages/sec
    
    assert!((stats.error_rate_percent - expected_error_rate).abs() < 0.1);
    assert!((stats.success_rate_percent - expected_success_rate).abs() < 0.1);
    assert_eq!(stats.throughput_messages_per_sec, expected_throughput as u32);
    
    // Verify timing statistics
    assert_eq!(stats.min_round_trip_time_us, 800);
    assert_eq!(stats.max_round_trip_time_us, 2000);
    assert!(stats.average_round_trip_time_us > 800);
    assert!(stats.average_round_trip_time_us < 2000);
    
    // Verify bidirectional success (should be true with >95% success rate)
    assert_eq!(stats.bidirectional_success, false); // False because success rate < 95%
    
    // Test with higher success rate
    stats.transmission_errors = 0;
    stats.reception_errors = 0;
    stats.timeout_errors = 0;
    stats.integrity_check_failures = 1; // Only 1 error
    stats.calculate_derived_stats();
    
    assert!(stats.success_rate_percent >= 95.0);
    assert_eq!(stats.bidirectional_success, true);
}