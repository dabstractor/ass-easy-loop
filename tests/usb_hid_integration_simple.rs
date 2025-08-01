//! Simplified USB HID integration tests that can run in the test environment
//! 
//! This demonstrates the testing approach for USB HID functionality including:
//! - Message serialization and deserialization
//! - Queue operations and overflow handling
//! - Performance impact measurement
//! - Error recovery scenarios
//! 
//! Requirements: 10.2, 10.3, 10.4, 10.5

#![cfg(test)]

use ass_easy_loop::logging::{LogLevel, LogMessage, LogQueue, LogReport};

/// Test helper function to create a mock timestamp function
fn mock_timestamp() -> u32 {
    42
}

// ============================================================================
// USB HID Report Serialization Tests (Requirements: 10.2)
// ============================================================================

#[test]
fn test_log_message_to_hid_report_serialization() {
    // Test that log messages can be properly serialized to HID reports
    let message = LogMessage::new(12345, LogLevel::Info, "TEST", "Test message");
    let report = LogReport::from_log_message(&message);
    
    // Verify report structure
    assert_eq!(report.data.len(), 64);
    
    // Verify serialized content
    assert_eq!(report.data[0], LogLevel::Info as u8); // Log level
    assert_eq!(&report.data[1..5], b"TEST"); // Module name
    assert_eq!(&report.data[9..21], b"Test message"); // Message content
    
    // Verify timestamp (little-endian u32 at bytes 57-60)
    let timestamp_bytes = 12345u32.to_le_bytes();
    assert_eq!(&report.data[57..61], &timestamp_bytes);
}

#[test]
fn test_hid_report_to_log_message_deserialization() {
    // Test that HID reports can be properly deserialized back to log messages
    let original_message = LogMessage::new(54321, LogLevel::Error, "MODULE", "Error occurred");
    let report = LogReport::from_log_message(&original_message);
    
    // Deserialize back to log message
    let deserialized_message = report.to_log_message().unwrap();
    
    // Verify all fields match
    assert_eq!(deserialized_message.timestamp, original_message.timestamp);
    assert_eq!(deserialized_message.level, original_message.level);
    assert_eq!(deserialized_message.module_str(), original_message.module_str());
    assert_eq!(deserialized_message.message_str(), original_message.message_str());
}

#[test]
fn test_hid_report_serialization_roundtrip() {
    // Test complete roundtrip: message -> report -> message
    let test_cases = vec![
        (LogLevel::Debug, "DEBUG_MOD", "Debug message with special chars: !@#$%"),
        (LogLevel::Info, "INFO", "Information message"),
        (LogLevel::Warn, "WARNING", "Warning: system overload detected"),
        (LogLevel::Error, "ERROR", "Critical error in subsystem"),
    ];
    
    for (level, module, message_text) in test_cases {
        let original = LogMessage::new(99999, level, module, message_text);
        let report = LogReport::from_log_message(&original);
        let recovered = report.to_log_message().unwrap();
        
        assert_eq!(original.timestamp, recovered.timestamp);
        assert_eq!(original.level, recovered.level);
        assert_eq!(original.module_str(), recovered.module_str());
        assert_eq!(original.message_str(), recovered.message_str());
    }
}

#[test]
fn test_invalid_hid_report_deserialization() {
    // Test error handling for corrupted HID reports
    let mut corrupted_report = LogReport { data: [0u8; 64] };
    corrupted_report.data[0] = 255; // Invalid log level
    
    let result = corrupted_report.to_log_message();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Invalid log level");
}

// ============================================================================
// Queue Integration Tests (Requirements: 10.3)
// ============================================================================

#[test]
fn test_queue_hid_report_integration() {
    // Test integration between queue operations and HID report generation
    let mut queue: LogQueue<8> = LogQueue::new();
    
    // Fill queue with various message types
    let test_messages = vec![
        LogMessage::new(1000, LogLevel::Debug, "BATTERY", "ADC reading: 1500"),
        LogMessage::new(2000, LogLevel::Info, "PEMF", "Pulse cycle started"),
        LogMessage::new(3000, LogLevel::Warn, "SYSTEM", "Memory usage: 85%"),
        LogMessage::new(4000, LogLevel::Error, "USB", "Transmission failed"),
    ];
    
    // Enqueue all messages
    for message in &test_messages {
        assert!(queue.enqueue(message.clone()));
    }
    
    // Dequeue and convert to HID reports
    let mut reports = Vec::new();
    while let Some(message) = queue.dequeue() {
        let report = LogReport::from_log_message(&message);
        reports.push(report);
    }
    
    // Verify all messages were processed correctly
    assert_eq!(reports.len(), 4);
    
    for (i, report) in reports.iter().enumerate() {
        let recovered_message = report.to_log_message().unwrap();
        let original_message = &test_messages[i];
        
        assert_eq!(recovered_message.timestamp, original_message.timestamp);
        assert_eq!(recovered_message.level, original_message.level);
        assert_eq!(recovered_message.module_str(), original_message.module_str());
        assert_eq!(recovered_message.message_str(), original_message.message_str());
    }
}

#[test]
fn test_queue_overflow_with_hid_reports() {
    // Test queue overflow behavior with HID report generation
    let mut queue: LogQueue<4> = LogQueue::new(); // Small queue
    
    // Fill queue beyond capacity
    for i in 0..8 {
        let message = LogMessage::new(i, LogLevel::Info, "OVERFLOW", "Test message");
        queue.enqueue(message);
    }
    
    let stats = queue.stats();
    assert_eq!(stats.messages_sent, 8);
    assert_eq!(stats.messages_dropped, 4); // 8 - 4 capacity
    assert_eq!(queue.len(), 4);
    
    // Verify remaining messages can be converted to HID reports
    let mut report_count = 0;
    while let Some(message) = queue.dequeue() {
        let report = LogReport::from_log_message(&message);
        let recovered = report.to_log_message().unwrap();
        
        // Verify message integrity
        assert_eq!(recovered.level, LogLevel::Info);
        assert_eq!(recovered.module_str(), "OVERFLOW");
        assert_eq!(recovered.message_str(), "Test message");
        
        report_count += 1;
    }
    
    assert_eq!(report_count, 4);
}

// ============================================================================
// Performance Impact Tests (Requirements: 10.4)
// ============================================================================

#[test]
fn test_hid_report_generation_performance() {
    // Test performance impact of HID report generation
    use std::time::Instant;
    
    let message = LogMessage::new(12345, LogLevel::Info, "PERF", "Performance test message");
    
    // Measure baseline message creation time
    let start = Instant::now();
    for _ in 0..1000 {
        let _msg = LogMessage::new(12345, LogLevel::Info, "PERF", "Performance test message");
    }
    let baseline_time = start.elapsed();
    
    // Measure time with HID report generation
    let start = Instant::now();
    for _ in 0..1000 {
        let msg = LogMessage::new(12345, LogLevel::Info, "PERF", "Performance test message");
        let _report = LogReport::from_log_message(&msg);
    }
    let with_hid_time = start.elapsed();
    
    // Calculate overhead percentage
    let overhead_percent = if baseline_time.as_nanos() > 0 {
        ((with_hid_time.as_nanos() as f64 - baseline_time.as_nanos() as f64) / baseline_time.as_nanos() as f64) * 100.0
    } else {
        0.0
    };
    
    // HID report generation should add minimal overhead
    assert!(overhead_percent < 50.0, "HID report generation overhead too high: {:.2}%", overhead_percent);
    
    // Verify both operations complete in reasonable time
    assert!(baseline_time.as_millis() < 10, "Baseline message creation too slow");
    assert!(with_hid_time.as_millis() < 20, "HID report generation too slow");
}

#[test]
fn test_queue_operations_performance_impact() {
    // Test performance impact of queue operations with HID integration
    use std::time::Instant;
    
    let mut queue: LogQueue<32> = LogQueue::new();
    
    // Measure queue fill performance
    let start = Instant::now();
    for i in 0..1000 {
        let message = LogMessage::new(i, LogLevel::Info, "PERF", "Queue performance test");
        queue.enqueue(message);
    }
    let fill_time = start.elapsed();
    
    // Measure queue drain with HID report generation
    let start = Instant::now();
    let mut report_count = 0;
    while let Some(message) = queue.dequeue() {
        let _report = LogReport::from_log_message(&message);
        report_count += 1;
    }
    let drain_time = start.elapsed();
    
    // Verify performance characteristics
    assert!(fill_time.as_millis() < 50, "Queue fill too slow: {}ms", fill_time.as_millis());
    assert!(drain_time.as_millis() < 100, "Queue drain with HID too slow: {}ms", drain_time.as_millis());
    assert_eq!(report_count, 32); // Only queue capacity should remain after overflow
    
    let stats = queue.stats();
    assert_eq!(stats.messages_sent, 1000);
    assert_eq!(stats.messages_dropped, 968); // 1000 - 32 capacity
}

// ============================================================================
// Error Recovery Tests (Requirements: 10.5)
// ============================================================================

#[test]
fn test_hid_report_error_recovery() {
    // Test system behavior when HID report operations fail
    let mut queue: LogQueue<8> = LogQueue::new();
    
    // Add valid messages
    for i in 0..5 {
        let message = LogMessage::new(i, LogLevel::Info, "RECOVERY", "Valid message");
        queue.enqueue(message);
    }
    
    // Simulate processing with some failures
    let mut successful_reports = 0;
    let mut failed_reports = 0;
    
    while let Some(message) = queue.dequeue() {
        let report = LogReport::from_log_message(&message);
        
        // Simulate occasional processing failure
        if message.timestamp % 3 == 0 {
            // Simulate failure by trying to deserialize corrupted data
            let mut corrupted = report;
            corrupted.data[0] = 255; // Invalid log level
            
            if corrupted.to_log_message().is_err() {
                failed_reports += 1;
                continue; // Skip this report
            }
        }
        
        // Process successful report
        let recovered = report.to_log_message().unwrap();
        assert_eq!(recovered.module_str(), "RECOVERY");
        successful_reports += 1;
    }
    
    // Verify system handled failures gracefully
    assert!(successful_reports > 0, "No reports processed successfully");
    assert!(failed_reports > 0, "No failures simulated");
    assert_eq!(successful_reports + failed_reports, 5);
}

#[test]
fn test_queue_recovery_after_overflow() {
    // Test queue recovery after overflow conditions
    let mut queue: LogQueue<4> = LogQueue::new();
    
    // Cause overflow
    for i in 0..10 {
        let message = LogMessage::new(i, LogLevel::Warn, "OVERFLOW", "Overflow test");
        queue.enqueue(message);
    }
    
    let overflow_stats = queue.stats();
    assert_eq!(overflow_stats.messages_sent, 10);
    assert_eq!(overflow_stats.messages_dropped, 6);
    assert_eq!(queue.len(), 4);
    
    // Drain queue
    let mut drained_count = 0;
    while let Some(message) = queue.dequeue() {
        let report = LogReport::from_log_message(&message);
        let recovered = report.to_log_message().unwrap();
        
        // Verify message integrity after overflow
        assert_eq!(recovered.level, LogLevel::Warn);
        assert_eq!(recovered.module_str(), "OVERFLOW");
        drained_count += 1;
    }
    
    assert_eq!(drained_count, 4);
    assert_eq!(queue.len(), 0);
    
    // Verify queue can continue normal operation after recovery
    let recovery_message = LogMessage::new(100, LogLevel::Info, "RECOVERY", "System recovered");
    assert!(queue.enqueue(recovery_message));
    assert_eq!(queue.len(), 1);
    
    let recovered_msg = queue.dequeue().unwrap();
    let report = LogReport::from_log_message(&recovered_msg);
    let final_msg = report.to_log_message().unwrap();
    
    assert_eq!(final_msg.timestamp, 100);
    assert_eq!(final_msg.level, LogLevel::Info);
    assert_eq!(final_msg.module_str(), "RECOVERY");
    assert_eq!(final_msg.message_str(), "System recovered");
}

#[test]
fn test_concurrent_queue_and_hid_operations() {
    // Test concurrent access to queue with HID report generation
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    let queue = Arc::new(Mutex::new(LogQueue::<16>::new()));
    let mut handles = vec![];
    
    // Spawn producer threads
    for thread_id in 0..4 {
        let queue_clone = Arc::clone(&queue);
        let handle = thread::spawn(move || {
            for i in 0..10 {
                let message = LogMessage::new(
                    (thread_id * 100 + i) as u32,
                    LogLevel::Info,
                    "THREAD",
                    "Concurrent test message"
                );
                queue_clone.lock().unwrap().enqueue(message);
            }
        });
        handles.push(handle);
    }
    
    // Spawn consumer thread that generates HID reports
    let queue_clone = Arc::clone(&queue);
    let consumer_handle = thread::spawn(move || {
        let mut processed_reports = 0;
        
        // Process messages as they become available
        for _ in 0..100 { // Limit iterations to prevent infinite loop
            if let Some(message) = queue_clone.lock().unwrap().dequeue() {
                let report = LogReport::from_log_message(&message);
                let recovered = report.to_log_message().unwrap();
                
                // Verify message integrity
                assert_eq!(recovered.level, LogLevel::Info);
                assert_eq!(recovered.module_str(), "THREAD");
                processed_reports += 1;
            } else {
                thread::sleep(std::time::Duration::from_millis(1));
            }
        }
        
        processed_reports
    });
    
    // Wait for all producers to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Wait for consumer and get results
    let processed_count = consumer_handle.join().unwrap();
    
    // Verify concurrent operations worked correctly
    let final_stats = queue.lock().unwrap().stats();
    assert_eq!(final_stats.messages_sent, 40); // 4 threads * 10 messages
    assert!(processed_count > 0, "No messages were processed");
    
    // Some messages might remain in queue due to timing
    let remaining = queue.lock().unwrap().len();
    assert_eq!(processed_count + remaining, std::cmp::min(40, 16)); // Account for queue capacity
}

// ============================================================================
// Integration Test Summary
// ============================================================================

#[test]
fn test_complete_usb_hid_integration_workflow() {
    // Test complete workflow: message creation -> queue -> HID report -> recovery
    let mut queue: LogQueue<8> = LogQueue::new();
    
    // Step 1: Create and queue various message types (simulating different system components)
    let system_messages = vec![
        ("BOOT", LogLevel::Info, "System initialization complete"),
        ("BATTERY", LogLevel::Warn, "Battery voltage low: 3.2V"),
        ("PEMF", LogLevel::Debug, "Pulse timing: 2.001ms"),
        ("USB", LogLevel::Error, "HID transmission timeout"),
        ("SYSTEM", LogLevel::Info, "Memory usage: 45%"),
    ];
    
    for (module, level, message) in &system_messages {
        let log_msg = LogMessage::new(mock_timestamp(), *level, module, message);
        assert!(queue.enqueue(log_msg));
    }
    
    // Step 2: Process queue and generate HID reports (simulating USB HID task)
    let mut hid_reports = Vec::new();
    while let Some(message) = queue.dequeue() {
        let report = LogReport::from_log_message(&message);
        hid_reports.push(report);
    }
    
    assert_eq!(hid_reports.len(), 5);
    
    // Step 3: Simulate host-side processing (simulating host utility)
    let mut received_messages = Vec::new();
    for report in &hid_reports {
        match report.to_log_message() {
            Ok(message) => received_messages.push(message),
            Err(_) => panic!("Failed to parse HID report"),
        }
    }
    
    // Step 4: Verify end-to-end integrity
    assert_eq!(received_messages.len(), 5);
    
    for (i, received) in received_messages.iter().enumerate() {
        let (expected_module, expected_level, expected_message) = &system_messages[i];
        
        assert_eq!(received.level, *expected_level);
        assert_eq!(received.module_str(), *expected_module);
        assert_eq!(received.message_str(), *expected_message);
        assert_eq!(received.timestamp, mock_timestamp());
    }
    
    // Step 5: Verify system statistics
    let final_stats = queue.stats();
    assert_eq!(final_stats.messages_sent, 5);
    assert_eq!(final_stats.messages_dropped, 0);
    assert_eq!(queue.len(), 0);
    
    // Step 6: Verify system can continue operating after processing
    let post_test_message = LogMessage::new(999, LogLevel::Info, "TEST", "Integration test complete");
    assert!(queue.enqueue(post_test_message));
    
    let final_message = queue.dequeue().unwrap();
    let final_report = LogReport::from_log_message(&final_message);
    let final_recovered = final_report.to_log_message().unwrap();
    
    assert_eq!(final_recovered.timestamp, 999);
    assert_eq!(final_recovered.message_str(), "Integration test complete");
}