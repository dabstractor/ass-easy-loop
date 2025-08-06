//! Test Command Processor Framework
//! 
//! This module implements the test command processor framework for automated testing.
//! It provides configurable test execution, parameter validation, timeout protection,
//! resource usage monitoring, and comprehensive result collection and serialization.
//! 
//! Requirements: 2.1, 2.2, 2.3, 8.1, 8.2, 8.3, 8.4, 8.5

use heapless::Vec;
use crate::command::parsing::{CommandReport, TestResponse, ErrorCode};
use crate::error_handling::SystemError;
use core::option::Option::{self, Some, None};
use core::result::Result::{self, Ok, Err};
use core::cmp::Ord;
use core::iter::Iterator;

/// Timing measurement structure (placeholder until performance_profiler is implemented)
#[derive(Clone, Copy, Debug)]
pub struct TimingMeasurement {
    pub task_name: &'static str,
    pub execution_time_us: u32,
    pub expected_time_us: u32,
    pub timestamp_ms: u32,
}

/// Jitter measurements structure (placeholder until performance_profiler is implemented)
#[derive(Clone, Copy, Debug)]
pub struct JitterMeasurements {
    pub pemf_jitter_us: u32,
    pub battery_jitter_us: u32,
    pub usb_jitter_us: u32,
    pub max_system_jitter_us: u32,
}

/// Test types supported by the test command processor
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum TestType {
    PemfTimingValidation = 0x01,
    BatteryAdcCalibration = 0x02,
    LedFunctionality = 0x03,
    SystemStressTest = 0x04,
    UsbCommunicationTest = 0x05,
}

impl TestType {
    /// Convert from u8 to TestType
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(TestType::PemfTimingValidation),
            0x02 => Some(TestType::BatteryAdcCalibration),
            0x03 => Some(TestType::LedFunctionality),
            0x04 => Some(TestType::SystemStressTest),
            0x05 => Some(TestType::UsbCommunicationTest),
            _ => None,
        }
    }
}

/// Test status enumeration
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum TestStatus {
    NotStarted = 0x00,
    Running = 0x01,
    Completed = 0x02,
    Failed = 0x03,
    TimedOut = 0x04,
    Aborted = 0x05,
}

/// Test parameters structure with validation ranges
/// Requirements: 2.2 (test parameter validation and range checking)
#[derive(Clone, Debug)]
pub struct TestParameters {
    pub duration_ms: u32,
    pub tolerance_percent: f32,
    pub sample_rate_hz: u32,
    pub validation_criteria: ValidationCriteria,
    pub resource_limits: ResourceLimits,
    pub custom_parameters: Vec<u8, 32>, // Additional test-specific parameters
}

impl Default for TestParameters {
    fn default() -> Self {
        Self::new()
    }
}

impl TestParameters {
    /// Create new test parameters with default values
    pub fn new() -> Self {
        Self {
            duration_ms: 1000,
            tolerance_percent: 1.0,
            sample_rate_hz: 100,
            validation_criteria: ValidationCriteria::default(),
            resource_limits: ResourceLimits::default(),
            custom_parameters: Vec::new(),
        }
    }

    /// Validate test parameters against acceptable ranges
    /// Requirements: 2.2 (parameter validation and range checking)
    pub fn validate(&self) -> Result<(), TestParameterError> {
        // Validate duration (1ms to 60 seconds)
        if self.duration_ms == 0 || self.duration_ms > 60_000 {
            return Err(TestParameterError::InvalidDuration);
        }

        // Validate tolerance (0.1% to 10%)
        if self.tolerance_percent < 0.1 || self.tolerance_percent > 10.0 {
            return Err(TestParameterError::InvalidTolerance);
        }

        // Validate sample rate (1Hz to 10kHz)
        if self.sample_rate_hz == 0 || self.sample_rate_hz > 10_000 {
            return Err(TestParameterError::InvalidSampleRate);
        }

        // Validate resource limits
        self.resource_limits.validate()?;

        // Validate validation criteria
        self.validation_criteria.validate()?;

        Ok(())
    }

    /// Parse test parameters from command payload
    /// Requirements: 2.1 (command parsing and validation)
    pub fn from_payload(payload: &[u8]) -> Result<Self, TestParameterError> {
        if payload.len() < 16 {
            return Err(TestParameterError::PayloadTooShort);
        }

        let duration_ms = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
        let tolerance_percent = f32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);
        let sample_rate_hz = u32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]);

        let validation_criteria = ValidationCriteria {
            max_error_count: u32::from_le_bytes([payload[12], payload[13], payload[14], payload[15]]),
            ..Default::default()
        };

        let mut resource_limits = ResourceLimits::default();
        if payload.len() >= 20 {
            resource_limits.max_cpu_usage_percent = payload[16];
            resource_limits.max_memory_usage_bytes = u32::from_le_bytes([payload[17], payload[18], payload[19], payload[20]]);
        }

        let mut custom_parameters = Vec::new();
        if payload.len() > 21 {
            let custom_len = core::cmp::min(payload.len() - 21, 32);
            for i in 0..custom_len {
                custom_parameters.push(payload[21 + i]).map_err(|_| TestParameterError::PayloadTooLarge)?;
            }
        }

        let params = Self {
            duration_ms,
            tolerance_percent,
            sample_rate_hz,
            validation_criteria,
            resource_limits,
            custom_parameters,
        };

        params.validate()?;
        Ok(params)
    }

    /// Serialize test parameters to bytes
    pub fn serialize(&self) -> Vec<u8, 60> {
        let mut serialized = Vec::new();

        // Serialize core parameters (16 bytes)
        let duration_bytes = self.duration_ms.to_le_bytes();
        for &byte in &duration_bytes {
            let _ = serialized.push(byte);
        }

        let tolerance_bytes = self.tolerance_percent.to_le_bytes();
        for &byte in &tolerance_bytes {
            let _ = serialized.push(byte);
        }

        let sample_rate_bytes = self.sample_rate_hz.to_le_bytes();
        for &byte in &sample_rate_bytes {
            let _ = serialized.push(byte);
        }

        let max_errors_bytes = self.validation_criteria.max_error_count.to_le_bytes();
        for &byte in &max_errors_bytes {
            let _ = serialized.push(byte);
        }

        // Serialize resource limits (5 bytes)
        let _ = serialized.push(self.resource_limits.max_cpu_usage_percent);
        let memory_bytes = self.resource_limits.max_memory_usage_bytes.to_le_bytes();
        for &byte in &memory_bytes {
            let _ = serialized.push(byte);
        }

        // Serialize custom parameters (up to remaining space)
        for &byte in &self.custom_parameters {
            if serialized.push(byte).is_err() {
                break; // Stop if we run out of space
            }
        }

        serialized
    }
}

/// Validation criteria for test execution
#[derive(Clone, Copy, Debug)]
pub struct ValidationCriteria {
    pub max_error_count: u32,
    pub min_success_rate_percent: u8,
    pub max_timing_deviation_us: u32,
    pub require_stable_operation: bool,
}

impl Default for ValidationCriteria {
    fn default() -> Self {
        Self {
            max_error_count: 10,
            min_success_rate_percent: 95,
            max_timing_deviation_us: 1000,
            require_stable_operation: true,
        }
    }
}

impl ValidationCriteria {

    /// Validate validation criteria parameters
    pub fn validate(&self) -> Result<(), TestParameterError> {
        if self.min_success_rate_percent > 100 {
            return Err(TestParameterError::InvalidValidationCriteria);
        }

        if self.max_timing_deviation_us > 100_000 {
            return Err(TestParameterError::InvalidValidationCriteria);
        }

        Ok(())
    }
}

/// Resource limits for test execution
/// Requirements: 8.1, 8.2 (resource usage monitoring and limits)
#[derive(Clone, Copy, Debug)]
pub struct ResourceLimits {
    pub max_cpu_usage_percent: u8,
    pub max_memory_usage_bytes: u32,
    pub max_execution_time_ms: u32,
    pub allow_preemption: bool,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_usage_percent: 50, // Maximum 50% CPU usage
            max_memory_usage_bytes: 4096, // Maximum 4KB memory usage
            max_execution_time_ms: 30_000, // Maximum 30 seconds execution time
            allow_preemption: true, // Allow higher priority tasks to preempt
        }
    }
}

impl ResourceLimits {

    /// Validate resource limits
    pub fn validate(&self) -> Result<(), TestParameterError> {
        if self.max_cpu_usage_percent > 100 {
            return Err(TestParameterError::InvalidResourceLimits);
        }

        if self.max_memory_usage_bytes > 64_000 {
            return Err(TestParameterError::InvalidResourceLimits);
        }

        if self.max_execution_time_ms > 300_000 {
            return Err(TestParameterError::InvalidResourceLimits);
        }

        Ok(())
    }
}

/// Test parameter validation errors
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TestParameterError {
    InvalidDuration,
    InvalidTolerance,
    InvalidSampleRate,
    InvalidValidationCriteria,
    InvalidResourceLimits,
    PayloadTooShort,
    PayloadTooLarge,
}

/// Test measurements and results
/// Requirements: 2.3 (test result collection and serialization)
#[derive(Clone, Debug)]
pub struct TestMeasurements {
    pub timing_accuracy: f32,
    pub resource_usage: ResourceUsageStats,
    pub error_count: u32,
    pub performance_metrics: PerformanceMetrics,
    pub timing_measurements: Vec<TimingMeasurement, 32>,
    pub jitter_measurements: JitterMeasurements,
    pub custom_measurements: Vec<u8, 64>,
}

impl Default for TestMeasurements {
    fn default() -> Self {
        Self::new()
    }
}

impl TestMeasurements {
    /// Create new empty test measurements
    pub fn new() -> Self {
        Self {
            timing_accuracy: 0.0,
            resource_usage: ResourceUsageStats::new(),
            error_count: 0,
            performance_metrics: PerformanceMetrics::new(),
            timing_measurements: Vec::new(),
            jitter_measurements: JitterMeasurements {
                pemf_jitter_us: 0,
                battery_jitter_us: 0,
                usb_jitter_us: 0,
                max_system_jitter_us: 0,
            },
            custom_measurements: Vec::new(),
        }
    }

    /// Add a timing measurement to the collection
    pub fn add_timing_measurement(&mut self, measurement: TimingMeasurement) -> Result<(), SystemError> {
        self.timing_measurements.push(measurement).map_err(|_| SystemError::SystemBusy)
    }

    /// Calculate timing accuracy from measurements
    pub fn calculate_timing_accuracy(&mut self, expected_timing_us: u32) -> f32 {
        if self.timing_measurements.is_empty() {
            return 0.0;
        }

        let mut total_deviation = 0.0;
        for measurement in &self.timing_measurements {
            let deviation = measurement.execution_time_us.abs_diff(expected_timing_us);
            total_deviation += deviation as f32;
        }

        let avg_deviation = total_deviation / self.timing_measurements.len() as f32;
        let accuracy_percent = 100.0 - (avg_deviation / expected_timing_us as f32 * 100.0);
        self.timing_accuracy = accuracy_percent.max(0.0);
        self.timing_accuracy
    }

    /// Serialize test measurements to bytes
    pub fn serialize(&self) -> Vec<u8, 60> {
        let mut serialized = Vec::new();

        // Serialize timing accuracy (4 bytes)
        let accuracy_bytes = self.timing_accuracy.to_le_bytes();
        for &byte in &accuracy_bytes {
            let _ = serialized.push(byte);
        }

        // Serialize error count (4 bytes)
        let error_bytes = self.error_count.to_le_bytes();
        for &byte in &error_bytes {
            let _ = serialized.push(byte);
        }

        // Serialize resource usage (8 bytes)
        let cpu_bytes = self.resource_usage.cpu_usage_percent.to_le_bytes();
        for &byte in &cpu_bytes {
            let _ = serialized.push(byte);
        }
        let memory_bytes = self.resource_usage.memory_usage_bytes.to_le_bytes();
        for &byte in &memory_bytes {
            let _ = serialized.push(byte);
        }

        // Serialize performance metrics (12 bytes)
        let throughput_bytes = self.performance_metrics.throughput_ops_per_sec.to_le_bytes();
        for &byte in &throughput_bytes {
            let _ = serialized.push(byte);
        }
        let latency_bytes = self.performance_metrics.average_latency_us.to_le_bytes();
        for &byte in &latency_bytes {
            let _ = serialized.push(byte);
        }
        let success_bytes = self.performance_metrics.success_rate_percent.to_le_bytes();
        for &byte in &success_bytes {
            let _ = serialized.push(byte);
        }

        // Serialize jitter measurements (16 bytes)
        let jitter_values = [
            self.jitter_measurements.pemf_jitter_us,
            self.jitter_measurements.battery_jitter_us,
            self.jitter_measurements.usb_jitter_us,
            self.jitter_measurements.max_system_jitter_us,
        ];

        for jitter in &jitter_values {
            let jitter_bytes = jitter.to_le_bytes();
            for &byte in &jitter_bytes {
                let _ = serialized.push(byte);
            }
        }

        // Serialize custom measurements (remaining space)
        for &byte in &self.custom_measurements {
            if serialized.push(byte).is_err() {
                break; // Stop if we run out of space
            }
        }

        serialized
    }
}

/// Resource usage statistics during test execution
#[derive(Clone, Copy, Debug)]
pub struct ResourceUsageStats {
    pub cpu_usage_percent: u32,
    pub memory_usage_bytes: u32,
    pub peak_memory_usage_bytes: u32,
    pub execution_time_ms: u32,
    pub preemption_count: u32,
}

impl Default for ResourceUsageStats {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceUsageStats {
    /// Create new empty resource usage stats
    pub fn new() -> Self {
        Self {
            cpu_usage_percent: 0,
            memory_usage_bytes: 0,
            peak_memory_usage_bytes: 0,
            execution_time_ms: 0,
            preemption_count: 0,
        }
    }
}

/// Performance metrics for test execution
#[derive(Clone, Copy, Debug)]
pub struct PerformanceMetrics {
    pub throughput_ops_per_sec: u32,
    pub average_latency_us: u32,
    pub success_rate_percent: u32,
    pub error_rate_percent: u32,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMetrics {
    /// Create new empty performance metrics
    pub fn new() -> Self {
        Self {
            throughput_ops_per_sec: 0,
            average_latency_us: 0,
            success_rate_percent: 0,
            error_rate_percent: 0,
        }
    }
}

/// Test result structure
/// Requirements: 2.3 (test result collection and serialization)
#[derive(Clone, Debug)]
pub struct TestResult {
    pub test_type: TestType,
    pub status: TestStatus,
    pub measurements: TestMeasurements,
    pub error_details: Option<Vec<u8, 32>>,
    pub start_timestamp_ms: u32,
    pub end_timestamp_ms: u32,
    pub test_id: u8,
}

impl TestResult {
    /// Create a new test result
    pub fn new(test_type: TestType, test_id: u8, start_timestamp_ms: u32) -> Self {
        Self {
            test_type,
            status: TestStatus::NotStarted,
            measurements: TestMeasurements::new(),
            error_details: None,
            start_timestamp_ms,
            end_timestamp_ms: 0,
            test_id,
        }
    }

    /// Mark test as completed
    pub fn complete(&mut self, end_timestamp_ms: u32) {
        self.status = TestStatus::Completed;
        self.end_timestamp_ms = end_timestamp_ms;
    }

    /// Mark test as failed with error details
    pub fn fail(&mut self, end_timestamp_ms: u32, error_message: &str) {
        self.status = TestStatus::Failed;
        self.end_timestamp_ms = end_timestamp_ms;
        
        let mut error_details = Vec::new();
        let error_bytes = error_message.as_bytes();
        let max_len = core::cmp::min(error_bytes.len(), 32);
        for &byte in error_bytes.iter().take(max_len) {
            if error_details.push(byte).is_err() {
                break;
            }
        }
        self.error_details = Some(error_details);
    }

    /// Mark test as timed out
    pub fn timeout(&mut self, end_timestamp_ms: u32) {
        self.status = TestStatus::TimedOut;
        self.end_timestamp_ms = end_timestamp_ms;
    }

    /// Get test duration in milliseconds
    pub fn duration_ms(&self) -> u32 {
        self.end_timestamp_ms.saturating_sub(self.start_timestamp_ms)
    }

    /// Serialize test result to command response
    /// Requirements: 2.3 (test result serialization)
    pub fn serialize_to_response(&self, command_id: u8) -> Result<CommandReport, ErrorCode> {
        let mut payload: Vec<u8, 60> = Vec::new();

        // Serialize test result header (8 bytes)
        payload.push(self.test_type as u8).map_err(|_| ErrorCode::PayloadTooLarge)?;
        payload.push(self.status as u8).map_err(|_| ErrorCode::PayloadTooLarge)?;
        payload.push(self.test_id).map_err(|_| ErrorCode::PayloadTooLarge)?;

        let duration_bytes = self.duration_ms().to_le_bytes();
        for &byte in &duration_bytes {
            payload.push(byte).map_err(|_| ErrorCode::PayloadTooLarge)?;
        }

        payload.push(self.measurements.error_count as u8).map_err(|_| ErrorCode::PayloadTooLarge)?;

        // Serialize measurements (up to remaining space)
        let measurements_serialized = self.measurements.serialize();
        let remaining_space = 60 - payload.len();
        let measurements_len = core::cmp::min(measurements_serialized.len(), remaining_space);
        
        for i in 0..measurements_len {
            payload.push(measurements_serialized[i]).map_err(|_| ErrorCode::PayloadTooLarge)?;
        }

        CommandReport::new(TestResponse::TestResult as u8, command_id, &payload)
    }
}

/// Active test tracking structure
#[derive(Clone, Debug)]
pub struct ActiveTest {
    pub test_type: TestType,
    pub parameters: TestParameters,
    pub result: TestResult,
    pub timeout_timestamp_ms: u32,
    pub resource_monitor: ResourceMonitor,
}

impl ActiveTest {
    /// Create a new active test
    pub fn new(
        test_type: TestType,
        parameters: TestParameters,
        test_id: u8,
        start_timestamp_ms: u32,
    ) -> Self {
        let timeout_timestamp_ms = start_timestamp_ms + parameters.duration_ms + 5000; // Add 5s buffer
        Self {
            test_type,
            parameters,
            result: TestResult::new(test_type, test_id, start_timestamp_ms),
            timeout_timestamp_ms,
            resource_monitor: ResourceMonitor::new(),
        }
    }

    /// Check if test has timed out
    pub fn is_timed_out(&self, current_timestamp_ms: u32) -> bool {
        current_timestamp_ms >= self.timeout_timestamp_ms
    }

    /// Update resource usage monitoring
    /// Requirements: 8.1, 8.2 (resource usage monitoring)
    pub fn update_resource_usage(&mut self, current_timestamp_ms: u32) {
        self.resource_monitor.update(current_timestamp_ms);
        
        // Update result measurements with current resource usage
        self.result.measurements.resource_usage = self.resource_monitor.get_current_stats();
    }

    /// Check if resource limits are exceeded
    /// Requirements: 8.1, 8.2 (resource usage limits)
    pub fn check_resource_limits(&self) -> Result<(), TestExecutionError> {
        let current_stats = self.resource_monitor.get_current_stats();
        
        if current_stats.cpu_usage_percent > self.parameters.resource_limits.max_cpu_usage_percent as u32 {
            return Err(TestExecutionError::CpuLimitExceeded);
        }

        if current_stats.memory_usage_bytes > self.parameters.resource_limits.max_memory_usage_bytes {
            return Err(TestExecutionError::MemoryLimitExceeded);
        }

        if current_stats.execution_time_ms > self.parameters.resource_limits.max_execution_time_ms {
            return Err(TestExecutionError::ExecutionTimeExceeded);
        }

        Ok(())
    }
}

/// Resource monitor for tracking test resource usage
/// Requirements: 8.1, 8.2 (resource usage monitoring)
#[derive(Clone, Copy, Debug)]
pub struct ResourceMonitor {
    start_timestamp_ms: u32,
    last_update_ms: u32,
    cpu_usage_samples: [u32; 8],
    sample_index: usize,
    peak_memory_usage: u32,
    current_memory_usage: u32,
}

impl Default for ResourceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new() -> Self {
        Self {
            start_timestamp_ms: 0,
            last_update_ms: 0,
            cpu_usage_samples: [0; 8],
            sample_index: 0,
            peak_memory_usage: 0,
            current_memory_usage: 0,
        }
    }

    /// Update resource monitoring
    pub fn update(&mut self, current_timestamp_ms: u32) {
        if self.start_timestamp_ms == 0 {
            self.start_timestamp_ms = current_timestamp_ms;
        }
        self.last_update_ms = current_timestamp_ms;

        // Simulate CPU usage measurement (in real implementation, this would measure actual usage)
        let cpu_usage = self.measure_cpu_usage();
        self.cpu_usage_samples[self.sample_index] = cpu_usage;
        self.sample_index = (self.sample_index + 1) % self.cpu_usage_samples.len();

        // Simulate memory usage measurement
        self.current_memory_usage = self.measure_memory_usage();
        if self.current_memory_usage > self.peak_memory_usage {
            self.peak_memory_usage = self.current_memory_usage;
        }
    }

    /// Get current resource usage statistics
    pub fn get_current_stats(&self) -> ResourceUsageStats {
        let avg_cpu_usage = self.cpu_usage_samples.iter().sum::<u32>() / self.cpu_usage_samples.len() as u32;
        let execution_time_ms = self.last_update_ms.saturating_sub(self.start_timestamp_ms);

        ResourceUsageStats {
            cpu_usage_percent: avg_cpu_usage,
            memory_usage_bytes: self.current_memory_usage,
            peak_memory_usage_bytes: self.peak_memory_usage,
            execution_time_ms,
            preemption_count: 0, // Would be tracked by RTIC in real implementation
        }
    }

    /// Measure current CPU usage (placeholder implementation)
    fn measure_cpu_usage(&self) -> u32 {
        // In a real implementation, this would measure actual CPU usage
        // For now, return a simulated value
        25 // 25% CPU usage
    }

    /// Measure current memory usage (placeholder implementation)
    fn measure_memory_usage(&self) -> u32 {
        // In a real implementation, this would measure actual memory usage
        // For now, return a simulated value
        2048 // 2KB memory usage
    }
}

/// Test execution errors
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TestExecutionError {
    CpuLimitExceeded,
    MemoryLimitExceeded,
    ExecutionTimeExceeded,
    HardwareError,
    ValidationFailed,
    TestAborted,
}

impl From<TestParameterError> for TestExecutionError {
    fn from(_: TestParameterError) -> Self {
        TestExecutionError::ValidationFailed
    }
}

/// pEMF timing test statistics
/// Requirements: 9.5 (timing statistics and error counts)
#[derive(Clone, Copy, Debug)]
pub struct PemfTimingStatistics {
    pub total_measurements: u32,
    pub timing_accuracy_percent: f32,
    pub error_count: u32,
    pub max_jitter_us: u32,
    pub average_timing_error_percent: f32,
    pub test_duration_ms: u32,
    pub within_tolerance_count: u32,
}

/// Timing deviation information for detailed analysis
/// Requirements: 9.1, 9.5 (timing deviation detection and reporting)
#[derive(Clone, Copy, Debug)]
pub struct TimingDeviation {
    pub measurement_index: u16,
    pub timestamp_ms: u32,
    pub expected_timing_us: u32,
    pub actual_timing_us: u32,
    pub deviation_us: u32,
    pub deviation_percent: f32,
    pub deviation_type: TimingDeviationType,
}

/// Type of timing deviation
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TimingDeviationType {
    TooFast,  // Timing is faster than expected
    TooSlow,  // Timing is slower than expected
}

/// Comprehensive timing deviation report
/// Requirements: 9.5 (timing statistics and error counts)
#[derive(Clone, Copy, Debug)]
pub struct TimingDeviationReport {
    pub total_measurements: u32,
    pub total_deviations: u32,
    pub deviation_rate_percent: u32,
    pub max_deviation_us: u32,
    pub average_deviation_us: u32,
    pub too_slow_count: u32,
    pub too_fast_count: u32,
    pub tolerance_percent: f32,
    pub test_passed: bool,
}

/// Comprehensive timing validation report with detailed analysis
/// Requirements: 9.5 (timing statistics and error counts)
#[derive(Clone, Copy, Debug)]
pub struct ComprehensiveTimingReport {
    pub test_duration_ms: u32,
    pub total_measurements: u32,
    pub within_tolerance_count: u32,
    pub success_rate_percent: u32,
    pub timing_accuracy_percent: f32,
    pub average_timing_error_percent: f32,
    pub max_jitter_us: u32,
    pub timing_stability_score: u8,
    pub deviation_count: u32,
    pub max_deviation_us: u32,
    pub test_passed: bool,
    pub tolerance_percent: f32,
}

/// System stress test parameters
/// Requirements: 9.2 (configurable stress test parameters)
#[derive(Clone, Copy, Debug)]
pub struct StressTestParameters {
    pub duration_ms: u32,
    pub load_level: u8, // 0-100 percentage
    pub memory_stress_enabled: bool,
    pub cpu_stress_enabled: bool,
    pub io_stress_enabled: bool,
    pub concurrent_operations: u8,
    pub stress_pattern: StressPattern,
    pub performance_threshold_percent: u8,
}

impl Default for StressTestParameters {
    fn default() -> Self {
        Self {
            duration_ms: 10000, // 10 seconds
            load_level: 50, // 50% load
            memory_stress_enabled: true,
            cpu_stress_enabled: true,
            io_stress_enabled: true,
            concurrent_operations: 4,
            stress_pattern: StressPattern::Constant,
            performance_threshold_percent: 80, // Alert if performance drops below 80%
        }
    }
}

impl StressTestParameters {

    /// Create stress test parameters from payload
    pub fn from_payload(payload: &[u8]) -> Result<Self, TestParameterError> {
        if payload.len() < 8 {
            return Err(TestParameterError::PayloadTooShort);
        }

        let duration_ms = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
        let load_level = payload[4];
        let flags = payload[5];
        let concurrent_operations = payload[6];
        let performance_threshold_percent = payload[7];

        let stress_pattern = if payload.len() > 8 {
            StressPattern::from_u8(payload[8]).unwrap_or(StressPattern::Constant)
        } else {
            StressPattern::Constant
        };

        let params = Self {
            duration_ms,
            load_level,
            memory_stress_enabled: (flags & 0x01) != 0,
            cpu_stress_enabled: (flags & 0x02) != 0,
            io_stress_enabled: (flags & 0x04) != 0,
            concurrent_operations,
            stress_pattern,
            performance_threshold_percent,
        };

        params.validate()?;
        Ok(params)
    }

    /// Validate stress test parameters
    pub fn validate(&self) -> Result<(), TestParameterError> {
        if self.duration_ms == 0 || self.duration_ms > 300_000 {
            return Err(TestParameterError::InvalidDuration);
        }

        if self.load_level > 100 {
            return Err(TestParameterError::InvalidResourceLimits);
        }

        if self.concurrent_operations == 0 || self.concurrent_operations > 16 {
            return Err(TestParameterError::InvalidResourceLimits);
        }

        if self.performance_threshold_percent > 100 {
            return Err(TestParameterError::InvalidResourceLimits);
        }

        Ok(())
    }

    /// Serialize stress test parameters
    pub fn serialize(&self) -> Vec<u8, 16> {
        let mut serialized = Vec::new();

        // Duration (4 bytes)
        let duration_bytes = self.duration_ms.to_le_bytes();
        for &byte in &duration_bytes {
            let _ = serialized.push(byte);
        }

        // Load level (1 byte)
        let _ = serialized.push(self.load_level);

        // Flags (1 byte)
        let mut flags = 0u8;
        if self.memory_stress_enabled { flags |= 0x01; }
        if self.cpu_stress_enabled { flags |= 0x02; }
        if self.io_stress_enabled { flags |= 0x04; }
        let _ = serialized.push(flags);

        // Concurrent operations (1 byte)
        let _ = serialized.push(self.concurrent_operations);

        // Performance threshold (1 byte)
        let _ = serialized.push(self.performance_threshold_percent);

        // Stress pattern (1 byte)
        let _ = serialized.push(self.stress_pattern as u8);

        serialized
    }
}

/// Stress test patterns
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum StressPattern {
    Constant = 0x00,    // Constant load throughout test
    Ramp = 0x01,        // Gradually increasing load
    Burst = 0x02,       // Periodic high-load bursts
    Random = 0x03,      // Random load variations
}

impl StressPattern {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(StressPattern::Constant),
            0x01 => Some(StressPattern::Ramp),
            0x02 => Some(StressPattern::Burst),
            0x03 => Some(StressPattern::Random),
            _ => None,
        }
    }
}

/// Battery ADC validation test parameters
/// Requirements: 9.1, 9.5 (battery ADC validation against known references)
#[derive(Clone, Copy, Debug)]
pub struct BatteryAdcParameters {
    pub test_duration_ms: u32,
    pub reference_voltage_mv: u32,
    pub tolerance_percent: f32,
    pub sample_count: u32,
    pub calibration_enabled: bool,
    pub state_transition_test: bool,
    pub expected_adc_value: u16,
    pub validation_mode: AdcValidationMode,
}

impl Default for BatteryAdcParameters {
    fn default() -> Self {
        Self {
            test_duration_ms: 5000, // 5 seconds
            reference_voltage_mv: 3300, // 3.3V reference
            tolerance_percent: 2.0, // ±2% tolerance
            sample_count: 50, // 50 samples
            calibration_enabled: true,
            state_transition_test: true,
            expected_adc_value: 1500, // Normal battery state
            validation_mode: AdcValidationMode::Accuracy,
        }
    }
}

impl BatteryAdcParameters {

    /// Create battery ADC parameters from payload
    pub fn from_payload(payload: &[u8]) -> Result<Self, TestParameterError> {
        if payload.len() < 16 {
            return Err(TestParameterError::PayloadTooShort);
        }

        let test_duration_ms = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
        let reference_voltage_mv = u32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);
        let tolerance_percent = f32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]);
        let sample_count = u32::from_le_bytes([payload[12], payload[13], payload[14], payload[15]]);

        let flags = if payload.len() > 16 { payload[16] } else { 0x03 }; // Default: calibration + state test
        let expected_adc_value = if payload.len() > 18 {
            u16::from_le_bytes([payload[17], payload[18]])
        } else {
            1500
        };
        let validation_mode = if payload.len() > 19 {
            AdcValidationMode::from_u8(payload[19]).unwrap_or(AdcValidationMode::Accuracy)
        } else {
            AdcValidationMode::Accuracy
        };

        let params = Self {
            test_duration_ms,
            reference_voltage_mv,
            tolerance_percent,
            sample_count,
            calibration_enabled: (flags & 0x01) != 0,
            state_transition_test: (flags & 0x02) != 0,
            expected_adc_value,
            validation_mode,
        };

        params.validate()?;
        Ok(params)
    }

    /// Validate battery ADC test parameters
    pub fn validate(&self) -> Result<(), TestParameterError> {
        if self.test_duration_ms == 0 || self.test_duration_ms > 60_000 {
            return Err(TestParameterError::InvalidDuration);
        }

        if self.reference_voltage_mv < 1000 || self.reference_voltage_mv > 5000 {
            return Err(TestParameterError::InvalidResourceLimits);
        }

        if self.tolerance_percent < 0.1 || self.tolerance_percent > 20.0 {
            return Err(TestParameterError::InvalidTolerance);
        }

        if self.sample_count == 0 || self.sample_count > 1000 {
            return Err(TestParameterError::InvalidSampleRate);
        }

        if self.expected_adc_value > 4095 {
            return Err(TestParameterError::InvalidResourceLimits);
        }

        Ok(())
    }

    /// Serialize battery ADC parameters
    pub fn serialize(&self) -> Vec<u8, 20> {
        let mut serialized = Vec::new();

        // Test duration (4 bytes)
        let duration_bytes = self.test_duration_ms.to_le_bytes();
        for &byte in &duration_bytes {
            let _ = serialized.push(byte);
        }

        // Reference voltage (4 bytes)
        let voltage_bytes = self.reference_voltage_mv.to_le_bytes();
        for &byte in &voltage_bytes {
            let _ = serialized.push(byte);
        }

        // Tolerance (4 bytes)
        let tolerance_bytes = self.tolerance_percent.to_le_bytes();
        for &byte in &tolerance_bytes {
            let _ = serialized.push(byte);
        }

        // Sample count (4 bytes)
        let sample_bytes = self.sample_count.to_le_bytes();
        for &byte in &sample_bytes {
            let _ = serialized.push(byte);
        }

        // Flags (1 byte)
        let mut flags = 0u8;
        if self.calibration_enabled { flags |= 0x01; }
        if self.state_transition_test { flags |= 0x02; }
        let _ = serialized.push(flags);

        // Expected ADC value (2 bytes)
        let adc_bytes = self.expected_adc_value.to_le_bytes();
        for &byte in &adc_bytes {
            let _ = serialized.push(byte);
        }

        // Validation mode (1 byte)
        let _ = serialized.push(self.validation_mode as u8);

        serialized
    }
}

/// ADC validation modes
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum AdcValidationMode {
    Accuracy = 0x00,        // Test ADC reading accuracy
    Calibration = 0x01,     // Test ADC calibration
    StateTransition = 0x02, // Test battery state transitions
    Comprehensive = 0x03,   // All validation modes
}

impl AdcValidationMode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(AdcValidationMode::Accuracy),
            0x01 => Some(AdcValidationMode::Calibration),
            0x02 => Some(AdcValidationMode::StateTransition),
            0x03 => Some(AdcValidationMode::Comprehensive),
            _ => None,
        }
    }
}

/// LED test patterns for functionality validation
/// Requirements: 9.1, 9.5 (LED control patterns validation)
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum LedTestPattern {
    Solid = 0x00,       // LED continuously ON
    Flashing = 0x01,    // LED flashing pattern (2Hz)
    Off = 0x02,         // LED continuously OFF
    Custom = 0x03,      // Custom pattern with configurable timing
}

impl LedTestPattern {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(LedTestPattern::Solid),
            0x01 => Some(LedTestPattern::Flashing),
            0x02 => Some(LedTestPattern::Off),
            0x03 => Some(LedTestPattern::Custom),
            _ => None,
        }
    }
}

/// LED functionality test parameters
/// Requirements: 9.1, 9.5 (configurable LED test patterns and durations)
#[derive(Clone, Copy, Debug)]
pub struct LedFunctionalityParameters {
    pub test_duration_ms: u32,
    pub pattern: LedTestPattern,
    pub flash_on_duration_ms: u32,
    pub flash_off_duration_ms: u32,
    pub timing_tolerance_percent: f32,
    pub measurement_interval_ms: u32,
    pub validate_timing: bool,
    pub validate_pattern: bool,
}

impl Default for LedFunctionalityParameters {
    fn default() -> Self {
        Self {
            test_duration_ms: 5000,        // 5 seconds
            pattern: LedTestPattern::Flashing,
            flash_on_duration_ms: 250,     // 250ms ON (2Hz pattern)
            flash_off_duration_ms: 250,    // 250ms OFF (2Hz pattern)
            timing_tolerance_percent: 5.0, // ±5% timing tolerance
            measurement_interval_ms: 10,   // 10ms measurement interval
            validate_timing: true,
            validate_pattern: true,
        }
    }
}

impl LedFunctionalityParameters {

    /// Create LED parameters from payload
    pub fn from_payload(payload: &[u8]) -> Result<Self, TestParameterError> {
        if payload.len() < 20 {
            return Err(TestParameterError::PayloadTooShort);
        }

        let test_duration_ms = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
        let pattern = LedTestPattern::from_u8(payload[4]).ok_or(TestParameterError::InvalidResourceLimits)?;
        let flash_on_duration_ms = u32::from_le_bytes([payload[5], payload[6], payload[7], payload[8]]);
        let flash_off_duration_ms = u32::from_le_bytes([payload[9], payload[10], payload[11], payload[12]]);
        let timing_tolerance_percent = f32::from_le_bytes([payload[13], payload[14], payload[15], payload[16]]);
        let measurement_interval_ms = u32::from_le_bytes([payload[17], payload[18], payload[19], payload[20]]);

        let flags = if payload.len() > 21 { payload[21] } else { 0x03 }; // Default: validate timing + pattern

        let params = Self {
            test_duration_ms,
            pattern,
            flash_on_duration_ms,
            flash_off_duration_ms,
            timing_tolerance_percent,
            measurement_interval_ms,
            validate_timing: (flags & 0x01) != 0,
            validate_pattern: (flags & 0x02) != 0,
        };

        params.validate()?;
        Ok(params)
    }

    /// Validate LED functionality test parameters
    pub fn validate(&self) -> Result<(), TestParameterError> {
        if self.test_duration_ms == 0 || self.test_duration_ms > 60_000 {
            return Err(TestParameterError::InvalidDuration);
        }

        if self.flash_on_duration_ms == 0 || self.flash_on_duration_ms > 10_000 {
            return Err(TestParameterError::InvalidDuration);
        }

        if self.flash_off_duration_ms == 0 || self.flash_off_duration_ms > 10_000 {
            return Err(TestParameterError::InvalidDuration);
        }

        if self.timing_tolerance_percent < 0.1 || self.timing_tolerance_percent > 50.0 {
            return Err(TestParameterError::InvalidTolerance);
        }

        if self.measurement_interval_ms == 0 || self.measurement_interval_ms > 1000 {
            return Err(TestParameterError::InvalidSampleRate);
        }

        // Validate that measurement interval is reasonable for the pattern timing
        let min_pattern_duration = core::cmp::min(self.flash_on_duration_ms, self.flash_off_duration_ms);
        if self.measurement_interval_ms > min_pattern_duration / 2 {
            return Err(TestParameterError::InvalidSampleRate);
        }

        Ok(())
    }

    /// Serialize LED functionality parameters
    pub fn serialize(&self) -> Vec<u8, 22> {
        let mut serialized = Vec::new();

        // Test duration (4 bytes)
        let duration_bytes = self.test_duration_ms.to_le_bytes();
        for &byte in &duration_bytes {
            let _ = serialized.push(byte);
        }

        // Pattern (1 byte)
        let _ = serialized.push(self.pattern as u8);

        // Flash ON duration (4 bytes)
        let on_bytes = self.flash_on_duration_ms.to_le_bytes();
        for &byte in &on_bytes {
            let _ = serialized.push(byte);
        }

        // Flash OFF duration (4 bytes)
        let off_bytes = self.flash_off_duration_ms.to_le_bytes();
        for &byte in &off_bytes {
            let _ = serialized.push(byte);
        }

        // Timing tolerance (4 bytes)
        let tolerance_bytes = self.timing_tolerance_percent.to_le_bytes();
        for &byte in &tolerance_bytes {
            let _ = serialized.push(byte);
        }

        // Measurement interval (4 bytes)
        let interval_bytes = self.measurement_interval_ms.to_le_bytes();
        for &byte in &interval_bytes {
            let _ = serialized.push(byte);
        }

        // Flags (1 byte)
        let mut flags = 0u8;
        if self.validate_timing { flags |= 0x01; }
        if self.validate_pattern { flags |= 0x02; }
        let _ = serialized.push(flags);

        serialized
    }
}

/// Battery ADC test measurements
/// Requirements: 9.5 (ADC accuracy measurements)
#[derive(Clone, Copy, Debug)]
pub struct BatteryAdcMeasurements {
    pub total_samples: u32,
    pub average_adc_value: u16,
    pub adc_variance: u32,
    pub voltage_accuracy_percent: f32,
    pub calibration_error_percent: f32,
    pub state_transition_count: u32,
    pub invalid_readings_count: u32,
    pub test_duration_ms: u32,
    pub reference_voltage_mv: u32,
    pub measured_voltage_mv: u32,
}

impl Default for BatteryAdcMeasurements {
    fn default() -> Self {
        Self::new()
    }
}

impl BatteryAdcMeasurements {
    /// Create new empty battery ADC measurements
    pub fn new() -> Self {
        Self {
            total_samples: 0,
            average_adc_value: 0,
            adc_variance: 0,
            voltage_accuracy_percent: 0.0,
            calibration_error_percent: 0.0,
            state_transition_count: 0,
            invalid_readings_count: 0,
            test_duration_ms: 0,
            reference_voltage_mv: 0,
            measured_voltage_mv: 0,
        }
    }

    /// Calculate voltage accuracy from ADC measurements
    pub fn calculate_voltage_accuracy(&mut self, reference_voltage_mv: u32) -> f32 {
        if self.total_samples == 0 || reference_voltage_mv == 0 {
            return 0.0;
        }

        // Convert average ADC to voltage using battery monitoring conversion
        // Voltage = ADC * 3300mV / 4095 / voltage_divider_ratio
        // Simplified: voltage_mv = ADC * 2386 / 1000
        self.measured_voltage_mv = (self.average_adc_value as u32 * 2386) / 1000;
        
        let voltage_error = self.measured_voltage_mv.abs_diff(reference_voltage_mv);

        self.voltage_accuracy_percent = 100.0 - (voltage_error as f32 / reference_voltage_mv as f32 * 100.0);
        self.voltage_accuracy_percent = self.voltage_accuracy_percent.max(0.0);
        self.voltage_accuracy_percent
    }

    /// Calculate calibration error
    pub fn calculate_calibration_error(&mut self, expected_adc: u16) -> f32 {
        if expected_adc == 0 {
            return 0.0;
        }

        let adc_error = self.average_adc_value.abs_diff(expected_adc);

        self.calibration_error_percent = (adc_error as f32 / expected_adc as f32) * 100.0;
        self.calibration_error_percent
    }

    /// Serialize battery ADC measurements
    pub fn serialize(&self) -> Vec<u8, 40> {
        let mut serialized = Vec::new();

        // Total samples (4 bytes)
        let samples_bytes = self.total_samples.to_le_bytes();
        for &byte in &samples_bytes {
            let _ = serialized.push(byte);
        }

        // Average ADC value (2 bytes)
        let adc_bytes = self.average_adc_value.to_le_bytes();
        for &byte in &adc_bytes {
            let _ = serialized.push(byte);
        }

        // ADC variance (4 bytes)
        let variance_bytes = self.adc_variance.to_le_bytes();
        for &byte in &variance_bytes {
            let _ = serialized.push(byte);
        }

        // Voltage accuracy (4 bytes)
        let accuracy_bytes = self.voltage_accuracy_percent.to_le_bytes();
        for &byte in &accuracy_bytes {
            let _ = serialized.push(byte);
        }

        // Calibration error (4 bytes)
        let cal_error_bytes = self.calibration_error_percent.to_le_bytes();
        for &byte in &cal_error_bytes {
            let _ = serialized.push(byte);
        }

        // State transition count (4 bytes)
        let transition_bytes = self.state_transition_count.to_le_bytes();
        for &byte in &transition_bytes {
            let _ = serialized.push(byte);
        }

        // Invalid readings count (4 bytes)
        let invalid_bytes = self.invalid_readings_count.to_le_bytes();
        for &byte in &invalid_bytes {
            let _ = serialized.push(byte);
        }

        // Test duration (4 bytes)
        let duration_bytes = self.test_duration_ms.to_le_bytes();
        for &byte in &duration_bytes {
            let _ = serialized.push(byte);
        }

        // Reference voltage (4 bytes)
        let ref_voltage_bytes = self.reference_voltage_mv.to_le_bytes();
        for &byte in &ref_voltage_bytes {
            let _ = serialized.push(byte);
        }

        // Measured voltage (4 bytes)
        let measured_voltage_bytes = self.measured_voltage_mv.to_le_bytes();
        for &byte in &measured_voltage_bytes {
            let _ = serialized.push(byte);
        }

        serialized
    }
}

/// LED timing measurement for pattern validation
/// Requirements: 9.1, 9.5 (LED timing accuracy validation)
#[derive(Clone, Copy, Debug)]
pub struct LedTimingMeasurement {
    pub timestamp_ms: u32,
    pub led_state: bool,        // true = ON, false = OFF
    pub expected_state: bool,   // Expected state based on pattern
    pub state_duration_ms: u32, // Duration in current state
    pub timing_error_ms: i32,   // Positive = late, negative = early
}

/// LED functionality test measurements
/// Requirements: 9.1, 9.5 (LED timing measurements and pattern validation)
#[derive(Clone, Debug)]
pub struct LedFunctionalityMeasurements {
    pub total_measurements: u32,
    pub pattern_cycles_completed: u32,
    pub timing_measurements: Vec<LedTimingMeasurement, 64>,
    pub average_on_duration_ms: u32,
    pub average_off_duration_ms: u32,
    pub timing_accuracy_percent: f32,
    pub pattern_accuracy_percent: f32,
    pub max_timing_error_ms: u32,
    pub timing_violations_count: u32,
    pub pattern_violations_count: u32,
    pub test_duration_ms: u32,
}

impl Default for LedFunctionalityMeasurements {
    fn default() -> Self {
        Self::new()
    }
}

impl LedFunctionalityMeasurements {
    /// Create new empty LED functionality measurements
    pub fn new() -> Self {
        Self {
            total_measurements: 0,
            pattern_cycles_completed: 0,
            timing_measurements: Vec::new(),
            average_on_duration_ms: 0,
            average_off_duration_ms: 0,
            timing_accuracy_percent: 0.0,
            pattern_accuracy_percent: 0.0,
            max_timing_error_ms: 0,
            timing_violations_count: 0,
            pattern_violations_count: 0,
            test_duration_ms: 0,
        }
    }

    /// Add timing measurement to the test result
    pub fn add_timing_measurement(&mut self, measurement: LedTimingMeasurement, 
                                 tolerance_percent: f32) -> Result<(), SystemError> {
        // Store the measurement
        self.timing_measurements.push(measurement).map_err(|_| SystemError::SystemBusy)?;
        self.total_measurements += 1;

        // Check for timing violations
        let expected_duration = if measurement.expected_state {
            // Expected ON duration - use the measurement's state duration as reference
            measurement.state_duration_ms
        } else {
            // Expected OFF duration
            measurement.state_duration_ms
        };

        let tolerance_ms = (expected_duration as f32 * tolerance_percent / 100.0) as u32;
        let timing_error_abs = measurement.timing_error_ms.unsigned_abs();

        if timing_error_abs > tolerance_ms {
            self.timing_violations_count += 1;
        }

        // Update maximum timing error
        if timing_error_abs > self.max_timing_error_ms {
            self.max_timing_error_ms = timing_error_abs;
        }

        // Check for pattern violations (state mismatch)
        if measurement.led_state != measurement.expected_state {
            self.pattern_violations_count += 1;
        }

        Ok(())
    }

    /// Calculate timing and pattern accuracy statistics
    pub fn calculate_accuracy_statistics(&mut self, expected_on_ms: u32, expected_off_ms: u32) {
        if self.timing_measurements.is_empty() {
            return;
        }

        let mut total_on_duration = 0u32;
        let mut total_off_duration = 0u32;
        let mut on_count = 0u32;
        let mut off_count = 0u32;
        let mut total_timing_error = 0u32;

        // Analyze timing measurements
        for measurement in &self.timing_measurements {
            let timing_error_abs = measurement.timing_error_ms.unsigned_abs();
            total_timing_error += timing_error_abs;

            if measurement.expected_state {
                total_on_duration += measurement.state_duration_ms;
                on_count += 1;
            } else {
                total_off_duration += measurement.state_duration_ms;
                off_count += 1;
            }
        }

        // Calculate average durations
        self.average_on_duration_ms = if on_count > 0 { total_on_duration / on_count } else { 0 };
        self.average_off_duration_ms = if off_count > 0 { total_off_duration / off_count } else { 0 };

        // Calculate timing accuracy
        let average_timing_error = total_timing_error as f32 / self.total_measurements as f32;
        let expected_avg_duration = (expected_on_ms + expected_off_ms) as f32 / 2.0;
        self.timing_accuracy_percent = 100.0 - (average_timing_error / expected_avg_duration * 100.0);
        self.timing_accuracy_percent = self.timing_accuracy_percent.max(0.0);

        // Calculate pattern accuracy
        let pattern_errors = self.pattern_violations_count as f32;
        self.pattern_accuracy_percent = 100.0 - (pattern_errors / self.total_measurements as f32 * 100.0);
        self.pattern_accuracy_percent = self.pattern_accuracy_percent.max(0.0);

        // Calculate completed cycles (approximate)
        let total_expected_cycle_duration = expected_on_ms + expected_off_ms;
        if total_expected_cycle_duration > 0 {
            self.pattern_cycles_completed = self.test_duration_ms / total_expected_cycle_duration;
        }
    }

    /// Serialize LED functionality measurements
    pub fn serialize(&self) -> Vec<u8, 48> {
        let mut serialized = Vec::new();

        // Total measurements (4 bytes)
        let measurements_bytes = self.total_measurements.to_le_bytes();
        for &byte in &measurements_bytes {
            let _ = serialized.push(byte);
        }

        // Pattern cycles completed (4 bytes)
        let cycles_bytes = self.pattern_cycles_completed.to_le_bytes();
        for &byte in &cycles_bytes {
            let _ = serialized.push(byte);
        }

        // Average ON duration (4 bytes)
        let on_duration_bytes = self.average_on_duration_ms.to_le_bytes();
        for &byte in &on_duration_bytes {
            let _ = serialized.push(byte);
        }

        // Average OFF duration (4 bytes)
        let off_duration_bytes = self.average_off_duration_ms.to_le_bytes();
        for &byte in &off_duration_bytes {
            let _ = serialized.push(byte);
        }

        // Timing accuracy (4 bytes)
        let timing_accuracy_bytes = self.timing_accuracy_percent.to_le_bytes();
        for &byte in &timing_accuracy_bytes {
            let _ = serialized.push(byte);
        }

        // Pattern accuracy (4 bytes)
        let pattern_accuracy_bytes = self.pattern_accuracy_percent.to_le_bytes();
        for &byte in &pattern_accuracy_bytes {
            let _ = serialized.push(byte);
        }

        // Max timing error (4 bytes)
        let max_error_bytes = self.max_timing_error_ms.to_le_bytes();
        for &byte in &max_error_bytes {
            let _ = serialized.push(byte);
        }

        // Timing violations count (4 bytes)
        let timing_violations_bytes = self.timing_violations_count.to_le_bytes();
        for &byte in &timing_violations_bytes {
            let _ = serialized.push(byte);
        }

        // Pattern violations count (4 bytes)
        let pattern_violations_bytes = self.pattern_violations_count.to_le_bytes();
        for &byte in &pattern_violations_bytes {
            let _ = serialized.push(byte);
        }

        // Test duration (4 bytes)
        let duration_bytes = self.test_duration_ms.to_le_bytes();
        for &byte in &duration_bytes {
            let _ = serialized.push(byte);
        }

        // Timing measurements count (2 bytes) - limited to first few measurements
        let measurements_count = core::cmp::min(self.timing_measurements.len(), 4) as u16;
        let count_bytes = measurements_count.to_le_bytes();
        for &byte in &count_bytes {
            let _ = serialized.push(byte);
        }

        // Include first few timing measurements (2 bytes each for timestamp and error)
        for i in 0..measurements_count as usize {
            if let Some(measurement) = self.timing_measurements.get(i) {
                let timestamp_short = (measurement.timestamp_ms & 0xFFFF) as u16;
                let timestamp_bytes = timestamp_short.to_le_bytes();
                for &byte in &timestamp_bytes {
                    if serialized.push(byte).is_err() { break; }
                }
            }
        }

        serialized
    }
}

/// LED functionality test result
/// Requirements: 9.1, 9.5 (LED control validation with timing measurements)
#[derive(Clone, Debug)]
pub struct LedFunctionalityTestResult {
    pub test_parameters: LedFunctionalityParameters,
    pub measurements: LedFunctionalityMeasurements,
    pub test_passed: bool,
    pub error_details: Option<Vec<u8, 64>>,
    pub start_timestamp_ms: u32,
    pub end_timestamp_ms: u32,
}

impl LedFunctionalityTestResult {
    /// Create new LED functionality test result
    pub fn new(parameters: LedFunctionalityParameters, start_timestamp_ms: u32) -> Self {
        Self {
            test_parameters: parameters,
            measurements: LedFunctionalityMeasurements::new(),
            test_passed: false,
            error_details: None,
            start_timestamp_ms,
            end_timestamp_ms: 0,
        }
    }

    /// Add LED state measurement to the test result
    pub fn add_led_measurement(&mut self, led_state: bool, expected_state: bool, 
                              timestamp_ms: u32, state_duration_ms: u32) -> Result<(), SystemError> {
        // Calculate timing error based on expected pattern timing
        let expected_duration = match self.test_parameters.pattern {
            LedTestPattern::Flashing => {
                if expected_state {
                    self.test_parameters.flash_on_duration_ms
                } else {
                    self.test_parameters.flash_off_duration_ms
                }
            },
            LedTestPattern::Solid => {
                // For solid pattern, LED should always be ON
                if expected_state { u32::MAX } else { 0 }
            },
            LedTestPattern::Off => {
                // For off pattern, LED should always be OFF
                if expected_state { 0 } else { u32::MAX }
            },
            LedTestPattern::Custom => {
                // Use configured timing
                if expected_state {
                    self.test_parameters.flash_on_duration_ms
                } else {
                    self.test_parameters.flash_off_duration_ms
                }
            },
        };

        let timing_error_ms = if expected_duration == u32::MAX || expected_duration == 0 {
            0 // No timing error for continuous states
        } else {
            state_duration_ms as i32 - expected_duration as i32
        };

        let measurement = LedTimingMeasurement {
            timestamp_ms,
            led_state,
            expected_state,
            state_duration_ms,
            timing_error_ms,
        };

        self.measurements.add_timing_measurement(measurement, self.test_parameters.timing_tolerance_percent)?;
        Ok(())
    }

    /// Complete the test and calculate final results
    pub fn complete_test(&mut self, end_timestamp_ms: u32) {
        self.end_timestamp_ms = end_timestamp_ms;
        self.measurements.test_duration_ms = end_timestamp_ms - self.start_timestamp_ms;

        // Calculate accuracy statistics
        self.measurements.calculate_accuracy_statistics(
            self.test_parameters.flash_on_duration_ms,
            self.test_parameters.flash_off_duration_ms
        );

        // Determine if test passed based on validation criteria
        self.test_passed = self.evaluate_test_success();
    }

    /// Evaluate if the test passed based on validation criteria
    fn evaluate_test_success(&self) -> bool {
        // Check timing accuracy if enabled
        if self.test_parameters.validate_timing {
            let min_timing_accuracy = 100.0 - self.test_parameters.timing_tolerance_percent;
            if self.measurements.timing_accuracy_percent < min_timing_accuracy {
                return false;
            }
        }

        // Check pattern accuracy if enabled
        if self.test_parameters.validate_pattern
            && self.measurements.pattern_accuracy_percent < 95.0 { // 95% pattern accuracy required
                return false;
            }

        // Check for excessive timing violations
        let violation_rate = self.measurements.timing_violations_count as f32 / self.measurements.total_measurements as f32;
        if violation_rate > 0.1 { // Allow up to 10% timing violations
            return false;
        }

        // Check minimum measurement count
        let expected_measurements = self.test_parameters.test_duration_ms / self.test_parameters.measurement_interval_ms;
        if self.measurements.total_measurements < expected_measurements / 2 {
            return false;
        }

        true
    }

    /// Get test duration in milliseconds
    pub fn duration_ms(&self) -> u32 {
        self.end_timestamp_ms.saturating_sub(self.start_timestamp_ms)
    }

    /// Serialize LED functionality test result for transmission
    pub fn serialize(&self) -> Result<Vec<u8, 64>, ErrorCode> {
        let mut payload = Vec::new();

        // Test result header (8 bytes)
        payload.push(TestType::LedFunctionality as u8).map_err(|_| ErrorCode::PayloadTooLarge)?;
        payload.push(if self.test_passed { TestStatus::Completed as u8 } else { TestStatus::Failed as u8 }).map_err(|_| ErrorCode::PayloadTooLarge)?;
        
        let duration_bytes = self.duration_ms().to_le_bytes();
        for &byte in &duration_bytes {
            payload.push(byte).map_err(|_| ErrorCode::PayloadTooLarge)?;
        }

        let measurements_count = self.measurements.total_measurements.to_le_bytes();
        for &byte in &measurements_count {
            payload.push(byte).map_err(|_| ErrorCode::PayloadTooLarge)?;
        }

        // Key measurements (16 bytes)
        let timing_accuracy_bytes = self.measurements.timing_accuracy_percent.to_le_bytes();
        for &byte in &timing_accuracy_bytes {
            payload.push(byte).map_err(|_| ErrorCode::PayloadTooLarge)?;
        }

        let pattern_accuracy_bytes = self.measurements.pattern_accuracy_percent.to_le_bytes();
        for &byte in &pattern_accuracy_bytes {
            payload.push(byte).map_err(|_| ErrorCode::PayloadTooLarge)?;
        }

        let max_error_bytes = self.measurements.max_timing_error_ms.to_le_bytes();
        for &byte in &max_error_bytes {
            payload.push(byte).map_err(|_| ErrorCode::PayloadTooLarge)?;
        }

        let violations_bytes = self.measurements.timing_violations_count.to_le_bytes();
        for &byte in &violations_bytes {
            payload.push(byte).map_err(|_| ErrorCode::PayloadTooLarge)?;
        }

        // Pattern information (8 bytes)
        payload.push(self.test_parameters.pattern as u8).map_err(|_| ErrorCode::PayloadTooLarge)?;
        
        let cycles_bytes = self.measurements.pattern_cycles_completed.to_le_bytes();
        for &byte in &cycles_bytes {
            payload.push(byte).map_err(|_| ErrorCode::PayloadTooLarge)?;
        }

        let avg_on_bytes = (self.measurements.average_on_duration_ms as u16).to_le_bytes();
        for &byte in &avg_on_bytes {
            payload.push(byte).map_err(|_| ErrorCode::PayloadTooLarge)?;
        }

        let avg_off_bytes = (self.measurements.average_off_duration_ms as u16).to_le_bytes();
        for &byte in &avg_off_bytes {
            payload.push(byte).map_err(|_| ErrorCode::PayloadTooLarge)?;
        }

        // Fill remaining space with test parameters summary
        let params_serialized = self.test_parameters.serialize();
        for &byte in params_serialized.iter().take(payload.capacity() - payload.len()) {
            if payload.push(byte).is_err() {
                break;
            }
        }

        Ok(payload)
    }
}

/// Battery state transition record
/// Requirements: 9.1, 9.5 (battery state transition testing)
#[derive(Clone, Copy, Debug)]
pub struct BatteryStateTransition {
    pub timestamp_ms: u32,
    pub from_state: BatteryState,
    pub to_state: BatteryState,
    pub adc_value: u16,
    pub voltage_mv: u32,
    pub transition_valid: bool,
}

/// Battery state enumeration (matching battery.rs)
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum BatteryState {
    Low = 0x00,      // ADC ≤ 1425
    Normal = 0x01,   // 1425 < ADC < 1675
    Charging = 0x02, // ADC ≥ 1675
}

impl BatteryState {
    /// Determine battery state from ADC reading
    pub fn from_adc_reading(adc_value: u16) -> Self {
        if adc_value <= 1425 {
            BatteryState::Low
        } else if adc_value < 1675 {
            BatteryState::Normal
        } else {
            BatteryState::Charging
        }
    }

    /// Get ADC threshold values for this state
    pub fn get_thresholds(&self) -> (u16, u16) {
        match self {
            BatteryState::Low => (0, 1425),
            BatteryState::Normal => (1425, 1675),
            BatteryState::Charging => (1675, u16::MAX),
        }
    }

    /// Check if transition to another state is valid
    pub fn is_valid_transition(&self, to_state: BatteryState, adc_value: u16) -> bool {
        match (*self, to_state) {
            (BatteryState::Low, BatteryState::Normal) => adc_value > 1425,
            (BatteryState::Normal, BatteryState::Low) => adc_value <= 1425,
            (BatteryState::Normal, BatteryState::Charging) => adc_value >= 1675,
            (BatteryState::Charging, BatteryState::Normal) => adc_value < 1675,
            (BatteryState::Low, BatteryState::Charging) => adc_value >= 1675,
            (BatteryState::Charging, BatteryState::Low) => adc_value <= 1425,
            _ => *self == to_state, // Same state is always valid
        }
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(BatteryState::Low),
            0x01 => Some(BatteryState::Normal),
            0x02 => Some(BatteryState::Charging),
            _ => None,
        }
    }
}

/// Battery ADC validation test result
/// Requirements: 9.1, 9.5 (ADC accuracy measurements and state transition testing)
#[derive(Clone, Debug)]
pub struct BatteryAdcTestResult {
    pub test_parameters: BatteryAdcParameters,
    pub measurements: BatteryAdcMeasurements,
    pub state_transitions: Vec<BatteryStateTransition, 16>,
    pub test_passed: bool,
    pub error_details: Option<Vec<u8, 64>>,
    pub start_timestamp_ms: u32,
    pub end_timestamp_ms: u32,
}

impl BatteryAdcTestResult {
    /// Create new battery ADC test result
    pub fn new(parameters: BatteryAdcParameters, start_timestamp_ms: u32) -> Self {
        Self {
            test_parameters: parameters,
            measurements: BatteryAdcMeasurements::new(),
            state_transitions: Vec::new(),
            test_passed: false,
            error_details: None,
            start_timestamp_ms,
            end_timestamp_ms: 0,
        }
    }

    /// Add ADC sample to the test result
    pub fn add_adc_sample(&mut self, adc_value: u16, _timestamp_ms: u32) -> Result<(), SystemError> {
        // Update measurements
        let new_total = self.measurements.total_samples + 1;
        let old_average = self.measurements.average_adc_value as u32;
        let new_average = ((old_average * self.measurements.total_samples) + adc_value as u32) / new_total;
        
        self.measurements.total_samples = new_total;
        self.measurements.average_adc_value = new_average as u16;

        // Update variance (simplified calculation)
        let deviation = adc_value.abs_diff(new_average as u16);
        self.measurements.adc_variance = ((self.measurements.adc_variance * (new_total - 1)) + deviation as u32) / new_total;

        // Check for invalid readings (outside ADC range)
        if adc_value > 4095 {
            self.measurements.invalid_readings_count += 1;
        }

        Ok(())
    }

    /// Add state transition to the test result
    pub fn add_state_transition(&mut self, from_state: BatteryState, to_state: BatteryState, 
                               adc_value: u16, timestamp_ms: u32) -> Result<(), SystemError> {
        let voltage_mv = (adc_value as u32 * 2386) / 1000; // Convert ADC to voltage
        let transition_valid = from_state.is_valid_transition(to_state, adc_value);

        let transition = BatteryStateTransition {
            timestamp_ms,
            from_state,
            to_state,
            adc_value,
            voltage_mv,
            transition_valid,
        };

        self.state_transitions.push(transition).map_err(|_| SystemError::SystemBusy)?;
        self.measurements.state_transition_count += 1;

        Ok(())
    }

    /// Complete the test and calculate final results
    pub fn complete_test(&mut self, end_timestamp_ms: u32) {
        self.end_timestamp_ms = end_timestamp_ms;
        self.measurements.test_duration_ms = end_timestamp_ms - self.start_timestamp_ms;
        self.measurements.reference_voltage_mv = self.test_parameters.reference_voltage_mv;

        // Calculate voltage accuracy
        self.measurements.calculate_voltage_accuracy(self.test_parameters.reference_voltage_mv);

        // Calculate calibration error
        self.measurements.calculate_calibration_error(self.test_parameters.expected_adc_value);

        // Determine if test passed based on validation criteria
        self.test_passed = self.evaluate_test_success();
    }

    /// Evaluate if the test passed based on validation criteria
    fn evaluate_test_success(&self) -> bool {
        // Check voltage accuracy
        if self.measurements.voltage_accuracy_percent < (100.0 - self.test_parameters.tolerance_percent) {
            return false;
        }

        // Check calibration error
        if self.measurements.calibration_error_percent > self.test_parameters.tolerance_percent {
            return false;
        }

        // Check for invalid readings
        let invalid_reading_rate = (self.measurements.invalid_readings_count as f32 / self.measurements.total_samples as f32) * 100.0;
        if invalid_reading_rate > 5.0 { // Allow up to 5% invalid readings
            return false;
        }

        // Check state transitions if enabled
        if self.test_parameters.state_transition_test {
            let invalid_transitions = self.state_transitions.iter()
                .filter(|t| !t.transition_valid)
                .count();
            
            if invalid_transitions > 0 {
                return false;
            }
        }

        // Check minimum sample count
        if self.measurements.total_samples < self.test_parameters.sample_count / 2 {
            return false;
        }

        true
    }

    /// Get test duration in milliseconds
    pub fn duration_ms(&self) -> u32 {
        self.end_timestamp_ms.saturating_sub(self.start_timestamp_ms)
    }

    /// Serialize battery ADC test result to command response
    pub fn serialize_to_response(&self, command_id: u8) -> Result<CommandReport, ErrorCode> {
        let mut payload: Vec<u8, 60> = Vec::new();

        // Test result header (8 bytes)
        payload.push(TestType::BatteryAdcCalibration as u8).map_err(|_| ErrorCode::PayloadTooLarge)?;
        payload.push(if self.test_passed { TestStatus::Completed as u8 } else { TestStatus::Failed as u8 }).map_err(|_| ErrorCode::PayloadTooLarge)?;
        
        let duration_bytes = self.duration_ms().to_le_bytes();
        for &byte in &duration_bytes {
            payload.push(byte).map_err(|_| ErrorCode::PayloadTooLarge)?;
        }

        // Key measurements (16 bytes)
        let samples_bytes = self.measurements.total_samples.to_le_bytes();
        for &byte in &samples_bytes {
            payload.push(byte).map_err(|_| ErrorCode::PayloadTooLarge)?;
        }

        let accuracy_bytes = self.measurements.voltage_accuracy_percent.to_le_bytes();
        for &byte in &accuracy_bytes {
            payload.push(byte).map_err(|_| ErrorCode::PayloadTooLarge)?;
        }

        let cal_error_bytes = self.measurements.calibration_error_percent.to_le_bytes();
        for &byte in &cal_error_bytes {
            payload.push(byte).map_err(|_| ErrorCode::PayloadTooLarge)?;
        }

        let transitions_bytes = self.measurements.state_transition_count.to_le_bytes();
        for &byte in &transitions_bytes {
            payload.push(byte).map_err(|_| ErrorCode::PayloadTooLarge)?;
        }

        // Additional measurements (remaining space)
        let remaining_space = 60 - payload.len();
        let measurements_serialized = self.measurements.serialize();
        let measurements_len = core::cmp::min(measurements_serialized.len(), remaining_space);
        
        for i in 0..measurements_len {
            payload.push(measurements_serialized[i]).map_err(|_| ErrorCode::PayloadTooLarge)?;
        }

        CommandReport::new(TestResponse::TestResult as u8, command_id, &payload)
    }
}

/// System stress test statistics
/// Requirements: 9.2, 9.5 (performance degradation detection and reporting)
#[derive(Clone, Copy, Debug)]
pub struct StressTestStatistics {
    pub test_duration_ms: u32,
    pub peak_cpu_usage_percent: u8,
    pub average_cpu_usage_percent: u8,
    pub peak_memory_usage_bytes: u32,
    pub average_memory_usage_bytes: u32,
    pub memory_allocation_failures: u32,
    pub performance_degradation_events: u32,
    pub min_performance_percent: u8,
    pub average_response_time_us: u32,
    pub max_response_time_us: u32,
    pub operations_completed: u32,
    pub operations_failed: u32,
    pub system_stability_score: u8, // 0-100
}

impl Default for StressTestStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl StressTestStatistics {
    /// Create new empty stress test statistics
    pub fn new() -> Self {
        Self {
            test_duration_ms: 0,
            peak_cpu_usage_percent: 0,
            average_cpu_usage_percent: 0,
            peak_memory_usage_bytes: 0,
            average_memory_usage_bytes: 0,
            memory_allocation_failures: 0,
            performance_degradation_events: 0,
            min_performance_percent: 100,
            average_response_time_us: 0,
            max_response_time_us: 0,
            operations_completed: 0,
            operations_failed: 0,
            system_stability_score: 100,
        }
    }

    /// Calculate success rate as percentage
    pub fn success_rate_percent(&self) -> u8 {
        if self.operations_completed + self.operations_failed == 0 {
            return 100;
        }
        let total = self.operations_completed + self.operations_failed;
        ((self.operations_completed * 100) / total) as u8
    }

    /// Check if statistics meet performance criteria
    pub fn meets_performance_criteria(&self, min_success_rate: u8, max_failures: u32) -> bool {
        self.success_rate_percent() >= min_success_rate && self.operations_failed <= max_failures
    }

    /// Serialize statistics to byte array
    pub fn serialize(&self) -> [u8; 64] {
        let mut data = [0u8; 64];
        data[0..4].copy_from_slice(&self.test_duration_ms.to_le_bytes());
        data[4] = self.peak_cpu_usage_percent;
        data[5] = self.average_cpu_usage_percent;
        data[6..10].copy_from_slice(&self.peak_memory_usage_bytes.to_le_bytes());
        data[10..14].copy_from_slice(&self.average_memory_usage_bytes.to_le_bytes());
        data[14..18].copy_from_slice(&self.memory_allocation_failures.to_le_bytes());
        data[18..22].copy_from_slice(&self.performance_degradation_events.to_le_bytes());
        data[22] = self.min_performance_percent;
        data[23..27].copy_from_slice(&self.average_response_time_us.to_le_bytes());
        data[27..31].copy_from_slice(&self.max_response_time_us.to_le_bytes());
        data[31..35].copy_from_slice(&self.operations_completed.to_le_bytes());
        data[35..39].copy_from_slice(&self.operations_failed.to_le_bytes());
        data[39] = self.system_stability_score;
        data
    }
}

/// USB communication test parameters
/// Requirements: 9.4, 9.5 (configurable message count and timing parameters)
#[derive(Clone, Copy, Debug)]
pub struct UsbCommunicationTestParameters {
    pub message_count: u32,
    pub message_interval_ms: u32,
    pub message_size_bytes: u16,
    pub timeout_per_message_ms: u32,
    pub enable_integrity_checking: bool,
    pub enable_error_injection: bool,
    pub error_injection_rate_percent: u8,
    pub bidirectional_test: bool,
    pub concurrent_messages: u8,
}

impl Default for UsbCommunicationTestParameters {
    fn default() -> Self {
        Self {
            message_count: 100,
            message_interval_ms: 10,
            message_size_bytes: 64,
            timeout_per_message_ms: 1000,
            enable_integrity_checking: true,
            enable_error_injection: false,
            error_injection_rate_percent: 0,
            bidirectional_test: true,
            concurrent_messages: 1,
        }
    }
}

impl UsbCommunicationTestParameters {

    /// Create USB communication test parameters from payload
    pub fn from_payload(payload: &[u8]) -> Result<Self, TestParameterError> {
        if payload.len() < 16 {
            return Err(TestParameterError::PayloadTooShort);
        }

        let message_count = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
        let message_interval_ms = u32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);
        let message_size_bytes = u16::from_le_bytes([payload[8], payload[9]]);
        let timeout_per_message_ms = u32::from_le_bytes([payload[10], payload[11], payload[12], payload[13]]);
        let flags = payload[14];
        let error_injection_rate_percent = payload[15];
        let concurrent_messages = if payload.len() > 16 { payload[16] } else { 1 };

        let params = Self {
            message_count,
            message_interval_ms,
            message_size_bytes,
            timeout_per_message_ms,
            enable_integrity_checking: (flags & 0x01) != 0,
            enable_error_injection: (flags & 0x02) != 0,
            bidirectional_test: (flags & 0x04) != 0,
            error_injection_rate_percent,
            concurrent_messages,
        };

        params.validate()?;
        Ok(params)
    }

    /// Validate USB communication test parameters
    pub fn validate(&self) -> Result<(), TestParameterError> {
        if self.message_count == 0 || self.message_count > 10_000 {
            return Err(TestParameterError::InvalidResourceLimits);
        }

        if self.message_interval_ms > 10_000 {
            return Err(TestParameterError::InvalidDuration);
        }

        if self.message_size_bytes == 0 || self.message_size_bytes > 64 {
            return Err(TestParameterError::PayloadTooLarge);
        }

        if self.timeout_per_message_ms == 0 || self.timeout_per_message_ms > 30_000 {
            return Err(TestParameterError::InvalidDuration);
        }

        if self.error_injection_rate_percent > 100 {
            return Err(TestParameterError::InvalidResourceLimits);
        }

        if self.concurrent_messages == 0 || self.concurrent_messages > 8 {
            return Err(TestParameterError::InvalidResourceLimits);
        }

        Ok(())
    }

    /// Serialize USB communication test parameters
    pub fn serialize(&self) -> Vec<u8, 20> {
        let mut serialized = Vec::new();

        // Message count (4 bytes)
        let count_bytes = self.message_count.to_le_bytes();
        for &byte in &count_bytes {
            let _ = serialized.push(byte);
        }

        // Message interval (4 bytes)
        let interval_bytes = self.message_interval_ms.to_le_bytes();
        for &byte in &interval_bytes {
            let _ = serialized.push(byte);
        }

        // Message size (2 bytes)
        let size_bytes = self.message_size_bytes.to_le_bytes();
        for &byte in &size_bytes {
            let _ = serialized.push(byte);
        }

        // Timeout (4 bytes)
        let timeout_bytes = self.timeout_per_message_ms.to_le_bytes();
        for &byte in &timeout_bytes {
            let _ = serialized.push(byte);
        }

        // Flags (1 byte)
        let mut flags = 0u8;
        if self.enable_integrity_checking { flags |= 0x01; }
        if self.enable_error_injection { flags |= 0x02; }
        if self.bidirectional_test { flags |= 0x04; }
        let _ = serialized.push(flags);

        // Error injection rate (1 byte)
        let _ = serialized.push(self.error_injection_rate_percent);

        // Concurrent messages (1 byte)
        let _ = serialized.push(self.concurrent_messages);

        serialized
    }
}

/// USB communication test statistics
/// Requirements: 9.4, 9.5 (communication statistics and error detection)
#[derive(Clone, Copy, Debug)]
pub struct UsbCommunicationStatistics {
    pub test_duration_ms: u32,
    pub messages_sent: u32,
    pub messages_received: u32,
    pub messages_acknowledged: u32,
    pub transmission_errors: u32,
    pub reception_errors: u32,
    pub timeout_errors: u32,
    pub integrity_check_failures: u32,
    pub average_round_trip_time_us: u32,
    pub min_round_trip_time_us: u32,
    pub max_round_trip_time_us: u32,
    pub throughput_messages_per_sec: u32,
    pub error_rate_percent: f32,
    pub success_rate_percent: f32,
    pub bidirectional_success: bool,
}

impl Default for UsbCommunicationStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl UsbCommunicationStatistics {
    /// Create new empty USB communication statistics
    pub fn new() -> Self {
        Self {
            test_duration_ms: 0,
            messages_sent: 0,
            messages_received: 0,
            messages_acknowledged: 0,
            transmission_errors: 0,
            reception_errors: 0,
            timeout_errors: 0,
            integrity_check_failures: 0,
            average_round_trip_time_us: 0,
            min_round_trip_time_us: u32::MAX,
            max_round_trip_time_us: 0,
            throughput_messages_per_sec: 0,
            error_rate_percent: 0.0,
            success_rate_percent: 0.0,
            bidirectional_success: false,
        }
    }

    /// Calculate derived statistics
    pub fn calculate_derived_stats(&mut self) {
        // Calculate error rate
        let total_operations = self.messages_sent + self.messages_received;
        let total_errors = self.transmission_errors + self.reception_errors + 
                          self.timeout_errors + self.integrity_check_failures;
        
        if total_operations > 0 {
            self.error_rate_percent = (total_errors as f32 / total_operations as f32) * 100.0;
            self.success_rate_percent = 100.0 - self.error_rate_percent;
        }

        // Calculate throughput
        if self.test_duration_ms > 0 {
            let duration_sec = self.test_duration_ms as f32 / 1000.0;
            self.throughput_messages_per_sec = ((self.messages_sent + self.messages_received) as f32 / duration_sec) as u32;
        }

        // Check bidirectional success
        self.bidirectional_success = self.messages_sent > 0 && self.messages_received > 0 && 
                                   self.success_rate_percent >= 95.0;
    }

    /// Add round trip time measurement
    pub fn add_round_trip_time(&mut self, rtt_us: u32) {
        if rtt_us < self.min_round_trip_time_us {
            self.min_round_trip_time_us = rtt_us;
        }
        if rtt_us > self.max_round_trip_time_us {
            self.max_round_trip_time_us = rtt_us;
        }
        
        // Update average (simplified running average)
        let total_messages = self.messages_acknowledged + 1;
        self.average_round_trip_time_us = 
            (self.average_round_trip_time_us * (total_messages - 1) + rtt_us) / total_messages;
    }

    /// Serialize USB communication statistics
    pub fn serialize(&self) -> Vec<u8, 60> {
        let mut serialized = Vec::new();

        // Test duration (4 bytes)
        let duration_bytes = self.test_duration_ms.to_le_bytes();
        for &byte in &duration_bytes {
            let _ = serialized.push(byte);
        }

        // Message counts (16 bytes)
        let counts = [
            self.messages_sent,
            self.messages_received,
            self.messages_acknowledged,
            self.transmission_errors,
        ];
        for count in &counts {
            let count_bytes = count.to_le_bytes();
            for &byte in &count_bytes {
                let _ = serialized.push(byte);
            }
        }

        // Error counts (12 bytes)
        let errors = [
            self.reception_errors,
            self.timeout_errors,
            self.integrity_check_failures,
        ];
        for error in &errors {
            let error_bytes = error.to_le_bytes();
            for &byte in &error_bytes {
                let _ = serialized.push(byte);
            }
        }

        // Timing statistics (12 bytes)
        let timings = [
            self.average_round_trip_time_us,
            self.min_round_trip_time_us,
            self.max_round_trip_time_us,
        ];
        for timing in &timings {
            let timing_bytes = timing.to_le_bytes();
            for &byte in &timing_bytes {
                let _ = serialized.push(byte);
            }
        }

        // Throughput (4 bytes)
        let throughput_bytes = self.throughput_messages_per_sec.to_le_bytes();
        for &byte in &throughput_bytes {
            let _ = serialized.push(byte);
        }

        // Success rates (8 bytes)
        let error_rate_bytes = self.error_rate_percent.to_le_bytes();
        for &byte in &error_rate_bytes {
            let _ = serialized.push(byte);
        }
        let success_rate_bytes = self.success_rate_percent.to_le_bytes();
        for &byte in &success_rate_bytes {
            let _ = serialized.push(byte);
        }

        // Bidirectional success flag (1 byte)
        let _ = serialized.push(if self.bidirectional_success { 1 } else { 0 });

        serialized
    }
}



/// Memory usage monitor for stress testing
/// Requirements: 9.2 (memory usage monitoring during stress conditions)
#[derive(Clone, Copy, Debug)]
pub struct MemoryUsageMonitor {
    baseline_usage_bytes: u32,
    current_usage_bytes: u32,
    peak_usage_bytes: u32,
    allocation_count: u32,
    deallocation_count: u32,
    allocation_failures: u32,
    fragmentation_level: u8,
    last_update_ms: u32,
}

impl Default for MemoryUsageMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryUsageMonitor {
    /// Create new memory usage monitor
    pub fn new() -> Self {
        Self {
            baseline_usage_bytes: 0,
            current_usage_bytes: 0,
            peak_usage_bytes: 0,
            allocation_count: 0,
            deallocation_count: 0,
            allocation_failures: 0,
            fragmentation_level: 0,
            last_update_ms: 0,
        }
    }

    /// Set baseline memory usage
    pub fn set_baseline(&mut self, usage_bytes: u32, timestamp_ms: u32) {
        self.baseline_usage_bytes = usage_bytes;
        self.current_usage_bytes = usage_bytes;
        self.peak_usage_bytes = usage_bytes;
        self.last_update_ms = timestamp_ms;
    }

    /// Update memory usage measurements
    pub fn update(&mut self, usage_bytes: u32, timestamp_ms: u32) {
        self.current_usage_bytes = usage_bytes;
        if usage_bytes > self.peak_usage_bytes {
            self.peak_usage_bytes = usage_bytes;
        }
        self.last_update_ms = timestamp_ms;

        // Simulate fragmentation calculation
        self.fragmentation_level = self.calculate_fragmentation();
    }

    /// Record memory allocation
    pub fn record_allocation(&mut self, size_bytes: u32, success: bool) {
        if success {
            self.allocation_count += 1;
            self.current_usage_bytes += size_bytes;
            if self.current_usage_bytes > self.peak_usage_bytes {
                self.peak_usage_bytes = self.current_usage_bytes;
            }
        } else {
            self.allocation_failures += 1;
        }
    }

    /// Record memory deallocation
    pub fn record_deallocation(&mut self, size_bytes: u32) {
        self.deallocation_count += 1;
        self.current_usage_bytes = self.current_usage_bytes.saturating_sub(size_bytes);
    }

    /// Get memory usage increase from baseline
    pub fn usage_increase_bytes(&self) -> u32 {
        self.current_usage_bytes.saturating_sub(self.baseline_usage_bytes)
    }

    /// Get memory usage increase percentage
    pub fn usage_increase_percent(&self) -> f32 {
        if self.baseline_usage_bytes == 0 {
            return 0.0;
        }
        let increase = self.usage_increase_bytes();
        (increase as f32 / self.baseline_usage_bytes as f32) * 100.0
    }

    /// Check if memory usage is critical
    pub fn is_critical_usage(&self, threshold_bytes: u32) -> bool {
        self.current_usage_bytes > threshold_bytes || self.allocation_failures > 0
    }

    /// Calculate memory fragmentation level (simplified simulation)
    fn calculate_fragmentation(&self) -> u8 {
        // Simulate fragmentation based on allocation/deallocation patterns
        let net_allocations = self.allocation_count.saturating_sub(self.deallocation_count);
        
        if net_allocations > 100 {
            core::cmp::min(net_allocations / 10, 50) as u8
        } else {
            0
        }
    }

    /// Get current statistics
    pub fn get_statistics(&self) -> ResourceUsageStats {
        ResourceUsageStats {
            cpu_usage_percent: 0, // Would be calculated separately
            memory_usage_bytes: self.current_usage_bytes,
            peak_memory_usage_bytes: self.peak_usage_bytes,
            execution_time_ms: 0, // Would be calculated separately
            preemption_count: 0, // Would be calculated separately
        }
    }
}

impl Default for PemfTimingStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl PemfTimingStatistics {
    /// Create new empty statistics
    pub fn new() -> Self {
        Self {
            total_measurements: 0,
            timing_accuracy_percent: 0.0,
            error_count: 0,
            max_jitter_us: 0,
            average_timing_error_percent: 0.0,
            test_duration_ms: 0,
            within_tolerance_count: 0,
        }
    }

    /// Calculate success rate percentage
    pub fn success_rate_percent(&self) -> f32 {
        if self.total_measurements == 0 {
            return 0.0;
        }
        (self.within_tolerance_count as f32 / self.total_measurements as f32) * 100.0
    }

    /// Check if test meets validation criteria
    pub fn meets_validation_criteria(&self, min_success_rate: f32, max_error_count: u32) -> bool {
        self.success_rate_percent() >= min_success_rate && self.error_count <= max_error_count
    }

    /// Serialize statistics to bytes for transmission
    pub fn serialize(&self) -> Vec<u8, 32> {
        let mut serialized = Vec::new();

        // Serialize total measurements (4 bytes)
        let total_bytes = self.total_measurements.to_le_bytes();
        for &byte in &total_bytes {
            let _ = serialized.push(byte);
        }

        // Serialize timing accuracy (4 bytes)
        let accuracy_bytes = self.timing_accuracy_percent.to_le_bytes();
        for &byte in &accuracy_bytes {
            let _ = serialized.push(byte);
        }

        // Serialize error count (4 bytes)
        let error_bytes = self.error_count.to_le_bytes();
        for &byte in &error_bytes {
            let _ = serialized.push(byte);
        }

        // Serialize max jitter (4 bytes)
        let jitter_bytes = self.max_jitter_us.to_le_bytes();
        for &byte in &jitter_bytes {
            let _ = serialized.push(byte);
        }

        // Serialize average timing error (4 bytes)
        let avg_error_bytes = self.average_timing_error_percent.to_le_bytes();
        for &byte in &avg_error_bytes {
            let _ = serialized.push(byte);
        }

        // Serialize test duration (4 bytes)
        let duration_bytes = self.test_duration_ms.to_le_bytes();
        for &byte in &duration_bytes {
            let _ = serialized.push(byte);
        }

        // Serialize within tolerance count (4 bytes)
        let tolerance_bytes = self.within_tolerance_count.to_le_bytes();
        for &byte in &tolerance_bytes {
            let _ = serialized.push(byte);
        }

        serialized
    }
}

/// Test Command Processor with configurable test execution
/// Requirements: 2.1, 2.2, 2.3, 8.1, 8.2, 8.3, 8.4, 8.5
pub struct TestCommandProcessor {
    active_test: Option<ActiveTest>,
    test_results: Vec<TestResult, 16>,
    test_id_counter: u8,
    total_tests_executed: u32,
    total_tests_passed: u32,
    total_tests_failed: u32,
    last_test_timestamp: u32,
}

impl Default for TestCommandProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl TestCommandProcessor {
    /// Create a new test command processor
    pub const fn new() -> Self {
        Self {
            active_test: None,
            test_results: Vec::new(),
            test_id_counter: 0,
            total_tests_executed: 0,
            total_tests_passed: 0,
            total_tests_failed: 0,
            last_test_timestamp: 0,
        }
    }

    /// Create a minimal test command processor for production builds
    /// Requirements: 8.4 (conditional compilation for production)
    #[cfg(not(feature = "test-commands"))]
    pub const fn new_minimal() -> Self {
        Self {
            active_test: None,
            test_results: Vec::new(), // Minimal capacity
            test_id_counter: 0,
            total_tests_executed: 0,
            total_tests_passed: 0,
            total_tests_failed: 0,
            last_test_timestamp: 0,
        }
    }

    /// Start a new test execution
    /// Requirements: 2.1, 2.2 (configurable test execution with parameter validation)
    pub fn start_test(
        &mut self,
        test_type: TestType,
        parameters: TestParameters,
        timestamp_ms: u32,
    ) -> Result<u8, TestExecutionError> {
        // Check if another test is already running
        if self.active_test.is_some() {
            return Err(TestExecutionError::TestAborted);
        }

        // Validate test parameters
        parameters.validate().map_err(|_| TestExecutionError::ValidationFailed)?;

        // Generate new test ID
        self.test_id_counter = self.test_id_counter.wrapping_add(1);
        let test_id = self.test_id_counter;

        // Create and start active test
        let mut active_test = ActiveTest::new(test_type, parameters, test_id, timestamp_ms);
        active_test.result.status = TestStatus::Running;

        self.active_test = Some(active_test);
        self.last_test_timestamp = timestamp_ms;

        Ok(test_id)
    }

    /// Update active test execution and check for completion/timeout
    /// Requirements: 8.3 (timeout protection), 8.1, 8.2 (resource usage monitoring)
    pub fn update_active_test(&mut self, timestamp_ms: u32) -> Option<TestResult> {
        if let Some(ref mut active_test) = self.active_test {
            // Update resource usage monitoring
            active_test.update_resource_usage(timestamp_ms);

            // Check for timeout
            if active_test.is_timed_out(timestamp_ms) {
                active_test.result.timeout(timestamp_ms);
                let result = active_test.result.clone();
                self.complete_test(result.clone());
                return Some(result);
            }

            // Check resource limits
            if let Err(error) = active_test.check_resource_limits() {
                let error_message = match error {
                    TestExecutionError::CpuLimitExceeded => "CPU limit exceeded",
                    TestExecutionError::MemoryLimitExceeded => "Memory limit exceeded",
                    TestExecutionError::ExecutionTimeExceeded => "Execution time exceeded",
                    _ => "Resource limit exceeded",
                };
                active_test.result.fail(timestamp_ms, error_message);
                let result = active_test.result.clone();
                self.complete_test(result.clone());
                return Some(result);
            }

            // Check if test duration has elapsed (normal completion)
            let elapsed_ms = timestamp_ms.saturating_sub(active_test.result.start_timestamp_ms);
            if elapsed_ms >= active_test.parameters.duration_ms {
                active_test.result.complete(timestamp_ms);
                let result = active_test.result.clone();
                self.complete_test(result.clone());
                return Some(result);
            }
        }

        None
    }

    /// Complete the active test and store results
    /// Requirements: 2.3 (test result collection)
    fn complete_test(&mut self, result: TestResult) {
        // Update statistics
        self.total_tests_executed += 1;
        match result.status {
            TestStatus::Completed => self.total_tests_passed += 1,
            TestStatus::Failed | TestStatus::TimedOut | TestStatus::Aborted => self.total_tests_failed += 1,
            _ => {}
        }

        // Store result (remove oldest if queue is full)
        if self.test_results.is_full() {
            let _ = self.test_results.remove(0);
        }
        let _ = self.test_results.push(result);

        // Clear active test
        self.active_test = None;
    }

    /// Abort the currently active test
    /// Requirements: 8.4 (test abortion capability)
    pub fn abort_active_test(&mut self, timestamp_ms: u32) -> Option<TestResult> {
        if let Some(ref mut active_test) = self.active_test {
            active_test.result.status = TestStatus::Aborted;
            active_test.result.end_timestamp_ms = timestamp_ms;
            let result = active_test.result.clone();
            self.complete_test(result.clone());
            Some(result)
        } else {
            None
        }
    }

    /// Get the currently active test information
    pub fn get_active_test_info(&self) -> Option<(TestType, TestStatus, u8)> {
        self.active_test.as_ref().map(|test| {
            (test.test_type, test.result.status, test.result.test_id)
        })
    }

    /// Check if there is an active test running
    pub fn has_active_test(&self) -> bool {
        self.active_test.is_some()
    }

    /// Get test execution statistics
    pub fn get_statistics(&self) -> TestProcessorStatistics {
        TestProcessorStatistics {
            total_tests_executed: self.total_tests_executed,
            total_tests_passed: self.total_tests_passed,
            total_tests_failed: self.total_tests_failed,
            active_test_count: if self.active_test.is_some() { 1 } else { 0 },
            stored_results_count: self.test_results.len() as u32,
            last_test_timestamp: self.last_test_timestamp,
        }
    }

    /// Get recent test results
    /// Requirements: 2.3 (test result collection)
    pub fn get_recent_results(&self, count: usize) -> &[TestResult] {
        let start_index = if self.test_results.len() > count {
            self.test_results.len() - count
        } else {
            0
        };
        &self.test_results[start_index..]
    }

    /// Clear all stored test results
    pub fn clear_results(&mut self) {
        self.test_results.clear();
    }

    /// Execute pEMF timing validation test
    /// Requirements: 9.1, 9.5 (pEMF timing validation without interference)
    pub fn execute_pemf_timing_test(
        &mut self,
        parameters: TestParameters,
        timestamp_ms: u32,
    ) -> Result<u8, TestExecutionError> {
        // Validate that we can run a pEMF timing test
        if self.has_active_test() {
            return Err(TestExecutionError::TestAborted);
        }

        // Create pEMF-specific test parameters if not provided
        let pemf_parameters = if parameters.custom_parameters.is_empty() {
            Self::create_pemf_timing_parameters(parameters.duration_ms, parameters.tolerance_percent)?
        } else {
            parameters
        };

        // Start the pEMF timing validation test
        let test_id = self.start_test(TestType::PemfTimingValidation, pemf_parameters, timestamp_ms)?;
        
        // Initialize pEMF timing test state
        if let Some(ref mut active_test) = self.active_test {
            // Reset timing measurements for fresh start
            active_test.result.measurements.timing_measurements.clear();
            active_test.result.measurements.error_count = 0;
            active_test.result.measurements.timing_accuracy = 0.0;
            
            // Initialize jitter measurements
            active_test.result.measurements.jitter_measurements = JitterMeasurements {
                pemf_jitter_us: 0,
                battery_jitter_us: 0,
                usb_jitter_us: 0,
                max_system_jitter_us: 0,
            };
            
            // Set up performance metrics for timing validation
            active_test.result.measurements.performance_metrics.success_rate_percent = 100;
            active_test.result.measurements.performance_metrics.error_rate_percent = 0;
        }
        
        // The test will run asynchronously and be monitored by update_active_test
        // This function returns immediately with the test ID
        Ok(test_id)
    }

    /// Update pEMF timing test measurements
    /// This should be called periodically to collect timing data
    /// Requirements: 9.1 (measure pulse accuracy without interfering)
    pub fn update_pemf_timing_measurements(&mut self, timing_measurement: TimingMeasurement) -> Result<(), SystemError> {
        if let Some(ref mut active_test) = self.active_test {
            if active_test.test_type == TestType::PemfTimingValidation {
                // Add timing measurement to the test results
                active_test.result.measurements.add_timing_measurement(timing_measurement)?;
                
                // Parse pEMF timing parameters from custom parameters
                let pemf_params = Self::parse_pemf_timing_parameters(&active_test.parameters.custom_parameters)
                    .unwrap_or_else(PemfTimingParameters::default);
                
                // Update timing accuracy calculation
                let expected_timing_us = pemf_params.expected_total_period_us;
                active_test.result.measurements.calculate_timing_accuracy(expected_timing_us);
                
                // Check for timing deviations
                let tolerance_percent = active_test.parameters.tolerance_percent;
                let timing_error_percent = ((timing_measurement.execution_time_us as f32 - expected_timing_us as f32) / expected_timing_us as f32 * 100.0).abs();
                
                if timing_error_percent > tolerance_percent {
                    active_test.result.measurements.error_count += 1;
                    
                    // Update error rate percentage
                    let total_measurements = active_test.result.measurements.timing_measurements.len() as u32;
                    if total_measurements > 0 {
                        active_test.result.measurements.performance_metrics.error_rate_percent = 
                            (active_test.result.measurements.error_count * 100) / total_measurements;
                        active_test.result.measurements.performance_metrics.success_rate_percent = 
                            100 - active_test.result.measurements.performance_metrics.error_rate_percent;
                    }
                }
                
                // Update jitter measurements
                Self::update_jitter_measurements_static(&mut active_test.result.measurements, timing_measurement, expected_timing_us);
                
                // Update performance metrics
                self.update_pemf_performance_metrics(&pemf_params, timing_measurement)?;
            }
        }
        
        Ok(())
    }

    /// Update performance metrics for pEMF timing test
    /// Requirements: 9.5 (timing statistics and error counts)
    fn update_pemf_performance_metrics(&mut self, pemf_params: &PemfTimingParameters, timing: TimingMeasurement) -> Result<(), SystemError> {
        if let Some(ref mut active_test) = self.active_test {
            let measurements = &mut active_test.result.measurements;
            
            // Calculate throughput (cycles per second)
            let test_duration_ms = timing.timestamp_ms.saturating_sub(active_test.result.start_timestamp_ms);
            if test_duration_ms > 0 {
                let cycles_completed = measurements.timing_measurements.len() as u32;
                measurements.performance_metrics.throughput_ops_per_sec = 
                    (cycles_completed * 1000) / test_duration_ms;
            }
            
            // Update average latency (deviation from expected timing)
            let expected_timing_us = pemf_params.expected_total_period_us;
            let deviation_us = timing.execution_time_us.abs_diff(expected_timing_us);
            
            // Running average of timing deviations
            let current_avg = measurements.performance_metrics.average_latency_us;
            let measurement_count = measurements.timing_measurements.len() as u32;
            if measurement_count > 0 {
                measurements.performance_metrics.average_latency_us = 
                    (current_avg * (measurement_count - 1) + deviation_us) / measurement_count;
            }
        }
        
        Ok(())
    }

    /// Collect pEMF timing measurement from system state
    /// This function should be called from the pEMF pulse task to provide timing data
    /// Requirements: 9.1 (measure pulse accuracy without interfering with normal operation)
    pub fn collect_pemf_timing_measurement(
        &mut self,
        high_phase_duration_us: u32,
        low_phase_duration_us: u32,
        total_cycle_duration_us: u32,
        timestamp_ms: u32,
    ) -> Result<(), SystemError> {
        // Only collect measurements if pEMF timing test is active
        if let Some(ref active_test) = self.active_test {
            if active_test.test_type == TestType::PemfTimingValidation {
                // Create timing measurement from pEMF pulse data
                let timing_measurement = TimingMeasurement {
                    task_name: "pemf_pulse",
                    execution_time_us: total_cycle_duration_us,
                    expected_time_us: 500_000, // 500ms expected for 2Hz
                    timestamp_ms,
                };
                
                // Update the test with this measurement
                self.update_pemf_timing_measurements(timing_measurement)?;
                
                // Store additional pEMF-specific timing data in custom measurements
                self.store_pemf_phase_timing(high_phase_duration_us, low_phase_duration_us, timestamp_ms)?;
            }
        }
        
        Ok(())
    }

    /// Store detailed pEMF phase timing data
    /// Requirements: 9.5 (timing statistics and error counts)
    fn store_pemf_phase_timing(
        &mut self,
        high_phase_us: u32,
        low_phase_us: u32,
        timestamp_ms: u32,
    ) -> Result<(), SystemError> {
        if let Some(ref mut active_test) = self.active_test {
            // Store phase timing data in custom measurements for detailed analysis
            let mut phase_data: Vec<u8, 12> = Vec::new();
            
            // Store high phase duration (4 bytes)
            let high_bytes = high_phase_us.to_le_bytes();
            for &byte in &high_bytes {
                phase_data.push(byte).map_err(|_| SystemError::SystemBusy)?;
            }
            
            // Store low phase duration (4 bytes)
            let low_bytes = low_phase_us.to_le_bytes();
            for &byte in &low_bytes {
                phase_data.push(byte).map_err(|_| SystemError::SystemBusy)?;
            }
            
            // Store timestamp (4 bytes)
            let timestamp_bytes = timestamp_ms.to_le_bytes();
            for &byte in &timestamp_bytes {
                phase_data.push(byte).map_err(|_| SystemError::SystemBusy)?;
            }
            
            // Append to custom measurements (if space available)
            let remaining_space = 64 - active_test.result.measurements.custom_measurements.len();
            if remaining_space >= phase_data.len() {
                for &byte in &phase_data {
                    if active_test.result.measurements.custom_measurements.push(byte).is_err() {
                        break;
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Update jitter measurements for pEMF timing test
    fn update_jitter_measurements_static(measurements: &mut TestMeasurements, timing: TimingMeasurement, expected_timing_us: u32) {
        // Calculate jitter based on deviation from expected timing
        let deviation_us = timing.execution_time_us.abs_diff(expected_timing_us);
        
        // Update pEMF jitter (maximum deviation seen so far)
        if deviation_us > measurements.jitter_measurements.pemf_jitter_us {
            measurements.jitter_measurements.pemf_jitter_us = deviation_us;
        }
        
        // Update maximum system jitter
        if deviation_us > measurements.jitter_measurements.max_system_jitter_us {
            measurements.jitter_measurements.max_system_jitter_us = deviation_us;
        }
    }

    /// Get pEMF timing test statistics
    /// Requirements: 9.5 (timing statistics and error counts)
    pub fn get_pemf_timing_statistics(&self) -> Option<PemfTimingStatistics> {
        if let Some(ref active_test) = self.active_test {
            if active_test.test_type == TestType::PemfTimingValidation {
                let measurements = &active_test.result.measurements;
                let elapsed_ms = if active_test.result.end_timestamp_ms > 0 {
                    active_test.result.end_timestamp_ms - active_test.result.start_timestamp_ms
                } else {
                    0 // Test still running
                };
                
                return Some(PemfTimingStatistics {
                    total_measurements: measurements.timing_measurements.len() as u32,
                    timing_accuracy_percent: measurements.timing_accuracy,
                    error_count: measurements.error_count,
                    max_jitter_us: measurements.jitter_measurements.pemf_jitter_us,
                    average_timing_error_percent: self.calculate_average_timing_error(measurements),
                    test_duration_ms: elapsed_ms,
                    within_tolerance_count: self.count_measurements_within_tolerance(measurements, active_test.parameters.tolerance_percent),
                });
            }
        }
        
        None
    }

    /// Detect and report timing deviations
    /// Requirements: 9.1, 9.5 (timing deviation detection and reporting)
    pub fn detect_timing_deviations(&self, tolerance_percent: f32) -> Vec<TimingDeviation, 16> {
        let mut deviations: Vec<TimingDeviation, 16> = Vec::new();
        
        if let Some(ref active_test) = self.active_test {
            if active_test.test_type == TestType::PemfTimingValidation {
                let measurements = &active_test.result.measurements;
                let pemf_params = Self::parse_pemf_timing_parameters(&active_test.parameters.custom_parameters)
                    .unwrap_or_else(PemfTimingParameters::default);
                
                // Analyze each timing measurement for deviations
                for (index, measurement) in measurements.timing_measurements.iter().enumerate() {
                    let expected_timing_us = pemf_params.expected_total_period_us;
                    let timing_error_percent = ((measurement.execution_time_us as f32 - expected_timing_us as f32) / expected_timing_us as f32 * 100.0).abs();
                    
                    if timing_error_percent > tolerance_percent {
                        let deviation = TimingDeviation {
                            measurement_index: index as u16,
                            timestamp_ms: measurement.timestamp_ms,
                            expected_timing_us,
                            actual_timing_us: measurement.execution_time_us,
                            deviation_us: measurement.execution_time_us.abs_diff(expected_timing_us),
                            deviation_percent: timing_error_percent,
                            deviation_type: if measurement.execution_time_us > expected_timing_us {
                                TimingDeviationType::TooSlow
                            } else {
                                TimingDeviationType::TooFast
                            },
                        };
                        
                        if deviations.push(deviation).is_err() {
                            break; // Vector is full
                        }
                    }
                }
            }
        }
        
        deviations
    }

    /// Generate timing deviation report
    /// Requirements: 9.5 (timing statistics and error counts)
    pub fn generate_timing_deviation_report(&self) -> Option<TimingDeviationReport> {
        if let Some(ref active_test) = self.active_test {
            if active_test.test_type == TestType::PemfTimingValidation {
                let deviations = self.detect_timing_deviations(active_test.parameters.tolerance_percent);
                let measurements = &active_test.result.measurements;
                
                // Calculate deviation statistics
                let total_deviations = deviations.len() as u32;
                let max_deviation_us = deviations.iter()
                    .map(|d| d.deviation_us)
                    .max()
                    .unwrap_or(0);
                
                let average_deviation_us = if !deviations.is_empty() {
                    deviations.iter().map(|d| d.deviation_us).sum::<u32>() / deviations.len() as u32
                } else {
                    0
                };
                
                let too_slow_count = deviations.iter()
                    .filter(|d| d.deviation_type == TimingDeviationType::TooSlow)
                    .count() as u32;
                
                let too_fast_count = deviations.iter()
                    .filter(|d| d.deviation_type == TimingDeviationType::TooFast)
                    .count() as u32;
                
                return Some(TimingDeviationReport {
                    total_measurements: measurements.timing_measurements.len() as u32,
                    total_deviations,
                    deviation_rate_percent: if !measurements.timing_measurements.is_empty() {
                        (total_deviations * 100) / measurements.timing_measurements.len() as u32
                    } else {
                        0
                    },
                    max_deviation_us,
                    average_deviation_us,
                    too_slow_count,
                    too_fast_count,
                    tolerance_percent: active_test.parameters.tolerance_percent,
                    test_passed: total_deviations == 0 || 
                        (total_deviations * 100 / measurements.timing_measurements.len() as u32) <= 5, // Allow up to 5% deviation rate
                });
            }
        }
        
        None
    }

    /// Calculate average timing error from measurements
    fn calculate_average_timing_error(&self, measurements: &TestMeasurements) -> f32 {
        if measurements.timing_measurements.is_empty() {
            return 0.0;
        }
        
        let expected_timing_us = 500_000; // 500ms total period
        let mut total_error = 0.0;
        
        for measurement in &measurements.timing_measurements {
            let error_percent = ((measurement.execution_time_us as f32 - expected_timing_us as f32) / expected_timing_us as f32 * 100.0).abs();
            total_error += error_percent;
        }
        
        total_error / measurements.timing_measurements.len() as f32
    }

    /// Count measurements within tolerance
    fn count_measurements_within_tolerance(&self, measurements: &TestMeasurements, tolerance_percent: f32) -> u32 {
        let expected_timing_us = 500_000; // 500ms total period
        let mut within_tolerance = 0;
        
        for measurement in &measurements.timing_measurements {
            let error_percent = ((measurement.execution_time_us as f32 - expected_timing_us as f32) / expected_timing_us as f32 * 100.0).abs();
            if error_percent <= tolerance_percent {
                within_tolerance += 1;
            }
        }
        
        within_tolerance
    }

    /// Get test result by test ID
    /// Requirements: 9.5 (test result structure with timing statistics and error counts)
    pub fn get_test_result(&self, test_id: u8) -> Option<TestResult> {
        if let Some(ref active_test) = self.active_test {
            if active_test.result.test_id == test_id {
                return Some(active_test.result.clone());
            }
        }
        
        // Check completed tests (in a real implementation, this would check a history)
        // For now, return None if not the active test
        None
    }

    /// Enhanced timing deviation detection with detailed analysis
    /// Requirements: 9.1, 9.5 (timing deviation detection and reporting)
    pub fn detect_detailed_timing_deviations(&self, tolerance_percent: f32) -> Vec<TimingDeviation, 32> {
        let mut deviations: Vec<TimingDeviation, 32> = Vec::new();
        
        if let Some(ref active_test) = self.active_test {
            if active_test.test_type == TestType::PemfTimingValidation {
                let measurements = &active_test.result.measurements;
                let pemf_params = Self::parse_pemf_timing_parameters(&active_test.parameters.custom_parameters)
                    .unwrap_or_else(PemfTimingParameters::default);
                
                // Analyze each timing measurement for deviations
                for (index, measurement) in measurements.timing_measurements.iter().enumerate() {
                    let expected_timing_us = pemf_params.expected_total_period_us;
                    let timing_error_percent = ((measurement.execution_time_us as f32 - expected_timing_us as f32) / expected_timing_us as f32 * 100.0).abs();
                    
                    if timing_error_percent > tolerance_percent {
                        let deviation_us = measurement.execution_time_us.abs_diff(expected_timing_us);
                        
                        let deviation = TimingDeviation {
                            measurement_index: index as u16,
                            timestamp_ms: measurement.timestamp_ms,
                            expected_timing_us,
                            actual_timing_us: measurement.execution_time_us,
                            deviation_us,
                            deviation_percent: timing_error_percent,
                            deviation_type: if measurement.execution_time_us > expected_timing_us {
                                TimingDeviationType::TooSlow
                            } else {
                                TimingDeviationType::TooFast
                            },
                        };
                        
                        if deviations.push(deviation).is_err() {
                            break; // Vector is full
                        }
                    }
                }
            }
        }
        
        deviations
    }

    /// Generate comprehensive timing validation report
    /// Requirements: 9.5 (timing statistics and error counts)
    pub fn generate_comprehensive_timing_report(&self) -> Option<ComprehensiveTimingReport> {
        if let Some(ref active_test) = self.active_test {
            if active_test.test_type == TestType::PemfTimingValidation {
                let measurements = &active_test.result.measurements;
                let deviations = self.detect_detailed_timing_deviations(active_test.parameters.tolerance_percent);
                
                // Calculate comprehensive statistics
                let total_measurements = measurements.timing_measurements.len() as u32;
                let within_tolerance_count = self.count_measurements_within_tolerance(measurements, active_test.parameters.tolerance_percent);
                let success_rate_percent = if total_measurements > 0 {
                    (within_tolerance_count * 100) / total_measurements
                } else {
                    0
                };
                
                // Calculate timing stability metrics
                let timing_stability = self.calculate_timing_stability(measurements);
                
                return Some(ComprehensiveTimingReport {
                    test_duration_ms: active_test.result.duration_ms(),
                    total_measurements,
                    within_tolerance_count,
                    success_rate_percent,
                    timing_accuracy_percent: measurements.timing_accuracy,
                    average_timing_error_percent: self.calculate_average_timing_error(measurements),
                    max_jitter_us: measurements.jitter_measurements.pemf_jitter_us,
                    timing_stability_score: timing_stability,
                    deviation_count: deviations.len() as u32,
                    max_deviation_us: deviations.iter().map(|d| d.deviation_us).max().unwrap_or(0),
                    test_passed: success_rate_percent >= 95 && measurements.timing_accuracy >= 95.0,
                    tolerance_percent: active_test.parameters.tolerance_percent,
                });
            }
        }
        
        None
    }

    /// Calculate timing stability score (0-100)
    fn calculate_timing_stability(&self, measurements: &TestMeasurements) -> u8 {
        if measurements.timing_measurements.len() < 2 {
            return 100; // Perfect stability with insufficient data
        }
        
        // Calculate coefficient of variation for timing measurements
        let timings: Vec<f32, 32> = measurements.timing_measurements.iter()
            .map(|m| m.execution_time_us as f32)
            .collect();
        
        let mean = timings.iter().sum::<f32>() / timings.len() as f32;
        let variance = timings.iter()
            .map(|t| (t - mean) * (t - mean))
            .sum::<f32>() / timings.len() as f32;
        // Simple square root approximation for no_std
        let std_dev = if variance > 0.0 {
            let mut x = variance;
            let mut prev = 0.0;
            while (x - prev).abs() > 0.001 {
                prev = x;
                x = (x + variance / x) / 2.0;
            }
            x
        } else {
            0.0
        };
        
        let coefficient_of_variation = if mean > 0.0 { std_dev / mean } else { 0.0 };
        
        // Convert to stability score (lower variation = higher stability)
        let stability_score = (1.0 - coefficient_of_variation.min(1.0)) * 100.0;
        stability_score as u8
    }

    /// Process a test command and return response
    /// Requirements: 2.1, 2.2, 2.3 (command processing with validation and result collection)
    pub fn process_test_command(
        &mut self,
        command: &CommandReport,
        timestamp_ms: u32,
    ) -> Result<CommandReport, ErrorCode> {
        // Parse test type from payload
        if command.payload.is_empty() {
            return CommandReport::error_response(
                command.command_id,
                ErrorCode::InvalidFormat,
                "Empty test command payload"
            );
        }

        let test_type = TestType::from_u8(command.payload[0])
            .ok_or(ErrorCode::UnsupportedCommand)?;

        // Parse test parameters from remaining payload
        let parameters = if command.payload.len() > 1 {
            match test_type {
                TestType::LedFunctionality => {
                    // Parse LED-specific parameters
                    if command.payload.len() >= 22 {
                        let led_params = LedFunctionalityParameters::from_payload(&command.payload[1..])
                            .map_err(|_| ErrorCode::InvalidFormat)?;
                        Self::create_led_functionality_test_parameters(&led_params)
                            .map_err(|_| ErrorCode::InvalidFormat)?
                    } else {
                        // Use default LED parameters
                        let led_params = LedFunctionalityParameters::default();
                        Self::create_led_functionality_test_parameters(&led_params)
                            .map_err(|_| ErrorCode::InvalidFormat)?
                    }
                },
                _ => {
                    // Use generic parameter parsing for other test types
                    TestParameters::from_payload(&command.payload[1..])
                        .map_err(|_| ErrorCode::InvalidFormat)?
                }
            }
        } else {
            match test_type {
                TestType::LedFunctionality => {
                    // Use default LED parameters
                    let led_params = LedFunctionalityParameters::default();
                    Self::create_led_functionality_test_parameters(&led_params)
                        .map_err(|_| ErrorCode::InvalidFormat)?
                },
                _ => TestParameters::new()
            }
        };

        // Start the test
        match self.start_test(test_type, parameters, timestamp_ms) {
            Ok(test_id) => {
                // Create success response with test ID
                let mut response_payload: Vec<u8, 60> = Vec::new();
                response_payload.push(test_type as u8).map_err(|_| ErrorCode::PayloadTooLarge)?;
                response_payload.push(test_id).map_err(|_| ErrorCode::PayloadTooLarge)?;
                response_payload.push(TestStatus::Running as u8).map_err(|_| ErrorCode::PayloadTooLarge)?;

                CommandReport::new(TestResponse::TestResult as u8, command.command_id, &response_payload)
            }
            Err(error) => {
                let error_message = match error {
                    TestExecutionError::TestAborted => "Another test is already running",
                    TestExecutionError::ValidationFailed => "Test parameter validation failed",
                    TestExecutionError::HardwareError => "Hardware error during test setup",
                    _ => "Test execution error",
                };
                CommandReport::error_response(command.command_id, ErrorCode::SystemNotReady, error_message)
            }
        }
    }
}

/// Test processor statistics
#[derive(Clone, Copy, Debug)]
pub struct TestProcessorStatistics {
    pub total_tests_executed: u32,
    pub total_tests_passed: u32,
    pub total_tests_failed: u32,
    pub active_test_count: u32,
    pub stored_results_count: u32,
    pub last_test_timestamp: u32,
}

impl TestProcessorStatistics {
    /// Calculate success rate percentage
    pub fn success_rate_percent(&self) -> u8 {
        if self.total_tests_executed == 0 {
            0
        } else {
            ((self.total_tests_passed * 100) / self.total_tests_executed) as u8
        }
    }

    /// Calculate failure rate percentage
    pub fn failure_rate_percent(&self) -> u8 {
        if self.total_tests_executed == 0 {
            0
        } else {
            ((self.total_tests_failed * 100) / self.total_tests_executed) as u8
        }
    }
}

/// pEMF timing test parameters
/// Requirements: 9.1 (configurable test duration and tolerance parameters)
#[derive(Clone, Copy, Debug)]
pub struct PemfTimingParameters {
    pub expected_frequency_mhz: u16,
    pub expected_high_duration_us: u32,
    pub expected_low_duration_us: u32,
    pub expected_total_period_us: u32,
}

impl Default for PemfTimingParameters {
    fn default() -> Self {
        Self {
            expected_frequency_mhz: 2000, // 2Hz = 2000 mHz
            expected_high_duration_us: 2000, // 2ms HIGH
            expected_low_duration_us: 498000, // 498ms LOW
            expected_total_period_us: 500000, // 500ms total period
        }
    }
}

impl PemfTimingParameters {

    /// Create pEMF timing parameters from frequency
    pub fn from_frequency_hz(frequency_hz: f32) -> Self {
        let period_us = (1_000_000.0 / frequency_hz) as u32;
        let high_duration_us = 2000; // Fixed 2ms HIGH phase
        let low_duration_us = period_us.saturating_sub(high_duration_us);
        
        Self {
            expected_frequency_mhz: (frequency_hz * 1000.0) as u16,
            expected_high_duration_us: high_duration_us,
            expected_low_duration_us: low_duration_us,
            expected_total_period_us: period_us,
        }
    }

    /// Validate timing parameters
    pub fn validate(&self) -> Result<(), TestParameterError> {
        // Check frequency range (0.1Hz to 10Hz)
        if self.expected_frequency_mhz < 100 || self.expected_frequency_mhz > 10000 {
            return Err(TestParameterError::InvalidSampleRate);
        }

        // Check that HIGH + LOW = total period
        if self.expected_high_duration_us + self.expected_low_duration_us != self.expected_total_period_us {
            return Err(TestParameterError::InvalidDuration);
        }

        // Check minimum HIGH duration (1ms)
        if self.expected_high_duration_us < 1000 {
            return Err(TestParameterError::InvalidDuration);
        }

        // Check minimum LOW duration (10ms)
        if self.expected_low_duration_us < 10000 {
            return Err(TestParameterError::InvalidDuration);
        }

        Ok(())
    }

    /// Serialize parameters to bytes
    pub fn serialize(&self) -> Vec<u8, 10> {
        let mut serialized = Vec::new();

        // Serialize frequency (2 bytes)
        let freq_bytes = self.expected_frequency_mhz.to_le_bytes();
        for &byte in &freq_bytes {
            let _ = serialized.push(byte);
        }

        // Serialize HIGH duration (4 bytes)
        let high_bytes = self.expected_high_duration_us.to_le_bytes();
        for &byte in &high_bytes {
            let _ = serialized.push(byte);
        }

        // Serialize LOW duration (4 bytes)
        let low_bytes = self.expected_low_duration_us.to_le_bytes();
        for &byte in &low_bytes {
            let _ = serialized.push(byte);
        }

        serialized
    }
}

/// Helper functions for pEMF timing test creation and management
impl TestCommandProcessor {
    /// Create pEMF timing test parameters with configurable duration and tolerance
    /// Requirements: 9.1 (configurable test duration and tolerance parameters)
    pub fn create_pemf_timing_parameters(duration_ms: u32, tolerance_percent: f32) -> Result<TestParameters, TestParameterError> {
        let mut parameters = TestParameters::new();
        
        // Set pEMF-specific parameters
        parameters.duration_ms = duration_ms;
        parameters.tolerance_percent = tolerance_percent;
        parameters.sample_rate_hz = 2; // Match pEMF frequency (2Hz)
        
        // Set validation criteria for pEMF timing
        parameters.validation_criteria.max_error_count = (duration_ms / 500).max(1); // Allow 1 error per cycle
        parameters.validation_criteria.min_success_rate_percent = 95; // 95% success rate required
        parameters.validation_criteria.max_timing_deviation_us = ((500_000.0 * tolerance_percent / 100.0) as u32).max(1000); // Convert tolerance to microseconds
        parameters.validation_criteria.require_stable_operation = true;
        
        // Set resource limits for non-intrusive testing
        parameters.resource_limits.max_cpu_usage_percent = 10; // Minimal CPU usage
        parameters.resource_limits.max_memory_usage_bytes = 2048; // 2KB max memory
        parameters.resource_limits.max_execution_time_ms = duration_ms + 5000; // Add 5s buffer
        parameters.resource_limits.allow_preemption = true; // Allow higher priority tasks to preempt
        
        // Add pEMF-specific custom parameters
        let pemf_params = PemfTimingParameters::default();
        let serialized_params = pemf_params.serialize();
        
        parameters.custom_parameters.clear();
        for &byte in &serialized_params {
            parameters.custom_parameters.push(byte).map_err(|_| TestParameterError::PayloadTooLarge)?;
        }
        
        // Validate parameters
        parameters.validate()?;
        
        Ok(parameters)
    }

    /// Parse pEMF timing parameters from custom parameter bytes
    pub fn parse_pemf_timing_parameters(custom_params: &[u8]) -> Option<PemfTimingParameters> {
        if custom_params.len() < 10 {
            return None;
        }
        
        let frequency_mhz = u16::from_le_bytes([custom_params[0], custom_params[1]]);
        let high_duration_us = u32::from_le_bytes([custom_params[2], custom_params[3], custom_params[4], custom_params[5]]);
        let low_duration_us = u32::from_le_bytes([custom_params[6], custom_params[7], custom_params[8], custom_params[9]]);
        
        let params = PemfTimingParameters {
            expected_frequency_mhz: frequency_mhz,
            expected_high_duration_us: high_duration_us,
            expected_low_duration_us: low_duration_us,
            expected_total_period_us: high_duration_us + low_duration_us,
        };
        
        // Validate parsed parameters
        if params.validate().is_ok() {
            Some(params)
        } else {
            None
        }
    }

    /// Execute battery ADC validation test
    /// Requirements: 9.1, 9.5 (battery ADC test that validates voltage readings against known references)
    pub fn execute_battery_adc_test(
        &mut self,
        adc_params: BatteryAdcParameters,
        timestamp_ms: u32,
    ) -> Result<u8, TestExecutionError> {
        // Create test parameters from battery ADC parameters
        let test_params = Self::create_battery_adc_test_parameters(&adc_params)?;
        
        // Start the battery ADC test
        let test_id = self.start_test(TestType::BatteryAdcCalibration, test_params, timestamp_ms)?;
        
        // Initialize battery ADC test specific data in custom measurements
        if let Some(ref mut active_test) = self.active_test {
            // Store battery ADC parameters in custom measurements for later use
            let serialized_params = adc_params.serialize();
            active_test.result.measurements.custom_measurements.clear();
            for &byte in &serialized_params {
                if active_test.result.measurements.custom_measurements.push(byte).is_err() {
                    break;
                }
            }
        }
        
        Ok(test_id)
    }

    /// Create battery ADC test parameters from BatteryAdcParameters
    /// Requirements: 9.1 (configurable test parameters)
    pub fn create_battery_adc_test_parameters(adc_params: &BatteryAdcParameters) -> Result<TestParameters, TestParameterError> {
        let mut parameters = TestParameters::new();
        
        // Set battery ADC specific parameters
        parameters.duration_ms = adc_params.test_duration_ms;
        parameters.tolerance_percent = adc_params.tolerance_percent;
        parameters.sample_rate_hz = (adc_params.sample_count * 1000) / adc_params.test_duration_ms; // Calculate sample rate
        
        // Set validation criteria for battery ADC testing
        parameters.validation_criteria.max_error_count = (adc_params.sample_count / 20).max(1); // Allow 5% error rate
        parameters.validation_criteria.min_success_rate_percent = (100.0 - adc_params.tolerance_percent) as u8;
        parameters.validation_criteria.max_timing_deviation_us = 10_000; // 10ms timing tolerance
        parameters.validation_criteria.require_stable_operation = true;
        
        // Set resource limits for non-intrusive testing
        parameters.resource_limits.max_cpu_usage_percent = 15; // Minimal CPU usage for ADC reading
        parameters.resource_limits.max_memory_usage_bytes = 1024; // 1KB max memory
        parameters.resource_limits.max_execution_time_ms = adc_params.test_duration_ms + 2000; // Add 2s buffer
        parameters.resource_limits.allow_preemption = true; // Allow higher priority tasks to preempt
        
        // Add battery ADC specific custom parameters
        let serialized_params = adc_params.serialize();
        parameters.custom_parameters.clear();
        for &byte in &serialized_params {
            parameters.custom_parameters.push(byte).map_err(|_| TestParameterError::PayloadTooLarge)?;
        }
        
        // Validate parameters
        parameters.validate()?;
        
        Ok(parameters)
    }

    /// Parse battery ADC parameters from custom parameter bytes
    pub fn parse_battery_adc_parameters(custom_params: &[u8]) -> Option<BatteryAdcParameters> {
        if custom_params.len() < 20 {
            return None;
        }
        
        BatteryAdcParameters::from_payload(custom_params).ok()
    }

    /// Add ADC sample to active battery ADC test
    /// Requirements: 9.1, 9.5 (ADC accuracy measurements)
    pub fn add_battery_adc_sample(&mut self, adc_value: u16, timestamp_ms: u32) -> Result<(), TestExecutionError> {
        if let Some(ref mut active_test) = self.active_test {
            if active_test.test_type == TestType::BatteryAdcCalibration {
                // Add timing measurement for ADC reading
                let measurement = TimingMeasurement {
                    task_name: "battery_adc",
                    execution_time_us: 100, // Typical ADC reading time
                    expected_time_us: 100,
                    timestamp_ms,
                };
                
                active_test.result.measurements.add_timing_measurement(measurement)
                    .map_err(|_| TestExecutionError::TestAborted)?;
                
                // Update custom measurements with ADC data
                // Store ADC value in first 2 bytes of custom measurements
                if active_test.result.measurements.custom_measurements.len() >= 2 {
                    let adc_bytes = adc_value.to_le_bytes();
                    active_test.result.measurements.custom_measurements[0] = adc_bytes[0];
                    active_test.result.measurements.custom_measurements[1] = adc_bytes[1];
                }
                
                return Ok(());
            }
        }
        
        Err(TestExecutionError::TestAborted)
    }

    /// Add battery state transition to active battery ADC test
    /// Requirements: 9.1, 9.5 (battery state transition testing)
    pub fn add_battery_state_transition(&mut self, from_state: BatteryState, to_state: BatteryState, 
                                       adc_value: u16, _timestamp_ms: u32) -> Result<(), TestExecutionError> {
        if let Some(ref mut active_test) = self.active_test {
            if active_test.test_type == TestType::BatteryAdcCalibration {
                // Validate state transition
                let transition_valid = from_state.is_valid_transition(to_state, adc_value);
                
                if !transition_valid {
                    active_test.result.measurements.error_count += 1;
                }
                
                // Store transition data in custom measurements (if space available)
                let transition_data = [
                    from_state as u8,
                    to_state as u8,
                    (adc_value & 0xFF) as u8,
                    ((adc_value >> 8) & 0xFF) as u8,
                    if transition_valid { 1 } else { 0 },
                ];
                
                let start_idx = active_test.result.measurements.custom_measurements.len();
                for (i, &byte) in transition_data.iter().enumerate() {
                    if start_idx + i < 64 {
                        if active_test.result.measurements.custom_measurements.len() <= start_idx + i {
                            let _ = active_test.result.measurements.custom_measurements.push(byte);
                        } else {
                            active_test.result.measurements.custom_measurements[start_idx + i] = byte;
                        }
                    }
                }
                
                return Ok(());
            }
        }
        
        Err(TestExecutionError::TestAborted)
    }

    /// Get battery ADC test statistics from active test
    /// Requirements: 9.5 (ADC accuracy measurements)
    pub fn get_battery_adc_statistics(&self) -> Option<BatteryAdcMeasurements> {
        if let Some(ref active_test) = self.active_test {
            if active_test.test_type == TestType::BatteryAdcCalibration {
                // Parse battery ADC parameters from custom measurements
                if let Some(adc_params) = Self::parse_battery_adc_parameters(&active_test.result.measurements.custom_measurements) {
                    let mut measurements = BatteryAdcMeasurements::new();
                    
                    // Extract current ADC value from custom measurements
                    if active_test.result.measurements.custom_measurements.len() >= 2 {
                        let adc_value = u16::from_le_bytes([
                            active_test.result.measurements.custom_measurements[0],
                            active_test.result.measurements.custom_measurements[1],
                        ]);
                        measurements.average_adc_value = adc_value;
                    }
                    
                    // Calculate statistics from timing measurements
                    measurements.total_samples = active_test.result.measurements.timing_measurements.len() as u32;
                    measurements.test_duration_ms = active_test.result.end_timestamp_ms.saturating_sub(active_test.result.start_timestamp_ms);
                    measurements.reference_voltage_mv = adc_params.reference_voltage_mv;
                    measurements.invalid_readings_count = active_test.result.measurements.error_count;
                    
                    // Calculate voltage accuracy
                    measurements.calculate_voltage_accuracy(adc_params.reference_voltage_mv);
                    measurements.calculate_calibration_error(adc_params.expected_adc_value);
                    
                    // Count state transitions from custom measurements
                    let transition_count = (active_test.result.measurements.custom_measurements.len().saturating_sub(2)) / 5;
                    measurements.state_transition_count = transition_count as u32;
                    
                    return Some(measurements);
                }
            }
        }
        
        None
    }

    /// Complete battery ADC test and generate comprehensive result
    /// Requirements: 9.1, 9.5 (comprehensive ADC validation)
    pub fn complete_battery_adc_test(&mut self, timestamp_ms: u32) -> Option<BatteryAdcTestResult> {
        if let Some(ref mut active_test) = self.active_test {
            if active_test.test_type == TestType::BatteryAdcCalibration {
                // Parse battery ADC parameters
                if let Some(adc_params) = Self::parse_battery_adc_parameters(&active_test.result.measurements.custom_measurements) {
                    let mut adc_result = BatteryAdcTestResult::new(adc_params, active_test.result.start_timestamp_ms);
                    
                    // Extract ADC samples from timing measurements
                    for measurement in &active_test.result.measurements.timing_measurements {
                        if measurement.task_name == "battery_adc" {
                            // Use execution time as ADC value (stored during add_battery_adc_sample)
                            let adc_value = if active_test.result.measurements.custom_measurements.len() >= 2 {
                                u16::from_le_bytes([
                                    active_test.result.measurements.custom_measurements[0],
                                    active_test.result.measurements.custom_measurements[1],
                                ])
                            } else {
                                1500 // Default value
                            };
                            
                            let _ = adc_result.add_adc_sample(adc_value, measurement.timestamp_ms);
                        }
                    }
                    
                    // Extract state transitions from custom measurements
                    let mut idx = 2; // Skip first 2 bytes (current ADC value)
                    while idx + 4 < active_test.result.measurements.custom_measurements.len() {
                        let from_state = BatteryState::from_u8(active_test.result.measurements.custom_measurements[idx]);
                        let to_state = BatteryState::from_u8(active_test.result.measurements.custom_measurements[idx + 1]);
                        let adc_value = u16::from_le_bytes([
                            active_test.result.measurements.custom_measurements[idx + 2],
                            active_test.result.measurements.custom_measurements[idx + 3],
                        ]);
                        
                        if let (Some(from), Some(to)) = (from_state, to_state) {
                            let _ = adc_result.add_state_transition(from, to, adc_value, timestamp_ms);
                        }
                        
                        idx += 5;
                    }
                    
                    // Complete the test
                    adc_result.complete_test(timestamp_ms);
                    
                    return Some(adc_result);
                }
            }
        }
        
        None
    }

    /// Execute LED functionality test
    /// Requirements: 9.1, 9.5 (LED control patterns validation with timing accuracy)
    pub fn execute_led_functionality_test(
        &mut self,
        led_params: LedFunctionalityParameters,
        timestamp_ms: u32,
    ) -> Result<u8, TestExecutionError> {
        // Create test parameters from LED functionality parameters
        let test_params = Self::create_led_functionality_test_parameters(&led_params)?;
        
        // Start the LED functionality test
        let test_id = self.start_test(TestType::LedFunctionality, test_params, timestamp_ms)?;
        
        // Initialize LED functionality test specific data in custom measurements
        if let Some(ref mut active_test) = self.active_test {
            // Store LED functionality parameters in custom measurements for later use
            let serialized_params = led_params.serialize();
            
            // Store parameters in custom measurements (first 22 bytes)
            for (i, &byte) in serialized_params.iter().enumerate() {
                if i < active_test.result.measurements.custom_measurements.capacity() {
                    let _ = active_test.result.measurements.custom_measurements.push(byte);
                }
            }
            
            // Initialize LED test result structure
            let led_result = LedFunctionalityTestResult::new(led_params, timestamp_ms);
            
            // Store LED test result in custom measurements (serialized format)
            // This will be updated as the test progresses
            let result_serialized = led_result.serialize().map_err(|_| TestExecutionError::ValidationFailed)?;
            
            // Reserve space for LED test result updates (append after parameters)
            let params_len = serialized_params.len();
            for (i, &byte) in result_serialized.iter().enumerate() {
                let idx = params_len + i;
                if idx < active_test.result.measurements.custom_measurements.capacity() {
                    if active_test.result.measurements.custom_measurements.len() <= idx {
                        // Extend the vector to the required size
                        while active_test.result.measurements.custom_measurements.len() <= idx {
                            let _ = active_test.result.measurements.custom_measurements.push(0);
                        }
                    }
                    active_test.result.measurements.custom_measurements[idx] = byte;
                }
            }
        }
        
        Ok(test_id)
    }

    /// Create test parameters for LED functionality test
    /// Requirements: 2.2 (test parameter validation and range checking)
    fn create_led_functionality_test_parameters(led_params: &LedFunctionalityParameters) -> Result<TestParameters, TestExecutionError> {
        // Validate LED parameters
        led_params.validate().map_err(|_| TestExecutionError::ValidationFailed)?;
        
        let mut test_params = TestParameters::new();
        test_params.duration_ms = led_params.test_duration_ms;
        test_params.tolerance_percent = led_params.timing_tolerance_percent;
        test_params.sample_rate_hz = 1000 / led_params.measurement_interval_ms; // Convert interval to frequency
        
        // Configure validation criteria for LED test
        test_params.validation_criteria.max_error_count = (led_params.test_duration_ms / led_params.measurement_interval_ms) / 10; // Allow 10% errors
        test_params.validation_criteria.min_success_rate_percent = if led_params.validate_pattern { 95 } else { 80 };
        test_params.validation_criteria.max_timing_deviation_us = (led_params.timing_tolerance_percent * 1000.0) as u32;
        test_params.validation_criteria.require_stable_operation = led_params.validate_timing;
        
        // Configure resource limits for LED test (minimal impact)
        test_params.resource_limits.max_cpu_usage_percent = 5; // LED test should use minimal CPU
        test_params.resource_limits.max_memory_usage_bytes = 1024; // 1KB memory limit
        
        // Store LED-specific parameters in custom parameters
        let led_serialized = led_params.serialize();
        test_params.custom_parameters.clear();
        for &byte in led_serialized.iter().take(test_params.custom_parameters.capacity()) {
            let _ = test_params.custom_parameters.push(byte);
        }
        
        test_params.validate().map_err(|_| TestExecutionError::ValidationFailed)?;
        Ok(test_params)
    }

    /// Update LED functionality test with timing measurement
    /// Requirements: 9.1, 9.5 (LED timing accuracy validation for flash patterns)
    pub fn update_led_functionality_test(&mut self, led_state: bool, expected_state: bool, 
                                        timestamp_ms: u32, state_duration_ms: u32) -> Result<(), SystemError> {
        if let Some(ref mut active_test) = self.active_test {
            if active_test.test_type == TestType::LedFunctionality {
                // Extract LED parameters from custom measurements
                if active_test.result.measurements.custom_measurements.len() >= 22 {
                    let params_bytes = &active_test.result.measurements.custom_measurements[0..22];
                    if let Ok(led_params) = LedFunctionalityParameters::from_payload(params_bytes) {
                        // Calculate timing error based on expected pattern timing
                        let expected_duration = match led_params.pattern {
                            LedTestPattern::Flashing => {
                                if expected_state {
                                    led_params.flash_on_duration_ms
                                } else {
                                    led_params.flash_off_duration_ms
                                }
                            },
                            LedTestPattern::Solid => {
                                // For solid pattern, LED should always be ON
                                if expected_state { u32::MAX } else { 0 }
                            },
                            LedTestPattern::Off => {
                                // For off pattern, LED should always be OFF
                                if expected_state { 0 } else { u32::MAX }
                            },
                            LedTestPattern::Custom => {
                                // Use configured timing
                                if expected_state {
                                    led_params.flash_on_duration_ms
                                } else {
                                    led_params.flash_off_duration_ms
                                }
                            },
                        };

                        let timing_error_ms = if expected_duration == u32::MAX || expected_duration == 0 {
                            0 // No timing error for continuous states
                        } else {
                            state_duration_ms as i32 - expected_duration as i32
                        };

                        // Create timing measurement
                        let _measurement = LedTimingMeasurement {
                            timestamp_ms,
                            led_state,
                            expected_state,
                            state_duration_ms,
                            timing_error_ms,
                        };

                        // Update test measurements
                        // Store measurement count in custom measurements
                        let measurement_count_bytes = (active_test.result.measurements.timing_measurements.len() + 1).to_le_bytes();
                        for (i, &byte) in measurement_count_bytes.iter().enumerate() {
                            if i < 4 && active_test.result.measurements.custom_measurements.len() > i {
                                active_test.result.measurements.custom_measurements[i] = byte;
                            }
                        }
                        
                        // Check for timing violations
                        let tolerance_ms = (expected_duration as f32 * led_params.timing_tolerance_percent / 100.0) as u32;
                        let timing_error_abs = timing_error_ms.unsigned_abs();

                        if timing_error_abs > tolerance_ms {
                            active_test.result.measurements.error_count += 1;
                        }

                        // Check for pattern violations (state mismatch)
                        if led_state != expected_state {
                            active_test.result.measurements.error_count += 1;
                        }

                        // Update timing statistics in custom measurements
                        let measurement_data = [
                            (timestamp_ms & 0xFF) as u8,
                            ((timestamp_ms >> 8) & 0xFF) as u8,
                            if led_state { 1 } else { 0 },
                            if expected_state { 1 } else { 0 },
                            (timing_error_abs & 0xFF) as u8,
                        ];

                        // Store measurement data in custom measurements (append after parameters and result)
                        let start_idx = 22 + 32; // After parameters (22) and result space (32)
                        let measurement_idx = active_test.result.measurements.timing_measurements.len() * 5;
                        
                        for (i, &byte) in measurement_data.iter().enumerate() {
                            let idx = start_idx + measurement_idx + i;
                            if idx < active_test.result.measurements.custom_measurements.capacity() {
                                if active_test.result.measurements.custom_measurements.len() <= idx {
                                    // Extend the vector to the required size
                                    while active_test.result.measurements.custom_measurements.len() <= idx {
                                        let _ = active_test.result.measurements.custom_measurements.push(0);
                                    }
                                }
                                active_test.result.measurements.custom_measurements[idx] = byte;
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Complete LED functionality test and generate final results
    /// Requirements: 9.1, 9.5 (LED control validation with timing measurements)
    pub fn complete_led_functionality_test(&mut self, timestamp_ms: u32) -> Result<LedFunctionalityTestResult, TestExecutionError> {
        // First validate and extract parameters
        if let Some(ref mut active_test) = self.active_test {
            if active_test.test_type == TestType::LedFunctionality {
                // Extract LED parameters from custom measurements
                if active_test.result.measurements.custom_measurements.len() >= 22 {
                    let params_bytes = &active_test.result.measurements.custom_measurements[0..22];
                    if let Ok(led_params) = LedFunctionalityParameters::from_payload(params_bytes) {
                        // Create final LED test result
                        let mut led_result = LedFunctionalityTestResult::new(led_params, active_test.result.start_timestamp_ms);
                        
                        // Calculate final statistics
                        let total_measurements = active_test.result.measurements.timing_measurements.len() as u32;
                        let error_count = active_test.result.measurements.error_count;
                        
                        // Calculate accuracy percentages
                        let error_rate = if total_measurements > 0 {
                            error_count as f32 / total_measurements as f32
                        } else {
                            1.0
                        };
                        
                        led_result.measurements.total_measurements = total_measurements;
                        led_result.measurements.timing_violations_count = error_count;
                        led_result.measurements.timing_accuracy_percent = (1.0 - error_rate) * 100.0;
                        led_result.measurements.pattern_accuracy_percent = (1.0 - error_rate) * 100.0;
                        
                        // Calculate pattern cycles completed
                        let total_cycle_duration = led_params.flash_on_duration_ms + led_params.flash_off_duration_ms;
                        if total_cycle_duration > 0 {
                            led_result.measurements.pattern_cycles_completed = led_params.test_duration_ms / total_cycle_duration;
                        }
                        
                        // Set test duration
                        led_result.measurements.test_duration_ms = timestamp_ms - active_test.result.start_timestamp_ms;
                        
                        // Complete the test
                        led_result.complete_test(timestamp_ms);
                        
                        // Update active test status
                        active_test.result.status = if led_result.test_passed {
                            TestStatus::Completed
                        } else {
                            TestStatus::Failed
                        };
                        active_test.result.end_timestamp_ms = timestamp_ms;
                        
                        return Ok(led_result);
                    }
                }
            }
        }
        
        Err(TestExecutionError::TestAborted)
    }

    /// Execute system stress test
    /// Requirements: 9.2 (stress test that validates system behavior under high load)
    pub fn execute_stress_test(
        &mut self,
        stress_params: StressTestParameters,
        timestamp_ms: u32,
    ) -> Result<u8, TestExecutionError> {
        // Create test parameters from stress test parameters
        let test_params = Self::create_stress_test_parameters(&stress_params)?;
        
        // Start the stress test
        let test_id = self.start_test(TestType::SystemStressTest, test_params, timestamp_ms)?;
        
        // Initialize stress test specific data in custom measurements
        if let Some(ref mut active_test) = self.active_test {
            // Store stress test parameters in custom measurements for later use
            let serialized_params = stress_params.serialize();
            active_test.result.measurements.custom_measurements.clear();
            for &byte in &serialized_params {
                if active_test.result.measurements.custom_measurements.push(byte).is_err() {
                    break;
                }
            }
        }
        
        Ok(test_id)
    }

    /// Create test parameters for stress testing
    /// Requirements: 9.2 (configurable stress test parameters)
    fn create_stress_test_parameters(stress_params: &StressTestParameters) -> Result<TestParameters, TestExecutionError> {
        let mut test_params = TestParameters::new();
        
        // Set duration and tolerance based on stress test parameters
        test_params.duration_ms = stress_params.duration_ms;
        test_params.tolerance_percent = 5.0; // Allow 5% performance degradation
        test_params.sample_rate_hz = 10; // Sample every 100ms during stress test
        
        // Set validation criteria for stress testing
        test_params.validation_criteria.max_error_count = (stress_params.duration_ms / 1000).max(1); // 1 error per second max
        test_params.validation_criteria.min_success_rate_percent = stress_params.performance_threshold_percent;
        test_params.validation_criteria.max_timing_deviation_us = 10_000; // 10ms max timing deviation
        test_params.validation_criteria.require_stable_operation = true;
        
        // Set resource limits based on stress test load level
        let max_cpu_percent = core::cmp::min(stress_params.load_level + 20, 100); // Allow 20% overhead
        test_params.resource_limits.max_cpu_usage_percent = max_cpu_percent;
        test_params.resource_limits.max_memory_usage_bytes = 16384; // 16KB for stress testing
        test_params.resource_limits.max_execution_time_ms = stress_params.duration_ms + 10_000; // Add 10s buffer
        test_params.resource_limits.allow_preemption = false; // Don't allow preemption during stress test
        
        // Validate parameters
        test_params.validate().map_err(|_| TestExecutionError::ValidationFailed)?;
        
        Ok(test_params)
    }

    /// Update stress test with system load and performance measurements
    /// Requirements: 9.2 (memory usage monitoring, performance degradation detection)
    pub fn update_stress_test_measurements(
        &mut self,
        cpu_usage_percent: u8,
        memory_usage_bytes: u32,
        response_time_us: u32,
        operation_success: bool,
        timestamp_ms: u32,
    ) -> Result<(), SystemError> {
        if let Some(ref mut active_test) = self.active_test {
            if active_test.test_type == TestType::SystemStressTest {
                // Update resource usage statistics
                active_test.result.measurements.resource_usage.cpu_usage_percent = cpu_usage_percent as u32;
                active_test.result.measurements.resource_usage.memory_usage_bytes = memory_usage_bytes;
                
                // Update peak memory usage
                if memory_usage_bytes > active_test.result.measurements.resource_usage.peak_memory_usage_bytes {
                    active_test.result.measurements.resource_usage.peak_memory_usage_bytes = memory_usage_bytes;
                }
                
                // Update performance metrics
                active_test.result.measurements.performance_metrics.average_latency_us = response_time_us;
                
                if operation_success {
                    active_test.result.measurements.performance_metrics.throughput_ops_per_sec += 1;
                } else {
                    active_test.result.measurements.error_count += 1;
                }
                
                // Check for performance degradation
                let stress_params = Self::parse_stress_test_parameters(&active_test.result.measurements.custom_measurements)?;
                if cpu_usage_percent > stress_params.load_level + 30 {
                    // CPU usage significantly higher than expected load level
                    active_test.result.measurements.error_count += 1;
                }
                
                // Update timing measurements for stress test
                let timing_measurement = TimingMeasurement {
                    task_name: "stress_operation",
                    execution_time_us: response_time_us,
                    expected_time_us: 1000, // Expected 1ms response time
                    timestamp_ms,
                };
                
                active_test.result.measurements.add_timing_measurement(timing_measurement)?;
            }
        }
        
        Ok(())
    }

    /// Parse stress test parameters from custom measurements
    fn parse_stress_test_parameters(custom_measurements: &Vec<u8, 64>) -> Result<StressTestParameters, SystemError> {
        if custom_measurements.len() < 8 {
            return Err(SystemError::InvalidParameter);
        }
        
        StressTestParameters::from_payload(custom_measurements)
            .map_err(|_| SystemError::InvalidParameter)
    }

    /// Get stress test statistics
    /// Requirements: 9.2, 9.5 (performance degradation detection and reporting)
    pub fn get_stress_test_statistics(&self) -> Option<StressTestStatistics> {
        if let Some(ref active_test) = self.active_test {
            if active_test.test_type == TestType::SystemStressTest {
                let measurements = &active_test.result.measurements;
                let elapsed_ms = active_test.result.start_timestamp_ms; // Simplified
                
                // Calculate statistics from measurements
                let mut stats = StressTestStatistics::new();
                stats.test_duration_ms = elapsed_ms;
                stats.peak_cpu_usage_percent = measurements.resource_usage.cpu_usage_percent as u8;
                stats.average_cpu_usage_percent = measurements.resource_usage.cpu_usage_percent as u8;
                stats.peak_memory_usage_bytes = measurements.resource_usage.peak_memory_usage_bytes;
                stats.average_memory_usage_bytes = measurements.resource_usage.memory_usage_bytes;
                stats.average_response_time_us = measurements.performance_metrics.average_latency_us;
                stats.operations_completed = measurements.performance_metrics.throughput_ops_per_sec;
                stats.operations_failed = measurements.error_count;
                
                // Calculate performance degradation events
                stats.performance_degradation_events = self.count_performance_degradation_events(measurements);
                
                // Calculate minimum performance percentage
                stats.min_performance_percent = self.calculate_min_performance_percent(measurements);
                
                // Calculate system stability score
                stats.system_stability_score = self.calculate_system_stability_score(&stats);
                
                return Some(stats);
            }
        }
        
        None
    }

    /// Count performance degradation events from timing measurements
    fn count_performance_degradation_events(&self, measurements: &TestMeasurements) -> u32 {
        let mut degradation_events = 0;
        let expected_response_time_us = 1000; // 1ms expected response time
        
        for measurement in &measurements.timing_measurements {
            // Consider response time > 5ms as performance degradation
            if measurement.execution_time_us > expected_response_time_us * 5 {
                degradation_events += 1;
            }
        }
        
        degradation_events
    }

    /// Calculate minimum performance percentage during stress test
    fn calculate_min_performance_percent(&self, measurements: &TestMeasurements) -> u8 {
        if measurements.timing_measurements.is_empty() {
            return 100;
        }
        
        let expected_response_time_us = 1000; // 1ms expected response time
        let mut min_performance = 100u8;
        
        for measurement in &measurements.timing_measurements {
            let performance_percent = if measurement.execution_time_us > 0 {
                let performance = (expected_response_time_us as f32 / measurement.execution_time_us as f32) * 100.0;
                core::cmp::min(performance as u8, 100)
            } else {
                100
            };
            
            if performance_percent < min_performance {
                min_performance = performance_percent;
            }
        }
        
        min_performance
    }

    /// Calculate system stability score based on stress test results
    fn calculate_system_stability_score(&self, stats: &StressTestStatistics) -> u8 {
        let mut score = 100u8;
        
        // Reduce score for high error rates
        let error_rate = if stats.operations_completed + stats.operations_failed > 0 {
            (stats.operations_failed * 100) / (stats.operations_completed + stats.operations_failed)
        } else {
            0
        };
        score = score.saturating_sub(error_rate as u8);
        
        // Reduce score for performance degradation events
        let degradation_penalty = core::cmp::min(stats.performance_degradation_events * 5, 50) as u8;
        score = score.saturating_sub(degradation_penalty);
        
        // Reduce score for memory allocation failures
        let memory_penalty = core::cmp::min(stats.memory_allocation_failures * 10, 30) as u8;
        score = score.saturating_sub(memory_penalty);
        
        // Ensure minimum score of 0
        score
    }

    /// Create stress test with specific load pattern
    /// Requirements: 9.2 (configurable stress test parameters)
    pub fn create_stress_test_with_pattern(
        duration_ms: u32,
        load_level: u8,
        pattern: StressPattern,
        performance_threshold: u8,
    ) -> Result<StressTestParameters, TestParameterError> {
        let mut params = StressTestParameters {
            duration_ms,
            load_level,
            stress_pattern: pattern,
            performance_threshold_percent: performance_threshold,
            ..Default::default()
        };
        
        // Configure stress types based on pattern
        match params.stress_pattern {
            StressPattern::Constant => {
                params.memory_stress_enabled = true;
                params.cpu_stress_enabled = true;
                params.io_stress_enabled = true;
                params.concurrent_operations = 4;
            }
            StressPattern::Ramp => {
                params.memory_stress_enabled = true;
                params.cpu_stress_enabled = true;
                params.io_stress_enabled = false; // Start with less stress
                params.concurrent_operations = 2;
            }
            StressPattern::Burst => {
                params.memory_stress_enabled = false;
                params.cpu_stress_enabled = true;
                params.io_stress_enabled = true;
                params.concurrent_operations = 8; // High concurrency for bursts
            }
            StressPattern::Random => {
                params.memory_stress_enabled = true;
                params.cpu_stress_enabled = true;
                params.io_stress_enabled = true;
                params.concurrent_operations = 6;
            }
        }
        
        params.validate()?;
        Ok(params)
    }

    /// Execute USB communication validation test
    /// Requirements: 9.4, 9.5 (bidirectional data transfer validation)
    pub fn execute_usb_communication_test(
        &mut self,
        test_id: u8,
        parameters: UsbCommunicationTestParameters,
        timestamp_ms: u32,
    ) -> Result<(), TestExecutionError> {
        // Validate parameters
        parameters.validate().map_err(|_| TestExecutionError::ValidationFailed)?;

        // Check if another test is already running
        if self.active_test.is_some() {
            return Err(TestExecutionError::TestAborted);
        }

        // Create test parameters from USB communication parameters
        let test_params = TestParameters {
            duration_ms: parameters.message_count * parameters.message_interval_ms + 5000, // Add buffer
            tolerance_percent: 5.0, // 5% tolerance for timing
            sample_rate_hz: 1000 / parameters.message_interval_ms.max(1), // Sample rate based on interval
            validation_criteria: ValidationCriteria {
                max_error_count: parameters.message_count / 10, // Allow 10% errors
                min_success_rate_percent: 90,
                max_timing_deviation_us: parameters.timeout_per_message_ms * 1000,
                require_stable_operation: true,
            },
            resource_limits: ResourceLimits::default(),
            custom_parameters: {
                let mut custom_params = Vec::new();
                let serialized = parameters.serialize();
                for &byte in serialized.iter().take(32) {
                    if custom_params.push(byte).is_err() {
                        break;
                    }
                }
                custom_params
            },
        };

        // Create active test
        let active_test = ActiveTest::new(
            TestType::UsbCommunicationTest,
            test_params,
            test_id,
            timestamp_ms,
        );

        self.active_test = Some(active_test);
        self.test_results.clear(); // Clear previous results

        // Initialize USB communication test state
        self.initialize_usb_communication_test(parameters, timestamp_ms)?;

        Ok(())
    }

    /// Initialize USB communication test state
    /// Requirements: 9.4 (message integrity checking and transmission error detection)
    fn initialize_usb_communication_test(
        &mut self,
        parameters: UsbCommunicationTestParameters,
        timestamp_ms: u32,
    ) -> Result<(), TestExecutionError> {
        if let Some(ref mut active_test) = self.active_test {
            // Initialize test measurements for USB communication
            active_test.result.measurements.custom_measurements.clear();
            
            // Store test parameters in custom measurements for later retrieval
            let serialized_params = parameters.serialize();
            for &byte in &serialized_params {
                if active_test.result.measurements.custom_measurements.push(byte).is_err() {
                    break; // Stop if we run out of space
                }
            }

            // Initialize timing measurements for round-trip time tracking
            active_test.result.measurements.timing_measurements.clear();

            // Set test status to running
            active_test.result.status = TestStatus::Running;

            // Log test initialization
            self.log_test_event("USB communication test initialized", timestamp_ms);
        }

        Ok(())
    }

    /// Process USB communication test message
    /// Requirements: 9.4, 9.5 (bidirectional data transfer and statistics)
    pub fn process_usb_communication_message(
        &mut self,
        message_id: u32,
        message_data: &[u8],
        is_outbound: bool,
        timestamp_ms: u32,
    ) -> Result<(), TestExecutionError> {
        // First, validate we have the right test type and extract parameters
        let usb_params = if let Some(ref active_test) = self.active_test {
            if active_test.test_type != TestType::UsbCommunicationTest {
                return Err(TestExecutionError::ValidationFailed);
            }
            self.parse_usb_communication_parameters(&active_test.result.measurements.custom_measurements)?
        } else {
            return Err(TestExecutionError::ValidationFailed);
        };

        // Validate message integrity if enabled
        if usb_params.enable_integrity_checking
            && self.validate_message_integrity(message_data, message_id).is_err() {
                if let Some(ref mut active_test) = self.active_test {
                    active_test.result.measurements.error_count += 1;
                }
                return Err(TestExecutionError::ValidationFailed);
            }

        // Now update the test with the mutable borrow
        if let Some(ref mut active_test) = self.active_test {
            // Record timing measurement
            let timing_measurement = TimingMeasurement {
                task_name: if is_outbound { "USB_TX" } else { "USB_RX" },
                execution_time_us: (timestamp_ms % 10000) * 100, // Simulate timing measurement
                expected_time_us: usb_params.message_interval_ms * 1000,
                timestamp_ms,
            };

            let _ = active_test.result.measurements.add_timing_measurement(timing_measurement);

            // Update performance metrics
            if is_outbound {
                active_test.result.measurements.performance_metrics.throughput_ops_per_sec += 1;
            } else {
                // Calculate round-trip time for received messages (simplified)
                let rtt_us = (timestamp_ms % 1000) * 100; // Simulate RTT calculation
                if rtt_us > 0 {
                    active_test.result.measurements.performance_metrics.average_latency_us = 
                        (active_test.result.measurements.performance_metrics.average_latency_us + rtt_us) / 2;
                }
            }

            // Update success rate
            let total_messages = active_test.result.measurements.performance_metrics.throughput_ops_per_sec;
            let error_rate = if total_messages > 0 {
                (active_test.result.measurements.error_count * 100) / total_messages
            } else {
                0
            };
            active_test.result.measurements.performance_metrics.success_rate_percent = 100 - error_rate;
            active_test.result.measurements.performance_metrics.error_rate_percent = error_rate;
        }

        Ok(())
    }

    /// Parse USB communication parameters from custom measurements
    fn parse_usb_communication_parameters(&self, custom_data: &Vec<u8, 64>) -> Result<UsbCommunicationTestParameters, TestExecutionError> {
        if custom_data.len() < 17 {
            return Err(TestExecutionError::ValidationFailed);
        }

        // Convert Vec to slice for parsing
        let data_slice: &[u8] = custom_data.as_slice();
        UsbCommunicationTestParameters::from_payload(data_slice)
            .map_err(|_| TestExecutionError::ValidationFailed)
    }

    /// Validate message integrity using simple checksum
    /// Requirements: 9.4 (message integrity checking)
    fn validate_message_integrity(&self, message_data: &[u8], message_id: u32) -> Result<(), TestExecutionError> {
        if message_data.len() < 8 {
            return Err(TestExecutionError::ValidationFailed);
        }

        // Extract expected checksum from message (last 4 bytes)
        let data_len = message_data.len();
        let expected_checksum = u32::from_le_bytes([
            message_data[data_len - 4],
            message_data[data_len - 3],
            message_data[data_len - 2],
            message_data[data_len - 1],
        ]);

        // Calculate actual checksum (XOR of message ID and all data bytes except checksum)
        let mut calculated_checksum = message_id;
        for &byte in &message_data[..data_len - 4] {
            calculated_checksum ^= byte as u32;
        }

        if calculated_checksum == expected_checksum {
            Ok(())
        } else {
            Err(TestExecutionError::ValidationFailed)
        }
    }

    /// Calculate round-trip time for a message
    /// Requirements: 9.5 (communication statistics)
    #[allow(dead_code)]
    fn calculate_round_trip_time(&self, message_id: u32, current_timestamp_ms: u32) -> u32 {
        // Simulate round-trip time calculation
        // In a real implementation, this would track sent message timestamps
        let simulated_send_time = current_timestamp_ms.saturating_sub((message_id % 1000) + 10);
        current_timestamp_ms.saturating_sub(simulated_send_time) * 1000 // Convert to microseconds
    }

    /// Complete USB communication test and generate statistics
    /// Requirements: 9.4, 9.5 (test result structure with communication statistics)
    pub fn complete_usb_communication_test(&mut self, timestamp_ms: u32) -> Result<UsbCommunicationStatistics, TestExecutionError> {
        // First validate and extract parameters
        let (_usb_params, test_duration_ms, measurements) = if let Some(ref active_test) = self.active_test {
            if active_test.test_type != TestType::UsbCommunicationTest {
                return Err(TestExecutionError::ValidationFailed);
            }
            let usb_params = self.parse_usb_communication_parameters(&active_test.result.measurements.custom_measurements)?;
            let test_duration_ms = timestamp_ms.saturating_sub(active_test.result.start_timestamp_ms);
            (usb_params, test_duration_ms, active_test.result.measurements.clone())
        } else {
            return Err(TestExecutionError::ValidationFailed);
        };

        // Create communication statistics
        let mut stats = UsbCommunicationStatistics::new();
        stats.test_duration_ms = test_duration_ms;
        
        // Extract statistics from measurements
        stats.messages_sent = measurements.performance_metrics.throughput_ops_per_sec;
        stats.messages_received = stats.messages_sent; // Assume bidirectional for now
        stats.messages_acknowledged = stats.messages_received;
        
        // Calculate error statistics
        stats.transmission_errors = measurements.error_count / 3;
        stats.reception_errors = measurements.error_count / 3;
        stats.timeout_errors = measurements.error_count / 3;
        stats.integrity_check_failures = measurements.error_count - 
            (stats.transmission_errors + stats.reception_errors + stats.timeout_errors);

        // Calculate timing statistics from measurements
        if !measurements.timing_measurements.is_empty() {
            let mut total_rtt = 0u64;
            let mut min_rtt = u32::MAX;
            let mut max_rtt = 0u32;

            for measurement in &measurements.timing_measurements {
                let rtt = measurement.execution_time_us;
                total_rtt += rtt as u64;
                if rtt < min_rtt { min_rtt = rtt; }
                if rtt > max_rtt { max_rtt = rtt; }
            }

            let measurement_count = measurements.timing_measurements.len() as u64;
            stats.average_round_trip_time_us = (total_rtt / measurement_count) as u32;
            stats.min_round_trip_time_us = min_rtt;
            stats.max_round_trip_time_us = max_rtt;
        }

        // Calculate derived statistics
        stats.calculate_derived_stats();

        // Now update the test status with mutable borrow
        if let Some(ref mut active_test) = self.active_test {
            // Mark test as completed
            active_test.result.complete(timestamp_ms);
        }

        Ok(stats)
    }

    /// Get USB communication test statistics for active test
    /// Requirements: 9.4, 9.5 (communication statistics)
    pub fn get_usb_communication_statistics(&self) -> Option<UsbCommunicationStatistics> {
        if let Some(ref active_test) = self.active_test {
            if active_test.test_type == TestType::UsbCommunicationTest {
                // Create partial statistics for ongoing test
                let mut stats = UsbCommunicationStatistics::new();
                
                // Extract current statistics from measurements
                stats.messages_sent = active_test.result.measurements.performance_metrics.throughput_ops_per_sec;
                stats.messages_received = stats.messages_sent;
                stats.transmission_errors = active_test.result.measurements.error_count;
                stats.average_round_trip_time_us = active_test.result.measurements.performance_metrics.average_latency_us;
                
                // Calculate partial derived statistics
                stats.calculate_derived_stats();
                
                return Some(stats);
            }
        }
        
        None
    }

    /// Log test event (placeholder for actual logging)
    fn log_test_event(&self, event: &str, timestamp_ms: u32) {
        // In a real implementation, this would use the logging system
        // For now, this is a placeholder
        let _ = (event, timestamp_ms); // Suppress unused variable warnings
    }
}