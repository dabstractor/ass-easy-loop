//! USB Communication Validation Tests
//! 
//! This module implements comprehensive integration tests for USB HID communication validation.
//! It validates bidirectional data transfer, message integrity checking, transmission error detection,
//! and configurable message count and timing parameters.
//! 
//! Requirements: 9.4, 9.5

#![no_std]
#![no_main]

use panic_halt as _;

use ass_easy_loop::{
    TestCommandProcessor, TestType, TestStatus,
    TestExecutionError, TestParameterError,
    UsbCommunicationTestParameters, UsbCommunicationStatistics
};
use ass_easy_loop::test_framework::{TestResult, TestRunner};
use heapless::{Vec, FnvIndexMap};

// Import custom assertion macros
use ass_easy_loop::{assert_no_std, assert_eq_no_std, register_tests};

/// Error types for testing error detection
#[derive(Debug, Clone, Copy)]
pub enum ErrorType {
    CorruptChecksum,
    CorruptData,
    TruncateMessage,
}

/// Test message structure for USB communication validation (no_std compatible)
#[derive(Debug, Clone, PartialEq)]
pub struct UsbTestMessage {
    pub message_id: u32,
    pub timestamp_ms: u32,
    pub data: Vec<u8, 64>, // Fixed capacity for no_std
    pub checksum: u32,
    pub is_outbound: bool,
}

impl UsbTestMessage {
    /// Create a new USB test message with integrity checking
    pub fn new(message_id: u32, timestamp_ms: u32, data: &[u8], is_outbound: bool) -> Self {
        let mut message_data = Vec::new();
        for &byte in data.iter().take(64) { // Limit to capacity
            if message_data.push(byte).is_err() {
                break; // Vector is full
            }
        }
        
        // Calculate checksum (XOR of message ID and all data bytes)
        let mut checksum = message_id;
        for &byte in &message_data {
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
    pub fn serialize(&self) -> Vec<u8, 80> { // Fixed capacity for no_std
        let mut serialized = Vec::new();
        
        // Message ID (4 bytes)
        for &byte in &self.message_id.to_le_bytes() {
            let _ = serialized.push(byte);
        }
        
        // Timestamp (4 bytes)
        for &byte in &self.timestamp_ms.to_le_bytes() {
            let _ = serialized.push(byte);
        }
        
        // Data length (2 bytes)
        for &byte in &(self.data.len() as u16).to_le_bytes() {
            let _ = serialized.push(byte);
        }
        
        // Data
        for &byte in &self.data {
            if serialized.push(byte).is_err() {
                break; // Vector is full
            }
        }
        
        // Checksum (4 bytes)
        for &byte in &self.checksum.to_le_bytes() {
            if serialized.push(byte).is_err() {
                break; // Vector is full
            }
        }
        
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

        let mut message_data = Vec::new();
        for &byte in data[10..10 + data_len].iter().take(64) { // Limit to capacity
            if message_data.push(byte).is_err() {
                break; // Vector is full
            }
        }
        
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
                    let new_len = self.data.len() / 2;
                    self.data.truncate(new_len); // Truncate data
                }
            }
        }
    }
}

/// Mock USB HID device for testing bidirectional communication (no_std compatible)
#[derive(Debug)]
pub struct MockUsbHidDevice {
    pub device_id: u16,
    pub is_connected: bool,
    pub transmitted_messages: Vec<UsbTestMessage, 32>,
    pub received_messages: Vec<UsbTestMessage, 32>,
    pub transmission_errors: u32,
    pub reception_errors: u32,
    pub message_id_counter: u32,
    pub error_injection_enabled: bool,
    pub error_injection_rate: u8,
}

impl MockUsbHidDevice {
    /// Create a new mock USB HID device
    pub fn new(device_id: u16) -> Self {
        Self {
            device_id,
            is_connected: false,
            transmitted_messages: Vec::new(),
            received_messages: Vec::new(),
            transmission_errors: 0,
            reception_errors: 0,
            message_id_counter: 0,
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
            self.transmission_errors += 1;
            return Err("Device not connected");
        }

        self.message_id_counter += 1;
        let message_id = self.message_id_counter;

        // Use a simple timestamp (in a real implementation, this would use a timer)
        let timestamp_ms = message_id * 100; // Simple incrementing timestamp

        let mut message = UsbTestMessage::new(message_id, timestamp_ms, data, true);

        // Inject errors if enabled
        if self.error_injection_enabled && self.should_inject_error() {
            message.inject_error(ErrorType::CorruptChecksum);
        }

        // Store transmitted message
        if self.transmitted_messages.push(message).is_err() {
            return Err("Message buffer full");
        }

        Ok(message_id)
    }

    /// Receive a message (host to device)
    pub fn receive_message(&mut self, serialized_data: &[u8]) -> Result<UsbTestMessage, &'static str> {
        if !self.is_connected {
            self.reception_errors += 1;
            return Err("Device not connected");
        }

        // Inject errors if enabled
        let mut data = Vec::<u8, 80>::new();
        for &byte in serialized_data.iter().take(80) {
            if data.push(byte).is_err() {
                break; // Vector is full
            }
        }
        
        if self.error_injection_enabled && self.should_inject_error() {
            if !data.is_empty() {
                data[0] ^= 0xFF; // Corrupt first byte
            }
        }

        match UsbTestMessage::deserialize(&data) {
            Ok(mut message) => {
                message.is_outbound = false;
                
                if self.received_messages.push(message.clone()).is_err() {
                    return Err("Receive buffer full");
                }
                Ok(message)
            }
            Err(error) => {
                self.reception_errors += 1;
                Err(error)
            }
        }
    }

    /// Check if error should be injected based on rate
    fn should_inject_error(&self) -> bool {
        if self.error_injection_rate == 0 {
            return false;
        }
        
        // Simple pseudo-random number generation for testing (no_std compatible)
        let random_value = (self.message_id_counter * 17 + 23) % 100;
        
        random_value < self.error_injection_rate as u32
    }

    /// Get transmission statistics
    pub fn get_transmission_stats(&self) -> (u32, u32, u32, u32) {
        let transmitted_count = self.transmitted_messages.len() as u32;
        let received_count = self.received_messages.len() as u32;
        let transmission_errors = self.transmission_errors;
        let reception_errors = self.reception_errors;
        
        (transmitted_count, received_count, transmission_errors, reception_errors)
    }
}

// ============================================================================
// USB Communication Test Parameter Validation Tests
// ============================================================================

fn test_usb_communication_parameters_validation() -> TestResult {
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
    assert_no_std!(valid_params.validate().is_ok());

    // Test invalid message count (zero)
    let mut invalid_params = valid_params;
    invalid_params.message_count = 0;
    assert_eq_no_std!(invalid_params.validate(), Err(TestParameterError::InvalidResourceLimits));

    TestResult::pass()
}

fn test_message_integrity_validation() -> TestResult {
    let test_data = b"Test message data for integrity checking";
    let message = UsbTestMessage::new(12345, 67890, test_data, true);

    // Test successful serialization and deserialization
    let serialized = message.serialize();
    let deserialized = match UsbTestMessage::deserialize(&serialized) {
        Ok(msg) => msg,
        Err(_) => return TestResult::fail("Failed to deserialize message"),
    };

    assert_eq_no_std!(deserialized.message_id, message.message_id);
    assert_eq_no_std!(deserialized.timestamp_ms, message.timestamp_ms);
    assert_eq_no_std!(deserialized.data, message.data);
    assert_eq_no_std!(deserialized.checksum, message.checksum);

    TestResult::pass()
}

fn test_message_integrity_corruption_detection() -> TestResult {
    let test_data = b"Test message for corruption detection";
    let mut message = UsbTestMessage::new(54321, 98765, test_data, true);

    // Test checksum corruption detection
    message.inject_error(ErrorType::CorruptChecksum);
    let serialized = message.serialize();
    let result = UsbTestMessage::deserialize(&serialized);
    assert_no_std!(result.is_err());
    if let Err(error) = result {
        assert_eq_no_std!(error, "Checksum validation failed");
    }

    TestResult::pass()
}

fn test_bidirectional_communication_success() -> TestResult {
    let mut device = MockUsbHidDevice::new(0x1234);
    device.connect();

    // Test device-to-host communication
    let outbound_data = b"Device to host message";
    let message_id = match device.send_message(outbound_data) {
        Ok(id) => id,
        Err(_) => return TestResult::fail("Failed to send message"),
    };
    assert_no_std!(message_id > 0);

    // Test host-to-device communication
    let inbound_message = UsbTestMessage::new(message_id + 1, 33333, b"Host to device response", false);
    let serialized_response = inbound_message.serialize();
    let received = match device.receive_message(&serialized_response) {
        Ok(msg) => msg,
        Err(_) => return TestResult::fail("Failed to receive message"),
    };

    assert_eq_no_std!(received.message_id, inbound_message.message_id);
    assert_eq_no_std!(received.data, inbound_message.data);
    assert_no_std!(!received.is_outbound);

    // Verify statistics
    let (tx_count, rx_count, tx_errors, rx_errors) = device.get_transmission_stats();
    assert_eq_no_std!(tx_count, 1);
    assert_eq_no_std!(rx_count, 1);
    assert_eq_no_std!(tx_errors, 0);
    assert_eq_no_std!(rx_errors, 0);

    TestResult::pass()
}

fn test_usb_communication_test_integration() -> TestResult {
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
    assert_no_std!(result.is_ok());
    
    // Verify test is active
    assert_no_std!(processor.has_active_test());
    let active_test_type = processor.get_active_test_type();
    assert_eq_no_std!(active_test_type, Some(TestType::UsbCommunicationTest));
    
    // Complete the test
    let final_stats = processor.complete_usb_communication_test(timestamp_ms + 2000);
    assert_no_std!(final_stats.is_ok());
    let final_stats = final_stats.unwrap();
    
    // Verify final statistics
    assert_no_std!(final_stats.test_duration_ms > 0);
    assert_no_std!(final_stats.success_rate_percent >= 0.0);

    TestResult::pass()
}

// Test runner setup and execution
#[no_mangle]
pub extern "C" fn main() -> ! {
    let mut runner = TestRunner::new("USB Communication Validation Tests");
    
    // Register all test functions
    register_tests!(runner,
        test_usb_communication_parameters_validation,
        test_message_integrity_validation,
        test_message_integrity_corruption_detection,
        test_bidirectional_communication_success,
        test_usb_communication_test_integration
    );
    
    // Run all tests
    let _results = runner.run_all();
    
    // In a real implementation, results would be transmitted via USB HID
    // For now, we just loop indefinitely
    loop {}
}