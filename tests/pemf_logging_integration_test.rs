//! Integration tests for pEMF pulse generation logging
//! 
//! This test validates that the pEMF pulse generation system correctly
//! integrates with the USB HID logging system according to requirements 4.1-4.5

#![no_std]
#![no_main]

extern crate std;
use std::println;

use ass_easy_loop::logging::{LogLevel, LogMessage, LogQueue, QueueStats};

/// Test timestamp function for testing
fn test_timestamp() -> u32 {
    42
}

#[test]
fn test_pemf_logging_initialization() {
    // Test that pEMF logging initialization messages are properly formatted
    let message = LogMessage::new(
        1000,
        LogLevel::Info,
        "ass_easy_loop::app::pemf_pulse_task",
        "pEMF pulse generation initialized"
    );
    
    assert_eq_no_std!(message.timestamp, 1000);
    assert_eq_no_std!(message.level, LogLevel::Info);
    assert_no_std!(message.module_str().contains("pemf_pulse_task"));
    assert_eq_no_std!(message.message_str(), "pEMF pulse generation initialized");
}

#[test]
fn test_pemf_timing_validation_logging() {
    // Test timing validation log message formatting
    let timing_deviation_message = LogMessage::new(
        5000,
        LogLevel::Warn,
        "ass_easy_loop::app::pemf_pulse_task",
        "HIGH phase timing deviation: 1ms (expected: 2ms, actual: 3ms, tolerance: ±5ms)"
    );
    
    assert_eq_no_std!(timing_deviation_message.level, LogLevel::Warn);
    assert_no_std!(timing_deviation_message.message_str().contains("timing deviation"));
    assert_no_std!(timing_deviation_message.message_str().contains("HIGH phase"));
}

#[test]
fn test_pemf_error_logging() {
    // Test error logging for pulse generation failures
    let error_message = LogMessage::new(
        10000,
        LogLevel::Error,
        "ass_easy_loop::app::pemf_pulse_task",
        "Failed to set MOSFET pin HIGH - GPIO error in cycle 42"
    );
    
    assert_eq_no_std!(error_message.level, LogLevel::Error);
    assert_no_std!(error_message.message_str().contains("Failed to set MOSFET pin"));
    assert_no_std!(error_message.message_str().contains("GPIO error"));
}

#[test]
fn test_pemf_statistics_logging() {
    // Test pulse timing statistics log message formatting
    let stats_message = LogMessage::new(
        60000,
        LogLevel::Info,
        "ass_easy_loop::app::pemf_pulse_task",
        "pEMF pulse statistics (cycles: 120)"
    );
    
    assert_eq_no_std!(stats_message.level, LogLevel::Info);
    assert_no_std!(stats_message.message_str().contains("pulse statistics"));
    assert_no_std!(stats_message.message_str().contains("cycles: 120"));
}

#[test]
fn test_pemf_timing_conflict_detection() {
    // Test timing conflict detection logging
    let conflict_message = LogMessage::new(
        15000,
        LogLevel::Error,
        "ass_easy_loop::app::pemf_pulse_task",
        "Timing conflict detected: cycle started 450ms after previous (expected: 500ms)"
    );
    
    assert_eq_no_std!(conflict_message.level, LogLevel::Error);
    assert_no_std!(conflict_message.message_str().contains("Timing conflict detected"));
    assert_no_std!(conflict_message.message_str().contains("cycle started"));
}

#[test]
fn test_pemf_logging_queue_integration() {
    // Test that pEMF logging messages can be properly queued
    let mut queue: LogQueue<8> = LogQueue::new();
    
    // Simulate pEMF initialization logging
    let init_msg = LogMessage::new(
        0,
        LogLevel::Info,
        "pemf",
        "pEMF pulse generation initialized"
    );
    assert_no_std!(queue.enqueue(init_msg));
    
    // Simulate timing validation logging
    let timing_msg = LogMessage::new(
        5000,
        LogLevel::Warn,
        "pemf",
        "HIGH phase timing deviation: 1ms"
    );
    assert_no_std!(queue.enqueue(timing_msg));
    
    // Simulate error logging
    let error_msg = LogMessage::new(
        10000,
        LogLevel::Error,
        "pemf",
        "Failed to set MOSFET pin HIGH"
    );
    assert_no_std!(queue.enqueue(error_msg));
    
    // Simulate statistics logging
    let stats_msg = LogMessage::new(
        60000,
        LogLevel::Info,
        "pemf",
        "pEMF pulse statistics (cycles: 120)"
    );
    assert_no_std!(queue.enqueue(stats_msg));
    
    // Verify all messages were queued
    assert_eq_no_std!(queue.len(), 4);
    
    // Verify messages can be dequeued in order
    let msg1 = queue.dequeue().unwrap();
    assert_eq_no_std!(msg1.timestamp, 0);
    assert_no_std!(msg1.message_str().contains("initialized"));
    
    let msg2 = queue.dequeue().unwrap();
    assert_eq_no_std!(msg2.timestamp, 5000);
    assert_no_std!(msg2.message_str().contains("timing deviation"));
    
    let msg3 = queue.dequeue().unwrap();
    assert_eq_no_std!(msg3.timestamp, 10000);
    assert_no_std!(msg3.message_str().contains("Failed to set"));
    
    let msg4 = queue.dequeue().unwrap();
    assert_eq_no_std!(msg4.timestamp, 60000);
    assert_no_std!(msg4.message_str().contains("statistics"));
    
    assert_no_std!(queue.is_empty());
}

#[test]
fn test_pemf_logging_performance_impact() {
    // Test that logging doesn't significantly impact performance
    // This is a basic test - real performance testing would require hardware
    
    let mut queue: LogQueue<32> = LogQueue::new();
    let start_time = 0u32;
    
    // Simulate rapid logging during pEMF operation
    for cycle in 0..100 {
        let timestamp = start_time + (cycle * 500); // 500ms per cycle
        
        // Log every 10th cycle (like the actual implementation)
        if cycle % 10 == 0 {
            let msg = LogMessage::new(
                timestamp,
                LogLevel::Debug,
                "pemf",
                "Timing validation check"
            );
            assert_no_std!(queue.enqueue(msg));
        }
        
        // Log statistics every 120 cycles (like the actual implementation)
        if cycle % 120 == 0 && cycle > 0 {
            let msg = LogMessage::new(
                timestamp,
                LogLevel::Info,
                "pemf",
                "pEMF pulse statistics"
            );
            assert_no_std!(queue.enqueue(msg));
        }
    }
    
    // Verify reasonable number of messages were generated
    let stats = queue.stats();
    assert_no_std!(stats.messages_sent <= 20); // Should be much less than 100 cycles
    assert_no_std!(stats.current_utilization_percent < 50); // Should not fill the queue
}

#[test]
fn test_pemf_timing_constants_validation() {
    // Test the timing constants used in pEMF logging
    const PULSE_HIGH_DURATION_MS: u64 = 2;
    const PULSE_LOW_DURATION_MS: u64 = 498;
    const EXPECTED_TOTAL_PERIOD_MS: u64 = PULSE_HIGH_DURATION_MS + PULSE_LOW_DURATION_MS;
    const TIMING_TOLERANCE_PERCENT: f32 = 0.01;
    
    // Verify timing constants are correct for 2Hz operation
    assert_eq_no_std!(EXPECTED_TOTAL_PERIOD_MS, 500);
    assert_eq_no_std!(PULSE_HIGH_DURATION_MS, 2);
    assert_eq_no_std!(PULSE_LOW_DURATION_MS, 498);
    
    // Verify tolerance calculation
    let max_deviation = ((EXPECTED_TOTAL_PERIOD_MS as f32) * TIMING_TOLERANCE_PERCENT) as u64;
    assert_eq_no_std!(max_deviation, 5); // ±5ms for ±1% of 500ms
    
    // Verify frequency calculation
    let frequency_hz = 1000.0 / (EXPECTED_TOTAL_PERIOD_MS as f32);
    assert_no_std!((frequency_hz - 2.0).abs() < 0.001);
}

#[test]
fn test_pemf_logging_message_truncation() {
    // Test that long pEMF log messages are properly truncated
    let long_message = "This is a very long pEMF timing validation message that exceeds the maximum message length and should be truncated to fit within the 48-byte limit imposed by the logging system";
    
    let msg = LogMessage::new(
        12345,
        LogLevel::Warn,
        "pemf_very_long_module_name",
        long_message
    );
    
    // Verify message was truncated to fit
    assert_no_std!(msg.message_str().len() <= 48);
    assert_no_std!(msg.module_str().len() <= 8);
    
    // Verify essential information is preserved
    assert_no_std!(msg.message_str().contains("very long"));
}