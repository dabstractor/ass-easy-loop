//! Unit tests for the logging module
//! These tests run with std support to enable testing infrastructure

use ass_easy_loop::logging::*;

#[test]
fn test_log_level_as_str() {
    assert_eq_no_std!(LogLevel::Debug.as_str(), "DEBUG");
    assert_eq_no_std!(LogLevel::Info.as_str(), "INFO");
    assert_eq_no_std!(LogLevel::Warn.as_str(), "WARN");
    assert_eq_no_std!(LogLevel::Error.as_str(), "ERROR");
}

#[test]
fn test_log_message_creation() {
    let message = LogMessage::new(12345, LogLevel::Info, "TEST", "Hello world");
    
    assert_eq_no_std!(message.timestamp, 12345);
    assert_eq_no_std!(message.level, LogLevel::Info);
    assert_eq_no_std!(message.module_str(), "TEST");
    assert_eq_no_std!(message.message_str(), "Hello world");
}

#[test]
fn test_log_message_truncation() {
    let long_module = "VERYLONGMODULENAME";
    let long_message = "This is a very long message that should be truncated because it exceeds the maximum length";
    
    let message = LogMessage::new(0, LogLevel::Debug, long_module, long_message);
    
    assert_no_std!(message.module_str().len() <= MAX_MODULE_NAME_LENGTH);
    assert_no_std!(message.message_str().len() <= MAX_MESSAGE_LENGTH);
}

#[test]
fn test_log_queue_basic_operations() {
    let mut queue: LogQueue<4> = LogQueue::new();
    
    assert_no_std!(queue.is_empty());
    assert_no_std!(!queue.is_full());
    assert_eq_no_std!(queue.len(), 0);
    assert_eq_no_std!(queue.capacity(), 4);
    
    let message1 = LogMessage::new(1, LogLevel::Info, "TEST", "Message 1");
    assert_no_std!(queue.enqueue(message1));
    assert_eq_no_std!(queue.len(), 1);
    assert_no_std!(!queue.is_empty());
    
    let dequeued = queue.dequeue().unwrap();
    assert_eq_no_std!(dequeued.timestamp, 1);
    assert_eq_no_std!(dequeued.message_str(), "Message 1");
    assert_no_std!(queue.is_empty());
}

#[test]
fn test_log_queue_overflow_fifo_eviction() {
    let mut queue: LogQueue<2> = LogQueue::new();
    
    // Fill queue to capacity
    let msg1 = LogMessage::new(1, LogLevel::Info, "TEST", "Message 1");
    let msg2 = LogMessage::new(2, LogLevel::Info, "TEST", "Message 2");
    assert_no_std!(queue.enqueue(msg1));
    assert_no_std!(queue.enqueue(msg2));
    assert_no_std!(queue.is_full());
    assert_eq_no_std!(queue.len(), 2);
    
    // Adding another message should evict the oldest (FIFO)
    let msg3 = LogMessage::new(3, LogLevel::Info, "TEST", "Message 3");
    assert_no_std!(queue.enqueue(msg3));
    assert_eq_no_std!(queue.len(), 2); // Still full
    
    // First message should be evicted, second and third should remain
    let dequeued1 = queue.dequeue().unwrap();
    assert_eq_no_std!(dequeued1.timestamp, 2); // Message 1 was evicted
    
    let dequeued2 = queue.dequeue().unwrap();
    assert_eq_no_std!(dequeued2.timestamp, 3);
    
    assert_no_std!(queue.is_empty());
}

#[test]
fn test_log_queue_statistics_tracking() {
    let mut queue: LogQueue<3> = LogQueue::new();
    
    // Initial stats should be zero
    let initial_stats = queue.stats();
    assert_eq_no_std!(initial_stats.messages_sent, 0);
    assert_eq_no_std!(initial_stats.messages_dropped, 0);
    assert_eq_no_std!(initial_stats.peak_utilization, 0);
    assert_eq_no_std!(initial_stats.current_utilization_percent, 0);
    
    // Add messages and check stats
    let msg1 = LogMessage::new(1, LogLevel::Info, "TEST", "Message 1");
    let msg2 = LogMessage::new(2, LogLevel::Info, "TEST", "Message 2");
    let msg3 = LogMessage::new(3, LogLevel::Info, "TEST", "Message 3");
    
    queue.enqueue(msg1);
    let stats1 = queue.stats();
    assert_eq_no_std!(stats1.messages_sent, 1);
    assert_eq_no_std!(stats1.messages_dropped, 0);
    assert_eq_no_std!(stats1.peak_utilization, 1);
    assert_eq_no_std!(stats1.current_utilization_percent, 33); // 1/3 * 100
    
    queue.enqueue(msg2);
    queue.enqueue(msg3);
    let stats2 = queue.stats();
    assert_eq_no_std!(stats2.messages_sent, 3);
    assert_eq_no_std!(stats2.messages_dropped, 0);
    assert_eq_no_std!(stats2.peak_utilization, 3);
    assert_eq_no_std!(stats2.current_utilization_percent, 100); // 3/3 * 100
    
    // Overflow should increment dropped counter
    let msg4 = LogMessage::new(4, LogLevel::Info, "TEST", "Message 4");
    queue.enqueue(msg4);
    let stats3 = queue.stats();
    assert_eq_no_std!(stats3.messages_sent, 4);
    assert_eq_no_std!(stats3.messages_dropped, 1);
    assert_eq_no_std!(stats3.peak_utilization, 3);
    assert_eq_no_std!(stats3.current_utilization_percent, 100);
}

#[test]
fn test_log_queue_stats_reset() {
    let mut queue: LogQueue<2> = LogQueue::new();
    
    // Add some messages to generate stats
    let msg1 = LogMessage::new(1, LogLevel::Info, "TEST", "Message 1");
    let msg2 = LogMessage::new(2, LogLevel::Info, "TEST", "Message 2");
    let msg3 = LogMessage::new(3, LogLevel::Info, "TEST", "Message 3"); // This will cause a drop
    
    queue.enqueue(msg1);
    queue.enqueue(msg2);
    queue.enqueue(msg3);
    
    let stats_before = queue.stats();
    assert_eq_no_std!(stats_before.messages_sent, 3);
    assert_eq_no_std!(stats_before.messages_dropped, 1);
    assert_eq_no_std!(stats_before.peak_utilization, 2);
    
    // Reset stats
    queue.reset_stats();
    let stats_after = queue.stats();
    assert_eq_no_std!(stats_after.messages_sent, 0);
    assert_eq_no_std!(stats_after.messages_dropped, 0);
    assert_eq_no_std!(stats_after.peak_utilization, 2); // Should be current length
    assert_eq_no_std!(stats_after.current_utilization_percent, 100); // Queue still full
}

#[test]
fn test_log_queue_clear() {
    let mut queue: LogQueue<3> = LogQueue::new();
    
    // Fill queue
    let msg1 = LogMessage::new(1, LogLevel::Info, "TEST", "Message 1");
    let msg2 = LogMessage::new(2, LogLevel::Info, "TEST", "Message 2");
    queue.enqueue(msg1);
    queue.enqueue(msg2);
    assert_eq_no_std!(queue.len(), 2);
    
    // Clear queue
    queue.clear();
    assert_no_std!(queue.is_empty());
    assert_eq_no_std!(queue.len(), 0);
    
    // Stats should still reflect previous activity
    let stats = queue.stats();
    assert_eq_no_std!(stats.messages_sent, 2);
    assert_eq_no_std!(stats.current_utilization_percent, 0);
}

#[test]
fn test_log_queue_circular_buffer_behavior() {
    let mut queue: LogQueue<3> = LogQueue::new();
    
    // Fill and empty multiple times to test circular behavior
    for i in 0..10 {
        let msg = LogMessage::new(i, LogLevel::Info, "TEST", "Message");
        queue.enqueue(msg);
        
        if i % 2 == 1 {
            // Dequeue every other message
            queue.dequeue();
        }
    }
    
    // Should have some messages remaining
    assert_no_std!(!queue.is_empty());
    
    // Verify messages are in correct order (FIFO)
    let mut last_timestamp = 0;
    while let Some(msg) = queue.dequeue() {
        assert_no_std!(msg.timestamp > last_timestamp);
        last_timestamp = msg.timestamp;
    }
}

#[test]
fn test_queue_stats_utilization_calculation() {
    // Test edge cases for utilization calculation
    assert_eq_no_std!(QueueStats::calculate_utilization::<0>(0), 0); // Empty queue
    assert_eq_no_std!(QueueStats::calculate_utilization::<10>(0), 0); // 0/10 = 0%
    assert_eq_no_std!(QueueStats::calculate_utilization::<10>(5), 50); // 5/10 = 50%
    assert_eq_no_std!(QueueStats::calculate_utilization::<10>(10), 100); // 10/10 = 100%
    assert_eq_no_std!(QueueStats::calculate_utilization::<3>(1), 33); // 1/3 = 33%
    assert_eq_no_std!(QueueStats::calculate_utilization::<3>(2), 66); // 2/3 = 66%
}

#[test]
fn test_log_queue_dequeue_empty() {
    let mut queue: LogQueue<4> = LogQueue::new();
    
    // Dequeuing from empty queue should return None
    assert_no_std!(queue.dequeue().is_none());
    assert_no_std!(queue.dequeue().is_none());
    
    // Stats should remain zero
    let stats = queue.stats();
    assert_eq_no_std!(stats.messages_sent, 0);
    assert_eq_no_std!(stats.messages_dropped, 0);
}

#[test]
fn test_message_formatter() {
    let message = LogMessage::new(12345, LogLevel::Warn, "BATTERY", "Low voltage detected");
    let formatted = MessageFormatter::format_message(&message);
    
    let formatted_str = core::str::from_utf8(&formatted).unwrap();
    assert_no_std!(formatted_str.contains("[12345]"));
    assert_no_std!(formatted_str.contains("[WARN]"));
    assert_no_std!(formatted_str.contains("[BATTERY]"));
    assert_no_std!(formatted_str.contains("Low voltage detected"));
}

#[test]
fn test_queue_logger() {
    fn mock_timestamp() -> u32 { 42 }
    
    let mut logger: QueueLogger<8> = QueueLogger::new(mock_timestamp);
    
    logger.info("TEST", "Test message");
    logger.error("ERROR", "Error message");
    
    let queue = logger.queue();
    assert_eq_no_std!(queue.len(), 2);
    
    let msg1 = queue.dequeue().unwrap();
    assert_eq_no_std!(msg1.level, LogLevel::Info);
    assert_eq_no_std!(msg1.timestamp, 42);
    
    let msg2 = queue.dequeue().unwrap();
    assert_eq_no_std!(msg2.level, LogLevel::Error);
}

#[test]
fn test_log_message_serialization() {
    let message = LogMessage::new(0x12345678, LogLevel::Warn, "BATTERY", "Low voltage");
    let serialized = message.serialize();
    
    // Check serialized format
    assert_eq_no_std!(serialized.len(), 64);
    assert_eq_no_std!(serialized[0], LogLevel::Warn as u8); // Log level
    
    // Check module name (bytes 1-8)
    assert_eq_no_std!(&serialized[1..8], b"BATTERY");
    assert_eq_no_std!(serialized[8], 0); // Null padding
    
    // Check message content (bytes 9-56)
    assert_eq_no_std!(&serialized[9..20], b"Low voltage");
    // Rest should be null-padded
    for i in 20..57 {
        assert_eq_no_std!(serialized[i], 0);
    }
    
    // Check timestamp (bytes 57-60, little-endian)
    let timestamp_bytes = 0x12345678u32.to_le_bytes();
    assert_eq_no_std!(&serialized[57..61], &timestamp_bytes);
    
    // Check reserved bytes (61-63)
    for i in 61..64 {
        assert_eq_no_std!(serialized[i], 0);
    }
}

#[test]
fn test_log_message_deserialization() {
    // Create a test buffer with known values
    let mut buffer = [0u8; 64];
    buffer[0] = LogLevel::Error as u8;
    buffer[1..5].copy_from_slice(b"TEST");
    buffer[9..21].copy_from_slice(b"Test message");
    buffer[57..61].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());
    
    let message = LogMessage::deserialize(&buffer).unwrap();
    
    assert_eq_no_std!(message.level, LogLevel::Error);
    assert_eq_no_std!(message.timestamp, 0xDEADBEEF);
    assert_eq_no_std!(message.module_str(), "TEST");
    assert_eq_no_std!(message.message_str(), "Test message");
}

#[test]
fn test_log_message_serialization_roundtrip() {
    let original = LogMessage::new(0xABCDEF12, LogLevel::Debug, "MODULE", "Test message content");
    let serialized = original.serialize();
    let deserialized = LogMessage::deserialize(&serialized).unwrap();
    
    assert_eq_no_std!(original.timestamp, deserialized.timestamp);
    assert_eq_no_std!(original.level, deserialized.level);
    assert_eq_no_std!(original.module_str(), deserialized.module_str());
    assert_eq_no_std!(original.message_str(), deserialized.message_str());
}

#[test]
fn test_log_message_deserialization_invalid_level() {
    let mut buffer = [0u8; 64];
    buffer[0] = 255; // Invalid log level
    
    let result = LogMessage::deserialize(&buffer);
    assert_no_std!(result.is_err());
    assert_eq_no_std!(result.unwrap_err(), "Invalid log level");
}

#[test]
fn test_serialization_with_max_length_strings() {
    // Test with maximum length module and message
    let max_module = "12345678"; // Exactly 8 characters
    let max_message = &"A".repeat(48); // Exactly 48 characters
    
    let message = LogMessage::new(0, LogLevel::Info, max_module, max_message);
    let serialized = message.serialize();
    let deserialized = LogMessage::deserialize(&serialized).unwrap();
    
    assert_eq_no_std!(message.module_str(), deserialized.module_str());
    assert_eq_no_std!(message.message_str(), deserialized.message_str());
}

#[test]
fn test_serialization_with_empty_strings() {
    let message = LogMessage::new(42, LogLevel::Debug, "", "");
    let serialized = message.serialize();
    let deserialized = LogMessage::deserialize(&serialized).unwrap();
    
    assert_eq_no_std!(deserialized.timestamp, 42);
    assert_eq_no_std!(deserialized.level, LogLevel::Debug);
    assert_eq_no_std!(deserialized.module_str(), "");
    assert_eq_no_std!(deserialized.message_str(), "");
}

// Concurrent access tests - these test the thread-safety of the LogQueue
// Note: These tests use std::thread which is available in test environment

#[cfg(test)]
mod concurrent_tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_concurrent_enqueue_dequeue() {
        let queue = Arc::new(Mutex::new(LogQueue::<16>::new()));
        let mut handles = vec![];

        // Spawn multiple producer threads
        for thread_id in 0..4 {
            let queue_clone = Arc::clone(&queue);
            let handle = thread::spawn(move || {
                for i in 0..10 {
                    let msg = LogMessage::new(
                        (thread_id * 100 + i) as u32,
                        LogLevel::Info,
                        "THREAD",
                        &format!("Message from thread {}", thread_id)
                    );
                    queue_clone.lock().unwrap().enqueue(msg);
                    // Small delay to increase chance of interleaving
                    thread::sleep(Duration::from_millis(1));
                }
            });
            handles.push(handle);
        }

        // Spawn consumer threads
        let consumed_messages = Arc::new(Mutex::new(Vec::new()));
        for _ in 0..2 {
            let queue_clone = Arc::clone(&queue);
            let messages_clone = Arc::clone(&consumed_messages);
            let handle = thread::spawn(move || {
                for _ in 0..20 {
                    if let Some(msg) = queue_clone.lock().unwrap().dequeue() {
                        messages_clone.lock().unwrap().push(msg);
                    }
                    thread::sleep(Duration::from_millis(2));
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Drain any remaining messages
        let mut remaining_count = 0;
        while queue.lock().unwrap().dequeue().is_some() {
            remaining_count += 1;
        }

        let consumed_count = consumed_messages.lock().unwrap().len();
        let total_processed = consumed_count + remaining_count;

        // We should have processed most messages (some might be dropped due to overflow)
        // With a queue size of 16 and 40 messages, we expect at least the queue capacity
        assert_no_std!(total_processed >= 16); // At least the queue capacity should be processed
        
        // Check statistics
        let stats = queue.lock().unwrap().stats();
        assert_eq_no_std!(stats.messages_sent, 40); // 4 threads * 10 messages each
        // With concurrent access and queue size 16, more messages might be dropped
        assert_no_std!(stats.messages_dropped <= 24); // Allow for more drops due to concurrency
    }

    #[test]
    fn test_concurrent_statistics_consistency() {
        let queue = Arc::new(Mutex::new(LogQueue::<8>::new()));
        let mut handles = vec![];

        // Spawn threads that add messages rapidly to test statistics consistency
        for thread_id in 0..8 {
            let queue_clone = Arc::clone(&queue);
            let handle = thread::spawn(move || {
                for i in 0..25 {
                    let msg = LogMessage::new(
                        (thread_id * 1000 + i) as u32,
                        LogLevel::Debug,
                        "STATS",
                        "Statistics test message"
                    );
                    queue_clone.lock().unwrap().enqueue(msg);
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        let final_stats = queue.lock().unwrap().stats();
        
        // Should have sent exactly 200 messages (8 threads * 25 messages)
        assert_eq_no_std!(final_stats.messages_sent, 200);
        
        // With queue size 8, we should have dropped many messages
        assert_no_std!(final_stats.messages_dropped > 0);
        
        // Messages sent should equal messages in queue + messages dropped
        let queue_len = queue.lock().unwrap().len();
        assert_eq_no_std!(final_stats.messages_sent as usize, queue_len + final_stats.messages_dropped as usize);
        
        // Peak utilization should be at queue capacity
        assert_eq_no_std!(final_stats.peak_utilization, 8);
        
        // Current utilization should be 100% (queue should be full)
        assert_eq_no_std!(final_stats.current_utilization_percent, 100);
    }

    #[test]
    fn test_concurrent_queue_overflow_behavior() {
        let queue = Arc::new(Mutex::new(LogQueue::<4>::new()));
        let mut handles = vec![];

        // Create a scenario where queue will definitely overflow
        for thread_id in 0..6 {
            let queue_clone = Arc::clone(&queue);
            let handle = thread::spawn(move || {
                for i in 0..10 {
                    let msg = LogMessage::new(
                        (thread_id * 100 + i) as u32,
                        LogLevel::Warn,
                        "OVERFLOW",
                        "Testing overflow behavior"
                    );
                    queue_clone.lock().unwrap().enqueue(msg);
                    thread::sleep(Duration::from_millis(1));
                }
            });
            handles.push(handle);
        }

        // Wait for all producers to finish
        for handle in handles {
            handle.join().unwrap();
        }

        let stats = queue.lock().unwrap().stats();
        
        // Should have attempted to send 60 messages
        assert_eq_no_std!(stats.messages_sent, 60);
        
        // With queue size 4, should have dropped many messages
        assert_no_std!(stats.messages_dropped >= 56); // At least 56 should be dropped
        
        // Queue should be full
        assert_eq_no_std!(queue.lock().unwrap().len(), 4);
        assert_no_std!(queue.lock().unwrap().is_full());
        
        // Verify FIFO behavior - dequeue all messages and check they're in order
        let mut timestamps = Vec::new();
        while let Some(msg) = queue.lock().unwrap().dequeue() {
            timestamps.push(msg.timestamp);
        }
        
        // Should have exactly 4 messages
        assert_eq_no_std!(timestamps.len(), 4);
        
        // They should be the most recent ones (highest timestamps)
        timestamps.sort();
        // With concurrent access, we can't guarantee exact ordering, but the timestamps
        // should be reasonably high (from later in the sequence)
        assert_no_std!(timestamps[0] >= 40); // Should be among the later messages sent
    }

    #[test]
    fn test_concurrent_mixed_operations() {
        let queue = Arc::new(Mutex::new(LogQueue::<12>::new()));
        let mut handles = vec![];

        // Mixed workload: some threads enqueue, some dequeue, some do both
        
        // Pure producers
        for thread_id in 0..2 {
            let queue_clone = Arc::clone(&queue);
            let handle = thread::spawn(move || {
                for i in 0..15 {
                    let msg = LogMessage::new(
                        (thread_id * 1000 + i) as u32,
                        LogLevel::Info,
                        "PRODUCER",
                        "Producer message"
                    );
                    queue_clone.lock().unwrap().enqueue(msg);
                    thread::sleep(Duration::from_millis(1));
                }
            });
            handles.push(handle);
        }

        // Pure consumers
        let consumed_count = Arc::new(Mutex::new(0));
        for _ in 0..2 {
            let queue_clone = Arc::clone(&queue);
            let count_clone = Arc::clone(&consumed_count);
            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    if queue_clone.lock().unwrap().dequeue().is_some() {
                        *count_clone.lock().unwrap() += 1;
                    }
                    thread::sleep(Duration::from_millis(2));
                }
            });
            handles.push(handle);
        }

        // Mixed producer-consumer
        for thread_id in 0..2 {
            let queue_clone = Arc::clone(&queue);
            let handle = thread::spawn(move || {
                for i in 0..10 {
                    // Enqueue a message
                    let msg = LogMessage::new(
                        (thread_id * 2000 + i) as u32,
                        LogLevel::Debug,
                        "MIXED",
                        "Mixed operation message"
                    );
                    queue_clone.lock().unwrap().enqueue(msg);
                    
                    // Sometimes dequeue
                    if i % 3 == 0 {
                        queue_clone.lock().unwrap().dequeue();
                    }
                    
                    thread::sleep(Duration::from_millis(1));
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        let final_stats = queue.lock().unwrap().stats();
        let final_consumed = *consumed_count.lock().unwrap();
        
        // Should have sent 50 messages total (2*15 + 2*10)
        assert_eq_no_std!(final_stats.messages_sent, 50);
        
        // Some messages should have been consumed
        assert_no_std!(final_consumed > 0);
        
        // Statistics should be consistent
        let queue_len = queue.lock().unwrap().len();
        let total_processed = final_consumed + queue_len + final_stats.messages_dropped as usize;
        assert_no_std!(total_processed <= final_stats.messages_sent as usize);
    }

    #[test]
    fn test_atomic_operations_consistency() {
        // This test verifies that atomic operations maintain consistency
        // even under high contention
        let queue = Arc::new(Mutex::new(LogQueue::<6>::new()));
        let mut handles = vec![];

        // High-frequency operations to stress test atomics
        for thread_id in 0..10 {
            let queue_clone = Arc::clone(&queue);
            let handle = thread::spawn(move || {
                for i in 0..100 {
                    let msg = LogMessage::new(
                        (thread_id * 10000 + i) as u32,
                        LogLevel::Error,
                        "ATOMIC",
                        "Atomic test message"
                    );
                    
                    // Rapid enqueue/dequeue to test atomic consistency
                    queue_clone.lock().unwrap().enqueue(msg);
                    
                    if i % 2 == 0 {
                        queue_clone.lock().unwrap().dequeue();
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for completion
        for handle in handles {
            handle.join().unwrap();
        }

        let final_stats = queue.lock().unwrap().stats();
        
        // Should have attempted 1000 enqueues
        assert_eq_no_std!(final_stats.messages_sent, 1000);
        
        // Verify internal consistency
        let queue_len = queue.lock().unwrap().len();
        assert_no_std!(queue_len <= 6); // Should not exceed capacity
        
        // Total messages processed should be consistent
        let total_accounted = queue_len + final_stats.messages_dropped as usize;
        // Note: We can't easily count dequeued messages in this test,
        // but we can verify that the queue state is consistent
        assert_no_std!(total_accounted <= final_stats.messages_sent as usize);
    }
}