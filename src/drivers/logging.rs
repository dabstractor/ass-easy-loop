use crate::types::logging::{LogCategory, LogLevel, LogMessage, LogReport, LoggingConfig};
use heapless::spsc::Queue;
use core::sync::atomic::{AtomicU32, Ordering};

// Global timestamp counter
static mut TIMESTAMP_MS: AtomicU32 = AtomicU32::new(0);

// Message queue for log messages (32 message capacity as per requirements)
static mut LOG_QUEUE: Queue<LogMessage, 32> = Queue::new();

// Global logging configuration
static mut LOGGING_CONFIG: LoggingConfig = LoggingConfig {
    enabled_categories: 0xF, // All categories enabled by default
    verbosity_level: LogLevel::Debug,
    enabled: true,
};

/// Initialize the logging system
pub fn init() {
    unsafe {
        // Queue is already initialized as static
        // Reset configuration to defaults
        LOGGING_CONFIG = LoggingConfig {
            enabled_categories: 0xF, // All categories enabled by default
            verbosity_level: LogLevel::Debug,
            enabled: true,
        };
    }
}

/// Get the global logging configuration
pub fn get_config() -> LoggingConfig {
    unsafe { LOGGING_CONFIG }
}

/// Set the global logging configuration
pub fn set_config(config: LoggingConfig) {
    unsafe {
        LOGGING_CONFIG = config;
    }
}

/// Check if a category is enabled
pub fn is_category_enabled(category: LogCategory) -> bool {
    unsafe {
        LOGGING_CONFIG.enabled && (LOGGING_CONFIG.enabled_categories & (1 << (category as u8))) != 0
    }
}

/// Check if a log level should be logged based on verbosity
pub fn should_log(level: LogLevel) -> bool {
    unsafe { LOGGING_CONFIG.enabled && level as u8 >= LOGGING_CONFIG.verbosity_level as u8 }
}

/// Enqueue a log message if logging is enabled and the message meets filtering criteria
pub fn log_message(msg: LogMessage) {
    // Check if logging is enabled and message meets criteria
    if !should_log(msg.level) || !is_category_enabled(msg.category) {
        return;
    }

    unsafe {
        // Non-blocking enqueue - FIFO behavior, oldest message automatically discarded
        let _ = LOG_QUEUE.enqueue(msg);
    }
}

/// Dequeue a log message for transmission
pub fn dequeue_message() -> Option<LogMessage> {
    unsafe { LOG_QUEUE.dequeue() }
}

/// Format a log message into a HID report
pub fn format_log_message(msg: &LogMessage) -> LogReport {
    let mut report = LogReport { data: [0u8; 64] };

    // PATTERN: Proper byte ordering for multi-byte values (little-endian)
    report.data[0] = (msg.timestamp_ms & 0xFF) as u8;
    report.data[1] = ((msg.timestamp_ms >> 8) & 0xFF) as u8;
    report.data[2] = ((msg.timestamp_ms >> 16) & 0xFF) as u8;
    report.data[3] = ((msg.timestamp_ms >> 24) & 0xFF) as u8;
    report.data[4] = msg.level as u8;
    report.data[5] = msg.category as u8;
    report.data[6] = msg.content_len;
    report.data[7] = 0; // Reserved/padding

    // PATTERN: Content with truncation and "..." indicator
    let copy_len = core::cmp::min(msg.content_len as usize, 51);
    report.data[8..(8 + copy_len)].copy_from_slice(&msg.content[..copy_len]);

    // PATTERN: Add "..." indicator for truncated messages
    if msg.content_len as usize > 51 {
        report.data[59] = b'.';
        report.data[60] = b'.';
        report.data[61] = b'.';
    }

    // CRITICAL: Message must fit exactly in 64-byte HID report
    report
}

// Conditional compilation for feature-specific logging functions
#[cfg(feature = "battery-logs")]
pub fn log_battery_status(state: &str, adc_value: u16, voltage: f32) {
    let mut content = [0u8; 52];
    // Simple string formatting approach
    let msg = b"Battery: ";
    let msg_len = core::cmp::min(msg.len(), 51);
    content[..msg_len].copy_from_slice(&msg[..msg_len]);

    let msg = LogMessage {
        timestamp_ms: get_timestamp_ms(),
        level: LogLevel::Info,
        category: LogCategory::Battery,
        content,
        content_len: msg_len as u8,
    };

    log_message(msg);
}

#[cfg(feature = "pemf-logs")]
pub fn log_pemf_pulse(duration_ms: u32, frequency_hz: f32) {
    let mut content = [0u8; 52];
    // Simple string formatting approach
    let msg = b"PEMF pulse";
    let msg_len = core::cmp::min(msg.len(), 51);
    content[..msg_len].copy_from_slice(&msg[..msg_len]);

    let msg = LogMessage {
        timestamp_ms: get_timestamp_ms(),
        level: LogLevel::Info,
        category: LogCategory::Pemf,
        content,
        content_len: msg_len as u8,
    };

    log_message(msg);
}

#[cfg(feature = "system-logs")]
pub fn log_system_event(event: &str) {
    let mut content = [0u8; 52];
    // Use the actual event string instead of hardcoded message
    let event_bytes = event.as_bytes();
    let msg_len = core::cmp::min(event_bytes.len(), 51);
    content[..msg_len].copy_from_slice(&event_bytes[..msg_len]);

    let msg = LogMessage {
        timestamp_ms: get_timestamp_ms(),
        level: LogLevel::Info,
        category: LogCategory::System,
        content,
        content_len: msg_len as u8,
    };

    log_message(msg);
}

#[cfg(feature = "usb-logs")]
pub fn log_usb_event(event: &str) {
    let mut content = [0u8; 52];
    // Simple string formatting approach
    let msg = b"USB event";
    let msg_len = core::cmp::min(msg.len(), 51);
    content[..msg_len].copy_from_slice(&msg[..msg_len]);

    let msg = LogMessage {
        timestamp_ms: get_timestamp_ms(),
        level: LogLevel::Info,
        category: LogCategory::Usb,
        content,
        content_len: msg_len as u8,
    };

    log_message(msg);
}

/// Get timestamp in milliseconds
pub fn get_timestamp_ms() -> u32 {
    unsafe { TIMESTAMP_MS.load(Ordering::Relaxed) }
}

/// Set timestamp in milliseconds
pub fn set_timestamp_ms(timestamp: u32) {
    unsafe { TIMESTAMP_MS.store(timestamp, Ordering::Relaxed) }
}

/// Initialize the logging system with proper timestamp
pub fn init_with_timestamp() {
    // Initialize with 0, will be updated by the main loop
    unsafe {
        TIMESTAMP_MS.store(0, Ordering::Relaxed);
    }
}
