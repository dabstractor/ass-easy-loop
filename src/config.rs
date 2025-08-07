//! Compile-time Configuration Constants
//!
//! This module defines compile-time configuration constants for USB HID logging
//! and other system parameters.

use crate::logging::LogLevel;
use core::result::Result::{self, Ok, Err};
use core::option::Option::{self, Some, None};
use core::default::Default;

/// USB Device Configuration
pub mod usb {
    /// USB Vendor ID for the HID logging device
    /// Using a custom VID for development - should be replaced with registered VID for production
    pub const VENDOR_ID: u16 = 0x1234;

    /// USB Product ID for the HID logging device
    pub const PRODUCT_ID: u16 = 0x5678;

    /// USB device manufacturer string (used in validation functions)
    pub const MANUFACTURER: &str = "dabstractor";

    /// USB device product string (used in validation functions)
    pub const PRODUCT: &str = "Ass Easy Loop";

    /// USB device serial number string (used in validation functions)
    pub const SERIAL_NUMBER: &str = "001";

    /// USB device release number (BCD format)
    pub const DEVICE_RELEASE: u16 = 0x0100; // Version 1.0

    /// HID report descriptor size in bytes
    pub const HID_REPORT_SIZE: usize = 64;

    /// Maximum number of HID endpoints
    pub const MAX_HID_ENDPOINTS: usize = 1;
}

/// Logging Configuration
pub mod logging {
    use super::LogLevel;

    /// Maximum log level to include at compile time
    /// Messages above this level will be compiled out
    #[cfg(debug_assertions)]
    pub const MAX_LOG_LEVEL: LogLevel = LogLevel::Debug;

    #[cfg(not(debug_assertions))]
    pub const MAX_LOG_LEVEL: LogLevel = LogLevel::Info;

    /// Default log message queue size
    #[allow(dead_code)]
    pub const DEFAULT_QUEUE_SIZE: usize = 32;

    /// Maximum log message queue size
    pub const MAX_QUEUE_SIZE: usize = 128;

    /// Enable battery monitoring logs (compile-time feature flag)
    #[cfg(feature = "battery-logs")]
    pub const ENABLE_BATTERY_LOGS: bool = true;
    #[cfg(not(feature = "battery-logs"))]
    pub const ENABLE_BATTERY_LOGS: bool = false;

    /// Enable pEMF pulse monitoring logs (compile-time feature flag)
    #[cfg(feature = "pemf-logs")]
    pub const ENABLE_PEMF_LOGS: bool = true;
    #[cfg(not(feature = "pemf-logs"))]
    pub const ENABLE_PEMF_LOGS: bool = false;

    /// Enable system status and diagnostic logs (compile-time feature flag)
    #[cfg(feature = "system-logs")]
    pub const ENABLE_SYSTEM_LOGS: bool = true;
    #[cfg(not(feature = "system-logs"))]
    pub const ENABLE_SYSTEM_LOGS: bool = false;

    /// Enable USB HID communication logs (compile-time feature flag)
    #[cfg(feature = "usb-logs")]
    pub const ENABLE_USB_LOGS: bool = true;
    #[cfg(not(feature = "usb-logs"))]
    pub const ENABLE_USB_LOGS: bool = false;

    /// Log message transmission timeout in milliseconds
    pub const TRANSMISSION_TIMEOUT_MS: u32 = 100;

    /// Maximum retry attempts for failed log transmissions
    pub const MAX_RETRY_ATTEMPTS: u8 = 3;

    /// Battery periodic logging interval in samples (at 10Hz sampling rate)
    /// Default: 50 samples = 5 seconds of periodic voltage readings
    pub const BATTERY_PERIODIC_LOG_INTERVAL_SAMPLES: u32 = 50;
}

/// System Configuration
pub mod system {
    /// System clock frequency in Hz
    pub const SYSTEM_CLOCK_HZ: u32 = 125_000_000;

    /// External crystal frequency in Hz
    pub const EXTERNAL_CRYSTAL_HZ: u32 = 12_000_000;

    /// USB polling interval in milliseconds
    pub const USB_POLL_INTERVAL_MS: u64 = 1;

    /// HID transmission task interval in milliseconds
    pub const HID_TASK_INTERVAL_MS: u64 = 10;

    /// Maximum CPU usage percentage for USB tasks
    pub const MAX_USB_CPU_USAGE_PERCENT: u8 = 5;
}

/// Task Priority Configuration
pub mod priorities {
    /// pEMF pulse generation task priority (highest)
    pub const PEMF_PULSE_PRIORITY: u8 = 3;

    /// Battery monitoring task priority
    pub const BATTERY_MONITOR_PRIORITY: u8 = 2;

    /// LED control task priority
    pub const LED_CONTROL_PRIORITY: u8 = 1;

    /// USB HID transmission task priority
    pub const USB_HID_TASK_PRIORITY: u8 = 1;

    /// USB device polling task priority (lowest)
    pub const USB_POLL_TASK_PRIORITY: u8 = 0;
}

/// Hardware Pin Configuration
pub mod pins {
    /// MOSFET control pin (GPIO 15)
    pub const MOSFET_PIN: u8 = 15;

    /// LED control pin (GPIO 25)
    pub const LED_PIN: u8 = 25;

    /// ADC input pin for battery monitoring (GPIO 26)
    pub const BATTERY_ADC_PIN: u8 = 26;
}

/// Timing Configuration
pub mod timing {
    /// pEMF pulse frequency in Hz
    pub const PEMF_FREQUENCY_HZ: f32 = 2.0;

    /// pEMF pulse HIGH duration in milliseconds
    pub const PEMF_PULSE_HIGH_MS: u64 = 2;

    /// pEMF pulse LOW duration in milliseconds
    pub const PEMF_PULSE_LOW_MS: u64 = 498;

    /// Battery monitoring sampling frequency in Hz
    pub const BATTERY_SAMPLING_HZ: f32 = 10.0;

    /// Battery monitoring interval in milliseconds
    pub const BATTERY_MONITOR_INTERVAL_MS: u64 = 100;

    /// LED flash frequency for low battery indication in Hz
    pub const LED_FLASH_FREQUENCY_HZ: f32 = 2.0;

    /// LED flash ON duration in milliseconds
    pub const LED_FLASH_ON_MS: u64 = 250;

    /// LED flash OFF duration in milliseconds
    pub const LED_FLASH_OFF_MS: u64 = 250;

    /// Timing tolerance percentage (±1%)
    pub const TIMING_TOLERANCE_PERCENT: f32 = 0.01;
}

/// Feature Flags
pub mod features {
    /// Enable USB HID logging functionality
    pub const ENABLE_USB_HID_LOGGING: bool = true;

    /// Enable compile-time log level filtering
    pub const ENABLE_COMPILE_TIME_LOG_FILTERING: bool = true;

    /// Enable runtime log level control
    pub const ENABLE_RUNTIME_LOG_CONTROL: bool = true;

    /// Enable log message timestamps
    #[allow(dead_code)]
    pub const ENABLE_LOG_TIMESTAMPS: bool = true;

    /// Enable log message module names
    #[allow(dead_code)]
    pub const ENABLE_LOG_MODULE_NAMES: bool = true;

    /// Enable panic handler USB logging
    #[allow(dead_code)]
    pub const ENABLE_PANIC_USB_LOGGING: bool = true;

    /// Enable performance monitoring
    #[allow(dead_code)]
    pub const ENABLE_PERFORMANCE_MONITORING: bool = true;

    /// Enable memory usage tracking
    #[allow(dead_code)]
    pub const ENABLE_MEMORY_TRACKING: bool = true;
}

/// Runtime logging configuration structure
/// This struct holds runtime-configurable logging parameters that can be modified via USB control commands
/// Requirements: 8.1, 8.2, 8.3, 8.4, 8.5
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LogConfig {
    /// Runtime maximum log level (can be more restrictive than compile-time MAX_LOG_LEVEL)
    pub max_level: LogLevel,
    /// Log message queue size (must be <= MAX_QUEUE_SIZE)
    pub queue_size: usize,
    /// USB Vendor ID
    pub usb_vid: u16,
    /// USB Product ID
    pub usb_pid: u16,
    /// Enable battery monitoring logs at runtime
    pub enable_battery_logs: bool,
    /// Enable pEMF timing logs at runtime
    pub enable_pemf_logs: bool,
    /// Enable system status logs at runtime
    pub enable_system_logs: bool,
    /// Enable USB communication logs at runtime
    pub enable_usb_logs: bool,
    /// Log transmission timeout in milliseconds
    pub transmission_timeout_ms: u32,
    /// Maximum retry attempts for failed transmissions
    pub max_retry_attempts: u8,
}

impl LogConfig {
    /// Create a new LogConfig with default values based on compile-time constants
    pub const fn new() -> Self {
        Self {
            max_level: logging::MAX_LOG_LEVEL,
            queue_size: logging::DEFAULT_QUEUE_SIZE,
            usb_vid: usb::VENDOR_ID,
            usb_pid: usb::PRODUCT_ID,
            enable_battery_logs: logging::ENABLE_BATTERY_LOGS,
            enable_pemf_logs: logging::ENABLE_PEMF_LOGS,
            enable_system_logs: logging::ENABLE_SYSTEM_LOGS,
            enable_usb_logs: logging::ENABLE_USB_LOGS,
            transmission_timeout_ms: logging::TRANSMISSION_TIMEOUT_MS,
            max_retry_attempts: logging::MAX_RETRY_ATTEMPTS,
        }
    }

    /// Create a LogConfig with all logging categories enabled (for debugging)
    pub const fn debug_config() -> Self {
        Self {
            max_level: LogLevel::Debug,
            queue_size: logging::MAX_QUEUE_SIZE,
            usb_vid: usb::VENDOR_ID,
            usb_pid: usb::PRODUCT_ID,
            enable_battery_logs: true,
            enable_pemf_logs: true,
            enable_system_logs: true,
            enable_usb_logs: true,
            transmission_timeout_ms: logging::TRANSMISSION_TIMEOUT_MS,
            max_retry_attempts: logging::MAX_RETRY_ATTEMPTS,
        }
    }

    /// Create a LogConfig with minimal logging (for production)
    pub const fn minimal_config() -> Self {
        Self {
            max_level: LogLevel::Error,
            queue_size: 16, // Smaller queue for minimal logging
            usb_vid: usb::VENDOR_ID,
            usb_pid: usb::PRODUCT_ID,
            enable_battery_logs: false,
            enable_pemf_logs: false,
            enable_system_logs: false,
            enable_usb_logs: false,
            transmission_timeout_ms: 50, // Shorter timeout
            max_retry_attempts: 1, // Fewer retries
        }
    }

    /// Check if a log level should be processed based on runtime configuration
    /// This combines compile-time and runtime filtering
    pub fn should_log(&self, level: LogLevel, category: LogCategory) -> bool {
        // First check compile-time maximum level
        if level as u8 > logging::MAX_LOG_LEVEL as u8 {
            return false;
        }

        // Then check runtime maximum level
        if level as u8 > self.max_level as u8 {
            return false;
        }

        // Finally check category-specific runtime flags
        match category {
            LogCategory::Battery => self.enable_battery_logs && logging::ENABLE_BATTERY_LOGS,
            LogCategory::Pemf => self.enable_pemf_logs && logging::ENABLE_PEMF_LOGS,
            LogCategory::System => self.enable_system_logs && logging::ENABLE_SYSTEM_LOGS,
            LogCategory::Usb => self.enable_usb_logs && logging::ENABLE_USB_LOGS,
            LogCategory::General => true, // General logs are always enabled if level passes
        }
    }

    /// Update runtime log level (must not exceed compile-time maximum)
    pub fn set_max_level(&mut self, level: LogLevel) -> Result<(), ConfigError> {
        if level as u8 > logging::MAX_LOG_LEVEL as u8 {
            return Err(ConfigError::InvalidLogLevel);
        }
        self.max_level = level;
        Ok(())
    }

    /// Enable or disable battery logging at runtime
    pub fn set_battery_logs(&mut self, enabled: bool) -> Result<(), ConfigError> {
        if enabled && !logging::ENABLE_BATTERY_LOGS {
            return Err(ConfigError::CategoryDisabledAtCompileTime);
        }
        self.enable_battery_logs = enabled;
        Ok(())
    }

    /// Enable or disable pEMF logging at runtime
    pub fn set_pemf_logs(&mut self, enabled: bool) -> Result<(), ConfigError> {
        if enabled && !logging::ENABLE_PEMF_LOGS {
            return Err(ConfigError::CategoryDisabledAtCompileTime);
        }
        self.enable_pemf_logs = enabled;
        Ok(())
    }

    /// Enable or disable system logging at runtime
    pub fn set_system_logs(&mut self, enabled: bool) -> Result<(), ConfigError> {
        if enabled && !logging::ENABLE_SYSTEM_LOGS {
            return Err(ConfigError::CategoryDisabledAtCompileTime);
        }
        self.enable_system_logs = enabled;
        Ok(())
    }

    /// Enable or disable USB logging at runtime
    pub fn set_usb_logs(&mut self, enabled: bool) -> Result<(), ConfigError> {
        if enabled && !logging::ENABLE_USB_LOGS {
            return Err(ConfigError::CategoryDisabledAtCompileTime);
        }
        self.enable_usb_logs = enabled;
        Ok(())
    }

    /// Set transmission timeout (with validation)
    pub fn set_transmission_timeout(&mut self, timeout_ms: u32) -> Result<(), ConfigError> {
        if timeout_ms == 0 || timeout_ms > 10000 {
            return Err(ConfigError::InvalidTimeout);
        }
        self.transmission_timeout_ms = timeout_ms;
        Ok(())
    }

    /// Set maximum retry attempts (with validation)
    pub fn set_max_retry_attempts(&mut self, attempts: u8) -> Result<(), ConfigError> {
        if attempts == 0 || attempts > 10 {
            return Err(ConfigError::InvalidRetryCount);
        }
        self.max_retry_attempts = attempts;
        Ok(())
    }

    /// Validate the current configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate log level doesn't exceed compile-time maximum
        if self.max_level as u8 > logging::MAX_LOG_LEVEL as u8 {
            return Err(ConfigError::InvalidLogLevel);
        }

        // Validate queue size
        if self.queue_size == 0 || self.queue_size > logging::MAX_QUEUE_SIZE {
            return Err(ConfigError::InvalidQueueSize);
        }

        // Validate USB IDs
        if self.usb_vid == 0 || self.usb_pid == 0 {
            return Err(ConfigError::InvalidUsbId);
        }

        // Validate timeout
        if self.transmission_timeout_ms == 0 || self.transmission_timeout_ms > 10000 {
            return Err(ConfigError::InvalidTimeout);
        }

        // Validate retry attempts
        if self.max_retry_attempts == 0 || self.max_retry_attempts > 10 {
            return Err(ConfigError::InvalidRetryCount);
        }

        // Validate category flags don't exceed compile-time capabilities
        if self.enable_battery_logs && !logging::ENABLE_BATTERY_LOGS {
            return Err(ConfigError::CategoryDisabledAtCompileTime);
        }
        if self.enable_pemf_logs && !logging::ENABLE_PEMF_LOGS {
            return Err(ConfigError::CategoryDisabledAtCompileTime);
        }
        if self.enable_system_logs && !logging::ENABLE_SYSTEM_LOGS {
            return Err(ConfigError::CategoryDisabledAtCompileTime);
        }
        if self.enable_usb_logs && !logging::ENABLE_USB_LOGS {
            return Err(ConfigError::CategoryDisabledAtCompileTime);
        }

        Ok(())
    }

    /// Serialize configuration to bytes for USB transmission
    pub fn serialize(&self) -> [u8; 16] {
        let mut buffer = [0u8; 16];
        
        buffer[0] = self.max_level as u8;
        buffer[1] = (self.queue_size & 0xFF) as u8;
        buffer[2] = ((self.queue_size >> 8) & 0xFF) as u8;
        buffer[3] = (self.usb_vid & 0xFF) as u8;
        buffer[4] = ((self.usb_vid >> 8) & 0xFF) as u8;
        buffer[5] = (self.usb_pid & 0xFF) as u8;
        buffer[6] = ((self.usb_pid >> 8) & 0xFF) as u8;
        
        // Pack boolean flags into a single byte
        let mut flags = 0u8;
        if self.enable_battery_logs { flags |= 0x01; }
        if self.enable_pemf_logs { flags |= 0x02; }
        if self.enable_system_logs { flags |= 0x04; }
        if self.enable_usb_logs { flags |= 0x08; }
        buffer[7] = flags;
        
        // Transmission timeout (4 bytes, little-endian)
        let timeout_bytes = self.transmission_timeout_ms.to_le_bytes();
        buffer[8..12].copy_from_slice(&timeout_bytes);
        
        buffer[12] = self.max_retry_attempts;
        // Bytes 13-15 reserved for future use
        
        buffer
    }

    /// Deserialize configuration from bytes received via USB
    pub fn deserialize(buffer: &[u8; 16]) -> Result<Self, ConfigError> {
        let max_level = match buffer[0] {
            0 => LogLevel::Debug,
            1 => LogLevel::Info,
            2 => LogLevel::Warn,
            3 => LogLevel::Error,
            _ => return Err(ConfigError::InvalidLogLevel),
        };

        let queue_size = (buffer[1] as usize) | ((buffer[2] as usize) << 8);
        let usb_vid = (buffer[3] as u16) | ((buffer[4] as u16) << 8);
        let usb_pid = (buffer[5] as u16) | ((buffer[6] as u16) << 8);
        
        let flags = buffer[7];
        let enable_battery_logs = (flags & 0x01) != 0;
        let enable_pemf_logs = (flags & 0x02) != 0;
        let enable_system_logs = (flags & 0x04) != 0;
        let enable_usb_logs = (flags & 0x08) != 0;
        
        let mut timeout_bytes = [0u8; 4];
        timeout_bytes.copy_from_slice(&buffer[8..12]);
        let transmission_timeout_ms = u32::from_le_bytes(timeout_bytes);
        
        let max_retry_attempts = buffer[12];

        let config = Self {
            max_level,
            queue_size,
            usb_vid,
            usb_pid,
            enable_battery_logs,
            enable_pemf_logs,
            enable_system_logs,
            enable_usb_logs,
            transmission_timeout_ms,
            max_retry_attempts,
        };

        // Validate the deserialized configuration
        config.validate()?;
        Ok(config)
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Logging categories for runtime control
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LogCategory {
    Battery,
    Pemf,
    System,
    Usb,
    General,
}

/// Configuration error types
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ConfigError {
    InvalidLogLevel,
    InvalidQueueSize,
    InvalidUsbId,
    InvalidTimeout,
    InvalidRetryCount,
    CategoryDisabledAtCompileTime,
    SerializationError,
    DeserializationError,
}

impl ConfigError {
    /// Get a human-readable error message
    pub fn as_str(&self) -> &'static str {
        match self {
            ConfigError::InvalidLogLevel => "Invalid log level",
            ConfigError::InvalidQueueSize => "Invalid queue size",
            ConfigError::InvalidUsbId => "Invalid USB VID/PID",
            ConfigError::InvalidTimeout => "Invalid timeout value",
            ConfigError::InvalidRetryCount => "Invalid retry count",
            ConfigError::CategoryDisabledAtCompileTime => "Category disabled at compile time",
            ConfigError::SerializationError => "Configuration serialization failed",
            ConfigError::DeserializationError => "Configuration deserialization failed",
        }
    }
}

/// USB HID control commands for runtime configuration
#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(test, derive(Debug))]
#[repr(u8)]
pub enum UsbControlCommand {
    GetConfig = 0x01,
    SetConfig = 0x02,
    SetLogLevel = 0x03,
    EnableCategory = 0x04,
    DisableCategory = 0x05,
    ResetConfig = 0x06,
    GetStats = 0x07,
}

impl UsbControlCommand {
    /// Convert from u8 to UsbControlCommand
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(UsbControlCommand::GetConfig),
            0x02 => Some(UsbControlCommand::SetConfig),
            0x03 => Some(UsbControlCommand::SetLogLevel),
            0x04 => Some(UsbControlCommand::EnableCategory),
            0x05 => Some(UsbControlCommand::DisableCategory),
            0x06 => Some(UsbControlCommand::ResetConfig),
            0x07 => Some(UsbControlCommand::GetStats),
            _ => None,
        }
    }
}

/// Configuration validation utilities
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate USB configuration constants
    pub fn validate_usb_config() -> bool {
        // Validate VID/PID are non-zero
        if usb::VENDOR_ID == 0 || usb::PRODUCT_ID == 0 {
            return false;
        }

        // Validate HID report size is reasonable
        if usb::HID_REPORT_SIZE < 8 || usb::HID_REPORT_SIZE > 256 {
            return false;
        }

        // Validate string lengths
        if usb::MANUFACTURER.len() > 32 || usb::PRODUCT.len() > 32 || usb::SERIAL_NUMBER.len() > 32
        {
            return false;
        }

        true
    }

    /// Validate logging configuration constants
    pub fn validate_logging_config() -> bool {
        // Validate queue sizes
        if logging::DEFAULT_QUEUE_SIZE == 0 || logging::DEFAULT_QUEUE_SIZE > logging::MAX_QUEUE_SIZE
        {
            return false;
        }

        // Validate timeout values
        if logging::TRANSMISSION_TIMEOUT_MS == 0 || logging::TRANSMISSION_TIMEOUT_MS > 10000 {
            return false;
        }

        // Validate retry attempts
        if logging::MAX_RETRY_ATTEMPTS == 0 || logging::MAX_RETRY_ATTEMPTS > 10 {
            return false;
        }

        true
    }

    /// Validate timing configuration constants
    pub fn validate_timing_config() -> bool {
        // Validate pEMF timing
        let total_period_ms = timing::PEMF_PULSE_HIGH_MS + timing::PEMF_PULSE_LOW_MS;
        let expected_period_ms = (1000.0 / timing::PEMF_FREQUENCY_HZ) as u64;

        if total_period_ms != expected_period_ms {
            return false;
        }

        // Validate battery monitoring timing
        let expected_battery_interval = (1000.0 / timing::BATTERY_SAMPLING_HZ) as u64;
        if timing::BATTERY_MONITOR_INTERVAL_MS != expected_battery_interval {
            return false;
        }

        // Validate LED flash timing
        let led_total_period = timing::LED_FLASH_ON_MS + timing::LED_FLASH_OFF_MS;
        let expected_led_period = (1000.0 / timing::LED_FLASH_FREQUENCY_HZ) as u64;

        if led_total_period != expected_led_period {
            return false;
        }

        // Validate timing tolerance
        if timing::TIMING_TOLERANCE_PERCENT <= 0.0 || timing::TIMING_TOLERANCE_PERCENT > 0.1 {
            return false;
        }

        true
    }

    /// Validate system configuration constants
    pub fn validate_system_config() -> bool {
        // Validate clock frequencies
        if system::SYSTEM_CLOCK_HZ == 0 || system::EXTERNAL_CRYSTAL_HZ == 0 {
            return false;
        }

        // Validate USB polling interval
        if system::USB_POLL_INTERVAL_MS == 0 || system::USB_POLL_INTERVAL_MS > 100 {
            return false;
        }

        // Validate HID task interval
        if system::HID_TASK_INTERVAL_MS == 0 || system::HID_TASK_INTERVAL_MS > 1000 {
            return false;
        }

        // Validate CPU usage limit
        if system::MAX_USB_CPU_USAGE_PERCENT == 0 || system::MAX_USB_CPU_USAGE_PERCENT > 50 {
            return false;
        }

        true
    }

    /// Validate all configuration constants
    pub fn validate_all_config() -> bool {
        Self::validate_usb_config()
            && Self::validate_logging_config()
            && Self::validate_timing_config()
            && Self::validate_system_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Removed unused imports

    #[test]
    fn test_usb_config_validation() {
        assert!(ConfigValidator::validate_usb_config());
        assert_ne!(usb::VENDOR_ID, 0);
        assert_ne!(usb::PRODUCT_ID, 0);
        assert!(usb::HID_REPORT_SIZE >= 8);
        assert!(usb::HID_REPORT_SIZE <= 256);
    }

    #[test]
    fn test_logging_config_validation() {
        assert!(ConfigValidator::validate_logging_config());
        assert!(logging::DEFAULT_QUEUE_SIZE > 0);
        assert!(logging::DEFAULT_QUEUE_SIZE <= logging::MAX_QUEUE_SIZE);
        assert!(logging::TRANSMISSION_TIMEOUT_MS > 0);
        assert!(logging::MAX_RETRY_ATTEMPTS > 0);
    }

    #[test]
    fn test_timing_config_validation() {
        assert!(ConfigValidator::validate_timing_config());

        // Test pEMF timing calculation
        let total_period = timing::PEMF_PULSE_HIGH_MS + timing::PEMF_PULSE_LOW_MS;
        let expected_period = (1000.0 / timing::PEMF_FREQUENCY_HZ) as u64;
        assert_eq!(total_period, expected_period);

        // Test battery monitoring timing
        let expected_battery_interval = (1000.0 / timing::BATTERY_SAMPLING_HZ) as u64;
        assert_eq!(
            timing::BATTERY_MONITOR_INTERVAL_MS,
            expected_battery_interval
        );
    }

    #[test]
    fn test_system_config_validation() {
        assert!(ConfigValidator::validate_system_config());
        assert!(system::SYSTEM_CLOCK_HZ > 0);
        assert!(system::EXTERNAL_CRYSTAL_HZ > 0);
        assert!(system::USB_POLL_INTERVAL_MS > 0);
        assert!(system::HID_TASK_INTERVAL_MS > 0);
    }

    #[test]
    fn test_all_config_validation() {
        assert!(ConfigValidator::validate_all_config());
    }

    #[test]
    fn test_priority_ordering() {
        // Ensure priorities are correctly ordered (higher number = higher priority)
        assert!(priorities::PEMF_PULSE_PRIORITY > priorities::BATTERY_MONITOR_PRIORITY);
        assert!(priorities::BATTERY_MONITOR_PRIORITY > priorities::LED_CONTROL_PRIORITY);
        assert!(priorities::LED_CONTROL_PRIORITY >= priorities::USB_HID_TASK_PRIORITY);
        assert!(priorities::USB_HID_TASK_PRIORITY > priorities::USB_POLL_TASK_PRIORITY);
    }

    #[test]
    fn test_feature_flags() {
        // Test that essential features are enabled
        assert!(features::ENABLE_USB_HID_LOGGING);
        assert!(features::ENABLE_LOG_TIMESTAMPS);
        assert!(features::ENABLE_LOG_MODULE_NAMES);
    }
}

