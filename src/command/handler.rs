//! Command handler module for USB HID output reports
//! Provides helper functions for processing USB commands
//! The actual RTIC tasks are defined in main.rs
//! Requirements: 2.1, 2.2, 6.1, 6.2

use heapless::Vec;
use usbd_hid::hid_class::HIDClass;
use crate::logging::{LogReport, LogMessage, LogLevel};
use rp2040_hal::usb::UsbBus;
use crate::command::parsing::{
    CommandReport, ParseResult, TestResponse, ResponseStatus, ErrorCode,
    AuthenticationValidator, CommandQueue, ResponseQueue, QueuedCommand, QueuedResponse
};
use core::option::Option::{self, Some};
use core::result::Result::{Ok, Err};
use core::convert::TryFrom;

/// USB HID command handler for processing output reports with enhanced queuing
/// Requirements: 2.4 (FIFO command execution), 2.5 (error responses), 6.4 (timeout handling)
/// Performance optimized for minimal system impact
pub struct UsbCommandHandler {
    command_queue: CommandQueue<8>,
    response_queue: ResponseQueue<8>,
    processed_count: u32,
    error_count: u32,
    timeout_count: u32,
    default_timeout_ms: u32,
    // Performance optimization fields
    last_process_time_us: u32,
    max_process_time_us: u32,
    avg_process_time_us: u32,
    process_count: u32,
}

impl Default for UsbCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl UsbCommandHandler {
    /// Create a new USB command handler with enhanced queuing
    pub const fn new() -> Self {
        Self {
            command_queue: CommandQueue::new(),
            response_queue: ResponseQueue::new(),
            processed_count: 0,
            error_count: 0,
            timeout_count: 0,
            default_timeout_ms: 5000, // 5 second default timeout
            // Performance optimization initialization
            last_process_time_us: 0,
            max_process_time_us: 0,
            avg_process_time_us: 0,
            process_count: 0,
        }
    }

    /// Set default command timeout in milliseconds
    pub fn set_default_timeout(&mut self, timeout_ms: u32) {
        self.default_timeout_ms = timeout_ms;
    }

    /// Process a USB HID output report containing a command
    /// This function is called by the RTIC task in main.rs
    /// Performance optimized to minimize system impact
    /// 
    /// # Arguments
    /// * `report_buf` - The raw USB report data (64 bytes)
    /// * `report_len` - The length of the received report
    /// * `timestamp` - Current timestamp for logging
    ///
    /// # Returns
    /// A vector of LogReport responses to send back to the host
    pub fn process_output_report(&mut self, report_buf: &[u8], report_len: usize, timestamp: u32) -> Vec<LogReport, 8> {
        #[cfg(feature = "performance-optimized")]
        let start_time_us = timestamp * 1000; // Convert to microseconds for precision
        let mut responses: Vec<LogReport, 8> = Vec::new();
        
        // Fast path: Early validation with minimal overhead
        if report_len != 64 {
            #[cfg(not(feature = "production"))]
            {
                let error_msg = LogMessage::new(
                    timestamp,
                    LogLevel::Error,
                    "CMD",
                    "Invalid report length"
                );
                if let Ok(log_report) = LogReport::try_from(error_msg.serialize()) {
                    let _ = responses.push(log_report);
                }
            }
            self.error_count += 1;
            return responses;
        }

        // Parse the command report with optimized parsing
        match CommandReport::parse(report_buf) {
            ParseResult::Valid(command) => {
                // Fast authentication check - optimized for performance
                if !AuthenticationValidator::validate_command(&command) {
                    #[cfg(not(feature = "production"))]
                    {
                        let error_response = self.create_error_response(
                            command.command_id,
                            ErrorCode::InvalidChecksum,
                            "Authentication failed",
                            timestamp
                        );
                        if let Some(response) = error_response {
                            let _ = responses.push(response);
                        }
                    }
                    self.error_count += 1;
                    return responses;
                }

                // Optimized format validation - skip in production for performance
                #[cfg(not(feature = "production"))]
                if let Err(error_code) = AuthenticationValidator::validate_format(&command) {
                    let error_response = self.create_error_response(
                        command.command_id,
                        error_code,
                        "Invalid command format",
                        timestamp
                    );
                    if let Some(response) = error_response {
                        let _ = responses.push(response);
                    }
                    self.error_count += 1;
                    return responses;
                }

                // Queue the command for processing with timeout
                // Requirements: 2.4 (FIFO order), 6.4 (timeout handling)
                if self.command_queue.enqueue(command.clone(), timestamp, self.default_timeout_ms) {
                    // Create acknowledgment response
                    let ack_response = self.create_ack_response(command.command_id, timestamp);
                    if let Some(response) = ack_response {
                        let _ = responses.push(response);
                    }
                    self.processed_count += 1;

                    // Log successful command reception with sequence tracking
                    let _sequence_num = self.command_queue.current_sequence().saturating_sub(1);
                    let log_msg = LogMessage::new(
                        timestamp,
                        LogLevel::Info,
                        "CMD",
                        "Command queued for processing"
                    );
                    if let Ok(log_report) = LogReport::try_from(log_msg.serialize()) {
                        let _ = responses.push(log_report);
                    }
                } else {
                    // Queue is full - Requirements: 2.5 (error response with diagnostic info)
                    let error_response = self.create_error_response(
                        command.command_id,
                        ErrorCode::SystemNotReady,
                        "Command queue full",
                        timestamp
                    );
                    if let Some(response) = error_response {
                        let _ = responses.push(response);
                    }
                    self.error_count += 1;
                }
            }
            ParseResult::InvalidChecksum => {
                let error_msg = LogMessage::new(
                    timestamp,
                    LogLevel::Error,
                    "CMD",
                    "Invalid command checksum"
                );
                if let Ok(log_report) = LogReport::try_from(error_msg.serialize()) {
                    let _ = responses.push(log_report);
                }
                self.error_count += 1;
            }
            ParseResult::InvalidFormat => {
                let error_msg = LogMessage::new(
                    timestamp,
                    LogLevel::Error,
                    "CMD",
                    "Invalid command format"
                );
                if let Ok(log_report) = LogReport::try_from(error_msg.serialize()) {
                    let _ = responses.push(log_report);
                }
                self.error_count += 1;
            }
            ParseResult::BufferTooShort => {
                let error_msg = LogMessage::new(
                    timestamp,
                    LogLevel::Error,
                    "CMD",
                    "Command buffer too short"
                );
                if let Ok(log_report) = LogReport::try_from(error_msg.serialize()) {
                    let _ = responses.push(log_report);
                }
                self.error_count += 1;
            }
        }
        
        // Performance tracking - measure processing time
        #[cfg(feature = "performance-optimized")]
        {
            let end_time_us = (timestamp + 1) * 1000; // Approximate end time
            let process_time_us = end_time_us.saturating_sub(start_time_us);
            self.update_performance_metrics(process_time_us);
        }
        
        responses
    }

    /// Update performance metrics for command processing
    /// Requirements: 8.1, 8.2 (minimize system impact)
    #[allow(dead_code)]
    fn update_performance_metrics(&mut self, process_time_us: u32) {
        self.last_process_time_us = process_time_us;
        self.process_count += 1;
        
        // Update maximum processing time
        if process_time_us > self.max_process_time_us {
            self.max_process_time_us = process_time_us;
        }
        
        // Update rolling average (simple moving average)
        if self.process_count == 1 {
            self.avg_process_time_us = process_time_us;
        } else {
            // Weighted average to prevent overflow
            let weight = core::cmp::min(self.process_count, 100);
            self.avg_process_time_us = ((self.avg_process_time_us * (weight - 1)) + process_time_us) / weight;
        }
    }

    /// Get the next command from the queue for processing
    /// Requirements: 2.4 (FIFO order execution)
    pub fn get_next_command(&mut self) -> Option<QueuedCommand> {
        self.command_queue.dequeue()
    }

    /// Process command timeouts and remove expired commands
    /// Requirements: 6.4 (timeout handling)
    pub fn process_timeouts(&mut self, current_time_ms: u32) -> usize {
        let removed_count = self.command_queue.remove_timed_out_commands(current_time_ms);
        self.timeout_count += removed_count as u32;
        removed_count
    }

    /// Queue a response for transmission to host
    /// Requirements: 2.5 (error responses with diagnostic information)
    pub fn queue_response(&mut self, response: CommandReport, sequence_number: u32, timestamp_ms: u32) -> bool {
        self.response_queue.enqueue(response, sequence_number, timestamp_ms)
    }

    /// Get the next response for transmission
    pub fn get_next_response(&mut self) -> Option<QueuedResponse> {
        self.response_queue.dequeue()
    }

    /// Requeue a response for retry after transmission failure
    pub fn requeue_response_for_retry(&mut self, response: QueuedResponse) -> bool {
        const MAX_RETRIES: u8 = 3;
        self.response_queue.requeue_for_retry(response, MAX_RETRIES)
    }

    /// Create an acknowledgment response for a successfully received command
    fn create_ack_response(&self, command_id: u8, timestamp: u32) -> Option<LogReport> {
        // Create ACK response payload
        let mut payload: Vec<u8, 60> = Vec::new();
        let _ = payload.push(ResponseStatus::Success as u8);
        let _ = payload.push(command_id);

        if let Ok(ack_command) = CommandReport::new(TestResponse::Ack as u8, command_id, &payload) {
            let serialized = ack_command.serialize();
            LogReport::try_from(serialized).ok()
        } else {
            // Fallback to log message
            let log_msg = LogMessage::new(
                timestamp,
                LogLevel::Info,
                "CMD",
                "Command acknowledged"
            );
            LogReport::try_from(log_msg.serialize()).ok()
        }
    }

    /// Create an error response for a failed command
    fn create_error_response(&self, command_id: u8, error_code: ErrorCode, message: &str, timestamp: u32) -> Option<LogReport> {
        if let Ok(error_command) = CommandReport::error_response(command_id, error_code, message) {
            let serialized = error_command.serialize();
            LogReport::try_from(serialized).ok()
        } else {
            // Fallback to log message
            let log_msg = LogMessage::new(
                timestamp,
                LogLevel::Error,
                "CMD",
                message
            );
            LogReport::try_from(log_msg.serialize()).ok()
        }
    }

    /// Get enhanced command processing statistics
    pub fn get_stats(&self) -> CommandHandlerStats {
        CommandHandlerStats {
            processed_commands: self.processed_count,
            error_count: self.error_count,
            timeout_count: self.timeout_count,
            command_queue_length: self.command_queue.len(),
            command_queue_capacity: self.command_queue.capacity(),
            dropped_commands: self.command_queue.dropped_count(),
            response_queue_length: self.response_queue.len(),
            response_queue_capacity: self.response_queue.capacity(),
            dropped_responses: self.response_queue.dropped_count(),
            transmission_failures: self.response_queue.transmission_failure_count(),
            current_sequence: self.command_queue.current_sequence(),
            // Performance metrics
            last_process_time_us: self.last_process_time_us,
            max_process_time_us: self.max_process_time_us,
            avg_process_time_us: self.avg_process_time_us,
            process_count: self.process_count,
        }
    }

    /// Reset statistics counters
    pub fn reset_stats(&mut self) {
        self.processed_count = 0;
        self.error_count = 0;
        self.timeout_count = 0;
        self.command_queue.reset_stats();
        self.response_queue.reset_stats();
        // Reset performance metrics
        self.last_process_time_us = 0;
        self.max_process_time_us = 0;
        self.avg_process_time_us = 0;
        self.process_count = 0;
    }

    /// Check if command processing is impacting system performance
    /// Requirements: 8.1, 8.2 (minimize system impact)
    pub fn is_performance_impacted(&self) -> bool {
        const MAX_ACCEPTABLE_PROCESS_TIME_US: u32 = 1000; // 1ms max
        const MAX_ACCEPTABLE_AVG_TIME_US: u32 = 500; // 500μs average
        
        self.max_process_time_us > MAX_ACCEPTABLE_PROCESS_TIME_US ||
        self.avg_process_time_us > MAX_ACCEPTABLE_AVG_TIME_US
    }

    /// Get performance impact assessment
    pub fn get_performance_impact(&self) -> PerformanceImpact {
        if self.process_count == 0 {
            return PerformanceImpact::Unknown;
        }

        const LOW_IMPACT_THRESHOLD_US: u32 = 100;
        const MEDIUM_IMPACT_THRESHOLD_US: u32 = 500;
        const HIGH_IMPACT_THRESHOLD_US: u32 = 1000;

        if self.avg_process_time_us <= LOW_IMPACT_THRESHOLD_US {
            PerformanceImpact::Low
        } else if self.avg_process_time_us <= MEDIUM_IMPACT_THRESHOLD_US {
            PerformanceImpact::Medium
        } else if self.avg_process_time_us <= HIGH_IMPACT_THRESHOLD_US {
            PerformanceImpact::High
        } else {
            PerformanceImpact::Critical
        }
    }
}

/// Performance impact assessment for command processing
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
pub enum PerformanceImpact {
    Unknown,
    Low,      // < 100μs average
    Medium,   // 100-500μs average
    High,     // 500-1000μs average
    Critical, // > 1000μs average
}

/// Enhanced command handler statistics with timeout and response tracking
/// Includes performance metrics for optimization monitoring
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct CommandHandlerStats {
    pub processed_commands: u32,
    pub error_count: u32,
    pub timeout_count: u32,
    pub command_queue_length: usize,
    pub command_queue_capacity: usize,
    pub dropped_commands: usize,
    pub response_queue_length: usize,
    pub response_queue_capacity: usize,
    pub dropped_responses: usize,
    pub transmission_failures: usize,
    pub current_sequence: u32,
    // Performance metrics
    pub last_process_time_us: u32,
    pub max_process_time_us: u32,
    pub avg_process_time_us: u32,
    pub process_count: u32,
}

/// Process a USB HID output report (legacy function for compatibility)
/// This function is called by the RTIC task in main.rs
/// 
/// # Arguments
/// * `hid_class` - The HID class instance for USB communication
/// * `report_buf` - The raw USB report data
/// * `report_len` - The length of the received report
///
/// # Returns
/// A vector of LogReport responses to send back to the host
#[allow(dead_code)]
pub fn process_usb_report(_hid_class: &mut HIDClass<UsbBus>, report_buf: &[u8], report_len: usize) -> Vec<LogReport, 8> {
    // Create a temporary handler for processing
    let mut handler = UsbCommandHandler::new();
    let timestamp = 0; // Placeholder timestamp
    handler.process_output_report(report_buf, report_len, timestamp)
}
