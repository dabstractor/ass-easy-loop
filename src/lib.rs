//! Library crate for ass-easy-loop embedded application
//! 
//! This library provides the core functionality for the pEMF device
//! including battery state management, USB HID logging, and configuration.

#![cfg_attr(not(test), no_std)]
#![allow(dead_code)] // Allow unused code for development and testing

pub mod battery;
pub mod logging;
pub mod config;
pub mod error_handling;
pub mod resource_management;
pub mod command;
pub mod bootloader;
pub mod system_state;
pub mod test_processor;

pub use battery::{BatteryState, BatteryMonitor, AdcError};
pub use logging::{LogLevel, LogMessage, LogQueue, Logger, QueueLogger, MessageFormatter};
pub use config::*;
pub use error_handling::{SystemError, SystemResult, ErrorRecovery};
pub use resource_management::{ResourceValidator, SafeLoggingAccess, ResourceLeakDetector};
pub use command::handler;
pub use command::parsing::{
    CommandReport, ParseResult, CommandQueue, ResponseQueue, QueuedCommand, QueuedResponse,
    TestCommand, TestResponse, ErrorCode, AuthenticationValidator
};
pub use bootloader::{
    BootloaderEntryManager, BootloaderError, TaskPriority, TaskShutdownStatus, 
    HardwareState, BootloaderEntryState, init_bootloader_manager, get_bootloader_manager,
    should_task_shutdown, mark_task_shutdown_complete, mark_task_shutdown_failed
};
pub use system_state::{
    SystemStateHandler, StateQueryType, SystemHealthData, TaskPerformanceData,
    HardwareStatusData, ConfigurationData, StateHandlerStats
};
pub use test_processor::{
    TestCommandProcessor, TestType, TestStatus, TestParameters, TestResult,
    TestMeasurements, ResourceUsageStats, PerformanceMetrics, TestProcessorStatistics,
    ActiveTest, ResourceMonitor, TestExecutionError, TestParameterError,
    UsbCommunicationTestParameters, UsbCommunicationStatistics, TimingMeasurement,
    PemfTimingStatistics, PemfTimingParameters, TimingDeviation, TimingDeviationReport,
    TimingDeviationType, ComprehensiveTimingReport
};