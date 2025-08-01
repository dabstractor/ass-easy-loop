//! Library crate for ass-easy-loop embedded application
//! 
//! This library provides the core functionality for the pEMF device
//! including battery state management, USB HID logging, and configuration.

#![cfg_attr(not(test), no_std)]

pub mod battery;
pub mod logging;
pub mod config;
pub mod error_handling;
pub mod resource_management;

pub use battery::{BatteryState, BatteryMonitor, AdcError};
pub use logging::{LogLevel, LogMessage, LogQueue, Logger, QueueLogger, MessageFormatter};
pub use config::*;
pub use error_handling::{SystemError, SystemResult, ErrorRecovery};
pub use resource_management::{ResourceValidator, SafeLoggingAccess, ResourceLeakDetector};