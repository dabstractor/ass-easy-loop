//! Comprehensive integration tests for USB HID functionality
//! 
//! This test file validates the complete USB HID logging system including:
//! - USB device enumeration and HID report transmission
//! - End-to-end communication tests between device and host utility
//! - Performance tests to measure timing impact on existing pEMF/battery tasks
//! - Error recovery tests for USB disconnection/reconnection scenarios
//! 
//! Requirements: 10.2, 10.3, 10.4, 10.5

#![cfg(test)]

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use ass_easy_loop::logging::{LogLevel, LogMessage, LogQueue, LogReport};

/// Mock USB HID device for testing USB enumeration and communication
#[derive(Debug, Clone)]
pub struct MockUsbHidDevice {
    pub vendor_id: u16,
    pub product_id: u16,
    pub device_release: u16,
    pub is_enumerated: bool,
    pub is_connected: bool,
    pub transmitted_reports: Arc<Mutex<VecDeque<LogReport>>>,
    pub enumeration_failures: Arc<Mutex<u32>>,
    pub transmission_errors: Arc<Mutex<u32>>,
    pub connection_state_changes: Arc<Mutex<Vec<bool>>>,
}

impl MockUsbHidDevice {
    pub fn new(vendor_id: u16, product_id: u16) -> Self {
        Self {
            vendor_id,
            product_id,
            device_release: 0x0100,
            is_enumerated: false,
            is_connected: false,
            transmitted_reports: Arc::new(Mutex::new(VecDeque::new())),
            enumeration_failures: Arc::new(Mutex::new(0)),
            transmission_errors: Arc::new(Mutex::new(0)),
            connection_state_changes: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Simulate USB device enumeration
    pub fn enumerate(&mut self) -> Result<(), &'static str> {
        if !self.is_connected {
            *self.enumeration_failures.lock().unwrap() += 1;
            return Err("Device not connected");
        }

        // Simulate enumeration delay
        thread::sleep(Duration::from_millis(10));
        
        self.is_enumerated = true;
        Ok(())
    }

    /// Simulate USB connection
    pub fn connect(&mut self) {
        self.is_connected = true;
        self.connection_state_changes.lock().unwrap().push(true);
    }

    /// Simulate USB disconnection
    pub fn disconnect(&mut self) {
        self.is_connected = false;
        self.is_enumerated = false;
        self.connection_state_changes.lock().unwrap().push(false);
    }

    /// Simulate HID report transmission
    pub fn transmit_report(&mut self, report: LogReport) -> Result<(), &'static str> {
        if !self.is_connected || !self.is_enumerated {
            *self.transmission_errors.lock().unwrap() += 1;
            return Err("Device not ready for transmission");
        }

        // Simulate transmission delay
        thread::sleep(Duration::from_millis(1));
        
        self.transmitted_reports.lock().unwrap().push_back(report);
        Ok(())
    }

    /// Get transmitted reports for verification
    pub fn get_transmitted_reports(&self) -> Vec<LogReport> {
        self.transmitted_reports.lock().unwrap().iter().cloned().collect()
    }

    /// Get enumeration failure count
    pub fn get_enumeration_failures(&self) -> u32 {
        *self.enumeration_failures.lock().unwrap()
    }

    /// Get transmission error count
    pub fn get_transmission_errors(&self) -> u32 {
        *self.transmission_errors.lock().unwrap()
    }

    /// Get connection state change history
    pub fn get_connection_history(&self) -> Vec<bool> {
        self.connection_state_changes.lock().unwrap().clone()
    }
}

/// Mock host utility for testing end-to-end communication
#[derive(Debug)]
pub struct MockHostUtility {
    pub received_reports: Arc<Mutex<VecDeque<LogReport>>>,
    pub parsed_messages: Arc<Mutex<VecDeque<LogMessage>>>,
    pub connection_errors: Arc<Mutex<u32>>,
    pub parsing_errors: Arc<Mutex<u32>>,
    pub is_running: Arc<Mutex<bool>>,
}

impl MockHostUtility {
    pub fn new() -> Self {
        Self {
            received_reports: Arc::new(Mutex::new(VecDeque::new())),
            parsed_messages: Arc::new(Mutex::new(VecDeque::new())),
            connection_errors: Arc::new(Mutex::new(0)),
            parsing_errors: Arc::new(Mutex::new(0)),
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start the host utility (simulate hidapi connection)
    pub fn start(&self) -> Result<(), &'static str> {
        *self.is_running.lock().unwrap() = true;
        Ok(())
    }

    /// Stop the host utility
    pub fn stop(&self) {
        *self.is_running.lock().unwrap() = false;
    }

    /// Simulate receiving HID reports from device
    pub fn receive_report(&self, report: LogReport) -> Result<(), &'static str> {
        if !*self.is_running.lock().unwrap() {
            *self.connection_errors.lock().unwrap() += 1;
            return Err("Host utility not running");
        }

        self.received_reports.lock().unwrap().push_back(report);

        // Parse the report into a log message
        match report.to_log_message() {
            Ok(message) => {
                self.parsed_messages.lock().unwrap().push_back(message);
                Ok(())
            }
            Err(_) => {
                *self.parsing_errors.lock().unwrap() += 1;
                Err("Failed to parse log message")
            }
        }
    }

    /// Get received reports
    pub fn get_received_reports(&self) -> Vec<LogReport> {
        self.received_reports.lock().unwrap().iter().cloned().collect()
    }

    /// Get parsed messages
    pub fn get_parsed_messages(&self) -> Vec<LogMessage> {
        self.parsed_messages.lock().unwrap().iter().cloned().collect()
    }

    /// Get error counts
    pub fn get_error_counts(&self) -> (u32, u32) {
        (
            *self.connection_errors.lock().unwrap(),
            *self.parsing_errors.lock().unwrap(),
        )
    }
}

/// Test helper function to create a mock timestamp function
fn mock_timestamp() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u32
}

// ============================================================================
// USB Device Enumeration and HID Report Transmission Tests
// ============================================================================

#[test]
fn test_usb_device_enumeration_success() {
    // Test successful USB device enumeration
    // Requirements: 10.2
    
    let mut device = MockUsbHidDevice::new(0x1234, 0x5678);
    
    // Initially not connected or enumerated
    assert!(!device.is_connected);
    assert!(!device.is_enumerated);
    
    // Connect device
    device.connect();
    assert!(device.is_connected);
    assert!(!device.is_enumerated);
    
    // Enumerate device
    let result = device.enumerate();
    assert!(result.is_ok());
    assert!(device.is_enumerated);
    assert_eq!(device.get_enumeration_failures(), 0);
    
    // Verify device descriptors
    assert_eq!(device.vendor_id, 0x1234);
    assert_eq!(device.product_id, 0x5678);
    assert_eq!(device.device_release, 0x0100);
}

#[test]
fn test_usb_device_enumeration_failure() {
    // Test USB device enumeration failure when not connected
    // Requirements: 10.2
    
    let mut device = MockUsbHidDevice::new(0x1234, 0x5678);
    
    // Attempt enumeration without connection
    let result = device.enumerate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Device not connected");
    assert!(!device.is_enumerated);
    assert_eq!(device.get_enumeration_failures(), 1);
    
    // Multiple failed attempts should increment failure counter
    let _ = device.enumerate();
    let _ = device.enumerate();
    assert_eq!(device.get_enumeration_failures(), 3);
}

#[test]
fn test_hid_report_transmission_success() {
    // Test successful HID report transmission
    // Requirements: 10.2
    
    let mut device = MockUsbHidDevice::new(0x1234, 0x5678);
    device.connect();
    device.enumerate().unwrap();
    
    // Create test log message and report
    let message = LogMessage::new(12345, LogLevel::Info, "TEST", "Test message");
    let report = LogReport::from_log_message(&message);
    
    // Transmit report
    let result = device.transmit_report(report);
    assert!(result.is_ok());
    assert_eq!(device.get_transmission_errors(), 0);
    
    // Verify report was transmitted
    let transmitted = device.get_transmitted_reports();
    assert_eq!(transmitted.len(), 1);
    
    // Verify report content
    let received_message = transmitted[0].to_log_message().unwrap();
    assert_eq!(received_message.timestamp, 12345);
    assert_eq!(received_message.level, LogLevel::Info);
    assert_eq!(received_message.module_str(), "TEST");
    assert_eq!(received_message.message_str(), "Test message");
}

#[test]
fn test_hid_report_transmission_failure() {
    // Test HID report transmission failure when device not ready
    // Requirements: 10.2
    
    let mut device = MockUsbHidDevice::new(0x1234, 0x5678);
    
    let message = LogMessage::new(12345, LogLevel::Error, "TEST", "Error message");
    let report = LogReport::from_log_message(&message);
    
    // Attempt transmission without connection
    let result = device.transmit_report(report.clone());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Device not ready for transmission");
    assert_eq!(device.get_transmission_errors(), 1);
    
    // Connect but don't enumerate
    device.connect();
    let result = device.transmit_report(report);
    assert!(result.is_err());
    assert_eq!(device.get_transmission_errors(), 2);
}

#[test]
fn test_multiple_hid_report_transmission() {
    // Test transmission of multiple HID reports in sequence
    // Requirements: 10.2
    
    let mut device = MockUsbHidDevice::new(0x1234, 0x5678);
    device.connect();
    device.enumerate().unwrap();
    
    // Create multiple test messages
    let messages = vec![
        LogMessage::new(1000, LogLevel::Debug, "MODULE1", "Debug message"),
        LogMessage::new(2000, LogLevel::Info, "MODULE2", "Info message"),
        LogMessage::new(3000, LogLevel::Warn, "MODULE3", "Warning message"),
        LogMessage::new(4000, LogLevel::Error, "MODULE4", "Error message"),
    ];
    
    // Transmit all reports
    for message in &messages {
        let report = LogReport::from_log_message(message);
        let result = device.transmit_report(report);
        assert!(result.is_ok());
    }
    
    // Verify all reports were transmitted
    let transmitted = device.get_transmitted_reports();
    assert_eq!(transmitted.len(), 4);
    assert_eq!(device.get_transmission_errors(), 0);
    
    // Verify report order and content
    for (i, transmitted_report) in transmitted.iter().enumerate() {
        let received_message = transmitted_report.to_log_message().unwrap();
        let original_message = &messages[i];
        
        assert_eq!(received_message.timestamp, original_message.timestamp);
        assert_eq!(received_message.level, original_message.level);
        assert_eq!(received_message.module_str(), original_message.module_str());
        assert_eq!(received_message.message_str(), original_message.message_str());
    }
}

// ============================================================================
// End-to-End Communication Tests
// ============================================================================

#[test]
fn test_end_to_end_communication_success() {
    // Test complete end-to-end communication between device and host
    // Requirements: 10.3
    
    let mut device = MockUsbHidDevice::new(0x1234, 0x5678);
    let host = MockHostUtility::new();
    
    // Set up device and host
    device.connect();
    device.enumerate().unwrap();
    host.start().unwrap();
    
    // Create test messages
    let test_messages = vec![
        LogMessage::new(1000, LogLevel::Info, "BATTERY", "Battery voltage: 3.7V"),
        LogMessage::new(2000, LogLevel::Warn, "PEMF", "Timing deviation detected"),
        LogMessage::new(3000, LogLevel::Error, "SYSTEM", "Memory usage high"),
    ];
    
    // Simulate device-to-host communication
    for message in &test_messages {
        let report = LogReport::from_log_message(message);
        
        // Device transmits report
        device.transmit_report(report.clone()).unwrap();
        
        // Host receives report
        host.receive_report(report).unwrap();
    }
    
    // Verify communication success
    let (connection_errors, parsing_errors) = host.get_error_counts();
    assert_eq!(connection_errors, 0);
    assert_eq!(parsing_errors, 0);
    
    // Verify all messages were received and parsed correctly
    let received_reports = host.get_received_reports();
    let parsed_messages = host.get_parsed_messages();
    
    assert_eq!(received_reports.len(), 3);
    assert_eq!(parsed_messages.len(), 3);
    
    // Verify message content integrity
    for (i, parsed_message) in parsed_messages.iter().enumerate() {
        let original_message = &test_messages[i];
        
        assert_eq!(parsed_message.timestamp, original_message.timestamp);
        assert_eq!(parsed_message.level, original_message.level);
        assert_eq!(parsed_message.module_str(), original_message.module_str());
        assert_eq!(parsed_message.message_str(), original_message.message_str());
    }
}

#[test]
fn test_end_to_end_communication_with_host_errors() {
    // Test end-to-end communication with host-side errors
    // Requirements: 10.3
    
    let mut device = MockUsbHidDevice::new(0x1234, 0x5678);
    let host = MockHostUtility::new();
    
    device.connect();
    device.enumerate().unwrap();
    // Don't start host utility to simulate connection error
    
    let message = LogMessage::new(1000, LogLevel::Info, "TEST", "Test message");
    let report = LogReport::from_log_message(&message);
    
    // Device transmits successfully
    device.transmit_report(report.clone()).unwrap();
    
    // Host fails to receive due to not being started
    let result = host.receive_report(report);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Host utility not running");
    
    let (connection_errors, parsing_errors) = host.get_error_counts();
    assert_eq!(connection_errors, 1);
    assert_eq!(parsing_errors, 0);
}

#[test]
fn test_end_to_end_communication_with_parsing_errors() {
    // Test end-to-end communication with message parsing errors
    // Requirements: 10.3
    
    let mut device = MockUsbHidDevice::new(0x1234, 0x5678);
    let host = MockHostUtility::new();
    
    device.connect();
    device.enumerate().unwrap();
    host.start().unwrap();
    
    // Create corrupted report with invalid log level
    let mut corrupted_report = LogReport { data: [0u8; 64] };
    corrupted_report.data[0] = 255; // Invalid log level
    
    // Device transmits corrupted report
    device.transmit_report(corrupted_report.clone()).unwrap();
    
    // Host receives but fails to parse
    let result = host.receive_report(corrupted_report);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Failed to parse log message");
    
    let (connection_errors, parsing_errors) = host.get_error_counts();
    assert_eq!(connection_errors, 0);
    assert_eq!(parsing_errors, 1);
}

// ============================================================================
// Performance Impact Tests
// ============================================================================

#[test]
fn test_usb_logging_performance_impact_on_pemf_timing() {
    // Test that USB logging has minimal impact on pEMF pulse timing
    // Requirements: 10.4, 7.1 (±1% tolerance)
    
    let mut baseline_times = Vec::new();
    let mut with_logging_times = Vec::new();
    
    // Measure baseline timing without USB logging
    for _ in 0..100 {
        let start = Instant::now();
        // Simulate pEMF pulse generation work
        thread::sleep(Duration::from_micros(100));
        baseline_times.push(start.elapsed());
    }
    
    // Measure timing with USB logging active
    let mut queue: LogQueue<32> = LogQueue::new();
    for i in 0..100 {
        let start = Instant::now();
        // Simulate pEMF pulse generation work
        thread::sleep(Duration::from_micros(100));
        
        // Add USB logging overhead
        let message = LogMessage::new(i, LogLevel::Info, "PEMF", "Pulse cycle");
        let _ = queue.enqueue(message);
        
        with_logging_times.push(start.elapsed());
    }
    
    // Calculate average times
    let baseline_avg: Duration = baseline_times.iter().sum::<Duration>() / baseline_times.len() as u32;
    let logging_avg: Duration = with_logging_times.iter().sum::<Duration>() / with_logging_times.len() as u32;
    
    // Calculate performance impact percentage
    let baseline_us = baseline_avg.as_micros() as f64;
    let logging_us = logging_avg.as_micros() as f64;
    
    let performance_impact_percent = if baseline_us > 0.0 {
        ((logging_us - baseline_us) / baseline_us) * 100.0
    } else {
        0.0
    };
    
    // Verify performance impact is within acceptable limits
    // Requirements: 7.1 (pEMF pulse timing SHALL remain within ±1% tolerance)
    assert!(performance_impact_percent < 1.0, 
            "USB logging impact on pEMF timing exceeds 1%: {:.2}%", performance_impact_percent);
}

#[test]
fn test_memory_usage_impact_with_usb_logging() {
    // Test memory usage impact of USB logging system
    // Requirements: 10.4, 7.2
    
    // Test queue memory usage scaling
    let small_queue: LogQueue<8> = LogQueue::new();
    let medium_queue: LogQueue<32> = LogQueue::new();
    let large_queue: LogQueue<128> = LogQueue::new();
    
    let small_size = std::mem::size_of_val(&small_queue);
    let medium_size = std::mem::size_of_val(&medium_queue);
    let large_size = std::mem::size_of_val(&large_queue);
    
    // Verify memory usage scales reasonably with queue size
    assert!(medium_size > small_size);
    assert!(large_size > medium_size);
    
    // Test that queue doesn't leak memory during operation
    let mut test_queue: LogQueue<16> = LogQueue::new();
    let initial_stats = test_queue.stats();
    
    // Fill and empty queue multiple times
    for cycle in 0..10 {
        // Fill queue
        for i in 0..20 { // Overfill to test overflow handling
            let message = LogMessage::new(
                (cycle * 100 + i) as u32, 
                LogLevel::Info, 
                "MEMORY", 
                "Memory test message"
            );
            test_queue.enqueue(message);
        }
        
        // Empty queue
        while test_queue.dequeue().is_some() {}
    }
    
    let final_stats = test_queue.stats();
    
    // Verify queue handled overflow correctly
    assert_eq!(final_stats.messages_sent, 200); // 10 cycles * 20 messages
    assert!(final_stats.messages_dropped > 0); // Should have dropped some due to overflow
    assert_eq!(test_queue.len(), 0); // Queue should be empty
}

// ============================================================================
// Error Recovery Tests
// ============================================================================

#[test]
fn test_usb_disconnection_recovery() {
    // Test system behavior during USB disconnection and reconnection
    // Requirements: 10.5, 7.3
    
    let mut device = MockUsbHidDevice::new(0x1234, 0x5678);
    let host = MockHostUtility::new();
    
    // Initial setup
    device.connect();
    device.enumerate().unwrap();
    host.start().unwrap();
    
    // Normal operation
    let message1 = LogMessage::new(1000, LogLevel::Info, "TEST", "Before disconnect");
    let report1 = LogReport::from_log_message(&message1);
    device.transmit_report(report1.clone()).unwrap();
    host.receive_report(report1).unwrap();
    
    // Simulate USB disconnection
    device.disconnect();
    
    // Verify device state after disconnection
    assert!(!device.is_connected);
    assert!(!device.is_enumerated);
    
    // Attempt transmission during disconnection (should fail gracefully)
    let message2 = LogMessage::new(2000, LogLevel::Warn, "TEST", "During disconnect");
    let report2 = LogReport::from_log_message(&message2);
    let result = device.transmit_report(report2);
    assert!(result.is_err());
    assert_eq!(device.get_transmission_errors(), 1);
    
    // Simulate USB reconnection
    device.connect();
    device.enumerate().unwrap();
    
    // Verify device state after reconnection
    assert!(device.is_connected);
    assert!(device.is_enumerated);
    
    // Resume normal operation
    let message3 = LogMessage::new(3000, LogLevel::Info, "TEST", "After reconnect");
    let report3 = LogReport::from_log_message(&message3);
    device.transmit_report(report3.clone()).unwrap();
    host.receive_report(report3).unwrap();
    
    // Verify connection history
    let connection_history = device.get_connection_history();
    assert_eq!(connection_history, vec![true, false, true]); // connect, disconnect, reconnect
    
    // Verify final state
    let transmitted = device.get_transmitted_reports();
    assert_eq!(transmitted.len(), 2); // Only successful transmissions
    
    let parsed_messages = host.get_parsed_messages();
    assert_eq!(parsed_messages.len(), 2);
    assert_eq!(parsed_messages[0].message_str(), "Before disconnect");
    assert_eq!(parsed_messages[1].message_str(), "After reconnect");
}

#[test]
fn test_queue_overflow_recovery_during_usb_errors() {
    // Test queue behavior during USB transmission errors
    // Requirements: 10.5, 7.4, 7.5
    
    let mut device = MockUsbHidDevice::new(0x1234, 0x5678);
    let mut queue: LogQueue<8> = LogQueue::new(); // Small queue to force overflow
    
    // Fill queue while USB is disconnected
    for i in 0..16 { // More messages than queue capacity
        let message = LogMessage::new(i, LogLevel::Info, "OVERFLOW", "Test message");
        queue.enqueue(message);
    }
    
    let stats_after_fill = queue.stats();
    assert_eq!(stats_after_fill.messages_sent, 16);
    assert_eq!(stats_after_fill.messages_dropped, 8); // 16 - 8 capacity
    assert_eq!(queue.len(), 8); // Queue should be full
    
    // Connect USB and attempt to drain queue
    device.connect();
    device.enumerate().unwrap();
    
    let mut transmitted_count = 0;
    let mut transmission_errors = 0;
    
    // Simulate USB task draining queue
    while let Some(message) = queue.dequeue() {
        let report = LogReport::from_log_message(&message);
        
        if device.transmit_report(report).is_ok() {
            transmitted_count += 1;
        } else {
            transmission_errors += 1;
        }
    }
    
    // Verify queue was drained successfully
    assert_eq!(transmitted_count, 8); // All remaining messages transmitted
    assert_eq!(transmission_errors, 0);
    assert_eq!(queue.len(), 0); // Queue should be empty
    
    // Verify queue can continue normal operation
    let new_message = LogMessage::new(100, LogLevel::Info, "RECOVERY", "After recovery");
    queue.enqueue(new_message);
    assert_eq!(queue.len(), 1);
    
    let recovered_message = queue.dequeue().unwrap();
    assert_eq!(recovered_message.timestamp, 100);
    assert_eq!(recovered_message.message_str(), "After recovery");
}