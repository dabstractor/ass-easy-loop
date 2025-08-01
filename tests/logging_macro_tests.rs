//! Tests for the compile-time and runtime configuration system
//! 
//! This test file validates the LogConfig struct, conditional compilation,
//! runtime configuration control, and USB control commands.

#![cfg(test)]

extern crate std;
use std::assert;
use std::assert_eq;
use std::assert_ne;

use ass_easy_loop::config::{LogConfig, LogCategory, ConfigError, UsbControlCommand};
use ass_easy_loop::logging::LogLevel;

#[test]
fn test_log_config_creation() {
    // Test default configuration creation
    let default_config = LogConfig::new();
    assert_eq!(default_config.max_level, LogLevel::Info); // Should match compile-time default
    assert_eq!(default_config.queue_size, 32);
    assert_eq!(default_config.usb_vid, 0x1234);
    assert_eq!(default_config.usb_pid, 0x5678);
    
    // Test debug configuration
    let debug_config = LogConfig::debug_config();
    assert_eq!(debug_config.max_level, LogLevel::Debug);
    assert_eq!(debug_config.queue_size, 128);
    assert!(debug_config.enable_battery_logs);
    assert!(debug_config.enable_pemf_logs);
    assert!(debug_config.enable_system_logs);
    assert!(debug_config.enable_usb_logs);
    
    // Test minimal configuration
    let minimal_config = LogConfig::minimal_config();
    assert_eq!(minimal_config.max_level, LogLevel::Error);
    assert_eq!(minimal_config.queue_size, 16);
    assert!(!minimal_config.enable_battery_logs);
    assert!(!minimal_config.enable_pemf_logs);
    assert!(!minimal_config.enable_system_logs);
    assert!(!minimal_config.enable_usb_logs);
}

#[test]
fn test_log_level_filtering() {
    let mut config = LogConfig::new();
    
    // Test compile-time level filtering (should always respect MAX_LOG_LEVEL)
    assert!(config.should_log(LogLevel::Error, LogCategory::General));
    assert!(config.should_log(LogLevel::Warn, LogCategory::General));
    assert!(config.should_log(LogLevel::Info, LogCategory::General));
    
    // Debug level depends on compile-time configuration
    #[cfg(debug_assertions)]
    assert!(config.should_log(LogLevel::Debug, LogCategory::General));
    #[cfg(not(debug_assertions))]
    assert!(!config.should_log(LogLevel::Debug, LogCategory::General));
    
    // Test runtime level filtering
    config.set_max_level(LogLevel::Warn).unwrap();
    assert!(config.should_log(LogLevel::Error, LogCategory::General));
    assert!(config.should_log(LogLevel::Warn, LogCategory::General));
    assert!(!config.should_log(LogLevel::Info, LogCategory::General));
    assert!(!config.should_log(LogLevel::Debug, LogCategory::General));
}

#[test]
fn test_category_filtering() {
    let mut config = LogConfig::new();
    
    // Test category-specific filtering
    config.enable_battery_logs = true;
    config.enable_pemf_logs = false;
    config.enable_system_logs = true;
    config.enable_usb_logs = false;
    
    // Battery logs should be enabled (both compile-time and runtime)
    assert_eq!(config.should_log(LogLevel::Info, LogCategory::Battery), 
               config.enable_battery_logs && ass_easy_loop::config::logging::ENABLE_BATTERY_LOGS);
    
    // pEMF logs should be disabled at runtime
    assert!(!config.should_log(LogLevel::Info, LogCategory::Pemf));
    
    // System logs should be enabled (both compile-time and runtime)
    assert_eq!(config.should_log(LogLevel::Info, LogCategory::System),
               config.enable_system_logs && ass_easy_loop::config::logging::ENABLE_SYSTEM_LOGS);
    
    // USB logs should be disabled at runtime
    assert!(!config.should_log(LogLevel::Info, LogCategory::Usb));
    
    // General logs should always be enabled if level passes
    assert!(config.should_log(LogLevel::Info, LogCategory::General));
}

#[test]
fn test_configuration_validation() {
    let mut config = LogConfig::new();
    
    // Valid configuration should pass validation
    assert!(config.validate().is_ok());
    
    // Test invalid log level (exceeding compile-time maximum)
    config.max_level = LogLevel::Debug;
    #[cfg(not(debug_assertions))]
    assert_eq!(config.validate(), Err(ConfigError::InvalidLogLevel));
    #[cfg(debug_assertions)]
    assert!(config.validate().is_ok());
    
    // Reset to valid level
    config.max_level = LogLevel::Info;
    
    // Test invalid queue size
    config.queue_size = 0;
    assert_eq!(config.validate(), Err(ConfigError::InvalidQueueSize));
    config.queue_size = 200; // Exceeds MAX_QUEUE_SIZE
    assert_eq!(config.validate(), Err(ConfigError::InvalidQueueSize));
    config.queue_size = 32; // Reset to valid
    
    // Test invalid USB IDs
    config.usb_vid = 0;
    assert_eq!(config.validate(), Err(ConfigError::InvalidUsbId));
    config.usb_vid = 0x1234; // Reset to valid
    config.usb_pid = 0;
    assert_eq!(config.validate(), Err(ConfigError::InvalidUsbId));
    config.usb_pid = 0x5678; // Reset to valid
    
    // Test invalid timeout
    config.transmission_timeout_ms = 0;
    assert_eq!(config.validate(), Err(ConfigError::InvalidTimeout));
    config.transmission_timeout_ms = 15000; // Too high
    assert_eq!(config.validate(), Err(ConfigError::InvalidTimeout));
    config.transmission_timeout_ms = 100; // Reset to valid
    
    // Test invalid retry count
    config.max_retry_attempts = 0;
    assert_eq!(config.validate(), Err(ConfigError::InvalidRetryCount));
    config.max_retry_attempts = 15; // Too high
    assert_eq!(config.validate(), Err(ConfigError::InvalidRetryCount));
    config.max_retry_attempts = 3; // Reset to valid
    
    // Configuration should be valid again
    assert!(config.validate().is_ok());
}

#[test]
fn test_configuration_setters() {
    let mut config = LogConfig::new();
    
    // Test log level setter
    assert!(config.set_max_level(LogLevel::Warn).is_ok());
    assert_eq!(config.max_level, LogLevel::Warn);
    
    // Test setting level beyond compile-time maximum
    #[cfg(not(debug_assertions))]
    assert_eq!(config.set_max_level(LogLevel::Debug), Err(ConfigError::InvalidLogLevel));
    
    // Test category setters
    assert!(config.set_battery_logs(false).is_ok());
    assert!(!config.enable_battery_logs);
    
    assert!(config.set_pemf_logs(false).is_ok());
    assert!(!config.enable_pemf_logs);
    
    assert!(config.set_system_logs(false).is_ok());
    assert!(!config.enable_system_logs);
    
    assert!(config.set_usb_logs(false).is_ok());
    assert!(!config.enable_usb_logs);
    
    // Test enabling categories that are disabled at compile time
    // This should fail if the feature is not enabled
    #[cfg(not(feature = "battery-logs"))]
    assert_eq!(config.set_battery_logs(true), Err(ConfigError::CategoryDisabledAtCompileTime));
    
    // Test timeout setter
    assert!(config.set_transmission_timeout(200).is_ok());
    assert_eq!(config.transmission_timeout_ms, 200);
    
    assert_eq!(config.set_transmission_timeout(0), Err(ConfigError::InvalidTimeout));
    assert_eq!(config.set_transmission_timeout(15000), Err(ConfigError::InvalidTimeout));
    
    // Test retry attempts setter
    assert!(config.set_max_retry_attempts(5).is_ok());
    assert_eq!(config.max_retry_attempts, 5);
    
    assert_eq!(config.set_max_retry_attempts(0), Err(ConfigError::InvalidRetryCount));
    assert_eq!(config.set_max_retry_attempts(15), Err(ConfigError::InvalidRetryCount));
}

#[test]
fn test_configuration_serialization() {
    let config = LogConfig::debug_config();
    
    // Test serialization
    let serialized = config.serialize();
    assert_eq!(serialized.len(), 16);
    
    // Test deserialization
    let deserialized = LogConfig::deserialize(&serialized).unwrap();
    assert_eq!(config.max_level, deserialized.max_level);
    assert_eq!(config.queue_size, deserialized.queue_size);
    assert_eq!(config.usb_vid, deserialized.usb_vid);
    assert_eq!(config.usb_pid, deserialized.usb_pid);
    assert_eq!(config.enable_battery_logs, deserialized.enable_battery_logs);
    assert_eq!(config.enable_pemf_logs, deserialized.enable_pemf_logs);
    assert_eq!(config.enable_system_logs, deserialized.enable_system_logs);
    assert_eq!(config.enable_usb_logs, deserialized.enable_usb_logs);
    assert_eq!(config.transmission_timeout_ms, deserialized.transmission_timeout_ms);
    assert_eq!(config.max_retry_attempts, deserialized.max_retry_attempts);
}

#[test]
fn test_invalid_serialization() {
    // Test deserialization with invalid data
    let mut invalid_data = [0u8; 16];
    
    // Invalid log level
    invalid_data[0] = 255;
    assert_eq!(LogConfig::deserialize(&invalid_data), Err(ConfigError::InvalidLogLevel));
    
    // Reset to valid log level
    invalid_data[0] = LogLevel::Info as u8;
    
    // Invalid queue size (0)
    invalid_data[1] = 0;
    invalid_data[2] = 0;
    assert_eq!(LogConfig::deserialize(&invalid_data), Err(ConfigError::InvalidQueueSize));
    
    // Invalid USB VID (0)
    invalid_data[1] = 32; // Valid queue size
    invalid_data[3] = 0;
    invalid_data[4] = 0;
    assert_eq!(LogConfig::deserialize(&invalid_data), Err(ConfigError::InvalidUsbId));
}

#[test]
fn test_usb_control_commands() {
    // Test command conversion from u8
    assert_eq!(UsbControlCommand::from_u8(0x01), Some(UsbControlCommand::GetConfig));
    assert_eq!(UsbControlCommand::from_u8(0x02), Some(UsbControlCommand::SetConfig));
    assert_eq!(UsbControlCommand::from_u8(0x03), Some(UsbControlCommand::SetLogLevel));
    assert_eq!(UsbControlCommand::from_u8(0x04), Some(UsbControlCommand::EnableCategory));
    assert_eq!(UsbControlCommand::from_u8(0x05), Some(UsbControlCommand::DisableCategory));
    assert_eq!(UsbControlCommand::from_u8(0x06), Some(UsbControlCommand::ResetConfig));
    assert_eq!(UsbControlCommand::from_u8(0x07), Some(UsbControlCommand::GetStats));
    
    // Test invalid command
    assert_eq!(UsbControlCommand::from_u8(0xFF), None);
    assert_eq!(UsbControlCommand::from_u8(0x00), None);
}

#[test]
fn test_config_error_messages() {
    // Test that all error types have meaningful messages
    assert_ne!(ConfigError::InvalidLogLevel.as_str(), "");
    assert_ne!(ConfigError::InvalidQueueSize.as_str(), "");
    assert_ne!(ConfigError::InvalidUsbId.as_str(), "");
    assert_ne!(ConfigError::InvalidTimeout.as_str(), "");
    assert_ne!(ConfigError::InvalidRetryCount.as_str(), "");
    assert_ne!(ConfigError::CategoryDisabledAtCompileTime.as_str(), "");
    assert_ne!(ConfigError::SerializationError.as_str(), "");
    assert_ne!(ConfigError::DeserializationError.as_str(), "");
    
    // Test specific error messages
    assert_eq!(ConfigError::InvalidLogLevel.as_str(), "Invalid log level");
    assert_eq!(ConfigError::CategoryDisabledAtCompileTime.as_str(), "Category disabled at compile time");
}

#[test]
fn test_log_category_enum() {
    // Test that all log categories are distinct
    assert_ne!(LogCategory::Battery, LogCategory::Pemf);
    assert_ne!(LogCategory::Battery, LogCategory::System);
    assert_ne!(LogCategory::Battery, LogCategory::Usb);
    assert_ne!(LogCategory::Battery, LogCategory::General);
    
    assert_ne!(LogCategory::Pemf, LogCategory::System);
    assert_ne!(LogCategory::Pemf, LogCategory::Usb);
    assert_ne!(LogCategory::Pemf, LogCategory::General);
    
    assert_ne!(LogCategory::System, LogCategory::Usb);
    assert_ne!(LogCategory::System, LogCategory::General);
    
    assert_ne!(LogCategory::Usb, LogCategory::General);
}

#[test]
fn test_compile_time_feature_flags() {
    // Test that compile-time feature flags are properly configured
    // These tests will pass/fail based on which features are enabled during compilation
    
    #[cfg(feature = "battery-logs")]
    assert!(ass_easy_loop::config::logging::ENABLE_BATTERY_LOGS);
    #[cfg(not(feature = "battery-logs"))]
    assert!(!ass_easy_loop::config::logging::ENABLE_BATTERY_LOGS);
    
    #[cfg(feature = "pemf-logs")]
    assert!(ass_easy_loop::config::logging::ENABLE_PEMF_LOGS);
    #[cfg(not(feature = "pemf-logs"))]
    assert!(!ass_easy_loop::config::logging::ENABLE_PEMF_LOGS);
    
    #[cfg(feature = "system-logs")]
    assert!(ass_easy_loop::config::logging::ENABLE_SYSTEM_LOGS);
    #[cfg(not(feature = "system-logs"))]
    assert!(!ass_easy_loop::config::logging::ENABLE_SYSTEM_LOGS);
    
    #[cfg(feature = "usb-logs")]
    assert!(ass_easy_loop::config::logging::ENABLE_USB_LOGS);
    #[cfg(not(feature = "usb-logs"))]
    assert!(!ass_easy_loop::config::logging::ENABLE_USB_LOGS);
}

#[test]
fn test_configuration_combinations() {
    // Test various configuration combinations to ensure they work correctly
    
    // Test production-like configuration
    let mut prod_config = LogConfig::minimal_config();
    prod_config.set_battery_logs(true).ok(); // Enable only essential logging
    prod_config.set_system_logs(true).ok();
    
    assert_eq!(prod_config.max_level, LogLevel::Error);
    assert!(!prod_config.enable_pemf_logs);
    assert!(!prod_config.enable_usb_logs);
    
    // Test development configuration
    let dev_config = LogConfig::debug_config();
    assert_eq!(dev_config.max_level, LogLevel::Debug);
    assert!(dev_config.enable_battery_logs);
    assert!(dev_config.enable_pemf_logs);
    assert!(dev_config.enable_system_logs);
    assert!(dev_config.enable_usb_logs);
    
    // Test selective category configuration
    let mut selective_config = LogConfig::new();
    selective_config.set_battery_logs(true).ok();
    selective_config.set_pemf_logs(false).ok();
    selective_config.set_system_logs(true).ok();
    selective_config.set_usb_logs(false).ok();
    
    // Verify selective filtering works
    assert!(selective_config.should_log(LogLevel::Info, LogCategory::General));
    assert_eq!(selective_config.should_log(LogLevel::Info, LogCategory::Battery),
               selective_config.enable_battery_logs && ass_easy_loop::config::logging::ENABLE_BATTERY_LOGS);
    assert!(!selective_config.should_log(LogLevel::Info, LogCategory::Pemf));
    assert_eq!(selective_config.should_log(LogLevel::Info, LogCategory::System),
               selective_config.enable_system_logs && ass_easy_loop::config::logging::ENABLE_SYSTEM_LOGS);
    assert!(!selective_config.should_log(LogLevel::Info, LogCategory::Usb));
}