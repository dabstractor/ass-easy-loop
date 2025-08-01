//! USB HID Command Processing Module
//! 
//! This module provides command parsing, validation, and authentication for
//! automated testing and bootloader control via USB HID output reports.
//! 
//! Requirements: 2.1, 2.2, 6.1, 6.2

use heapless::Vec;
use core::fmt::Write;

/// Standard HID report size for commands (64 bytes)
pub const COMMAND_REPORT_SIZE: usize = 64;

/// Command type identifiers
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum CommandType {
    /// Enter bootloader mode
    EnterBootloader = 0x80,
    /// System state query
    SystemStateQuery = 0x81,
    /// Execute test command
    ExecuteTest = 0x82,
    /// Configuration query
    ConfigurationQuery = 0x83,
    /// Performance metrics query
    PerformanceMetrics = 0x84,
}

impl CommandType {
    /// Convert from u8 to CommandType
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x80 => Some(CommandType::EnterBootloader),
            0x81 => Some(CommandType::SystemStateQuery),
            0x82 => Some(CommandType::ExecuteTest),
            0x83 => Some(CommandType::ConfigurationQuery),
            0x84 => Some(CommandType::PerformanceMetrics),
            _ => None,
        }
    }
}

/// Command validation errors
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CommandError {
    /// Invalid command format
    InvalidFormat,
    /// Unknown command type
    UnknownCommandType,
    /// Authentication failed
    AuthenticationFailed,
    /// Invalid payload length
    InvalidPayloadLength,
    /// Invalid parameters
    InvalidParameters,
}

impl CommandError {
    /// Get error message as string
    pub fn as_str(&self) -> &'static str {
        match self {
            CommandError::InvalidFormat => "Invalid command format",
            CommandError::UnknownCommandType => "Unknown command type",
            CommandError::AuthenticationFailed => "Authentication failed",
            CommandError::InvalidPayloadLength => "Invalid payload length",
            CommandError::InvalidParameters => "Invalid parameters",
        }
    }
}

/// Parsed command structure
#[derive(Clone, Debug)]
pub struct Command {
    /// Command type
    pub command_type: CommandType,
    /// Command sequence ID for tracking
    pub command_id: u8,
    /// Payload data
    pub payload: Vec<u8, 61>, // Max 61 bytes payload (64 - 3 header bytes)
}

impl Command {
    /// Create a new command
    pub fn new(command_type: CommandType, command_id: u8, payload: &[u8]) -> Result<Self, CommandError> {
        if payload.len() > 61 {
            return Err(CommandError::InvalidPayloadLength);
        }
        
        let mut payload_vec = Vec::new();
        for &byte in payload {
            payload_vec.push(byte).map_err(|_| CommandError::InvalidPayloadLength)?;
        }
        
        Ok(Self {
            command_type,
            command_id,
            payload: payload_vec,
        })
    }
    
    /// Get payload as slice
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

/// Command response structure
#[derive(Clone, Debug)]
pub struct CommandResponse {
    /// Original command ID for correlation
    pub command_id: u8,
    /// Response status
    pub status: ResponseStatus,
    /// Response data
    pub data: Vec<u8, 60>, // Max 60 bytes data (64 - 4 header bytes)
}

/// Response status codes
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ResponseStatus {
    /// Command executed successfully
    Success = 0x00,
    /// Command failed with error
    Error = 0x01,
    /// Command in progress
    InProgress = 0x02,
    /// Command not supported
    NotSupported = 0x03,
}

impl ResponseStatus {
    /// Convert from u8 to ResponseStatus
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(ResponseStatus::Success),
            0x01 => Some(ResponseStatus::Error),
            0x02 => Some(ResponseStatus::InProgress),
            0x03 => Some(ResponseStatus::NotSupported),
            _ => None,
        }
    }
}

impl CommandResponse {
    /// Create a new command response
    pub fn new(command_id: u8, status: ResponseStatus, data: &[u8]) -> Result<Self, CommandError> {
        if data.len() > 60 {
            return Err(CommandError::InvalidPayloadLength);
        }
        
        let mut data_vec = Vec::new();
        for &byte in data {
            data_vec.push(byte).map_err(|_| CommandError::InvalidPayloadLength)?;
        }
        
        Ok(Self {
            command_id,
            status,
            data: data_vec,
        })
    }
    
    /// Create success response
    pub fn success(command_id: u8, data: &[u8]) -> Result<Self, CommandError> {
        Self::new(command_id, ResponseStatus::Success, data)
    }
    
    /// Create error response
    pub fn error(command_id: u8, error_msg: &str) -> Result<Self, CommandError> {
        let error_bytes = error_msg.as_bytes();
        let truncated_len = core::cmp::min(error_bytes.len(), 60);
        Self::new(command_id, ResponseStatus::Error, &error_bytes[..truncated_len])
    }
    
    /// Serialize response to HID report format
    pub fn serialize(&self) -> [u8; 64] {
        let mut buffer = [0u8; 64];
        
        // Byte 0: Response marker (0xFF to distinguish from log messages)
        buffer[0] = 0xFF;
        // Byte 1: Command ID
        buffer[1] = self.command_id;
        // Byte 2: Status
        buffer[2] = self.status as u8;
        // Byte 3: Data length
        buffer[3] = self.data.len() as u8;
        // Bytes 4-63: Response data
        for (i, &byte) in self.data.iter().enumerate() {
            if i < 60 {
                buffer[4 + i] = byte;
            }
        }
        
        buffer
    }
}

/// Authentication validator for command security
pub struct AuthenticationValidator {
    /// Expected magic value for simple authentication
    magic_value: u8,
}

impl AuthenticationValidator {
    /// Create new authentication validator
    pub const fn new() -> Self {
        Self {
            magic_value: 0x42, // Simple magic value for basic authentication
        }
    }
    
    /// Validate command authentication using simple checksum
    /// Requirements: 6.1, 6.2
    pub fn validate_command(&self, raw_command: &[u8; 64]) -> bool {
        if raw_command.len() < 4 {
            return false;
        }
        
        // Simple checksum validation:
        // auth_token = (command_type + command_id + payload_length) ^ magic_value
        let command_type = raw_command[0];
        let command_id = raw_command[1];
        let payload_length = raw_command[2];
        let auth_token = raw_command[3];
        
        let expected_token = (command_type.wrapping_add(command_id).wrapping_add(payload_length)) ^ self.magic_value;
        
        auth_token == expected_token
    }
    
    /// Generate authentication token for a command
    pub fn generate_auth_token(&self, command_type: u8, command_id: u8, payload_length: u8) -> u8 {
        (command_type.wrapping_add(command_id).wrapping_add(payload_length)) ^ self.magic_value
    }
}

/// Command parser for processing USB HID output reports
pub struct CommandParser {
    /// Authentication validator
    auth_validator: AuthenticationValidator,
}

impl CommandParser {
    /// Create new command parser
    pub const fn new() -> Self {
        Self {
            auth_validator: AuthenticationValidator::new(),
        }
    }
    
    /// Parse raw HID report into command structure
    /// Requirements: 2.1, 2.2, 6.1, 6.2
    pub fn parse_command(&self, raw_report: &[u8; 64]) -> Result<Command, CommandError> {
        // Validate report format
        if raw_report.len() != COMMAND_REPORT_SIZE {
            return Err(CommandError::InvalidFormat);
        }
        
        // Extract header fields
        let command_type_byte = raw_report[0];
        let command_id = raw_report[1];
        let payload_length = raw_report[2];
        
        // Validate payload length
        if payload_length > 61 {
            return Err(CommandError::InvalidPayloadLength);
        }
        
        // Validate authentication
        if !self.auth_validator.validate_command(raw_report) {
            return Err(CommandError::AuthenticationFailed);
        }
        
        // Parse command type
        let command_type = CommandType::from_u8(command_type_byte)
            .ok_or(CommandError::UnknownCommandType)?;
        
        // Extract payload
        let payload_end = 4 + payload_length as usize;
        if payload_end > 64 {
            return Err(CommandError::InvalidPayloadLength);
        }
        
        let payload_slice = &raw_report[4..payload_end];
        
        // Create command
        Command::new(command_type, command_id, payload_slice)
    }
    
    /// Validate command parameters based on command type
    pub fn validate_command_parameters(&self, command: &Command) -> Result<(), CommandError> {
        match command.command_type {
            CommandType::EnterBootloader => {
                // Bootloader command should have 4-byte timeout parameter
                if command.payload.len() != 4 {
                    return Err(CommandError::InvalidParameters);
                }
                
                // Extract timeout value (little-endian u32)
                let timeout_bytes = [
                    command.payload[0],
                    command.payload[1],
                    command.payload[2],
                    command.payload[3],
                ];
                let timeout_ms = u32::from_le_bytes(timeout_bytes);
                
                // Validate timeout range (100ms to 30 seconds)
                if timeout_ms < 100 || timeout_ms > 30000 {
                    return Err(CommandError::InvalidParameters);
                }
            }
            
            CommandType::SystemStateQuery => {
                // State query should have 1-byte query type parameter
                if command.payload.len() != 1 {
                    return Err(CommandError::InvalidParameters);
                }
                
                // Validate query type (0-4 are valid)
                let query_type = command.payload[0];
                if query_type > 4 {
                    return Err(CommandError::InvalidParameters);
                }
            }
            
            CommandType::ExecuteTest => {
                // Test command should have at least 1 byte for test type
                if command.payload.is_empty() {
                    return Err(CommandError::InvalidParameters);
                }
                
                // Validate test type (0-4 are valid)
                let test_type = command.payload[0];
                if test_type > 4 {
                    return Err(CommandError::InvalidParameters);
                }
            }
            
            CommandType::ConfigurationQuery => {
                // Config query should have 1-byte config type parameter
                if command.payload.len() != 1 {
                    return Err(CommandError::InvalidParameters);
                }
                
                // Validate config type (0-3 are valid)
                let config_type = command.payload[0];
                if config_type > 3 {
                    return Err(CommandError::InvalidParameters);
                }
            }
            
            CommandType::PerformanceMetrics => {
                // Performance metrics should have 1-byte metric type parameter
                if command.payload.len() != 1 {
                    return Err(CommandError::InvalidParameters);
                }
                
                // Validate metric type (0-3 are valid)
                let metric_type = command.payload[0];
                if metric_type > 3 {
                    return Err(CommandError::InvalidParameters);
                }
            }
        }
        
        Ok(())
    }
    
    /// Create a formatted command for transmission (for testing purposes)
    pub fn create_command_report(&self, command_type: CommandType, command_id: u8, payload: &[u8]) -> Result<[u8; 64], CommandError> {
        if payload.len() > 61 {
            return Err(CommandError::InvalidPayloadLength);
        }
        
        let mut report = [0u8; 64];
        
        // Header
        report[0] = command_type as u8;
        report[1] = command_id;
        report[2] = payload.len() as u8;
        
        // Generate authentication token
        report[3] = self.auth_validator.generate_auth_token(
            command_type as u8,
            command_id,
            payload.len() as u8
        );
        
        // Payload
        for (i, &byte) in payload.iter().enumerate() {
            report[4 + i] = byte;
        }
        
        Ok(report)
    }
}

/// Command queue for storing parsed commands
pub struct CommandQueue<const N: usize> {
    /// Internal command storage
    commands: [Option<Command>; N],
    /// Head pointer for circular buffer
    head: usize,
    /// Tail pointer for circular buffer
    tail: usize,
    /// Current count of commands
    count: usize,
}

impl<const N: usize> CommandQueue<N> {
    /// Create new command queue
    pub const fn new() -> Self {
        Self {
            commands: [const { None }; N],
            head: 0,
            tail: 0,
            count: 0,
        }
    }
    
    /// Enqueue a command
    pub fn enqueue(&mut self, command: Command) -> bool {
        if self.count >= N {
            return false; // Queue full
        }
        
        self.commands[self.head] = Some(command);
        self.head = (self.head + 1) % N;
        self.count += 1;
        
        true
    }
    
    /// Dequeue a command
    pub fn dequeue(&mut self) -> Option<Command> {
        if self.count == 0 {
            return None; // Queue empty
        }
        
        let command = self.commands[self.tail].take();
        self.tail = (self.tail + 1) % N;
        self.count -= 1;
        
        command
    }
    
    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    
    /// Check if queue is full
    pub fn is_full(&self) -> bool {
        self.count >= N
    }
    
    /// Get current queue length
    pub fn len(&self) -> usize {
        self.count
    }
    
    /// Get queue capacity
    pub const fn capacity(&self) -> usize {
        N
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_type_conversion() {
        assert_eq!(CommandType::from_u8(0x80), Some(CommandType::EnterBootloader));
        assert_eq!(CommandType::from_u8(0x81), Some(CommandType::SystemStateQuery));
        assert_eq!(CommandType::from_u8(0xFF), None);
    }

    #[test]
    fn test_authentication_validator() {
        let validator = AuthenticationValidator::new();
        
        // Test token generation
        let token = validator.generate_auth_token(0x80, 0x01, 0x04);
        assert_ne!(token, 0); // Should generate non-zero token
        
        // Test validation with correct token
        let mut report = [0u8; 64];
        report[0] = 0x80; // command_type
        report[1] = 0x01; // command_id
        report[2] = 0x04; // payload_length
        report[3] = token; // auth_token
        
        assert!(validator.validate_command(&report));
        
        // Test validation with incorrect token
        report[3] = token ^ 0xFF; // Corrupt the token
        assert!(!validator.validate_command(&report));
    }

    #[test]
    fn test_command_parser() {
        let parser = CommandParser::new();
        
        // Create valid bootloader command
        let bootloader_payload = [100u8, 0, 0, 0]; // 100ms timeout
        let report = parser.create_command_report(
            CommandType::EnterBootloader,
            0x01,
            &bootloader_payload
        ).unwrap();
        
        // Parse the command
        let parsed_command = parser.parse_command(&report).unwrap();
        assert_eq!(parsed_command.command_type, CommandType::EnterBootloader);
        assert_eq!(parsed_command.command_id, 0x01);
        assert_eq!(parsed_command.payload.len(), 4);
        
        // Validate parameters
        assert!(parser.validate_command_parameters(&parsed_command).is_ok());
    }

    #[test]
    fn test_command_queue() {
        let mut queue = CommandQueue::<4>::new();
        
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
        
        // Create test command
        let command = Command::new(CommandType::SystemStateQuery, 1, &[0]).unwrap();
        
        // Test enqueue
        assert!(queue.enqueue(command.clone()));
        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());
        
        // Test dequeue
        let dequeued = queue.dequeue().unwrap();
        assert_eq!(dequeued.command_type, CommandType::SystemStateQuery);
        assert_eq!(dequeued.command_id, 1);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_command_response() {
        // Test success response
        let response = CommandResponse::success(0x01, b"OK").unwrap();
        assert_eq!(response.command_id, 0x01);
        assert_eq!(response.status, ResponseStatus::Success);
        
        // Test serialization
        let serialized = response.serialize();
        assert_eq!(serialized[0], 0xFF); // Response marker
        assert_eq!(serialized[1], 0x01); // Command ID
        assert_eq!(serialized[2], ResponseStatus::Success as u8);
        assert_eq!(serialized[3], 2); // Data length
        assert_eq!(serialized[4], b'O');
        assert_eq!(serialized[5], b'K');
        
        // Test error response
        let error_response = CommandResponse::error(0x02, "Test error").unwrap();
        assert_eq!(error_response.status, ResponseStatus::Error);
    }
}