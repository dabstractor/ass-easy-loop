//! USB Communication Test Integration
//! 
//! Integration tests for USB communication validation functionality
//! that work within the embedded no_std environment.
//! 
//! Requirements: 9.4, 9.5

#![cfg(test)]
#![no_std]

use ass_easy_loop::{
    TestCommandProcessor, TestType, TestStatus, TestParameters, TestResult,
    TestMeasurements, ResourceUsageStats, PerformanceMetrics, TestProcessorStatistics,
    TestExecutionError, TestParameterError,
    UsbCommunicationTestParameters, UsbCommunicationStatistics, TimingMeasurement
};
use heapless::Vec;

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
// USB Communication Statistics Tests
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
    
    // Verify bidirectional success (should be false with <95% success rate)
    assert_eq!(stats.bidirectional_success, false);
    
    // Test with higher success rate
    stats.transmission_errors = 0;
    stats.reception_errors = 0;
    stats.timeout_errors = 0;
    stats.integrity_check_failures = 1; // Only 1 error
    stats.calculate_derived_stats();
    
    assert!(stats.success_rate_percent >= 95.0);
    assert_eq!(stats.bidirectional_success, true);
}

#[test]
fn test_usb_communication_statistics_serialization() {
    let mut stats = UsbCommunicationStatistics::new();
    stats.test_duration_ms = 10000;
    stats.messages_sent = 200;
    stats.messages_received = 195;
    stats.transmission_errors = 2;
    stats.reception_errors = 3;
    stats.average_round_trip_time_us = 1200;
    stats.min_round_trip_time_us = 800;
    stats.max_round_trip_time_us = 2000;
    stats.throughput_messages_per_sec = 39;
    stats.error_rate_percent = 2.5;
    stats.success_rate_percent = 97.5;
    stats.bidirectional_success = true;

    // Serialize statistics
    let serialized = stats.serialize();
    assert!(serialized.len() > 40); // Should have substantial data

    // Verify key fields are present in serialization
    // Test duration (first 4 bytes)
    let duration = u32::from_le_bytes([serialized[0], serialized[1], serialized[2], serialized[3]]);
    assert_eq!(duration, 10000);

    // Messages sent (bytes 4-7)
    let messages_sent = u32::from_le_bytes([serialized[4], serialized[5], serialized[6], serialized[7]]);
    assert_eq!(messages_sent, 200);
}

// ============================================================================
// Test Processor Integration Tests
// ============================================================================

#[test]
fn test_usb_communication_test_execution() {
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
    let active_test_info = processor.get_active_test_info();
    assert!(active_test_info.is_some());
    let (test_type, status, _test_id) = active_test_info.unwrap();
    assert_eq!(test_type, TestType::UsbCommunicationTest);
    assert_eq!(status, TestStatus::Running);
}

#[test]
fn test_usb_communication_message_processing() {
    let mut processor = TestCommandProcessor::new();
    
    // Set up test
    let params = UsbCommunicationTestParameters::default();
    processor.execute_usb_communication_test(1, params, 1000).unwrap();
    
    // Process some messages
    for i in 0..10 {
        let message_data = b"Test message data for USB communication validation";
        let result = processor.process_usb_communication_message(
            i,
            message_data,
            i % 2 == 0, // Alternate between outbound and inbound
            1000 + i * 20,
        );
        assert!(result.is_ok());
    }
    
    // Get current statistics
    let stats = processor.get_usb_communication_statistics();
    assert!(stats.is_some());
    let stats = stats.unwrap();
    assert!(stats.messages_sent > 0);
}

#[test]
fn test_usb_communication_test_completion() {
    let mut processor = TestCommandProcessor::new();
    
    // Set up and run test
    let params = UsbCommunicationTestParameters {
        message_count: 20,
        message_interval_ms: 10,
        message_size_bytes: 24,
        timeout_per_message_ms: 500,
        enable_integrity_checking: true,
        enable_error_injection: false,
        error_injection_rate_percent: 0,
        bidirectional_test: true,
        concurrent_messages: 1,
    };
    
    processor.execute_usb_communication_test(99, params, 54321).unwrap();
    
    // Process some messages
    for i in 0..10 {
        let message_data = b"Test message";
        let _ = processor.process_usb_communication_message(
            i,
            message_data,
            true,
            54321 + i * 10,
        );
    }
    
    // Complete test and verify statistics
    let final_stats = processor.complete_usb_communication_test(54321 + 1000);
    assert!(final_stats.is_ok());
    let final_stats = final_stats.unwrap();
    
    assert!(final_stats.test_duration_ms > 0);
    assert!(final_stats.success_rate_percent >= 0.0);
    assert!(final_stats.success_rate_percent <= 100.0);
}

#[test]
fn test_usb_communication_test_with_integrity_checking() {
    let mut processor = TestCommandProcessor::new();
    
    // Create test parameters with integrity checking enabled
    let params = UsbCommunicationTestParameters {
        message_count: 10,
        message_interval_ms: 50,
        message_size_bytes: 32,
        timeout_per_message_ms: 1000,
        enable_integrity_checking: true,
        enable_error_injection: false,
        error_injection_rate_percent: 0,
        bidirectional_test: true,
        concurrent_messages: 1,
    };
    
    processor.execute_usb_communication_test(123, params, 10000).unwrap();
    
    // Process messages with valid data
    let valid_message = b"Valid message data for integrity test";
    let result = processor.process_usb_communication_message(
        1,
        valid_message,
        true,
        10100,
    );
    assert!(result.is_ok());
    
    // Process message with invalid data (simulating corruption)
    let invalid_message = b""; // Empty message should fail integrity check
    let result = processor.process_usb_communication_message(
        2,
        invalid_message,
        true,
        10200,
    );
    // This should fail due to integrity checking
    assert!(result.is_err());
    
    // Complete test
    let final_stats = processor.complete_usb_communication_test(11000);
    assert!(final_stats.is_ok());
    let stats = final_stats.unwrap();
    
    // Should have some integrity check failures
    assert!(stats.integrity_check_failures > 0 || stats.transmission_errors > 0);
}

#[test]
fn test_usb_communication_test_parameter_edge_cases() {
    // Test minimum valid parameters
    let min_params = UsbCommunicationTestParameters {
        message_count: 1,
        message_interval_ms: 1,
        message_size_bytes: 1,
        timeout_per_message_ms: 1,
        enable_integrity_checking: false,
        enable_error_injection: false,
        error_injection_rate_percent: 0,
        bidirectional_test: false,
        concurrent_messages: 1,
    };
    assert!(min_params.validate().is_ok());
    
    // Test maximum valid parameters
    let max_params = UsbCommunicationTestParameters {
        message_count: 10_000,
        message_interval_ms: 10_000,
        message_size_bytes: 64,
        timeout_per_message_ms: 30_000,
        enable_integrity_checking: true,
        enable_error_injection: true,
        error_injection_rate_percent: 100,
        bidirectional_test: true,
        concurrent_messages: 8,
    };
    assert!(max_params.validate().is_ok());
}

#[test]
fn test_usb_communication_statistics_edge_cases() {
    let mut stats = UsbCommunicationStatistics::new();
    
    // Test with zero operations
    stats.calculate_derived_stats();
    assert_eq!(stats.error_rate_percent, 0.0);
    assert_eq!(stats.success_rate_percent, 0.0);
    assert_eq!(stats.throughput_messages_per_sec, 0);
    assert_eq!(stats.bidirectional_success, false);
    
    // Test with only errors
    stats.messages_sent = 0;
    stats.messages_received = 0;
    stats.transmission_errors = 5;
    stats.reception_errors = 3;
    stats.calculate_derived_stats();
    assert_eq!(stats.error_rate_percent, 0.0); // No operations, so no error rate
    assert_eq!(stats.success_rate_percent, 0.0);
    
    // Test with perfect success
    stats.messages_sent = 100;
    stats.messages_received = 100;
    stats.transmission_errors = 0;
    stats.reception_errors = 0;
    stats.timeout_errors = 0;
    stats.integrity_check_failures = 0;
    stats.test_duration_ms = 1000;
    stats.calculate_derived_stats();
    assert_eq!(stats.error_rate_percent, 0.0);
    assert_eq!(stats.success_rate_percent, 100.0);
    assert_eq!(stats.throughput_messages_per_sec, 200); // 200 operations per second
    assert_eq!(stats.bidirectional_success, true);
}

#[test]
fn test_multiple_usb_communication_tests() {
    let mut processor = TestCommandProcessor::new();
    
    // First test
    let params1 = UsbCommunicationTestParameters {
        message_count: 10,
        message_interval_ms: 100,
        message_size_bytes: 16,
        timeout_per_message_ms: 500,
        enable_integrity_checking: false,
        enable_error_injection: false,
        error_injection_rate_percent: 0,
        bidirectional_test: false,
        concurrent_messages: 1,
    };
    
    processor.execute_usb_communication_test(1, params1, 1000).unwrap();
    
    // Process some messages
    for i in 0..5 {
        let _ = processor.process_usb_communication_message(i, b"Test1", true, 1000 + i * 100);
    }
    
    // Complete first test
    let stats1 = processor.complete_usb_communication_test(2000).unwrap();
    assert!(stats1.test_duration_ms > 0);
    
    // Second test (should be able to start after first completes)
    let params2 = UsbCommunicationTestParameters {
        message_count: 20,
        message_interval_ms: 50,
        message_size_bytes: 32,
        timeout_per_message_ms: 1000,
        enable_integrity_checking: true,
        enable_error_injection: false,
        error_injection_rate_percent: 0,
        bidirectional_test: true,
        concurrent_messages: 2,
    };
    
    let result = processor.execute_usb_communication_test(2, params2, 3000);
    assert!(result.is_ok());
    
    // Verify second test is now active
    assert!(processor.has_active_test());
    let active_info = processor.get_active_test_info().unwrap();
    assert_eq!(active_info.0, TestType::UsbCommunicationTest);
    assert_eq!(active_info.2, 2); // Test ID should be 2
}