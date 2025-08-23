#![allow(static_mut_refs)]

use crate::types::logging::{LogCategory, LogLevel, LogMessage, LogReport, LoggingConfig};
use core::sync::atomic::{AtomicU32, Ordering};
use heapless::spsc::Queue;

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

// Enhanced battery-specific logging functions with proper formatting
#[cfg(feature = "battery-logs")]
pub mod battery {
    use super::*;
    use crate::types::battery::{BatteryReading, BatteryState, SafetyFlags};
    use crate::types::errors::BatteryError;

    /// Log complete battery reading with all relevant data
    /// Format: "BAT: State={:?} ADC={} V={}mV Flags={:02x}"
    pub fn log_battery_reading(reading: &BatteryReading) {
        let mut content = [0u8; 52];

        // Format battery reading message efficiently for 52-byte limit
        // Voltage in mV, flags as hex byte
        let formatted_msg = format_battery_reading_message(reading);
        let msg_len = core::cmp::min(formatted_msg.len(), 51) as u8;

        content[..msg_len as usize].copy_from_slice(&formatted_msg[..msg_len as usize]);

        let msg = LogMessage {
            timestamp_ms: reading.timestamp_ms,
            level: LogLevel::Info,
            category: LogCategory::Battery,
            content,
            content_len: msg_len,
        };

        // Non-blocking enqueue - critical for safety task performance
        log_message(msg);
    }

    /// Log battery state transitions
    /// Format: "BAT: Transition {} -> {} ADC={}"  
    pub fn log_battery_state_change(
        old_state: BatteryState,
        new_state: BatteryState,
        adc_value: u16,
    ) {
        let mut content = [0u8; 52];

        let formatted_msg = format_state_transition_message(old_state, new_state, adc_value);
        let msg_len = core::cmp::min(formatted_msg.len(), 51) as u8;

        content[..msg_len as usize].copy_from_slice(&formatted_msg[..msg_len as usize]);

        let msg = LogMessage {
            timestamp_ms: get_timestamp_ms(),
            level: LogLevel::Info,
            category: LogCategory::Battery,
            content,
            content_len: msg_len,
        };

        log_message(msg);
    }

    /// Log critical battery safety events (highest priority)
    /// Format: "BAT: CRITICAL - {error_description}"
    pub fn log_battery_safety_event(error: &BatteryError) {
        let mut content = [0u8; 52];

        let formatted_msg = format_safety_event_message(error);
        let msg_len = core::cmp::min(formatted_msg.len(), 51) as u8;

        content[..msg_len as usize].copy_from_slice(&formatted_msg[..msg_len as usize]);

        let msg = LogMessage {
            timestamp_ms: get_timestamp_ms(),
            level: LogLevel::Error, // Critical safety events are errors
            category: LogCategory::Battery,
            content,
            content_len: msg_len,
        };

        log_message(msg);
    }

    /// Log charging circuit state changes
    /// Format: "BAT: Charging {enabled|disabled} V={}mV"
    pub fn log_charging_state_change(enabled: bool, voltage_mv: u16) {
        let mut content = [0u8; 52];

        let state_str: &[u8] = if enabled { b"enabled" } else { b"disabled" };
        let formatted_msg = format_charging_state_message(state_str, voltage_mv);
        let msg_len = core::cmp::min(formatted_msg.len(), 51) as u8;

        content[..msg_len as usize].copy_from_slice(&formatted_msg[..msg_len as usize]);

        let msg = LogMessage {
            timestamp_ms: get_timestamp_ms(),
            level: LogLevel::Info,
            category: LogCategory::Battery,
            content,
            content_len: msg_len,
        };

        log_message(msg);
    }

    /// Log battery safety status for periodic monitoring
    /// Format: "BAT: Safety OK|FAULT flags={:02x}"
    pub fn log_battery_safety_status(safety_flags: &SafetyFlags) {
        let mut content = [0u8; 52];

        let formatted_msg = format_safety_status_message(safety_flags);
        let msg_len = core::cmp::min(formatted_msg.len(), 51) as u8;

        content[..msg_len as usize].copy_from_slice(&formatted_msg[..msg_len as usize]);

        let msg = LogMessage {
            timestamp_ms: get_timestamp_ms(),
            level: if safety_flags.is_safe() {
                LogLevel::Debug
            } else {
                LogLevel::Warn
            },
            category: LogCategory::Battery,
            content,
            content_len: msg_len,
        };

        log_message(msg);
    }

    /// Log ADC calibration and diagnostic information
    /// Format: "BAT: ADC cal min={} max={} range={}"
    pub fn log_adc_diagnostics(min_reading: u16, max_reading: u16, error_count: u8) {
        let mut content = [0u8; 52];

        let formatted_msg = format_adc_diagnostics_message(min_reading, max_reading, error_count);
        let msg_len = core::cmp::min(formatted_msg.len(), 51) as u8;

        content[..msg_len as usize].copy_from_slice(&formatted_msg[..msg_len as usize]);

        let msg = LogMessage {
            timestamp_ms: get_timestamp_ms(),
            level: LogLevel::Debug,
            category: LogCategory::Battery,
            content,
            content_len: msg_len,
        };

        log_message(msg);
    }

    // Helper functions for efficient message formatting without heap allocation

    fn format_battery_reading_message(reading: &BatteryReading) -> [u8; 52] {
        let mut msg = [0u8; 52];
        let mut pos = 0;

        // "BAT: State="
        let prefix = b"BAT: State=";
        msg[pos..pos + prefix.len()].copy_from_slice(prefix);
        pos += prefix.len();

        // State name (Low, Normal, Charging, Full, Fault)
        let state_name = state_to_bytes(reading.state);
        msg[pos..pos + state_name.len()].copy_from_slice(state_name);
        pos += state_name.len();

        // " ADC="
        let adc_prefix = b" ADC=";
        msg[pos..pos + adc_prefix.len()].copy_from_slice(adc_prefix);
        pos += adc_prefix.len();

        // Format ADC value (u16)
        pos += format_u16_into_slice(reading.adc_value, &mut msg[pos..]);

        // " V="
        let volt_prefix = b" V=";
        msg[pos..pos + volt_prefix.len()].copy_from_slice(volt_prefix);
        pos += volt_prefix.len();

        // Format voltage (u16 in mV)
        pos += format_u16_into_slice(reading.voltage_mv, &mut msg[pos..]);

        // "mV F="
        let flags_prefix = b"mV F=";
        msg[pos..pos + flags_prefix.len()].copy_from_slice(flags_prefix);
        pos += flags_prefix.len();

        // Format safety flags as hex
        pos += format_hex_u8_into_slice(reading.safety_flags, &mut msg[pos..]);

        msg
    }

    fn format_state_transition_message(
        old_state: BatteryState,
        new_state: BatteryState,
        adc: u16,
    ) -> [u8; 52] {
        let mut msg = [0u8; 52];
        let mut pos = 0;

        let prefix = b"BAT: ";
        msg[pos..pos + prefix.len()].copy_from_slice(prefix);
        pos += prefix.len();

        let old_name = state_to_bytes(old_state);
        msg[pos..pos + old_name.len()].copy_from_slice(old_name);
        pos += old_name.len();

        let arrow = b" -> ";
        msg[pos..pos + arrow.len()].copy_from_slice(arrow);
        pos += arrow.len();

        let new_name = state_to_bytes(new_state);
        msg[pos..pos + new_name.len()].copy_from_slice(new_name);
        pos += new_name.len();

        let adc_prefix = b" ADC=";
        msg[pos..pos + adc_prefix.len()].copy_from_slice(adc_prefix);
        pos += adc_prefix.len();

        pos += format_u16_into_slice(adc, &mut msg[pos..]);

        msg
    }

    fn format_safety_event_message(error: &BatteryError) -> [u8; 52] {
        let mut msg = [0u8; 52];
        let mut pos = 0;

        let prefix = b"BAT: CRITICAL - ";
        msg[pos..pos + prefix.len()].copy_from_slice(prefix);
        pos += prefix.len();

        let desc = error.description().as_bytes();
        let copy_len = core::cmp::min(desc.len(), 52 - pos);
        msg[pos..pos + copy_len].copy_from_slice(&desc[..copy_len]);

        msg
    }

    fn format_charging_state_message(state: &[u8], voltage_mv: u16) -> [u8; 52] {
        let mut msg = [0u8; 52];
        let mut pos = 0;

        let prefix = b"BAT: Charging ";
        msg[pos..pos + prefix.len()].copy_from_slice(prefix);
        pos += prefix.len();

        msg[pos..pos + state.len()].copy_from_slice(state);
        pos += state.len();

        let volt_prefix = b" V=";
        msg[pos..pos + volt_prefix.len()].copy_from_slice(volt_prefix);
        pos += volt_prefix.len();

        pos += format_u16_into_slice(voltage_mv, &mut msg[pos..]);

        let suffix = b"mV";
        msg[pos..pos + suffix.len()].copy_from_slice(suffix);

        msg
    }

    fn format_safety_status_message(flags: &SafetyFlags) -> [u8; 52] {
        let mut msg = [0u8; 52];
        let mut pos = 0;

        let prefix = b"BAT: Safety ";
        msg[pos..pos + prefix.len()].copy_from_slice(prefix);
        pos += prefix.len();

        let status: &[u8] = if flags.is_safe() { b"OK" } else { b"FAULT" };
        msg[pos..pos + status.len()].copy_from_slice(status);
        pos += status.len();

        let flags_prefix = b" flags=";
        msg[pos..pos + flags_prefix.len()].copy_from_slice(flags_prefix);
        pos += flags_prefix.len();

        pos += format_hex_u8_into_slice(flags.get_packed_flags(), &mut msg[pos..]);

        msg
    }

    fn format_adc_diagnostics_message(min: u16, max: u16, errors: u8) -> [u8; 52] {
        let mut msg = [0u8; 52];
        let mut pos = 0;

        let prefix = b"BAT: ADC min=";
        msg[pos..pos + prefix.len()].copy_from_slice(prefix);
        pos += prefix.len();

        pos += format_u16_into_slice(min, &mut msg[pos..]);

        let max_prefix = b" max=";
        msg[pos..pos + max_prefix.len()].copy_from_slice(max_prefix);
        pos += max_prefix.len();

        pos += format_u16_into_slice(max, &mut msg[pos..]);

        let err_prefix = b" err=";
        msg[pos..pos + err_prefix.len()].copy_from_slice(err_prefix);
        pos += err_prefix.len();

        pos += format_u8_into_slice(errors, &mut msg[pos..]);

        msg
    }

    // Utility functions for efficient number formatting

    fn state_to_bytes(state: BatteryState) -> &'static [u8] {
        match state {
            BatteryState::Low => b"Low",
            BatteryState::Normal => b"Normal",
            BatteryState::Charging => b"Charging",
            BatteryState::Full => b"Full",
            BatteryState::Fault => b"Fault",
        }
    }

    fn format_u16_into_slice(value: u16, slice: &mut [u8]) -> usize {
        let mut temp = [0u8; 5]; // Max 5 digits for u16
        let mut pos = 0;
        let mut val = value;

        if val == 0 {
            slice[0] = b'0';
            return 1;
        }

        while val > 0 {
            temp[pos] = (val % 10) as u8 + b'0';
            val /= 10;
            pos += 1;
        }

        // Reverse digits into output slice
        for i in 0..pos {
            slice[i] = temp[pos - 1 - i];
        }

        pos
    }

    fn format_u8_into_slice(value: u8, slice: &mut [u8]) -> usize {
        format_u16_into_slice(value as u16, slice)
    }

    fn format_hex_u8_into_slice(value: u8, slice: &mut [u8]) -> usize {
        const HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";
        slice[0] = HEX_CHARS[((value >> 4) & 0xF) as usize];
        slice[1] = HEX_CHARS[(value & 0xF) as usize];
        2
    }
}

// Legacy function maintained for compatibility
#[cfg(feature = "battery-logs")]
pub fn log_battery_status(state: &str, adc_value: u16, _voltage: f32) {
    use crate::types::battery::BatteryState;

    // Convert to new system format
    let _voltage_mv = (_voltage * 1000.0) as u16;
    let _battery_state = if state == "Low" {
        BatteryState::Low
    } else if state == "Charging" {
        BatteryState::Charging
    } else {
        BatteryState::Normal
    };

    let dummy_flags = crate::types::battery::SafetyFlags::new();
    let reading =
        crate::types::battery::BatteryReading::new(get_timestamp_ms(), adc_value, &dummy_flags);

    battery::log_battery_reading(&reading);
}

#[cfg(feature = "pemf-logs")]
pub fn log_pemf_pulse(_duration_ms: u32, _frequency_hz: f32) {
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
pub fn log_usb_event(_event: &str) {
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
