//! USB HID Logging Module
//! 
//! This module provides USB HID logging functionality for the RP2040 pEMF device.
//! It includes message formatting, queuing, and USB HID transmission capabilities.

use heapless::Vec;
use portable_atomic::{AtomicUsize, Ordering};
use core::iter::Iterator;
use core::option::Option::{self, Some, None};
use core::result::Result::{self, Ok, Err};
use core::ops::FnOnce;



/// Maximum length of a log message content
pub const MAX_MESSAGE_LENGTH: usize = 48;

/// Maximum length of a module name
pub const MAX_MODULE_NAME_LENGTH: usize = 8;

/// Default log message queue size
#[allow(dead_code)]
pub const DEFAULT_QUEUE_SIZE: usize = 32;


/// Log severity levels
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(any(test, feature = "system-logs"), derive(Debug))]
#[repr(u8)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

impl LogLevel {
    /// Convert log level to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

/// Internal log message representation
#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
pub struct LogMessage {
    pub timestamp: u32,                           // Milliseconds since boot
    pub level: LogLevel,                          // Debug/Info/Warn/Error
    pub module: [u8; MAX_MODULE_NAME_LENGTH],     // Source module (e.g., "BATTERY", "PEMF")
    pub message: [u8; MAX_MESSAGE_LENGTH],        // Formatted message content
}

impl LogMessage {
    /// Create a new log message
    pub fn new(timestamp: u32, level: LogLevel, module: &str, message: &str) -> Self {
        let mut module_bytes = [0u8; MAX_MODULE_NAME_LENGTH];
        let mut message_bytes = [0u8; MAX_MESSAGE_LENGTH];

        // Copy module name, truncating if necessary
        let module_len = core::cmp::min(module.len(), MAX_MODULE_NAME_LENGTH);
        module_bytes[..module_len].copy_from_slice(&module.as_bytes()[..module_len]);

        // Copy message content, truncating if necessary
        let message_len = core::cmp::min(message.len(), MAX_MESSAGE_LENGTH);
        message_bytes[..message_len].copy_from_slice(&message.as_bytes()[..message_len]);

        Self {
            timestamp,
            level,
            module: module_bytes,
            message: message_bytes,
        }
    }

    /// Get module name as string slice
    pub fn module_str(&self) -> &str {
        // Find the null terminator or use full length
        let len = self.module.iter().position(|&b| b == 0).unwrap_or(MAX_MODULE_NAME_LENGTH);
        core::str::from_utf8(&self.module[..len]).unwrap_or("<invalid>")
    }

    /// Get message content as string slice
    pub fn message_str(&self) -> &str {
        // Find the null terminator or use full length
        let len = self.message.iter().position(|&b| b == 0).unwrap_or(MAX_MESSAGE_LENGTH);
        core::str::from_utf8(&self.message[..len]).unwrap_or("<invalid>")
    }

    /// Serialize log message to binary format for HID transmission
    /// Format: [level:1][module:8][message:48][timestamp:4][reserved:3] = 64 bytes
    pub fn serialize(&self) -> [u8; 64] {
        let mut buffer = [0u8; 64];
        
        // Byte 0: Log level
        buffer[0] = self.level as u8;
        
        // Bytes 1-8: Module name (8 bytes, null-padded)
        buffer[1..9].copy_from_slice(&self.module);
        
        // Bytes 9-56: Message content (48 bytes, null-terminated)
        buffer[9..57].copy_from_slice(&self.message);
        
        // Bytes 57-60: Timestamp (little-endian u32)
        let timestamp_bytes = self.timestamp.to_le_bytes();
        buffer[57..61].copy_from_slice(&timestamp_bytes);
        
        // Bytes 61-63: Reserved/padding (already zeroed)
        
        buffer
    }

    /// Deserialize log message from binary format
    pub fn deserialize(buffer: &[u8; 64]) -> Result<Self, &'static str> {
        if buffer.len() != 64 {
            return Err("Invalid buffer size");
        }

        // Parse log level
        let level = match buffer[0] {
            0 => LogLevel::Debug,
            1 => LogLevel::Info,
            2 => LogLevel::Warn,
            3 => LogLevel::Error,
            _ => return Err("Invalid log level"),
        };

        // Parse module name
        let mut module = [0u8; MAX_MODULE_NAME_LENGTH];
        module.copy_from_slice(&buffer[1..9]);

        // Parse message content
        let mut message = [0u8; MAX_MESSAGE_LENGTH];
        message.copy_from_slice(&buffer[9..57]);

        // Parse timestamp
        let mut timestamp_bytes = [0u8; 4];
        timestamp_bytes.copy_from_slice(&buffer[57..61]);
        let timestamp = u32::from_le_bytes(timestamp_bytes);

        Ok(Self {
            timestamp,
            level,
            module,
            message,
        })
    }
}

#[cfg(not(test))]
/// USB HID report structure for log message transmission
/// Uses custom vendor-defined HID descriptor for 64-byte reports
#[derive(Clone, Copy, Debug)]
pub struct LogReport {
    /// Complete log message data (64 bytes: level + module + message + timestamp + padding)
    pub data: [u8; 64],
}

#[cfg(not(test))]
impl core::convert::TryFrom<[u8; 64]> for LogReport {
    type Error = &'static str;

    fn try_from(bytes: [u8; 64]) -> Result<Self, Self::Error> {
        Ok(Self { data: bytes })
    }
}

impl LogReport {
    /// Get the HID report descriptor for this report type
    /// Updated to support both input reports (device to host) and output reports (host to device)
    /// Requirements: 2.1, 2.2 - USB HID bidirectional communication
    pub fn descriptor() -> &'static [u8] {
        // Vendor-defined HID descriptor for 64-byte bidirectional reports
        &[
            0x06, 0x00, 0xFF,  // Usage Page (Vendor Defined 0xFF00)
            0x09, 0x01,        // Usage (0x01)
            0xA1, 0x01,        // Collection (Application)
            
            // Input Report (Device to Host) - for log messages
            0x09, 0x02,        //   Usage (0x02) - Log Data
            0x15, 0x00,        //   Logical Minimum (0)
            0x26, 0xFF, 0x00,  //   Logical Maximum (255)
            0x75, 0x08,        //   Report Size (8 bits)
            0x95, 0x40,        //   Report Count (64 bytes)
            0x81, 0x02,        //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
            
            // Output Report (Host to Device) - for commands
            0x09, 0x03,        //   Usage (0x03) - Command Data
            0x15, 0x00,        //   Logical Minimum (0)
            0x26, 0xFF, 0x00,  //   Logical Maximum (255)
            0x75, 0x08,        //   Report Size (8 bits)
            0x95, 0x40,        //   Report Count (64 bytes)
            0x91, 0x02,        //   Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
            
            0xC0,              // End Collection
        ]
    }
}

#[cfg(not(test))]
impl serde::Serialize for LogReport {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize the data array as a sequence of bytes
        use serde::ser::SerializeTuple;
        let mut seq = serializer.serialize_tuple(64)?;
        for byte in &self.data {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }
}

#[cfg(not(test))]
impl usbd_hid::descriptor::AsInputReport for LogReport {}

#[cfg(test)]
impl usbd_hid::descriptor::AsInputReport for LogReport {}

#[cfg(test)]
/// USB HID report structure for log message transmission (test version without HID descriptor)
#[derive(Clone, Copy, Debug)]
pub struct LogReport {
    /// Complete log message data (64 bytes: level + module + message + timestamp + padding)
    pub data: [u8; 64],
}

impl LogReport {
    /// Create a new LogReport from a LogMessage
    pub fn from_log_message(message: &LogMessage) -> Self {
        let data = message.serialize();
        Self { data }
    }

    /// Convert LogReport back to LogMessage
    pub fn to_log_message(self) -> Result<LogMessage, &'static str> {
        LogMessage::deserialize(&self.data)
    }

    /// Get the raw HID report bytes (64 bytes total)
    pub fn as_bytes(&self) -> [u8; 64] {
        self.data
    }

    /// Create LogReport from raw HID report bytes
    pub fn from_bytes(bytes: &[u8; 64]) -> Self {
        Self { data: *bytes }
    }

    /// Get timestamp from the embedded data
    pub fn timestamp(&self) -> u32 {
        // Extract timestamp from bytes 57-60 (little-endian u32)
        let mut timestamp_bytes = [0u8; 4];
        timestamp_bytes.copy_from_slice(&self.data[57..61]);
        u32::from_le_bytes(timestamp_bytes)
    }

    /// Get log level from the embedded data
    pub fn level(&self) -> Result<LogLevel, &'static str> {
        match self.data[0] {
            0 => Ok(LogLevel::Debug),
            1 => Ok(LogLevel::Info),
            2 => Ok(LogLevel::Warn),
            3 => Ok(LogLevel::Error),
            _ => Err("Invalid log level"),
        }
    }
}

/// Queue statistics for monitoring performance and behavior
#[derive(Clone, Copy, Debug, Default)]
pub struct QueueStats {
    /// Total number of messages successfully enqueued
    pub messages_sent: u32,
    /// Total number of messages dropped due to queue overflow
    pub messages_dropped: u32,
    /// Peak queue utilization (maximum number of messages held)
    pub peak_utilization: usize,
    /// Current queue utilization percentage (0-100)
    pub current_utilization_percent: u8,
}

impl QueueStats {
    /// Calculate current utilization percentage for a queue of size N
    pub fn calculate_utilization<const N: usize>(current_count: usize) -> u8 {
        if N == 0 {
            return 0;
        }
        ((current_count * 100) / N) as u8
    }
}

/// Performance monitoring statistics for USB tasks and system behavior
/// Requirements: 7.1, 7.2, 7.5
#[derive(Clone, Copy, Debug, Default)]
pub struct PerformanceStats {
    /// CPU usage statistics for USB tasks
    pub usb_cpu_usage: CpuUsageStats,
    /// Memory usage statistics for logging system
    pub memory_usage: MemoryUsageStats,
    /// Message processing performance metrics
    pub message_performance: MessagePerformanceStats,
    /// System timing impact measurements
    pub timing_impact: TimingImpactStats,
}

/// CPU usage statistics for USB tasks
#[derive(Clone, Copy, Debug, Default)]
pub struct CpuUsageStats {
    /// USB polling task CPU usage percentage (0-100)
    pub usb_poll_cpu_percent: u8,
    /// USB HID transmission task CPU usage percentage (0-100)
    pub usb_hid_cpu_percent: u8,
    /// Combined USB tasks CPU usage percentage (0-100)
    pub total_usb_cpu_percent: u8,
    /// Peak CPU usage recorded for USB tasks
    pub peak_usb_cpu_percent: u8,
    /// Number of CPU usage measurements taken
    pub measurement_count: u32,
    /// Average CPU usage over all measurements
    pub average_cpu_percent: u8,
}

/// Memory usage statistics for the logging system
#[derive(Clone, Copy, Debug, Default)]
pub struct MemoryUsageStats {
    /// Current memory usage by log queue in bytes
    pub queue_memory_bytes: usize,
    /// Peak memory usage by log queue in bytes
    pub peak_queue_memory_bytes: usize,
    /// USB buffer memory usage in bytes
    pub usb_buffer_memory_bytes: usize,
    /// Total logging system memory usage in bytes
    pub total_memory_bytes: usize,
    /// Memory utilization percentage of available RAM
    pub memory_utilization_percent: u8,
    /// Number of memory allocations (should be 0 for static allocation)
    pub allocation_count: u32,
}

/// Message processing performance metrics
#[derive(Clone, Copy, Debug, Default)]
pub struct MessagePerformanceStats {
    /// Average time to format a message in microseconds
    pub avg_format_time_us: u32,
    /// Average time to enqueue a message in microseconds
    pub avg_enqueue_time_us: u32,
    /// Average time to transmit a message via USB in microseconds
    pub avg_transmission_time_us: u32,
    /// Peak message processing time in microseconds
    pub peak_processing_time_us: u32,
    /// Total number of messages processed
    pub messages_processed: u32,
    /// Number of transmission failures
    pub transmission_failures: u32,
}

/// System timing impact measurements
#[derive(Clone, Copy, Debug, Default)]
pub struct TimingImpactStats {
    /// pEMF pulse timing deviation with USB logging active (microseconds)
    pub pemf_timing_deviation_us: u32,
    /// Battery monitoring timing deviation with USB logging active (microseconds)
    pub battery_timing_deviation_us: u32,
    /// Maximum observed timing deviation across all tasks (microseconds)
    pub max_timing_deviation_us: u32,
    /// Percentage of timing measurements within tolerance
    pub timing_accuracy_percent: u8,
    /// Number of timing violations detected
    pub timing_violations: u32,
}

impl PerformanceStats {
    /// Create new performance statistics with default values
    pub const fn new() -> Self {
        Self {
            usb_cpu_usage: CpuUsageStats::new(),
            memory_usage: MemoryUsageStats::new(),
            message_performance: MessagePerformanceStats::new(),
            timing_impact: TimingImpactStats::new(),
        }
    }

    /// Update CPU usage statistics
    pub fn update_cpu_usage(&mut self, poll_cpu: u8, hid_cpu: u8) {
        self.usb_cpu_usage.usb_poll_cpu_percent = poll_cpu;
        self.usb_cpu_usage.usb_hid_cpu_percent = hid_cpu;
        self.usb_cpu_usage.total_usb_cpu_percent = poll_cpu.saturating_add(hid_cpu);
        
        if self.usb_cpu_usage.total_usb_cpu_percent > self.usb_cpu_usage.peak_usb_cpu_percent {
            self.usb_cpu_usage.peak_usb_cpu_percent = self.usb_cpu_usage.total_usb_cpu_percent;
        }
        
        self.usb_cpu_usage.measurement_count = self.usb_cpu_usage.measurement_count.saturating_add(1);
        
        // Calculate running average
        let total_measurements = self.usb_cpu_usage.measurement_count;
        if total_measurements > 0 {
            let current_avg = self.usb_cpu_usage.average_cpu_percent as u32;
            let new_sample = self.usb_cpu_usage.total_usb_cpu_percent as u32;
            let new_avg = (current_avg * (total_measurements - 1) + new_sample) / total_measurements;
            self.usb_cpu_usage.average_cpu_percent = new_avg as u8;
        }
    }

    /// Update memory usage statistics
    pub fn update_memory_usage(&mut self, queue_bytes: usize, usb_buffer_bytes: usize) {
        self.memory_usage.queue_memory_bytes = queue_bytes;
        self.memory_usage.usb_buffer_memory_bytes = usb_buffer_bytes;
        self.memory_usage.total_memory_bytes = queue_bytes + usb_buffer_bytes;
        
        if queue_bytes > self.memory_usage.peak_queue_memory_bytes {
            self.memory_usage.peak_queue_memory_bytes = queue_bytes;
        }
        
        // Calculate memory utilization percentage (assuming 264KB total RAM on RP2040)
        const TOTAL_RAM_BYTES: usize = 264 * 1024;
        self.memory_usage.memory_utilization_percent = 
            ((self.memory_usage.total_memory_bytes * 100) / TOTAL_RAM_BYTES) as u8;
    }

    /// Update message performance statistics
    pub fn update_message_performance(&mut self, format_time_us: u32, enqueue_time_us: u32, transmission_time_us: u32) {
        let total_processing_time = format_time_us + enqueue_time_us + transmission_time_us;
        
        // Update running averages
        let count = self.message_performance.messages_processed;
        if count > 0 {
            self.message_performance.avg_format_time_us = 
                (self.message_performance.avg_format_time_us * count + format_time_us) / (count + 1);
            self.message_performance.avg_enqueue_time_us = 
                (self.message_performance.avg_enqueue_time_us * count + enqueue_time_us) / (count + 1);
            self.message_performance.avg_transmission_time_us = 
                (self.message_performance.avg_transmission_time_us * count + transmission_time_us) / (count + 1);
        } else {
            self.message_performance.avg_format_time_us = format_time_us;
            self.message_performance.avg_enqueue_time_us = enqueue_time_us;
            self.message_performance.avg_transmission_time_us = transmission_time_us;
        }
        
        if total_processing_time > self.message_performance.peak_processing_time_us {
            self.message_performance.peak_processing_time_us = total_processing_time;
        }
        
        self.message_performance.messages_processed = self.message_performance.messages_processed.saturating_add(1);
    }

    /// Record a transmission failure
    pub fn record_transmission_failure(&mut self) {
        self.message_performance.transmission_failures = 
            self.message_performance.transmission_failures.saturating_add(1);
    }

    /// Update timing impact statistics
    pub fn update_timing_impact(&mut self, pemf_deviation_us: u32, battery_deviation_us: u32) {
        self.timing_impact.pemf_timing_deviation_us = pemf_deviation_us;
        self.timing_impact.battery_timing_deviation_us = battery_deviation_us;
        
        let max_deviation = core::cmp::max(pemf_deviation_us, battery_deviation_us);
        if max_deviation > self.timing_impact.max_timing_deviation_us {
            self.timing_impact.max_timing_deviation_us = max_deviation;
        }
        
        // Check if timing is within tolerance (±1% = ±10ms for 1000ms period)
        const TIMING_TOLERANCE_US: u32 = 10_000; // 10ms in microseconds
        if max_deviation <= TIMING_TOLERANCE_US {
            // Timing is within tolerance - update accuracy percentage
            // This is a simplified calculation; in practice, you'd track more measurements
            if self.timing_impact.timing_accuracy_percent < 100 {
                self.timing_impact.timing_accuracy_percent = self.timing_impact.timing_accuracy_percent.saturating_add(1);
            }
        } else {
            self.timing_impact.timing_violations = self.timing_impact.timing_violations.saturating_add(1);
        }
    }

    /// Get a summary of performance impact
    pub fn get_performance_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            cpu_usage_ok: self.usb_cpu_usage.total_usb_cpu_percent <= crate::config::system::MAX_USB_CPU_USAGE_PERCENT,
            memory_usage_ok: self.memory_usage.memory_utilization_percent <= 10, // 10% threshold
            timing_impact_ok: self.timing_impact.timing_accuracy_percent >= 95, // 95% accuracy threshold
            overall_performance_ok: self.usb_cpu_usage.total_usb_cpu_percent <= crate::config::system::MAX_USB_CPU_USAGE_PERCENT &&
                                   self.memory_usage.memory_utilization_percent <= 10 &&
                                   self.timing_impact.timing_accuracy_percent >= 95,
        }
    }
}

impl CpuUsageStats {
    pub const fn new() -> Self {
        Self {
            usb_poll_cpu_percent: 0,
            usb_hid_cpu_percent: 0,
            total_usb_cpu_percent: 0,
            peak_usb_cpu_percent: 0,
            measurement_count: 0,
            average_cpu_percent: 0,
        }
    }
}

impl MemoryUsageStats {
    pub const fn new() -> Self {
        Self {
            queue_memory_bytes: 0,
            peak_queue_memory_bytes: 0,
            usb_buffer_memory_bytes: 0,
            total_memory_bytes: 0,
            memory_utilization_percent: 0,
            allocation_count: 0,
        }
    }
}

impl MessagePerformanceStats {
    pub const fn new() -> Self {
        Self {
            avg_format_time_us: 0,
            avg_enqueue_time_us: 0,
            avg_transmission_time_us: 0,
            peak_processing_time_us: 0,
            messages_processed: 0,
            transmission_failures: 0,
        }
    }
}

impl TimingImpactStats {
    pub const fn new() -> Self {
        Self {
            pemf_timing_deviation_us: 0,
            battery_timing_deviation_us: 0,
            max_timing_deviation_us: 0,
            timing_accuracy_percent: 100,
            timing_violations: 0,
        }
    }
}

/// Performance summary for quick health checks
#[derive(Clone, Copy, Debug)]
pub struct PerformanceSummary {
    pub cpu_usage_ok: bool,
    pub memory_usage_ok: bool,
    pub timing_impact_ok: bool,
    pub overall_performance_ok: bool,
}

/// Performance monitoring utilities
pub struct PerformanceMonitor;

impl PerformanceMonitor {
    /// Measure CPU usage for a task execution
    /// Returns the execution time in microseconds
    pub fn measure_task_execution<F, R>(task: F) -> (R, u32)
    where
        F: FnOnce() -> R,
    {
        // Get start time using RTIC monotonic timer
        let start_time = unsafe { 
            if let Some(get_timestamp) = TIMESTAMP_FUNCTION {
                get_timestamp()
            } else {
                0
            }
        };
        
        // Execute the task
        let result = task();
        
        // Get end time
        let end_time = unsafe { 
            if let Some(get_timestamp) = TIMESTAMP_FUNCTION {
                get_timestamp()
            } else {
                0
            }
        };
        
        // Calculate execution time in microseconds
        let execution_time_us = if end_time >= start_time {
            (end_time - start_time) * 1000 // Convert ms to us
        } else {
            0 // Handle timer overflow case
        };
        
        (result, execution_time_us)
    }

    /// Calculate CPU usage percentage based on execution time and period
    pub fn calculate_cpu_usage(execution_time_us: u32, period_us: u32) -> u8 {
        if period_us == 0 {
            return 0;
        }
        
        let usage_percent = (execution_time_us * 100) / period_us;
        core::cmp::min(usage_percent, 100) as u8
    }

    /// Estimate memory usage for a queue with N elements
    pub fn calculate_queue_memory_usage<const N: usize>(current_count: usize) -> usize {
        // Each LogMessage is approximately 64 bytes (timestamp + level + module + message)
        const MESSAGE_SIZE: usize = core::mem::size_of::<LogMessage>();
        // Queue overhead includes atomic counters and array storage
        let queue_overhead = core::mem::size_of::<LogQueue<N>>() - (N * MESSAGE_SIZE);
        
        queue_overhead + (current_count * MESSAGE_SIZE)
    }

    /// Optimize message formatting for minimal CPU overhead
    /// This is an optimized version of the standard message formatting
    pub fn format_message_optimized(message: &LogMessage) -> [u8; 64] {
        // Use direct memory operations instead of string formatting for better performance
        let mut buffer = [0u8; 64];
        
        // Directly serialize without intermediate string formatting
        buffer[0] = message.level as u8;
        buffer[1..9].copy_from_slice(&message.module);
        buffer[9..57].copy_from_slice(&message.message);
        
        // Use bit operations for timestamp encoding
        let timestamp_bytes = message.timestamp.to_le_bytes();
        buffer[57..61].copy_from_slice(&timestamp_bytes);
        
        buffer
    }

    /// Batch process multiple messages for improved efficiency
    pub fn batch_format_messages(messages: &[LogMessage]) -> Vec<[u8; 64], 32> {
        let mut formatted_messages = Vec::new();
        
        for message in messages.iter().take(32) { // Limit batch size
            let formatted = Self::format_message_optimized(message);
            if formatted_messages.push(formatted).is_err() {
                break; // Vec is full
            }
        }
        
        formatted_messages
    }
}

/// Thread-safe circular buffer for storing log messages with statistics tracking
pub struct LogQueue<const N: usize> {
    buffer: [Option<LogMessage>; N],
    head: AtomicUsize,
    tail: AtomicUsize,
    count: AtomicUsize,
    // Statistics tracking
    messages_sent: AtomicUsize,
    messages_dropped: AtomicUsize,
    peak_utilization: AtomicUsize,
}

impl<const N: usize> Default for LogQueue<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> LogQueue<N> {
    /// Create a new empty log queue
    pub const fn new() -> Self {
        Self {
            buffer: [const { None }; N],
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            count: AtomicUsize::new(0),
            messages_sent: AtomicUsize::new(0),
            messages_dropped: AtomicUsize::new(0),
            peak_utilization: AtomicUsize::new(0),
        }
    }

    /// Enqueue a log message (lock-free with FIFO eviction on overflow)
    /// Always returns true as messages are either queued or oldest is evicted
    pub fn enqueue(&mut self, message: LogMessage) -> bool {
        let current_count = self.count.load(Ordering::Acquire);
        let mut dropped = false;
        
        // If queue is full, evict oldest message (FIFO eviction)
        if current_count >= N
            && self.dequeue_internal().is_some() {
                self.messages_dropped.fetch_add(1, Ordering::Relaxed);
                dropped = true;
            }

        // Get current head position and store message
        let head = self.head.load(Ordering::Acquire);
        self.buffer[head] = Some(message);
        
        // Update head pointer (circular)
        let new_head = (head + 1) % N;
        self.head.store(new_head, Ordering::Release);
        
        // Update count (ensure it doesn't exceed capacity)
        let new_count = if dropped {
            current_count // Count stays the same if we evicted
        } else {
            current_count + 1
        };
        self.count.store(new_count, Ordering::Release);
        
        // Update statistics
        self.messages_sent.fetch_add(1, Ordering::Relaxed);
        
        // Update peak utilization
        let current_peak = self.peak_utilization.load(Ordering::Relaxed);
        if new_count > current_peak {
            self.peak_utilization.store(new_count, Ordering::Relaxed);
        }
        
        true
    }

    /// Dequeue a log message (lock-free)
    /// Returns None if queue is empty
    pub fn dequeue(&mut self) -> Option<LogMessage> {
        self.dequeue_internal()
    }

    /// Internal dequeue implementation for use by both public dequeue and overflow handling
    fn dequeue_internal(&mut self) -> Option<LogMessage> {
        let current_count = self.count.load(Ordering::Acquire);
        
        if current_count == 0 {
            return None;
        }

        let tail = self.tail.load(Ordering::Acquire);
        let message = self.buffer[tail].take();
        
        // Update tail pointer (circular)
        let new_tail = (tail + 1) % N;
        self.tail.store(new_tail, Ordering::Release);
        
        // Update count
        self.count.store(current_count - 1, Ordering::Release);
        
        message
    }

    /// Get current queue size
    pub fn len(&self) -> usize {
        self.count.load(Ordering::Acquire)
    }

    /// Get queue capacity
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if queue is full
    pub fn is_full(&self) -> bool {
        self.len() >= N
    }

    /// Get current queue statistics
    pub fn stats(&self) -> QueueStats {
        let current_count = self.len();
        QueueStats {
            messages_sent: self.messages_sent.load(Ordering::Relaxed) as u32,
            messages_dropped: self.messages_dropped.load(Ordering::Relaxed) as u32,
            peak_utilization: self.peak_utilization.load(Ordering::Relaxed),
            current_utilization_percent: QueueStats::calculate_utilization::<N>(current_count),
        }
    }

    /// Reset statistics counters (useful for testing or periodic monitoring)
    pub fn reset_stats(&mut self) {
        self.messages_sent.store(0, Ordering::Relaxed);
        self.messages_dropped.store(0, Ordering::Relaxed);
        self.peak_utilization.store(self.len(), Ordering::Relaxed);
    }

    /// Clear all messages from the queue
    pub fn clear(&mut self) {
        while self.dequeue().is_some() {}
    }
}

/// Message formatting utilities
pub struct MessageFormatter;

impl MessageFormatter {
    /// Format a log message into a human-readable string
    /// Format: [TIMESTAMP] [LEVEL] [MODULE] MESSAGE
    pub fn format_message(message: &LogMessage) -> Vec<u8, 128> {
        let mut formatted = Vec::new();
        
        // Add timestamp
        let _ = write_to_vec(&mut formatted, b"[");
        let _ = write_u32_to_vec(&mut formatted, message.timestamp);
        let _ = write_to_vec(&mut formatted, b"] [");
        
        // Add log level
        let _ = write_to_vec(&mut formatted, message.level.as_str().as_bytes());
        let _ = write_to_vec(&mut formatted, b"] [");
        
        // Add module name
        let _ = write_to_vec(&mut formatted, message.module_str().as_bytes());
        let _ = write_to_vec(&mut formatted, b"] ");
        
        // Add message content
        let _ = write_to_vec(&mut formatted, message.message_str().as_bytes());
        
        formatted
    }
}

/// Helper function to write bytes to a Vec
fn write_to_vec(vec: &mut Vec<u8, 128>, data: &[u8]) -> Result<(), ()> {
    for &byte in data {
        vec.push(byte).map_err(|_| ())?;
    }
    Ok(())
}

/// Helper function to write u32 as decimal string to a Vec
fn write_u32_to_vec(vec: &mut Vec<u8, 128>, mut value: u32) -> Result<(), ()> {
    if value == 0 {
        return vec.push(b'0').map_err(|_| ());
    }
    
    let mut digits = [0u8; 10]; // u32 max is 10 digits
    let mut count = 0;
    
    while value > 0 {
        digits[count] = (value % 10) as u8 + b'0';
        value /= 10;
        count += 1;
    }
    
    // Write digits in reverse order
    for i in (0..count).rev() {
        vec.push(digits[i]).map_err(|_| ())?;
    }
    
    Ok(())
}

/// Global logging queue for macro integration
static mut GLOBAL_QUEUE: Option<&'static mut LogQueue<32>> = None;
static mut TIMESTAMP_FUNCTION: Option<fn() -> u32> = None;

/// Initialize the global logging system with a queue and timestamp function
/// This must be called once during system initialization
/// 
/// # Safety
/// This function is unsafe because it modifies global state.
/// It should only be called once during system initialization.
pub unsafe fn init_global_logging(queue: *mut LogQueue<32>, get_timestamp: fn() -> u32) {
    GLOBAL_QUEUE = Some(&mut *queue);
    TIMESTAMP_FUNCTION = Some(get_timestamp);
}

/// Global runtime configuration for logging system
static mut GLOBAL_CONFIG: Option<&'static mut crate::config::LogConfig> = None;

/// Global performance statistics for monitoring system behavior
/// Requirements: 7.1, 7.2, 7.5
static mut GLOBAL_PERFORMANCE_STATS: Option<&'static mut PerformanceStats> = None;

/// Initialize the global performance monitoring system
/// This must be called once during system initialization
/// 
/// # Safety
/// This function is unsafe because it modifies global state.
/// It should only be called once during system initialization.
pub unsafe fn init_global_performance_monitoring(stats: *mut PerformanceStats) {
    GLOBAL_PERFORMANCE_STATS = Some(&mut *stats);
}

/// Get the current performance statistics
/// Returns None if performance monitoring hasn't been initialized
#[allow(static_mut_refs)]
pub fn get_global_performance_stats() -> Option<&'static PerformanceStats> {
    unsafe { GLOBAL_PERFORMANCE_STATS.as_deref() }
}

/// Update global performance statistics
/// Returns true if update was successful, false if performance monitoring is not initialized
#[allow(static_mut_refs)]
pub fn update_global_performance_stats<F>(update_fn: F) -> bool 
where
    F: FnOnce(&mut PerformanceStats)
{
    unsafe {
        if let Some(stats) = GLOBAL_PERFORMANCE_STATS.as_mut() {
            update_fn(stats);
            true
        } else {
            false
        }
    }
}

/// Record CPU usage for USB tasks
pub fn record_usb_cpu_usage(poll_cpu_percent: u8, hid_cpu_percent: u8) {
    update_global_performance_stats(|stats| {
        stats.update_cpu_usage(poll_cpu_percent, hid_cpu_percent);
    });
}

/// Record memory usage for logging system
pub fn record_memory_usage(queue_bytes: usize, usb_buffer_bytes: usize) {
    update_global_performance_stats(|stats| {
        stats.update_memory_usage(queue_bytes, usb_buffer_bytes);
    });
}

/// Record message processing performance
pub fn record_message_performance(format_time_us: u32, enqueue_time_us: u32, transmission_time_us: u32) {
    update_global_performance_stats(|stats| {
        stats.update_message_performance(format_time_us, enqueue_time_us, transmission_time_us);
    });
}

/// Record a transmission failure
pub fn record_transmission_failure() {
    update_global_performance_stats(|stats| {
        stats.record_transmission_failure();
    });
}

/// Record timing impact measurements
pub fn record_timing_impact(pemf_deviation_us: u32, battery_deviation_us: u32) {
    update_global_performance_stats(|stats| {
        stats.update_timing_impact(pemf_deviation_us, battery_deviation_us);
    });
}

/// Initialize the global logging configuration
/// This must be called once during system initialization
/// 
/// # Safety
/// This function is unsafe because it modifies global state.
/// It should only be called once during system initialization.
pub unsafe fn init_global_config(config: *mut crate::config::LogConfig) {
    GLOBAL_CONFIG = Some(&mut *config);
}

/// Get the current runtime logging configuration
/// Returns None if configuration hasn't been initialized
#[allow(static_mut_refs)]
pub fn get_global_config() -> Option<&'static crate::config::LogConfig> {
    unsafe { GLOBAL_CONFIG.as_deref() }
}

/// Update the global logging configuration
/// Returns true if update was successful, false if config is not initialized
#[allow(static_mut_refs)]
pub fn update_global_config<F>(update_fn: F) -> bool 
where
    F: FnOnce(&mut crate::config::LogConfig) -> Result<(), crate::config::ConfigError>
{
    unsafe {
        if let Some(config) = GLOBAL_CONFIG.as_mut() {
            update_fn(config).is_ok()
        } else {
            false
        }
    }
}

/// Core log message function that formats and queues messages
/// This function is called by the logging macros and handles:
/// - Timestamp generation using RTIC monotonic timer
/// - Module name tracking
/// - Message formatting and queuing
/// - Runtime configuration filtering
pub fn log_message(level: LogLevel, module: &str, message: &str) {
    log_message_with_category(level, module, message, crate::config::LogCategory::General)
}

/// Core log message function with category support for runtime filtering
/// This function provides category-specific logging control
#[allow(static_mut_refs)]
pub fn log_message_with_category(level: LogLevel, module: &str, message: &str, category: crate::config::LogCategory) {
    unsafe {
        // Check if logging system is initialized
        if let (Some(queue), Some(get_timestamp)) = (GLOBAL_QUEUE.as_mut(), TIMESTAMP_FUNCTION) {
            // Check runtime configuration if available
            if let Some(config) = GLOBAL_CONFIG.as_ref() {
                if !config.should_log(level, category) {
                    return; // Message filtered out by runtime configuration
                }
            }
            
            let timestamp = get_timestamp();
            let log_msg = LogMessage::new(timestamp, level, module, message);
            let _ = queue.enqueue(log_msg);
        }
    }
}

/// Logging macros for convenient usage throughout the codebase
/// These macros automatically capture the module name and format messages
///
/// Log a debug message
/// Usage: log_debug!("MESSAGE") or log_debug!("FORMAT", args...)
#[macro_export]
macro_rules! log_debug {
    ($msg:expr) => {
        $crate::logging::log_message($crate::logging::LogLevel::Debug, module_path!(), $msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        {
            use heapless::String;
            use core::fmt::Write;
            let mut formatted: String<48> = String::new();
            if write!(&mut formatted, $fmt, $($arg)*).is_ok() {
                $crate::logging::log_message($crate::logging::LogLevel::Debug, module_path!(), formatted.as_str())
            }
        }
    };
}

/// Log an info message
/// Usage: log_info!("MESSAGE") or log_info!("FORMAT", args...)
#[macro_export]
macro_rules! log_info {
    ($msg:expr) => {
        $crate::logging::log_message($crate::logging::LogLevel::Info, module_path!(), $msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        {
            use heapless::String;
            use core::fmt::Write;
            let mut formatted: String<48> = String::new();
            if write!(&mut formatted, $fmt, $($arg)*).is_ok() {
                $crate::logging::log_message($crate::logging::LogLevel::Info, module_path!(), formatted.as_str())
            }
        }
    };
}

/// Log a warning message
/// Usage: log_warn!("MESSAGE") or log_warn!("FORMAT", args...)
#[macro_export]
macro_rules! log_warn {
    ($msg:expr) => {
        $crate::logging::log_message($crate::logging::LogLevel::Warn, module_path!(), $msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        {
            use heapless::String;
            use core::fmt::Write;
            let mut formatted: String<48> = String::new();
            if write!(&mut formatted, $fmt, $($arg)*).is_ok() {
                $crate::logging::log_message($crate::logging::LogLevel::Warn, module_path!(), formatted.as_str())
            }
        }
    };
}

/// Log an error message
/// Usage: log_error!("MESSAGE") or log_error!("FORMAT", args...)
#[macro_export]
macro_rules! log_error {
    ($msg:expr) => {
        $crate::logging::log_message($crate::logging::LogLevel::Error, module_path!(), $msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        {
            use heapless::String;
            use core::fmt::Write;
            let mut formatted: String<48> = String::new();
            if write!(&mut formatted, $fmt, $($arg)*).is_ok() {
                $crate::logging::log_message($crate::logging::LogLevel::Error, module_path!(), formatted.as_str())
            }
        }
    };
}

/// Category-specific logging macros with conditional compilation support
/// These macros provide compile-time and runtime filtering for different log categories
///
/// Log a battery-related debug message
/// Usage: log_battery_debug!("MESSAGE") or log_battery_debug!("FORMAT", args...)
#[macro_export]
macro_rules! log_battery_debug {
    ($msg:expr) => {
        #[cfg(feature = "battery-logs")]
        $crate::logging::log_message_with_category(
            $crate::logging::LogLevel::Debug, 
            module_path!(), 
            $msg, 
            $crate::config::LogCategory::Battery
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "battery-logs")]
        {
            use heapless::String;
            use core::fmt::Write;
            let mut formatted: String<48> = String::new();
            if write!(&mut formatted, $fmt, $($arg)*).is_ok() {
                $crate::logging::log_message_with_category(
                    $crate::logging::LogLevel::Debug, 
                    module_path!(), 
                    formatted.as_str(), 
                    $crate::config::LogCategory::Battery
                )
            }
        }
    };
}

/// Log a battery-related info message
/// Usage: log_battery_info!("MESSAGE") or log_battery_info!("FORMAT", args...)
#[macro_export]
macro_rules! log_battery_info {
    ($msg:expr) => {
        #[cfg(feature = "battery-logs")]
        $crate::logging::log_message_with_category(
            $crate::logging::LogLevel::Info, 
            module_path!(), 
            $msg, 
            $crate::config::LogCategory::Battery
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "battery-logs")]
        {
            use heapless::String;
            use core::fmt::Write;
            let mut formatted: String<48> = String::new();
            if write!(&mut formatted, $fmt, $($arg)*).is_ok() {
                $crate::logging::log_message_with_category(
                    $crate::logging::LogLevel::Info, 
                    module_path!(), 
                    formatted.as_str(), 
                    $crate::config::LogCategory::Battery
                )
            }
        }
    };
}

/// Log a battery-related warning message
/// Usage: log_battery_warn!("MESSAGE") or log_battery_warn!("FORMAT", args...)
#[macro_export]
macro_rules! log_battery_warn {
    ($msg:expr) => {
        #[cfg(feature = "battery-logs")]
        $crate::logging::log_message_with_category(
            $crate::logging::LogLevel::Warn, 
            module_path!(), 
            $msg, 
            $crate::config::LogCategory::Battery
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "battery-logs")]
        {
            use heapless::String;
            use core::fmt::Write;
            let mut formatted: String<48> = String::new();
            if write!(&mut formatted, $fmt, $($arg)*).is_ok() {
                $crate::logging::log_message_with_category(
                    $crate::logging::LogLevel::Warn, 
                    module_path!(), 
                    formatted.as_str(), 
                    $crate::config::LogCategory::Battery
                )
            }
        }
    };
}

/// Log a battery-related error message
/// Usage: log_battery_error!("MESSAGE") or log_battery_error!("FORMAT", args...)
#[macro_export]
macro_rules! log_battery_error {
    ($msg:expr) => {
        #[cfg(feature = "battery-logs")]
        $crate::logging::log_message_with_category(
            $crate::logging::LogLevel::Error, 
            module_path!(), 
            $msg, 
            $crate::config::LogCategory::Battery
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "battery-logs")]
        {
            use heapless::String;
            use core::fmt::Write;
            let mut formatted: String<48> = String::new();
            if write!(&mut formatted, $fmt, $($arg)*).is_ok() {
                $crate::logging::log_message_with_category(
                    $crate::logging::LogLevel::Error, 
                    module_path!(), 
                    formatted.as_str(), 
                    $crate::config::LogCategory::Battery
                )
            }
        }
    };
}

/// Log a pEMF-related debug message
/// Usage: log_pemf_debug!("MESSAGE") or log_pemf_debug!("FORMAT", args...)
#[macro_export]
macro_rules! log_pemf_debug {
    ($msg:expr) => {
        #[cfg(feature = "pemf-logs")]
        $crate::logging::log_message_with_category(
            $crate::logging::LogLevel::Debug, 
            module_path!(), 
            $msg, 
            $crate::config::LogCategory::Pemf
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "pemf-logs")]
        {
            use heapless::String;
            use core::fmt::Write;
            let mut formatted: String<48> = String::new();
            if write!(&mut formatted, $fmt, $($arg)*).is_ok() {
                $crate::logging::log_message_with_category(
                    $crate::logging::LogLevel::Debug, 
                    module_path!(), 
                    formatted.as_str(), 
                    $crate::config::LogCategory::Pemf
                )
            }
        }
    };
}

/// Log a pEMF-related info message
/// Usage: log_pemf_info!("MESSAGE") or log_pemf_info!("FORMAT", args...)
#[macro_export]
macro_rules! log_pemf_info {
    ($msg:expr) => {
        #[cfg(feature = "pemf-logs")]
        $crate::logging::log_message_with_category(
            $crate::logging::LogLevel::Info, 
            module_path!(), 
            $msg, 
            $crate::config::LogCategory::Pemf
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "pemf-logs")]
        {
            use heapless::String;
            use core::fmt::Write;
            let mut formatted: String<48> = String::new();
            if write!(&mut formatted, $fmt, $($arg)*).is_ok() {
                $crate::logging::log_message_with_category(
                    $crate::logging::LogLevel::Info, 
                    module_path!(), 
                    formatted.as_str(), 
                    $crate::config::LogCategory::Pemf
                )
            }
        }
    };
}

/// Log a pEMF-related warning message
/// Usage: log_pemf_warn!("MESSAGE") or log_pemf_warn!("FORMAT", args...)
#[macro_export]
macro_rules! log_pemf_warn {
    ($msg:expr) => {
        #[cfg(feature = "pemf-logs")]
        $crate::logging::log_message_with_category(
            $crate::logging::LogLevel::Warn, 
            module_path!(), 
            $msg, 
            $crate::config::LogCategory::Pemf
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "pemf-logs")]
        {
            use heapless::String;
            use core::fmt::Write;
            let mut formatted: String<48> = String::new();
            if write!(&mut formatted, $fmt, $($arg)*).is_ok() {
                $crate::logging::log_message_with_category(
                    $crate::logging::LogLevel::Warn, 
                    module_path!(), 
                    formatted.as_str(), 
                    $crate::config::LogCategory::Pemf
                )
            }
        }
    };
}

/// Log a pEMF-related error message
/// Usage: log_pemf_error!("MESSAGE") or log_pemf_error!("FORMAT", args...)
#[macro_export]
macro_rules! log_pemf_error {
    ($msg:expr) => {
        #[cfg(feature = "pemf-logs")]
        $crate::logging::log_message_with_category(
            $crate::logging::LogLevel::Error, 
            module_path!(), 
            $msg, 
            $crate::config::LogCategory::Pemf
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "pemf-logs")]
        {
            use heapless::String;
            use core::fmt::Write;
            let mut formatted: String<48> = String::new();
            if write!(&mut formatted, $fmt, $($arg)*).is_ok() {
                $crate::logging::log_message_with_category(
                    $crate::logging::LogLevel::Error, 
                    module_path!(), 
                    formatted.as_str(), 
                    $crate::config::LogCategory::Pemf
                )
            }
        }
    };
}

/// Log a system-related debug message
/// Usage: log_system_debug!("MESSAGE") or log_system_debug!("FORMAT", args...)
#[macro_export]
macro_rules! log_system_debug {
    ($msg:expr) => {
        #[cfg(feature = "system-logs")]
        $crate::logging::log_message_with_category(
            $crate::logging::LogLevel::Debug, 
            module_path!(), 
            $msg, 
            $crate::config::LogCategory::System
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "system-logs")]
        {
            use heapless::String;
            use core::fmt::Write;
            let mut formatted: String<48> = String::new();
            if write!(&mut formatted, $fmt, $($arg)*).is_ok() {
                $crate::logging::log_message_with_category(
                    $crate::logging::LogLevel::Debug, 
                    module_path!(), 
                    formatted.as_str(), 
                    $crate::config::LogCategory::System
                )
            }
        }
    };
}

/// Log a system-related info message
/// Usage: log_system_info!("MESSAGE") or log_system_info!("FORMAT", args...)
#[macro_export]
macro_rules! log_system_info {
    ($msg:expr) => {
        #[cfg(feature = "system-logs")]
        $crate::logging::log_message_with_category(
            $crate::logging::LogLevel::Info, 
            module_path!(), 
            $msg, 
            $crate::config::LogCategory::System
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "system-logs")]
        {
            use heapless::String;
            use core::fmt::Write;
            let mut formatted: String<48> = String::new();
            if write!(&mut formatted, $fmt, $($arg)*).is_ok() {
                $crate::logging::log_message_with_category(
                    $crate::logging::LogLevel::Info, 
                    module_path!(), 
                    formatted.as_str(), 
                    $crate::config::LogCategory::System
                )
            }
        }
    };
}

/// Log a system-related warning message
/// Usage: log_system_warn!("MESSAGE") or log_system_warn!("FORMAT", args...)
#[macro_export]
macro_rules! log_system_warn {
    ($msg:expr) => {
        #[cfg(feature = "system-logs")]
        $crate::logging::log_message_with_category(
            $crate::logging::LogLevel::Warn, 
            module_path!(), 
            $msg, 
            $crate::config::LogCategory::System
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "system-logs")]
        {
            use heapless::String;
            use core::fmt::Write;
            let mut formatted: String<48> = String::new();
            if write!(&mut formatted, $fmt, $($arg)*).is_ok() {
                $crate::logging::log_message_with_category(
                    $crate::logging::LogLevel::Warn, 
                    module_path!(), 
                    formatted.as_str(), 
                    $crate::config::LogCategory::System
                )
            }
        }
    };
}

/// Log a system-related error message
/// Usage: log_system_error!("MESSAGE") or log_system_error!("FORMAT", args...)
#[macro_export]
macro_rules! log_system_error {
    ($msg:expr) => {
        #[cfg(feature = "system-logs")]
        $crate::logging::log_message_with_category(
            $crate::logging::LogLevel::Error, 
            module_path!(), 
            $msg, 
            $crate::config::LogCategory::System
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "system-logs")]
        {
            use heapless::String;
            use core::fmt::Write;
            let mut formatted: String<48> = String::new();
            if write!(&mut formatted, $fmt, $($arg)*).is_ok() {
                $crate::logging::log_message_with_category(
                    $crate::logging::LogLevel::Error, 
                    module_path!(), 
                    formatted.as_str(), 
                    $crate::config::LogCategory::System
                )
            }
        }
    };
}

/// Log a USB-related debug message
/// Usage: log_usb_debug!("MESSAGE") or log_usb_debug!("FORMAT", args...)
#[macro_export]
macro_rules! log_usb_debug {
    ($msg:expr) => {
        #[cfg(feature = "usb-logs")]
        $crate::logging::log_message_with_category(
            $crate::logging::LogLevel::Debug, 
            module_path!(), 
            $msg, 
            $crate::config::LogCategory::Usb
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "usb-logs")]
        {
            use heapless::String;
            use core::fmt::Write;
            let mut formatted: String<48> = String::new();
            if write!(&mut formatted, $fmt, $($arg)*).is_ok() {
                $crate::logging::log_message_with_category(
                    $crate::logging::LogLevel::Debug, 
                    module_path!(), 
                    formatted.as_str(), 
                    $crate::config::LogCategory::Usb
                )
            }
        }
    };
}

/// Log a USB-related info message
/// Usage: log_usb_info!("MESSAGE") or log_usb_info!("FORMAT", args...)
#[macro_export]
macro_rules! log_usb_info {
    ($msg:expr) => {
        #[cfg(feature = "usb-logs")]
        $crate::logging::log_message_with_category(
            $crate::logging::LogLevel::Info, 
            module_path!(), 
            $msg, 
            $crate::config::LogCategory::Usb
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "usb-logs")]
        {
            use heapless::String;
            use core::fmt::Write;
            let mut formatted: String<48> = String::new();
            if write!(&mut formatted, $fmt, $($arg)*).is_ok() {
                $crate::logging::log_message_with_category(
                    $crate::logging::LogLevel::Info, 
                    module_path!(), 
                    formatted.as_str(), 
                    $crate::config::LogCategory::Usb
                )
            }
        }
    };
}

/// Log a USB-related warning message
/// Usage: log_usb_warn!("MESSAGE") or log_usb_warn!("FORMAT", args...)
#[macro_export]
macro_rules! log_usb_warn {
    ($msg:expr) => {
        #[cfg(feature = "usb-logs")]
        $crate::logging::log_message_with_category(
            $crate::logging::LogLevel::Warn, 
            module_path!(), 
            $msg, 
            $crate::config::LogCategory::Usb
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "usb-logs")]
        {
            use heapless::String;
            use core::fmt::Write;
            let mut formatted: String<48> = String::new();
            if write!(&mut formatted, $fmt, $($arg)*).is_ok() {
                $crate::logging::log_message_with_category(
                    $crate::logging::LogLevel::Warn, 
                    module_path!(), 
                    formatted.as_str(), 
                    $crate::config::LogCategory::Usb
                )
            }
        }
    };
}

/// Log a USB-related error message
/// Usage: log_usb_error!("MESSAGE") or log_usb_error!("FORMAT", args...)
#[macro_export]
macro_rules! log_usb_error {
    ($msg:expr) => {
        #[cfg(feature = "usb-logs")]
        $crate::logging::log_message_with_category(
            $crate::logging::LogLevel::Error, 
            module_path!(), 
            $msg, 
            $crate::config::LogCategory::Usb
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature = "usb-logs")]
        {
            use heapless::String;
            use core::fmt::Write;
            let mut formatted: String<48> = String::new();
            if write!(&mut formatted, $fmt, $($arg)*).is_ok() {
                $crate::logging::log_message_with_category(
                    $crate::logging::LogLevel::Error, 
                    module_path!(), 
                    formatted.as_str(), 
                    $crate::config::LogCategory::Usb
                )
            }
        }
    };
}

/// Logging interface trait
pub trait Logger {
    /// Log a message with the specified level
    fn log(&mut self, level: LogLevel, module: &str, message: &str);
    
    /// Log a debug message
    fn debug(&mut self, module: &str, message: &str) {
        self.log(LogLevel::Debug, module, message);
    }
    
    /// Log an info message
    fn info(&mut self, module: &str, message: &str) {
        self.log(LogLevel::Info, module, message);
    }
    
    /// Log a warning message
    fn warn(&mut self, module: &str, message: &str) {
        self.log(LogLevel::Warn, module, message);
    }
    
    /// Log an error message
    fn error(&mut self, module: &str, message: &str) {
        self.log(LogLevel::Error, module, message);
    }
}

// Basic logger implementation using a message queue
pub struct QueueLogger<const N: usize> {
    queue: LogQueue<N>,
    get_timestamp: fn() -> u32,
}

impl<const N: usize> QueueLogger<N> {
    /// Create a new queue logger with a timestamp function
    pub fn new(get_timestamp: fn() -> u32) -> Self {
        Self {
            queue: LogQueue::new(),
            get_timestamp,
        }
    }
    
    /// Get a reference to the internal queue
    pub fn queue(&mut self) -> &mut LogQueue<N> {
        &mut self.queue
    }
}

impl<const N: usize> Logger for QueueLogger<N> {
    fn log(&mut self, level: LogLevel, module: &str, message: &str) {
        let timestamp = (self.get_timestamp)();
        let log_message = LogMessage::new(timestamp, level, module, message);
        let _ = self.queue.enqueue(log_message);
    }
}
