//! Command parsing module for 64-byte HID report format
//! Implements command validation and authentication with simple checksum
//! Requirements: 2.1, 2.2, 6.1, 6.2

use heapless::{Vec, spsc::Queue};
use portable_atomic::{AtomicUsize, Ordering};
use core::option::Option::{self, Some, None};
use core::result::Result::{self, Ok, Err};
use core::iter::Iterator;

/// Standardized 64-byte HID report format for command handling
/// Format: [Command Type:1][Command ID:1][Payload Length:1][Auth Token:1][Payload:60]
#[derive(Clone, Debug, PartialEq)]
pub struct CommandReport {
    /// Command type (0x80-0xFF for test commands)
    pub command_type: u8,
    /// Command ID (sequence number for tracking)
    pub command_id: u8,
    /// Payload length (0-60 bytes)
    pub payload_length: u8,
    /// Authentication token (simple checksum for validation)
    pub auth_token: u8,
    /// Command payload (command-specific data, up to 60 bytes)
    pub payload: Vec<u8, 60>,
}

/// Command parsing result
#[derive(Debug, PartialEq)]
pub enum ParseResult {
    Valid(CommandReport),
    InvalidChecksum,
    InvalidFormat,
    BufferTooShort,
}

/// Test command types
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum TestCommand {
    EnterBootloader = 0x80,
    SystemStateQuery = 0x81,
    ExecuteTest = 0x82,
    ConfigurationQuery = 0x83,
    PerformanceMetrics = 0x84,
    RunTestSuite = 0x85,
    GetTestResults = 0x86,
    ClearTestResults = 0x87,
}

/// Test response types
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum TestResponse {
    Ack = 0x90,
    StateData = 0x91,
    TestResult = 0x92,
    Error = 0x93,
    SuiteSummary = 0x94,
    StatusUpdate = 0x95,
    BatchStart = 0x96,
    BatchEnd = 0x97,
}

/// Response status codes
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
#[allow(dead_code)]
pub enum ResponseStatus {
    Success = 0x00,
    InvalidCommand = 0x01,
    AuthenticationFailed = 0x02,
    CommandTimeout = 0x03,
    SystemBusy = 0x04,
    HardwareError = 0x05,
}

/// Error codes for command processing
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum ErrorCode {
    InvalidFormat = 0x10,
    InvalidChecksum = 0x11,
    UnsupportedCommand = 0x12,
    PayloadTooLarge = 0x13,
    SystemNotReady = 0x14,
}

impl CommandReport {
    /// Create a new command report
    pub fn new(command_type: u8, command_id: u8, payload: &[u8]) -> Result<Self, ErrorCode> {
        if payload.len() > 60 {
            return Err(ErrorCode::PayloadTooLarge);
        }

        let mut payload_vec = Vec::new();
        for &byte in payload {
            payload_vec.push(byte).map_err(|_| ErrorCode::PayloadTooLarge)?;
        }

        let payload_length = payload.len() as u8;
        let auth_token = Self::calculate_checksum(command_type, command_id, payload_length, payload);

        Ok(Self {
            command_type,
            command_id,
            payload_length,
            auth_token,
            payload: payload_vec,
        })
    }

    /// Parse a 64-byte HID report into a CommandReport
    pub fn parse(buffer: &[u8]) -> ParseResult {
        // Verify buffer length
        if buffer.len() < 64 {
            return ParseResult::BufferTooShort;
        }

        // Extract header fields
        let command_type = buffer[0];
        let command_id = buffer[1];
        let payload_length = buffer[2];
        let auth_token = buffer[3];

        // Validate payload length
        if payload_length > 60 {
            return ParseResult::InvalidFormat;
        }

        // Extract payload
        let mut payload = Vec::new();
        for i in 0..payload_length as usize {
            if payload.push(buffer[4 + i]).is_err() {
                return ParseResult::InvalidFormat;
            }
        }

        // Validate checksum
        let expected_checksum = Self::calculate_checksum(command_type, command_id, payload_length, &payload);
        if auth_token != expected_checksum {
            return ParseResult::InvalidChecksum;
        }

        ParseResult::Valid(CommandReport {
            command_type,
            command_id,
            payload_length,
            auth_token,
            payload,
        })
    }

    /// Serialize command report to 64-byte buffer
    pub fn serialize(&self) -> [u8; 64] {
        let mut buffer = [0u8; 64];

        // Write header
        buffer[0] = self.command_type;
        buffer[1] = self.command_id;
        buffer[2] = self.payload_length;
        buffer[3] = self.auth_token;

        // Write payload
        for (i, &byte) in self.payload.iter().enumerate() {
            buffer[4 + i] = byte;
        }

        // Remaining bytes are already zeroed
        buffer
    }

    /// Calculate simple checksum for authentication
    /// Uses XOR of all header bytes and payload bytes
    fn calculate_checksum(command_type: u8, command_id: u8, payload_length: u8, payload: &[u8]) -> u8 {
        let mut checksum = command_type ^ command_id ^ payload_length;
        for &byte in payload {
            checksum ^= byte;
        }
        checksum
    }

    /// Create a success response
    pub fn success_response(command_id: u8, data: &[u8]) -> Result<Self, ErrorCode> {
        Self::new(TestResponse::Ack as u8, command_id, data)
    }

    /// Create an error response
    pub fn error_response(command_id: u8, error_code: ErrorCode, message: &str) -> Result<Self, ErrorCode> {
        let mut payload: Vec<u8, 60> = Vec::new();
        payload.push(error_code as u8).map_err(|_| ErrorCode::PayloadTooLarge)?;
        
        // Add error message (truncated to fit)
        let message_bytes = message.as_bytes();
        let max_message_len = core::cmp::min(message_bytes.len(), 59); // Reserve 1 byte for error code
        for &byte in &message_bytes[..max_message_len] {
            if payload.push(byte).is_err() {
                break;
            }
        }

        Self::new(TestResponse::Error as u8, command_id, &payload)
    }

    /// Get command type as TestCommand enum
    pub fn get_test_command(&self) -> Option<TestCommand> {
        match self.command_type {
            0x80 => Some(TestCommand::EnterBootloader),
            0x81 => Some(TestCommand::SystemStateQuery),
            0x82 => Some(TestCommand::ExecuteTest),
            0x83 => Some(TestCommand::ConfigurationQuery),
            0x84 => Some(TestCommand::PerformanceMetrics),
            0x85 => Some(TestCommand::RunTestSuite),
            0x86 => Some(TestCommand::GetTestResults),
            0x87 => Some(TestCommand::ClearTestResults),
            _ => None,
        }
    }

    /// Check if command is valid test command
    pub fn is_valid_test_command(&self) -> bool {
        self.command_type >= 0x80 && self.command_type <= 0x87
    }
}

/// Command with sequence tracking and timeout information
#[derive(Clone, Debug, PartialEq)]
pub struct QueuedCommand {
    pub command: CommandReport,
    pub sequence_number: u32,
    pub timestamp_ms: u32,
    pub timeout_ms: u32,
}

impl QueuedCommand {
    /// Create a new queued command with sequence tracking
    pub fn new(command: CommandReport, sequence_number: u32, timestamp_ms: u32, timeout_ms: u32) -> Self {
        Self {
            command,
            sequence_number,
            timestamp_ms,
            timeout_ms,
        }
    }

    /// Check if command has timed out
    pub fn is_timed_out(&self, current_time_ms: u32) -> bool {
        current_time_ms.saturating_sub(self.timestamp_ms) > self.timeout_ms
    }

    /// Get remaining timeout in milliseconds
    pub fn remaining_timeout_ms(&self, current_time_ms: u32) -> u32 {
        let elapsed = current_time_ms.saturating_sub(self.timestamp_ms);
        self.timeout_ms.saturating_sub(elapsed)
    }
}

/// Thread-safe command queue for storing incoming commands with sequence tracking
/// Requirements: 2.4 (FIFO order), 6.4 (timeout handling)
pub struct CommandQueue<const N: usize> {
    queue: Queue<QueuedCommand, N>,
    dropped_commands: AtomicUsize,
    sequence_counter: AtomicUsize,
    timeout_count: AtomicUsize,
}

impl<const N: usize> Default for CommandQueue<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> CommandQueue<N> {
    /// Create a new command queue
    pub const fn new() -> Self {
        Self {
            queue: Queue::new(),
            dropped_commands: AtomicUsize::new(0),
            sequence_counter: AtomicUsize::new(0),
            timeout_count: AtomicUsize::new(0),
        }
    }

    /// Enqueue a command with sequence tracking and timeout
    /// Requirements: 2.4 (FIFO order maintained by underlying queue)
    pub fn enqueue(&mut self, command: CommandReport, timestamp_ms: u32, timeout_ms: u32) -> bool {
        let sequence_number = self.sequence_counter.fetch_add(1, Ordering::Relaxed) as u32;
        let queued_command = QueuedCommand::new(command, sequence_number, timestamp_ms, timeout_ms);
        
        match self.queue.enqueue(queued_command) {
            Ok(()) => true,
            Err(_) => {
                self.dropped_commands.fetch_add(1, Ordering::Relaxed);
                false
            }
        }
    }

    /// Dequeue a command (FIFO order)
    /// Requirements: 2.4 (commands executed in FIFO order)
    pub fn dequeue(&mut self) -> Option<QueuedCommand> {
        self.queue.dequeue()
    }

    /// Peek at the next command without removing it
    pub fn peek(&self) -> Option<&QueuedCommand> {
        self.queue.peek()
    }

    /// Remove timed out commands from the queue
    /// Requirements: 6.4 (timeout handling)
    pub fn remove_timed_out_commands(&mut self, current_time_ms: u32) -> usize {
        let mut removed_count = 0;
        let mut temp_commands: Vec<QueuedCommand, N> = Vec::new();
        
        // Drain all commands and filter out timed out ones
        while let Some(queued_cmd) = self.queue.dequeue() {
            if queued_cmd.is_timed_out(current_time_ms) {
                removed_count += 1;
                self.timeout_count.fetch_add(1, Ordering::Relaxed);
            } else {
                // Keep non-timed-out commands
                if temp_commands.push(queued_cmd).is_err() {
                    // If we can't store it, we have to drop it
                    self.dropped_commands.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
        
        // Re-enqueue non-timed-out commands (maintains FIFO order)
        for cmd in temp_commands {
            if self.queue.enqueue(cmd).is_err() {
                // Queue is somehow full, count as dropped
                self.dropped_commands.fetch_add(1, Ordering::Relaxed);
            }
        }
        
        removed_count
    }

    /// Get current queue length
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Get queue capacity
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Check if queue is full
    pub fn is_full(&self) -> bool {
        self.queue.is_full()
    }

    /// Get number of dropped commands
    pub fn dropped_count(&self) -> usize {
        self.dropped_commands.load(Ordering::Relaxed)
    }

    /// Get number of timed out commands
    pub fn timeout_count(&self) -> usize {
        self.timeout_count.load(Ordering::Relaxed)
    }

    /// Get current sequence number
    pub fn current_sequence(&self) -> u32 {
        self.sequence_counter.load(Ordering::Relaxed) as u32
    }

    /// Reset all statistics
    pub fn reset_stats(&mut self) {
        self.dropped_commands.store(0, Ordering::Relaxed);
        self.timeout_count.store(0, Ordering::Relaxed);
    }
}

/// Command parser for validating and processing incoming commands
pub struct CommandParser {
    processed_commands: AtomicUsize,
    validation_failures: AtomicUsize,
}

impl Default for CommandParser {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandParser {
    /// Create a new command parser
    pub const fn new() -> Self {
        Self {
            processed_commands: AtomicUsize::new(0),
            validation_failures: AtomicUsize::new(0),
        }
    }

    /// Parse and validate a command from raw buffer
    pub fn parse_command(&self, buffer: &[u8]) -> ParseResult {
        let result = CommandReport::parse(buffer);
        
        match &result {
            ParseResult::Valid(_) => {
                self.processed_commands.fetch_add(1, Ordering::Relaxed);
            }
            _ => {
                self.validation_failures.fetch_add(1, Ordering::Relaxed);
            }
        }

        result
    }

    /// Get number of successfully processed commands
    pub fn processed_count(&self) -> usize {
        self.processed_commands.load(Ordering::Relaxed)
    }

    /// Get number of validation failures
    pub fn validation_failure_count(&self) -> usize {
        self.validation_failures.load(Ordering::Relaxed)
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        self.processed_commands.store(0, Ordering::Relaxed);
        self.validation_failures.store(0, Ordering::Relaxed);
    }
}

/// Authentication validator for command security
pub struct AuthenticationValidator;

impl AuthenticationValidator {
    /// Validate command authentication token
    pub fn validate_command(command: &CommandReport) -> bool {
        let expected_checksum = CommandReport::calculate_checksum(
            command.command_type,
            command.command_id,
            command.payload_length,
            &command.payload,
        );
        command.auth_token == expected_checksum
    }

    /// Validate command format and structure
    pub fn validate_format(command: &CommandReport) -> Result<(), ErrorCode> {
        // Check if command type is in valid range
        if !command.is_valid_test_command() {
            return Err(ErrorCode::UnsupportedCommand);
        }

        // Check payload length consistency
        if command.payload.len() != command.payload_length as usize {
            return Err(ErrorCode::InvalidFormat);
        }

        // Check payload length bounds
        if command.payload_length > 60 {
            return Err(ErrorCode::PayloadTooLarge);
        }

        Ok(())
    }
}

/// Response with sequence tracking for command results
#[derive(Clone, Debug, PartialEq)]
pub struct QueuedResponse {
    pub response: CommandReport,
    pub sequence_number: u32,
    pub timestamp_ms: u32,
    pub retry_count: u8,
}

impl QueuedResponse {
    /// Create a new queued response
    pub fn new(response: CommandReport, sequence_number: u32, timestamp_ms: u32) -> Self {
        Self {
            response,
            sequence_number,
            timestamp_ms,
            retry_count: 0,
        }
    }

    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count = self.retry_count.saturating_add(1);
    }

    /// Check if maximum retries exceeded
    pub fn max_retries_exceeded(&self, max_retries: u8) -> bool {
        self.retry_count >= max_retries
    }
}

/// Thread-safe response queue for sending command results back to host
/// Requirements: 2.5 (error responses with diagnostic information)
pub struct ResponseQueue<const N: usize> {
    queue: Queue<QueuedResponse, N>,
    dropped_responses: AtomicUsize,
    transmission_failures: AtomicUsize,
}

impl<const N: usize> Default for ResponseQueue<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> ResponseQueue<N> {
    /// Create a new response queue
    pub const fn new() -> Self {
        Self {
            queue: Queue::new(),
            dropped_responses: AtomicUsize::new(0),
            transmission_failures: AtomicUsize::new(0),
        }
    }

    /// Enqueue a response for transmission
    pub fn enqueue(&mut self, response: CommandReport, sequence_number: u32, timestamp_ms: u32) -> bool {
        let queued_response = QueuedResponse::new(response, sequence_number, timestamp_ms);
        
        match self.queue.enqueue(queued_response) {
            Ok(()) => true,
            Err(_) => {
                self.dropped_responses.fetch_add(1, Ordering::Relaxed);
                false
            }
        }
    }

    /// Dequeue a response for transmission
    pub fn dequeue(&mut self) -> Option<QueuedResponse> {
        self.queue.dequeue()
    }

    /// Peek at the next response without removing it
    pub fn peek(&self) -> Option<&QueuedResponse> {
        self.queue.peek()
    }

    /// Re-enqueue a response for retry (e.g., after transmission failure)
    pub fn requeue_for_retry(&mut self, mut response: QueuedResponse, max_retries: u8) -> bool {
        response.increment_retry();
        
        if response.max_retries_exceeded(max_retries) {
            self.transmission_failures.fetch_add(1, Ordering::Relaxed);
            false
        } else {
            match self.queue.enqueue(response) {
                Ok(()) => true,
                Err(_) => {
                    self.dropped_responses.fetch_add(1, Ordering::Relaxed);
                    false
                }
            }
        }
    }

    /// Get current queue length
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Get queue capacity
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Check if queue is full
    pub fn is_full(&self) -> bool {
        self.queue.is_full()
    }

    /// Get number of dropped responses
    pub fn dropped_count(&self) -> usize {
        self.dropped_responses.load(Ordering::Relaxed)
    }

    /// Get number of transmission failures
    pub fn transmission_failure_count(&self) -> usize {
        self.transmission_failures.load(Ordering::Relaxed)
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.dropped_responses.store(0, Ordering::Relaxed);
        self.transmission_failures.store(0, Ordering::Relaxed);
    }
}

/// Initialize command handler (placeholder for future initialization)
#[allow(dead_code)]
pub fn init_command_handler(_queue: &mut CommandQueue<8>) {
    // Command handler initialization
    // Currently no additional setup required
}
