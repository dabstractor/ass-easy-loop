//! Integration tests for the compile-time and runtime configuration system
//!
//! This test file validates the integration between the configuration system
//! and the actual logging functionality, including macro behavior with
//! different configuration settings.

#![cfg(test)]

extern crate std;
use std::assert;
use std::assert_eq;

use ass_easy_loop::config::{ConfigError, LogCategory, LogConfig};
use ass_easy_loop::logging::{LogLevel, LogMessage, LogQueue, Logger, QueueLogger};

/// Test helper function to create a mock timestamp function
fn mock_timestamp() -> u32 {
    12345
}

#[test]
fn test_queue_logger_with_config() {
    let mut logger = QueueLogger::<16>::new(mock_timestamp);

    // Test basic logging functionality
    logger.log(LogLevel::Info, "TEST", "Test message");
    logger.log(LogLevel::Error, "TEST", "Error message");
    logger.log(LogLevel::Debug, "TEST", "Debug message");

    let queue = logger.queue();

    // Should have 3 messages (or 2 if debug is filtered out at compile time)
    #[cfg(debug_assertions)]
    assert_eq!(queue.len(), 3);
    #[cfg(not(debug_assertions))]
    assert_eq!(queue.len(), 2); // Debug message filtered out

    // Test message content
    if let Some(msg) = queue.dequeue() {
        assert_eq!(msg.level, LogLevel::Info);
        assert_eq!(msg.module_str(), "TEST");
        assert_eq!(msg.message_str(), "Test message");
        assert_eq!(msg.timestamp, 12345);
    }

    if let Some(msg) = queue.dequeue() {
        assert_eq!(msg.level, LogLevel::Error);
        assert_eq!(msg.module_str(), "TEST");
        assert_eq!(msg.message_str(), "Error message");
    }

    // Check if debug message exists (depends on compile-time configuration)
    #[cfg(debug_assertions)]
    {
        if let Some(msg) = queue.dequeue() {
            assert_eq!(msg.level, LogLevel::Debug);
            assert_eq!(msg.module_str(), "TEST");
            assert_eq!(msg.message_str(), "Debug message");
        }
    }
}

#[test]
fn test_log_message_serialization_with_config() {
    let config = LogConfig::debug_config();

    // Create a test log message
    let message = LogMessage::new(
        config.transmission_timeout_ms as u32,
        LogLevel::Warn,
        "CONFIG",
        "Configuration test message",
    );

    // Test serialization
    let serialized = message.serialize();
    assert_eq!(serialized.len(), 64);

    // Test deserialization
    let deserialized = LogMessage::deserialize(&serialized).unwrap();
    assert_eq!(message.timestamp, deserialized.timestamp);
    assert_eq!(message.level, deserialized.level);
    assert_eq!(message.module_str(), deserialized.module_str());
    assert_eq!(message.message_str(), deserialized.message_str());
}

#[test]
fn test_configuration_affects_logging() {
    // Test that configuration changes affect logging behavior
    let mut config = LogConfig::new();

    // Test initial configuration allows info messages
    assert!(config.should_log(LogLevel::Info, LogCategory::General));
    assert!(config.should_log(LogLevel::Error, LogCategory::General));

    // Change to more restrictive level
    config.set_max_level(LogLevel::Error).unwrap();
    assert!(!config.should_log(LogLevel::Info, LogCategory::General));
    assert!(!config.should_log(LogLevel::Warn, LogCategory::General));
    assert!(config.should_log(LogLevel::Error, LogCategory::General));

    // Test category-specific filtering
    // Only test categories that are enabled at compile time
    #[cfg(feature = "battery-logs")]
    {
        config.set_battery_logs(false).unwrap();
        assert!(!config.should_log(LogLevel::Error, LogCategory::Battery));
    }

    #[cfg(feature = "system-logs")]
    {
        config.set_system_logs(true).unwrap();
        assert_eq!(
            config.should_log(LogLevel::Error, LogCategory::System),
            config.enable_system_logs && ass_easy_loop::config::logging::ENABLE_SYSTEM_LOGS
        );
    }

    #[cfg(not(feature = "system-logs"))]
    {
        // When system-logs feature is disabled, setting it should fail
        assert_eq!(
            config.set_system_logs(true),
            Err(ass_easy_loop::config::ConfigError::CategoryDisabledAtCompileTime)
        );
    }
}

#[test]
fn test_queue_behavior_with_different_configs() {
    // Test queue behavior with different configuration sizes
    let small_config = LogConfig::minimal_config();
    let large_config = LogConfig::debug_config();

    assert_eq!(small_config.queue_size, 16);
    assert_eq!(large_config.queue_size, 128);

    // Test that configurations are valid
    assert!(small_config.validate().is_ok());
    assert!(large_config.validate().is_ok());

    // Test timeout differences
    assert_eq!(small_config.transmission_timeout_ms, 50);
    assert_eq!(large_config.transmission_timeout_ms, 100);

    // Test retry attempt differences
    assert_eq!(small_config.max_retry_attempts, 1);
    assert_eq!(large_config.max_retry_attempts, 3);
}

#[test]
fn test_config_validation_edge_cases() {
    let mut config = LogConfig::new();

    // Test boundary values for queue size
    config.queue_size = 1; // Minimum valid size
    assert!(config.validate().is_ok());

    config.queue_size = 128; // Maximum valid size
    assert!(config.validate().is_ok());

    config.queue_size = 129; // Just over maximum
    assert_eq!(config.validate(), Err(ConfigError::InvalidQueueSize));

    // Reset to valid
    config.queue_size = 32;

    // Test boundary values for timeout
    config.transmission_timeout_ms = 1; // Minimum valid timeout
    assert!(config.validate().is_ok());

    config.transmission_timeout_ms = 10000; // Maximum valid timeout
    assert!(config.validate().is_ok());

    config.transmission_timeout_ms = 10001; // Just over maximum
    assert_eq!(config.validate(), Err(ConfigError::InvalidTimeout));

    // Reset to valid
    config.transmission_timeout_ms = 100;

    // Test boundary values for retry attempts
    config.max_retry_attempts = 1; // Minimum valid attempts
    assert!(config.validate().is_ok());

    config.max_retry_attempts = 10; // Maximum valid attempts
    assert!(config.validate().is_ok());

    config.max_retry_attempts = 11; // Just over maximum
    assert_eq!(config.validate(), Err(ConfigError::InvalidRetryCount));
}

#[test]
fn test_compile_time_vs_runtime_filtering() {
    let mut config = LogConfig::debug_config();

    // Test that runtime configuration cannot exceed compile-time limits
    #[cfg(not(debug_assertions))]
    {
        // In release mode, debug level should not be allowed
        assert_eq!(
            config.set_max_level(LogLevel::Debug),
            Err(ConfigError::InvalidLogLevel)
        );

        // But info level should be allowed
        assert!(config.set_max_level(LogLevel::Info).is_ok());
    }

    #[cfg(debug_assertions)]
    {
        // In debug mode, debug level should be allowed
        assert!(config.set_max_level(LogLevel::Debug).is_ok());
    }

    // Test category compile-time vs runtime interaction
    #[cfg(not(feature = "battery-logs"))]
    {
        // If battery logs are disabled at compile time, runtime enable should fail
        assert_eq!(
            config.set_battery_logs(true),
            Err(ConfigError::CategoryDisabledAtCompileTime)
        );
    }

    #[cfg(feature = "battery-logs")]
    {
        // If battery logs are enabled at compile time, runtime control should work
        assert!(config.set_battery_logs(true).is_ok());
        assert!(config.set_battery_logs(false).is_ok());
    }
}

#[test]
fn test_configuration_persistence() {
    // Test that configuration changes persist correctly
    let mut config = LogConfig::new();
    let original_level = config.max_level;

    // Change configuration
    config.set_max_level(LogLevel::Error).unwrap();
    config.set_battery_logs(false).unwrap();
    config.set_transmission_timeout(200).unwrap();
    config.set_max_retry_attempts(5).unwrap();

    // Verify changes persisted
    assert_ne!(config.max_level, original_level);
    assert_eq!(config.max_level, LogLevel::Error);
    assert!(!config.enable_battery_logs);
    assert_eq!(config.transmission_timeout_ms, 200);
    assert_eq!(config.max_retry_attempts, 5);

    // Test serialization preserves changes
    let serialized = config.serialize();
    let deserialized = LogConfig::deserialize(&serialized).unwrap();

    assert_eq!(config.max_level, deserialized.max_level);
    assert_eq!(config.enable_battery_logs, deserialized.enable_battery_logs);
    assert_eq!(
        config.transmission_timeout_ms,
        deserialized.transmission_timeout_ms
    );
    assert_eq!(config.max_retry_attempts, deserialized.max_retry_attempts);
}

#[test]
fn test_error_recovery() {
    // Test that the system can recover from configuration errors
    let mut config = LogConfig::new();

    // Introduce an invalid configuration
    config.queue_size = 0; // Invalid
    assert_eq!(config.validate(), Err(ConfigError::InvalidQueueSize));

    // Reset to default should fix the error
    config = LogConfig::new();
    assert!(config.validate().is_ok());

    // Test recovery from serialization errors
    let mut invalid_data = [0u8; 16];
    invalid_data[0] = 255; // Invalid log level

    assert_eq!(
        LogConfig::deserialize(&invalid_data),
        Err(ConfigError::InvalidLogLevel)
    );

    // But valid data should still work
    let valid_config = LogConfig::new();
    let valid_data = valid_config.serialize();
    assert!(LogConfig::deserialize(&valid_data).is_ok());
}

#[test]
fn test_feature_flag_consistency() {
    // Test that feature flags are consistent between compile-time and runtime
    let config = LogConfig::new();

    // Runtime flags should not exceed compile-time capabilities
    assert_eq!(
        config.enable_battery_logs,
        config.enable_battery_logs && ass_easy_loop::config::logging::ENABLE_BATTERY_LOGS
    );
    assert_eq!(
        config.enable_pemf_logs,
        config.enable_pemf_logs && ass_easy_loop::config::logging::ENABLE_PEMF_LOGS
    );
    assert_eq!(
        config.enable_system_logs,
        config.enable_system_logs && ass_easy_loop::config::logging::ENABLE_SYSTEM_LOGS
    );
    assert_eq!(
        config.enable_usb_logs,
        config.enable_usb_logs && ass_easy_loop::config::logging::ENABLE_USB_LOGS
    );
}

#[test]
fn test_configuration_defaults() {
    // Test that default configuration is sensible and valid
    let default_config = LogConfig::default();
    let new_config = LogConfig::new();

    // Default and new should be identical
    assert_eq!(default_config.max_level, new_config.max_level);
    assert_eq!(default_config.queue_size, new_config.queue_size);
    assert_eq!(default_config.usb_vid, new_config.usb_vid);
    assert_eq!(default_config.usb_pid, new_config.usb_pid);
    assert_eq!(
        default_config.enable_battery_logs,
        new_config.enable_battery_logs
    );
    assert_eq!(default_config.enable_pemf_logs, new_config.enable_pemf_logs);
    assert_eq!(
        default_config.enable_system_logs,
        new_config.enable_system_logs
    );
    assert_eq!(default_config.enable_usb_logs, new_config.enable_usb_logs);

    // Default configuration should be valid
    assert!(default_config.validate().is_ok());

    // Default configuration should have reasonable values
    assert!(default_config.queue_size > 0);
    assert!(default_config.queue_size <= 128);
    assert!(default_config.transmission_timeout_ms > 0);
    assert!(default_config.transmission_timeout_ms <= 10000);
    assert!(default_config.max_retry_attempts > 0);
    assert!(default_config.max_retry_attempts <= 10);
    assert_ne!(default_config.usb_vid, 0);
    assert_ne!(default_config.usb_pid, 0);
}
