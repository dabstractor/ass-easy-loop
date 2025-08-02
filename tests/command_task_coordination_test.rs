//! Unit tests for command task coordination
//! Tests the integration between USB polling and command processing tasks
//! Requirements: 2.2, 8.1, 8.2, 8.3

#![cfg(test)]

extern crate std;
use std::vec::Vec;

use ass_easy_loop::command::{
    CommandReport, ParseResult, CommandQueue, CommandParser, ResponseQueue, 
    AuthenticationValidator, TestCommand, ErrorCode
};

    /// Test command queue operations for RTIC task coordination
    #[test]
    fn test_command_queue_fifo_order() {
        let mut queue = CommandQueue::<4>::new();
        let timestamp = 1000;
        let timeout = 5000;

        // Create test commands
        let cmd1 = CommandReport::new(TestCommand::SystemStateQuery as u8, 1, &[0x01]).unwrap();
        let cmd2 = CommandReport::new(TestCommand::ExecuteTest as u8, 2, &[0x02]).unwrap();
        let cmd3 = CommandReport::new(TestCommand::ConfigurationQuery as u8, 3, &[0x03]).unwrap();

        // Enqueue commands
        assert!(queue.enqueue(cmd1.clone(), timestamp, timeout));
        assert!(queue.enqueue(cmd2.clone(), timestamp + 1, timeout));
        assert!(queue.enqueue(cmd3.clone(), timestamp + 2, timeout));

        // Verify FIFO order
        let dequeued1 = queue.dequeue().unwrap();
        assert_eq!(dequeued1.command.command_id, 1);
        assert_eq!(dequeued1.command.command_type, TestCommand::SystemStateQuery as u8);

        let dequeued2 = queue.dequeue().unwrap();
        assert_eq!(dequeued2.command.command_id, 2);
        assert_eq!(dequeued2.command.command_type, TestCommand::ExecuteTest as u8);

        let dequeued3 = queue.dequeue().unwrap();
        assert_eq!(dequeued3.command.command_id, 3);
        assert_eq!(dequeued3.command.command_type, TestCommand::ConfigurationQuery as u8);

        // Queue should be empty
        assert!(queue.dequeue().is_none());
    }

    /// Test command timeout handling for task coordination
    #[test]
    fn test_command_timeout_handling() {
        let mut queue = CommandQueue::<4>::new();
        let base_timestamp = 1000;
        let short_timeout = 100;
        let long_timeout = 5000;

        // Create commands with different timeouts
        let cmd1 = CommandReport::new(TestCommand::SystemStateQuery as u8, 1, &[0x01]).unwrap();
        let cmd2 = CommandReport::new(TestCommand::ExecuteTest as u8, 2, &[0x02]).unwrap();

        // Enqueue commands
        assert!(queue.enqueue(cmd1, base_timestamp, short_timeout));
        assert!(queue.enqueue(cmd2, base_timestamp, long_timeout));

        assert_eq!(queue.len(), 2);

        // Simulate time passing beyond short timeout
        let current_time = base_timestamp + short_timeout + 50;
        let removed_count = queue.remove_timed_out_commands(current_time);

        // Only the first command should be timed out
        assert_eq!(removed_count, 1);
        assert_eq!(queue.len(), 1);

        // Remaining command should be the second one
        let remaining = queue.dequeue().unwrap();
        assert_eq!(remaining.command.command_id, 2);
    }

    /// Test command authentication for secure task coordination
    #[test]
    fn test_command_authentication() {
        // Create a valid command
        let payload = [0xAA, 0xBB, 0xCC];
        let command = CommandReport::new(TestCommand::ExecuteTest as u8, 42, &payload).unwrap();

        // Verify authentication passes
        assert!(AuthenticationValidator::validate_command(&command));
        assert!(AuthenticationValidator::validate_format(&command).is_ok());

        // Test with corrupted checksum
        let mut corrupted_command = command.clone();
        corrupted_command.auth_token = corrupted_command.auth_token.wrapping_add(1);
        assert!(!AuthenticationValidator::validate_command(&corrupted_command));

        // Test with invalid command type
        let mut invalid_command = command.clone();
        invalid_command.command_type = 0x00; // Invalid command type
        assert_eq!(
            AuthenticationValidator::validate_format(&invalid_command),
            Err(ErrorCode::UnsupportedCommand)
        );
    }

    /// Test response queue operations for task coordination
    #[test]
    fn test_response_queue_operations() {
        let mut queue = ResponseQueue::<4>::new();
        let timestamp = 2000;

        // Create test responses
        let resp1 = CommandReport::success_response(1, &[0x01, 0x02]).unwrap();
        let resp2 = CommandReport::success_response(2, &[0x03, 0x04]).unwrap();

        // Test enqueueing
        assert!(queue.enqueue(resp1.clone(), 100, timestamp));
        assert!(queue.enqueue(resp2.clone(), 101, timestamp + 1));

        assert_eq!(queue.len(), 2);

        // Test dequeuing
        let dequeued1 = queue.dequeue().unwrap();
        assert_eq!(dequeued1.response.command_id, 1);
        assert_eq!(dequeued1.sequence_number, 100);

        let dequeued2 = queue.dequeue().unwrap();
        assert_eq!(dequeued2.response.command_id, 2);
        assert_eq!(dequeued2.sequence_number, 101);

        // Queue should be empty
        assert!(queue.dequeue().is_none());
    }

    /// Test command parsing for USB HID integration
    #[test]
    fn test_usb_command_parsing() {
        // Create a valid 64-byte USB HID report
        let mut report_buffer = [0u8; 64];
        report_buffer[0] = TestCommand::SystemStateQuery as u8; // Command type
        report_buffer[1] = 123; // Command ID
        report_buffer[2] = 3; // Payload length
        report_buffer[4] = 0xAA; // Payload
        report_buffer[5] = 0xBB;
        report_buffer[6] = 0xCC;
        
        // Calculate checksum
        let checksum = report_buffer[0] ^ report_buffer[1] ^ report_buffer[2] ^ 
                      report_buffer[4] ^ report_buffer[5] ^ report_buffer[6];
        report_buffer[3] = checksum;

        // Test parsing
        match CommandReport::parse(&report_buffer) {
            ParseResult::Valid(command) => {
                assert_eq!(command.command_type, TestCommand::SystemStateQuery as u8);
                assert_eq!(command.command_id, 123);
                assert_eq!(command.payload_length, 3);
                assert_eq!(command.payload.as_slice(), &[0xAA, 0xBB, 0xCC]);
                assert!(AuthenticationValidator::validate_command(&command));
            }
            _ => panic!("Command parsing failed"),
        }
    }

    /// Test command queue capacity limits for resource management
    #[test]
    fn test_command_queue_capacity_limits() {
        let mut queue = CommandQueue::<2>::new(); // Small queue for testing
        let timestamp = 3000;
        let timeout = 5000;

        // Fill the queue to capacity
        let cmd1 = CommandReport::new(TestCommand::SystemStateQuery as u8, 1, &[0x01]).unwrap();
        let cmd2 = CommandReport::new(TestCommand::ExecuteTest as u8, 2, &[0x02]).unwrap();
        let cmd3 = CommandReport::new(TestCommand::ConfigurationQuery as u8, 3, &[0x03]).unwrap();

        assert!(queue.enqueue(cmd1, timestamp, timeout));
        assert!(queue.enqueue(cmd2, timestamp + 1, timeout));
        
        // Queue should be full
        assert!(queue.is_full());
        assert_eq!(queue.len(), 2);

        // Attempting to enqueue another command should fail
        assert!(!queue.enqueue(cmd3, timestamp + 2, timeout));
        
        // Dropped count should increase
        assert_eq!(queue.dropped_count(), 1);
    }

    /// Test task priority coordination by verifying command processing doesn't block
    #[test]
    fn test_task_priority_simulation() {
        let mut command_queue = CommandQueue::<8>::new();
        let mut response_queue = ResponseQueue::<8>::new();
        let timestamp = 4000;

        // Simulate high-priority task creating commands
        for i in 0..5 {
            let cmd = CommandReport::new(
                TestCommand::PerformanceMetrics as u8, 
                i, 
                &[i as u8]
            ).unwrap();
            assert!(command_queue.enqueue(cmd, timestamp + i as u32, 5000));
        }

        // Simulate medium-priority command processing task
        let mut processed_count = 0;
        while let Some(queued_cmd) = command_queue.dequeue() {
            // Process command (simulate work)
            let response = CommandReport::success_response(
                queued_cmd.command.command_id,
                &[0xFF, queued_cmd.command.command_id]
            ).unwrap();

            // Queue response
            assert!(response_queue.enqueue(
                response, 
                queued_cmd.sequence_number, 
                timestamp + processed_count + 100
            ));
            
            processed_count += 1;
        }

        // Verify all commands were processed
        assert_eq!(processed_count, 5);
        assert_eq!(response_queue.len(), 5);
        assert_eq!(command_queue.len(), 0);
    }

    /// Test error handling in command processing coordination
    #[test]
    fn test_command_error_handling() {
        // Test invalid command format
        let mut invalid_buffer = [0u8; 64];
        invalid_buffer[0] = 0xFF; // Invalid command type
        invalid_buffer[1] = 1;
        invalid_buffer[2] = 0;
        invalid_buffer[3] = 0xFF ^ 1 ^ 0; // Correct checksum for invalid command

        match CommandReport::parse(&invalid_buffer) {
            ParseResult::Valid(command) => {
                // Command parses but should fail validation
                assert_eq!(
                    AuthenticationValidator::validate_format(&command),
                    Err(ErrorCode::UnsupportedCommand)
                );
            }
            _ => panic!("Expected valid parse result for format validation test"),
        }

        // Test buffer too short
        let short_buffer = [0u8; 32]; // Too short
        match CommandReport::parse(&short_buffer) {
            ParseResult::BufferTooShort => {
                // Expected result
            }
            _ => panic!("Expected BufferTooShort result"),
        }

        // Test invalid checksum
        let mut checksum_buffer = [0u8; 64];
        checksum_buffer[0] = TestCommand::SystemStateQuery as u8;
        checksum_buffer[1] = 1;
        checksum_buffer[2] = 0;
        checksum_buffer[3] = 0xFF; // Wrong checksum

        match CommandReport::parse(&checksum_buffer) {
            ParseResult::InvalidChecksum => {
                // Expected result
            }
            _ => panic!("Expected InvalidChecksum result"),
        }
    }
}