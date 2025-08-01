/// Error handling utilities for the pEMF/Battery Monitor system
/// Requirements: 7.1, 7.5 - Graceful error handling and error logging for debugging

use crate::logging::{LogLevel, log_message};
use heapless::String;
use core::fmt::Write;

/// System error types for graceful error handling
/// Requirements: 7.1 (graceful error handling for non-critical operations)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SystemError {
    /// ADC read operation failed
    AdcReadFailed,
    /// GPIO operation failed (non-critical)
    GpioOperationFailed,
}

impl SystemError {
    /// Get a human-readable description of the error
    pub fn description(&self) -> &'static str {
        match self {
            SystemError::AdcReadFailed => "ADC read operation failed",
            SystemError::GpioOperationFailed => "GPIO operation failed",
        }
    }

    /// Get the severity level of the error for logging
    pub fn severity(&self) -> LogLevel {
        match self {
            SystemError::AdcReadFailed => LogLevel::Error,
            SystemError::GpioOperationFailed => LogLevel::Warn,
        }
    }

    /// Determine if this error should cause a system panic
    /// Requirements: 7.1 (panic-halt for unrecoverable errors)
    pub fn is_critical(&self) -> bool {
        match self {
            SystemError::AdcReadFailed => false,        // Continue with last known value
            SystemError::GpioOperationFailed => false, // Log and continue
        }
    }
}

/// Result type for system operations that can fail gracefully
pub type SystemResult<T> = Result<T, SystemError>;

/// Error recovery strategies for different error types
/// Requirements: 7.1 (graceful error handling for non-critical operations)
pub struct ErrorRecovery;

impl ErrorRecovery {
    /// Handle a system error with appropriate recovery strategy
    /// Requirements: 7.5 (error logging for debugging purposes)
    pub fn handle_error(error: SystemError, context: &str) -> SystemResult<()> {
        // Log the error with context information
        let mut error_msg: String<64> = String::new();
        let _ = write!(&mut error_msg, "{}: {} in {}", error.severity().as_str(), error.description(), context);
        log_message(
            error.severity(),
            "ERROR_HANDLER",
            error_msg.as_str()
        );

        // Check if this is a critical error that requires panic
        if error.is_critical() {
            // For critical errors, we panic to ensure system safety
            panic!("Critical error: {} in {}", error.description(), context);
        }

        // For non-critical errors, implement recovery strategies
        match error {
            SystemError::AdcReadFailed => {
                log_message(LogLevel::Info, "ERROR_HANDLER", "ADC error recovery: using last known good value");
                // Recovery: Continue with last known good ADC value
                // The calling code should handle this by using cached values
                Ok(())
            }
            
            SystemError::GpioOperationFailed => {
                log_message(LogLevel::Info, "ERROR_HANDLER", "GPIO error recovery: operation will be retried");
                // Recovery: GPIO operations will be retried on next cycle
                // This is handled by the calling task's loop structure
                Ok(())
            }
        }
    }

    /// Retry an operation with exponential backoff
    /// Requirements: 7.1 (graceful error handling for non-critical operations)
    pub fn retry_with_backoff<T, F>(
        mut operation: F,
        max_retries: u8,
        context: &str,
    ) -> SystemResult<T>
    where
        F: FnMut() -> SystemResult<T>,
    {
        let mut attempts = 0;
        let mut last_error = SystemError::GpioOperationFailed;

        while attempts < max_retries {
            match operation() {
                Ok(result) => {
                    if attempts > 0 {
                        let mut success_msg: String<64> = String::new();
                        let _ = write!(&mut success_msg, "Operation succeeded after {} retries in {}", attempts, context);
                        log_message(
                            LogLevel::Info,
                            "ERROR_HANDLER",
                            success_msg.as_str()
                        );
                    }
                    return Ok(result);
                }
                Err(error) => {
                    last_error = error;
                    attempts += 1;

                    // Don't retry critical errors
                    if error.is_critical() {
                        return Self::handle_error(error, context).and(Err(error));
                    }

                    // Log retry attempt
                    if attempts < max_retries {
                        let mut retry_msg: String<64> = String::new();
                        let _ = write!(&mut retry_msg, "Retry {}/{} for {} in {}", attempts, max_retries, error.description(), context);
                        log_message(
                            LogLevel::Debug,
                            "ERROR_HANDLER",
                            retry_msg.as_str()
                        );

                        // Simple delay for backoff (in a real implementation, this would use proper timing)
                        for _ in 0..(1000 * attempts as u32) {
                            cortex_m::asm::nop();
                        }
                    }
                }
            }
        }

        // All retries exhausted
        let mut failure_msg: String<64> = String::new();
        let _ = write!(&mut failure_msg, "Operation failed after {} retries in {}: {}", max_retries, context, last_error.description());
        log_message(
            LogLevel::Error,
            "ERROR_HANDLER",
            failure_msg.as_str()
        );

        Self::handle_error(last_error, context).and(Err(last_error))
    }
}

/// Macro for safe error handling with automatic logging and recovery
/// Requirements: 7.5 (error logging for debugging purposes)
#[macro_export]
macro_rules! handle_error {
    ($result:expr, $context:expr) => {
        match $result {
            Ok(value) => Ok(value),
            Err(error) => {
                $crate::error_handling::ErrorRecovery::handle_error(error, $context)?;
                Err(error)
            }
        }
    };
}

/// Macro for operations that should continue on error with logging
/// Requirements: 7.1 (graceful error handling for non-critical operations)
#[macro_export]
macro_rules! continue_on_error {
    ($result:expr, $context:expr) => {
        match $result {
            Ok(value) => Some(value),
            Err(error) => {
                let _ = $crate::error_handling::ErrorRecovery::handle_error(error, $context);
                None
            }
        }
    };
}

/// Macro for operations that should retry on error
/// Requirements: 7.1 (graceful error handling for non-critical operations)
#[macro_export]
macro_rules! retry_on_error {
    ($operation:expr, $max_retries:expr, $context:expr) => {
        $crate::error_handling::ErrorRecovery::retry_with_backoff(
            || $operation,
            $max_retries,
            $context
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity() {
        assert_eq!(SystemError::AdcReadFailed.severity(), LogLevel::Error);
        assert_eq!(SystemError::GpioOperationFailed.severity(), LogLevel::Warn);
    }

    #[test]
    fn test_error_criticality() {
        assert!(!SystemError::AdcReadFailed.is_critical());
        assert!(!SystemError::GpioOperationFailed.is_critical());
    }

    #[test]
    fn test_error_descriptions() {
        assert_eq!(SystemError::AdcReadFailed.description(), "ADC read operation failed");
        assert_eq!(SystemError::GpioOperationFailed.description(), "GPIO operation failed");
    }
}