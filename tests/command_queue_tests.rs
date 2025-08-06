//! Unit tests for command queue and response system
//! Tests the enhanced command queuing with sequence tracking and timeout handling
//! Requirements: 2.4 (FIFO order), 2.5 (error responses), 6.4 (timeout handling)

#![cfg(test)]

use ass_easy_loop::command::{
    CommandReport, CommandQueue, ResponseQueue, TestCommand, ErrorCode, AuthenticationValidator
};

/// Test basic command queue FIFO ordering
/// Requirements: 2.4 (commands executed in FIFO order)
#[test]
fn test_command_queue_fifo_order() {
    let mut queue = CommandQueue::<4>::new();
    
    // Create test commands with different IDs
    let cmd1 = CommandReport::new(TestCommand::SystemStateQuery as u8, 1, &[0x01]).unwrap();
    let cmd2 = CommandReport::new(TestCommand::ExecuteTest as u8, 2, &[0x02]).unwrap();
    let cmd3 = CommandReport::new(TestCommand::ConfigurationQuery as u8, 3, &[0x03]).unwrap();
    
    // Enqueue commands with timestamps
    let base_time = 1000;
    let timeout = 5000;
    
    assert_no_std!(queue.enqueue(cmd1, base_time, timeout));
    assert_no_std!(queue.enqueue(cmd2, base_time + 100, timeout));
    assert_no_std!(queue.enqueue(cmd3, base_time + 200, timeout));
    
    // Verify FIFO order
    let dequeued1 = queue.dequeue().unwrap();
    assert_eq_no_std!(dequeued1.command.command_id, 1);
    assert_eq_no_std!(dequeued1.sequence_number, 0); // First command gets sequence 0
    
    let dequeued2 = queue.dequeue().unwrap();
    assert_eq_no_std!(dequeued2.command.command_id, 2);
    assert_eq_no_std!(dequeued2.sequence_number, 1); // Second command gets sequence 1
    
    let dequeued3 = queue.dequeue().unwrap();
    assert_eq_no_std!(dequeued3.command.command_id, 3);
    assert_eq_no_std!(dequeued3.sequence_number, 2); // Third command gets sequence 2
    
    // Queue should be empty
    assert_no_std!(queue.dequeue().is_none());
}

/// Test command queue capacity and overflow handling
#[test]
fn test_command_queue_overflow() {
    let mut queue = CommandQueue::<2>::new(); // Small queue for testing overflow
    
    let cmd1 = CommandReport::new(TestCommand::SystemStateQuery as u8, 1, &[]).unwrap();
    let cmd2 = CommandReport::new(TestCommand::ExecuteTest as u8, 2, &[]).unwrap();
    let cmd3 = CommandReport::new(TestCommand::ConfigurationQuery as u8, 3, &[]).unwrap();
    
    let base_time = 1000;
    let timeout = 5000;
    
    // Fill the queue
    assert_no_std!(queue.enqueue(cmd1, base_time, timeout));
    assert_no_std!(queue.enqueue(cmd2, base_time, timeout));
    
    // Verify queue is full
    assert_eq_no_std!(queue.len(), 2);
    assert_no_std!(queue.is_full());
    
    // Try to add one more (should fail and increment dropped count)
    assert_no_std!(!queue.enqueue(cmd3, base_time, timeout));
    assert_eq_no_std!(queue.dropped_count(), 1);
    
    // Queue length should remain the same
    assert_eq_no_std!(queue.len(), 2);
}

/// Test command timeout detection and removal
/// Requirements: 6.4 (timeout handling)
#[test]
fn test_command_timeout_handling() {
    let mut queue = CommandQueue::<4>::new();
    
    let cmd1 = CommandReport::new(TestCommand::SystemStateQuery as u8, 1, &[]).unwrap();
    let cmd2 = CommandReport::new(TestCommand::ExecuteTest as u8, 2, &[]).unwrap();
    let cmd3 = CommandReport::new(TestCommand::ConfigurationQuery as u8, 3, &[]).unwrap();
    
    let base_time = 1000;
    let short_timeout = 100; // 100ms timeout
    let long_timeout = 5000; // 5s timeout
    
    // Enqueue commands with different timeouts
    assert_no_std!(queue.enqueue(cmd1, base_time, short_timeout)); // Will timeout
    assert_no_std!(queue.enqueue(cmd2, base_time, long_timeout));  // Won't timeout
    assert_no_std!(queue.enqueue(cmd3, base_time, short_timeout)); // Will timeout
    
    assert_eq_no_std!(queue.len(), 3);
    
    // Simulate time passing beyond short timeout
    let current_time = base_time + 200; // 200ms later
    
    // Remove timed out commands
    let removed_count = queue.remove_timed_out_commands(current_time);
    assert_eq_no_std!(removed_count, 2); // cmd1 and cmd3 should be removed
    assert_eq_no_std!(queue.len(), 1); // Only cmd2 should remain
    assert_eq_no_std!(queue.timeout_count(), 2);
    
    // Verify the remaining command is cmd2
    let remaining = queue.dequeue().unwrap();
    assert_eq_no_std!(remaining.command.command_id, 2);
}

/// Test response queue basic functionality
#[test]
fn test_response_queue_basic() {
    let mut queue = ResponseQueue::<4>::new();
    
    // Create test responses
    let resp1 = CommandReport::success_response(1, &[0x01]).unwrap();
    let resp2 = CommandReport::success_response(2, &[0x02]).unwrap();
    
    let base_time = 1000;
    
    // Enqueue responses
    assert_no_std!(queue.enqueue(resp1, 0, base_time));
    assert_no_std!(queue.enqueue(resp2, 1, base_time + 100));
    
    assert_eq_no_std!(queue.len(), 2);
    assert_no_std!(!queue.is_empty());
    
    // Dequeue responses (FIFO order)
    let dequeued1 = queue.dequeue().unwrap();
    assert_eq_no_std!(dequeued1.response.command_id, 1);
    assert_eq_no_std!(dequeued1.sequence_number, 0);
    assert_eq_no_std!(dequeued1.retry_count, 0);
    
    let dequeued2 = queue.dequeue().unwrap();
    assert_eq_no_std!(dequeued2.response.command_id, 2);
    assert_eq_no_std!(dequeued2.sequence_number, 1);
    
    // Queue should be empty
    assert_no_std!(queue.dequeue().is_none());
    assert_no_std!(queue.is_empty());
}

/// Test response queue retry mechanism
#[test]
fn test_response_queue_retry() {
    let mut queue = ResponseQueue::<4>::new();
    
    let resp = CommandReport::success_response(1, &[0x01]).unwrap();
    let base_time = 1000;
    
    // Enqueue response
    assert_no_std!(queue.enqueue(resp, 0, base_time));
    
    // Dequeue for transmission
    let queued_resp = queue.dequeue().unwrap();
    assert_eq_no_std!(queued_resp.retry_count, 0);
    
    // Simulate transmission failure and retry
    assert_no_std!(queue.requeue_for_retry(queued_resp, 3)); // Max 3 retries
    
    // Dequeue again
    let queued_resp = queue.dequeue().unwrap();
    assert_eq_no_std!(queued_resp.retry_count, 1);
    
    // Retry again
    assert_no_std!(queue.requeue_for_retry(queued_resp, 3));
    let queued_resp = queue.dequeue().unwrap();
    assert_eq_no_std!(queued_resp.retry_count, 2);
    
    // One more retry
    assert_no_std!(queue.requeue_for_retry(queued_resp, 3));
    let queued_resp = queue.dequeue().unwrap();
    assert_eq_no_std!(queued_resp.retry_count, 3);
    
    // Should fail to retry (max retries exceeded)
    assert_no_std!(!queue.requeue_for_retry(queued_resp, 3));
    assert_eq_no_std!(queue.transmission_failure_count(), 1);
}

/// Test command authentication and validation
/// Requirements: 6.4 (command validation)
#[test]
fn test_command_authentication() {
    // Test valid command creation and validation
    let cmd = CommandReport::new(TestCommand::SystemStateQuery as u8, 1, &[0x01, 0x02]).unwrap();
    assert_no_std!(AuthenticationValidator::validate_command(&cmd));
    assert_no_std!(AuthenticationValidator::validate_format(&cmd).is_ok());
    
    // Test invalid command type
    let invalid_type_cmd = CommandReport::new(0x70, 1, &[]).unwrap(); // Invalid command type
    assert_eq_no_std!(AuthenticationValidator::validate_format(&invalid_type_cmd), 
               Err(ErrorCode::UnsupportedCommand));
}

/// Test error response creation
/// Requirements: 2.5 (error responses with diagnostic information)
#[test]
fn test_error_response_creation() {
    let error_resp = CommandReport::error_response(
        42, 
        ErrorCode::InvalidFormat, 
        "Test error message"
    ).unwrap();
    
    assert_eq_no_std!(error_resp.command_type, 0x93); // TestResponse::Error as u8
    assert_eq_no_std!(error_resp.command_id, 42);
    assert_no_std!(error_resp.payload.len() > 1); // Should contain error code + message
    assert_eq_no_std!(error_resp.payload[0], ErrorCode::InvalidFormat as u8);
    
    // Verify authentication
    assert_no_std!(AuthenticationValidator::validate_command(&error_resp));
}

/// Test command sequence tracking
#[test]
fn test_command_sequence_tracking() {
    let mut queue = CommandQueue::<4>::new();
    
    let cmd1 = CommandReport::new(TestCommand::SystemStateQuery as u8, 1, &[]).unwrap();
    let cmd2 = CommandReport::new(TestCommand::ExecuteTest as u8, 2, &[]).unwrap();
    
    let base_time = 1000;
    let timeout = 5000;
    
    // Initial sequence should be 0
    assert_eq_no_std!(queue.current_sequence(), 0);
    
    // Enqueue commands and verify sequence increments
    assert_no_std!(queue.enqueue(cmd1, base_time, timeout));
    assert_eq_no_std!(queue.current_sequence(), 1);
    
    assert_no_std!(queue.enqueue(cmd2, base_time, timeout));
    assert_eq_no_std!(queue.current_sequence(), 2);
    
    // Dequeue and verify sequence numbers
    let queued1 = queue.dequeue().unwrap();
    assert_eq_no_std!(queued1.sequence_number, 0);
    
    let queued2 = queue.dequeue().unwrap();
    assert_eq_no_std!(queued2.sequence_number, 1);
}

/// Test queue statistics and monitoring
#[test]
fn test_queue_statistics() {
    let mut cmd_queue = CommandQueue::<2>::new();
    let mut resp_queue = ResponseQueue::<2>::new();
    
    let cmd = CommandReport::new(TestCommand::SystemStateQuery as u8, 1, &[]).unwrap();
    let resp = CommandReport::success_response(1, &[]).unwrap();
    
    let base_time = 1000;
    let timeout = 100; // Short timeout for testing
    
    // Test command queue statistics
    assert_eq_no_std!(cmd_queue.len(), 0);
    assert_eq_no_std!(cmd_queue.dropped_count(), 0);
    assert_eq_no_std!(cmd_queue.timeout_count(), 0);
    
    // Add commands
    assert_no_std!(cmd_queue.enqueue(cmd, base_time, timeout));
    assert_no_std!(cmd_queue.enqueue(cmd, base_time, timeout));
    assert_eq_no_std!(cmd_queue.len(), 2);
    
    // Overflow
    assert_no_std!(!cmd_queue.enqueue(cmd, base_time, timeout));
    assert_eq_no_std!(cmd_queue.dropped_count(), 1);
    
    // Timeout
    let removed = cmd_queue.remove_timed_out_commands(base_time + 200);
    assert_eq_no_std!(removed, 2);
    assert_eq_no_std!(cmd_queue.timeout_count(), 2);
    
    // Test response queue statistics
    assert_eq_no_std!(resp_queue.len(), 0);
    assert_eq_no_std!(resp_queue.dropped_count(), 0);
    assert_eq_no_std!(resp_queue.transmission_failure_count(), 0);
    
    // Add responses
    assert_no_std!(resp_queue.enqueue(resp, 0, base_time));
    assert_no_std!(resp_queue.enqueue(resp, 1, base_time));
    assert_eq_no_std!(resp_queue.len(), 2);
    
    // Overflow
    assert_no_std!(!resp_queue.enqueue(resp, 2, base_time));
    assert_eq_no_std!(resp_queue.dropped_count(), 1);
    
    // Test retry failure
    let queued_resp = resp_queue.dequeue().unwrap();
    assert_no_std!(!resp_queue.requeue_for_retry(queued_resp, 0)); // Max retries = 0
    assert_eq_no_std!(resp_queue.transmission_failure_count(), 1);
    
    // Reset statistics
    cmd_queue.reset_stats();
    resp_queue.reset_stats();
    
    assert_eq_no_std!(cmd_queue.dropped_count(), 0);
    assert_eq_no_std!(cmd_queue.timeout_count(), 0);
    assert_eq_no_std!(resp_queue.dropped_count(), 0);
    assert_eq_no_std!(resp_queue.transmission_failure_count(), 0);
}