//! Library crate for ass-easy-loop embedded application
//!
//! This library provides the core functionality for the pEMF device
//! including battery state management, USB HID logging, and configuration.

#![cfg_attr(not(any(test, feature = "std-testing")), no_std)]
#![allow(dead_code)] // Allow unused code for development and testing

// Conditional compilation for testing
#[cfg(any(test, feature = "std-testing"))]
extern crate std;

// Re-export std types for testing when available
#[cfg(any(test, feature = "std-testing"))]
pub use std::{collections::HashMap, string::String, vec::Vec};

pub mod battery;
pub mod bootloader;
pub mod command;
pub mod config;
pub mod error_handling;
pub mod logging;
pub mod resource_management;
pub mod system_state;
pub mod test_execution_handler;
pub mod test_framework;
pub mod test_framework_validation;
pub mod test_performance_optimizer;
pub mod test_processor;
pub mod test_result_serializer;
pub mod types;

pub use battery::{AdcError, BatteryMonitor, BatteryState};
pub use bootloader::{
    get_bootloader_manager, init_bootloader_manager, mark_task_shutdown_complete,
    mark_task_shutdown_failed, should_task_shutdown, BootloaderEntryManager, BootloaderEntryState,
    BootloaderError, HardwareState, TaskPriority, TaskShutdownStatus,
};
pub use command::{handler, parsing::CommandParser};
pub use command::parsing::{
    AuthenticationValidator, CommandQueue, CommandReport, ErrorCode, ParseResult, QueuedCommand,
    QueuedResponse, ResponseQueue, TestCommand, TestResponse,
};
pub use config::*;
pub use error_handling::{ErrorRecovery, SystemError, SystemResult};
pub use logging::{
    LogLevel, LogMessage, LogQueue, Logger, MessageFormatter, QueueLogger, 
    init_global_config, init_global_performance_monitoring, get_global_config, update_global_config,
    record_timing_impact, get_global_performance_stats, init_global_logging, LogReport
};

// Logging macros are already exported at crate root due to #[macro_export] in logging.rs
pub use resource_management::{ResourceLeakDetector, ResourceValidator, SafeLoggingAccess};
pub use system_state::{
    ConfigurationData, HardwareStatusData, StateHandlerStats, StateQueryType, SystemHealthData,
    SystemStateHandler, TaskPerformanceData,
};
pub use test_execution_handler::{
    TestExecutionFlags, TestExecutionHandler, TestExecutionParams, TestExecutionStats,
    TestExecutionStatus,
};
pub use test_framework::{
    create_test_suite, TestCase, TestExecutionResult, TestResult as NoStdTestResult, TestRunner,
    TestSuiteResult, TestSuiteStats, MAX_ERROR_MESSAGE_LENGTH, MAX_NAME_LENGTH,
    MAX_TESTS_PER_SUITE, MAX_TEST_SUITES,
};
pub use test_framework_validation::validate_test_framework;
#[cfg(all(
    feature = "test-commands",
    not(feature = "exclude-test-infrastructure")
))]
pub use test_performance_optimizer::{
    OptimizationResult, OptimizationSettings, PerformanceStats, TestPerformanceOptimizer,
};
pub use test_processor::{
    ActiveTest, ComprehensiveTimingReport, PemfTimingParameters, PemfTimingStatistics,
    PerformanceMetrics, ResourceMonitor, ResourceUsageStats, TestCommandProcessor,
    TestExecutionError, TestMeasurements, TestParameterError, TestParameters,
    TestProcessorStatistics, TestResult, TestStatus, TestType, TimingDeviation,
    TimingDeviationReport, TimingDeviationType, TimingMeasurement, UsbCommunicationStatistics,
    UsbCommunicationTestParameters,
};
pub use test_result_serializer::{
    CollectorStats, SerializedSuiteSummary, SerializedTestResult, SerializerStats, TestReportType,
    TestResultCollector, TestResultSerializer, TestResultStatus,
};
pub use types::{
    AdcValue, Count, DurationMs, DurationUs, FrequencyHz, GpioPin, Index, MemoryBytes, Percentage,
    Priority, TimestampMs, UsbId, VoltageMillivolts,
};
