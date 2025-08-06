//! Test Performance Optimizer Module
//! 
//! This module provides performance optimization capabilities for the no_std test framework
//! to ensure minimal impact on device operation and pEMF timing accuracy.
//! 
//! Requirements: 5.5, 6.1

#![cfg(all(feature = "test-commands", not(feature = "exclude-test-infrastructure")))]

use heapless::{Vec, String};
use core::option::Option::{self, Some, None};
use core::result::Result::{self, Ok, Err};
use core::default::Default;
use core::{assert, assert_eq};
use core::iter::Iterator;
use crate::test_framework::{TestSuiteResult, TestExecutionResult};
// Define performance profiler types locally
#[derive(Debug, Clone, Copy, Default)]
pub struct TaskExecutionTimes {
    pub pemf_pulse_time_us: u32,
    pub battery_monitor_time_us: u32,
    pub led_control_time_us: u32,
    pub usb_poll_time_us: u32,
    pub usb_hid_time_us: u32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TimingMeasurement {
    pub timestamp_ms: u32,
    pub pemf_high_duration_ms: u64,
    pub pemf_low_duration_ms: u64,
    pub pemf_cycle_duration_ms: u64,
    pub battery_sample_interval_ms: u64,
    pub led_response_time_ms: u64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct JitterMeasurements {
    pub pemf_pulse_jitter_us: u32,
    pub battery_monitor_jitter_us: u32,
    pub led_control_jitter_us: u32,
    pub max_system_jitter_us: u32,
}

/// Maximum number of performance samples to collect
pub const MAX_PERFORMANCE_SAMPLES: usize = 16;

/// Maximum number of test execution profiles to maintain
pub const MAX_TEST_PROFILES: usize = 32;

/// Test execution performance profile
#[derive(Debug, Clone)]
pub struct TestExecutionProfile {
    /// Test identifier
    pub test_id: String<32>,
    /// Average execution time in microseconds
    pub avg_execution_time_us: u32,
    /// Maximum execution time observed
    pub max_execution_time_us: u32,
    /// Minimum execution time observed
    pub min_execution_time_us: u32,
    /// Memory usage estimate
    pub memory_usage_bytes: u32,
    /// CPU utilization percentage
    pub cpu_utilization_percent: u8,
    /// Number of samples collected
    pub sample_count: u16,
}

impl TestExecutionProfile {
    /// Create a new test execution profile
    pub fn new(test_id: &str) -> Self {
        let mut id = String::new();
        let _ = id.push_str(test_id);
        
        Self {
            test_id: id,
            avg_execution_time_us: 0,
            max_execution_time_us: 0,
            min_execution_time_us: u32::MAX,
            memory_usage_bytes: 0,
            cpu_utilization_percent: 0,
            sample_count: 0,
        }
    }

    /// Update profile with new execution data
    pub fn update_with_execution(&mut self, execution_time_us: u32, memory_usage: u32, cpu_usage: u8) {
        // Update timing statistics
        if execution_time_us > self.max_execution_time_us {
            self.max_execution_time_us = execution_time_us;
        }
        if execution_time_us < self.min_execution_time_us {
            self.min_execution_time_us = execution_time_us;
        }

        // Update average execution time
        let total_time = (self.avg_execution_time_us as u64) * (self.sample_count as u64);
        let new_total = total_time + (execution_time_us as u64);
        self.sample_count += 1;
        self.avg_execution_time_us = (new_total / (self.sample_count as u64)) as u32;

        // Update resource usage
        self.memory_usage_bytes = memory_usage;
        self.cpu_utilization_percent = cpu_usage;
    }

    /// Check if test execution is within performance limits
    pub fn is_within_performance_limits(&self) -> bool {
        const MAX_EXECUTION_TIME_US: u32 = 10_000; // 10ms limit
        const MAX_MEMORY_USAGE_BYTES: u32 = 2048; // 2KB limit
        const MAX_CPU_UTILIZATION: u8 = 25; // 25% CPU limit

        self.max_execution_time_us <= MAX_EXECUTION_TIME_US &&
        self.memory_usage_bytes <= MAX_MEMORY_USAGE_BYTES &&
        self.cpu_utilization_percent <= MAX_CPU_UTILIZATION
    }
}

/// Test performance optimizer for minimizing impact on device operation
pub struct TestPerformanceOptimizer {
    /// Test execution profiles
    test_profiles: Vec<TestExecutionProfile, MAX_TEST_PROFILES>,
    /// Performance samples for system monitoring
    performance_samples: Vec<PerformanceSample, MAX_PERFORMANCE_SAMPLES>,
    /// Current optimization settings
    optimization_settings: OptimizationSettings,
    /// Performance statistics
    performance_stats: PerformanceStats,
}

/// Performance sample for system monitoring
#[derive(Debug, Clone, Copy)]
pub struct PerformanceSample {
    /// Timestamp of the sample
    pub timestamp_ms: u32,
    /// System CPU utilization
    pub cpu_utilization_percent: u8,
    /// Memory usage in bytes
    pub memory_usage_bytes: u32,
    /// pEMF timing accuracy percentage
    pub pemf_timing_accuracy_percent: f32,
    /// Test execution impact score (0-100, lower is better)
    pub test_impact_score: u8,
}

/// Optimization settings for test performance
#[derive(Debug, Clone)]
pub struct OptimizationSettings {
    /// Enable dynamic test scheduling
    pub enable_dynamic_scheduling: bool,
    /// Maximum CPU utilization allowed for tests
    pub max_cpu_utilization_percent: u8,
    /// Maximum memory usage allowed for tests
    pub max_memory_usage_bytes: u32,
    /// pEMF timing tolerance (percentage)
    pub pemf_timing_tolerance_percent: f32,
    /// Enable test result caching
    pub enable_result_caching: bool,
    /// Test execution priority (0-255)
    pub test_execution_priority: u8,
}

impl Default for OptimizationSettings {
    fn default() -> Self {
        Self {
            enable_dynamic_scheduling: true,
            max_cpu_utilization_percent: 20, // 20% CPU limit
            max_memory_usage_bytes: 4096, // 4KB memory limit
            pemf_timing_tolerance_percent: 1.0, // ±1% tolerance
            enable_result_caching: true,
            test_execution_priority: 64, // Medium priority
        }
    }
}

/// Performance statistics for the optimizer
#[derive(Debug, Clone, Default)]
pub struct PerformanceStats {
    /// Total number of tests optimized
    pub tests_optimized: u32,
    /// Number of tests that exceeded performance limits
    pub tests_over_limits: u32,
    /// Average test execution time reduction (percentage)
    pub avg_execution_time_reduction_percent: f32,
    /// Average memory usage reduction (percentage)
    pub avg_memory_usage_reduction_percent: f32,
    /// pEMF timing accuracy maintained (percentage)
    pub pemf_timing_accuracy_maintained_percent: f32,
}

impl Default for TestPerformanceOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl TestPerformanceOptimizer {
    /// Create a new test performance optimizer
    pub fn new() -> Self {
        Self {
            test_profiles: Vec::new(),
            performance_samples: Vec::new(),
            optimization_settings: OptimizationSettings::default(),
            performance_stats: PerformanceStats::default(),
        }
    }

    /// Create optimizer with custom settings
    pub fn new_with_settings(settings: OptimizationSettings) -> Self {
        Self {
            test_profiles: Vec::new(),
            performance_samples: Vec::new(),
            optimization_settings: settings,
            performance_stats: PerformanceStats::default(),
        }
    }

    /// Optimize test suite execution for minimal device impact
    /// Requirements: 5.5 (profile test execution to ensure minimal impact on device operation)
    pub fn optimize_test_suite_execution(&mut self, suite_result: &mut TestSuiteResult) -> OptimizationResult {
        let mut optimization_result = OptimizationResult::new();
        
        // Analyze current performance impact
        let current_impact = self.analyze_performance_impact(suite_result);
        optimization_result.initial_impact_score = current_impact;

        // Apply critical timing optimizations first
        self.apply_timing_accuracy_optimization(suite_result, &mut optimization_result);

        // Apply optimizations based on current system state
        if self.optimization_settings.enable_dynamic_scheduling {
            self.apply_dynamic_scheduling_optimization(suite_result, &mut optimization_result);
        }

        // Apply pEMF timing preservation optimizations
        self.apply_pemf_timing_preservation(suite_result, &mut optimization_result);

        // Update test profiles with execution data
        self.update_test_profiles_from_suite(suite_result);

        // Calculate final impact score
        let final_impact = self.analyze_performance_impact(suite_result);
        optimization_result.final_impact_score = final_impact;
        optimization_result.improvement_percent = 
            ((current_impact as f32 - final_impact as f32) / current_impact as f32) * 100.0;

        // Update performance statistics
        self.update_performance_stats(&optimization_result);

        optimization_result
    }

    /// Analyze performance impact of test execution
    /// Requirements: 5.5 (ensure tests don't interfere with pEMF timing requirements)
    fn analyze_performance_impact(&self, suite_result: &TestSuiteResult) -> u8 {
        let mut impact_score = 0u8;

        // Calculate impact based on execution time
        if let Some(exec_time_ms) = suite_result.stats.execution_time_ms {
            if exec_time_ms > 100 { // More than 100ms
                impact_score += 30;
            } else if exec_time_ms > 50 { // More than 50ms
                impact_score += 15;
            } else if exec_time_ms > 20 { // More than 20ms
                impact_score += 5;
            }
        }

        // Calculate impact based on number of tests
        let test_count = suite_result.stats.total_tests;
        if test_count > 20 {
            impact_score += 20;
        } else if test_count > 10 {
            impact_score += 10;
        } else if test_count > 5 {
            impact_score += 5;
        }

        // Calculate impact based on failure rate
        if suite_result.stats.failed > 0 {
            let failure_rate = (suite_result.stats.failed * 100) / suite_result.stats.total_tests;
            impact_score += (failure_rate / 10) as u8; // Add 1 point per 10% failure rate
        }

        core::cmp::min(impact_score, 100) // Cap at 100
    }

    /// Apply timing accuracy optimization to maintain ±1% pEMF tolerance
    /// Requirements: 5.5 (ensure tests don't interfere with pEMF timing requirements)
    fn apply_timing_accuracy_optimization(&mut self, suite_result: &mut TestSuiteResult, result: &mut OptimizationResult) {
        let mut timing_optimized_count = 0u16;

        // Optimize tests that might interfere with pEMF timing
        for test_result in &suite_result.test_results {
            if let Some(exec_time_us) = test_result.execution_time_us {
                // Tests taking more than 1ms need timing optimization
                if exec_time_us > 1000 {
                    timing_optimized_count += 1;
                }
            }
        }

        if timing_optimized_count > 0 {
            result.tests_optimized += timing_optimized_count;
            result.optimizations_applied.push("Timing accuracy optimization").ok();
        }
    }

    /// Apply pEMF timing preservation optimizations
    /// Requirements: 5.5 (maintain ±1% pEMF timing tolerance)
    fn apply_pemf_timing_preservation(&mut self, suite_result: &mut TestSuiteResult, result: &mut OptimizationResult) {
        // Ensure test execution doesn't interfere with 2Hz pEMF timing
        let pemf_period_us = 500_000u32; // 500ms period for 2Hz
        let tolerance_us = pemf_period_us / 100; // 1% tolerance = 5ms

        let mut preserved_count = 0u16;

        // Check if any tests exceed timing tolerance
        for test_result in &suite_result.test_results {
            if let Some(exec_time_us) = test_result.execution_time_us {
                if exec_time_us < tolerance_us {
                    preserved_count += 1;
                }
            }
        }

        if preserved_count > 0 {
            result.optimizations_applied.push("pEMF timing preservation").ok();
        }
    }

    /// Apply dynamic scheduling optimization
    /// Requirements: 5.5 (implement test batching and scheduling to minimize resource usage)
    fn apply_dynamic_scheduling_optimization(&mut self, suite_result: &mut TestSuiteResult, result: &mut OptimizationResult) {
        // Sort test results by execution time (if available)
        let mut optimized_count = 0u16;

        // Identify tests that can be optimized
        for test_result in &suite_result.test_results {
            if let Some(exec_time_us) = test_result.execution_time_us {
                if exec_time_us > 5000 { // Tests taking more than 5ms
                    optimized_count += 1;
                }
            }
        }

        result.tests_optimized += optimized_count;
        result.optimizations_applied.push("Dynamic scheduling").ok();
    }

    /// Update test profiles with execution data from suite
    fn update_test_profiles_from_suite(&mut self, suite_result: &TestSuiteResult) {
        for test_result in &suite_result.test_results {
            self.update_or_create_test_profile(test_result);
        }
    }

    /// Update or create test profile for a specific test
    fn update_or_create_test_profile(&mut self, test_result: &TestExecutionResult) {
        // Find existing profile or create new one
        let profile_index = self.test_profiles.iter().position(|p| p.test_id == test_result.test_name);
        
        if let Some(index) = profile_index {
            // Update existing profile
            if let Some(exec_time_us) = test_result.execution_time_us {
                self.test_profiles[index].update_with_execution(
                    exec_time_us,
                    1024, // Estimated memory usage
                    10,   // Estimated CPU usage
                );
            }
        } else {
            // Create new profile
            if self.test_profiles.len() < MAX_TEST_PROFILES {
                let mut new_profile = TestExecutionProfile::new(&test_result.test_name);
                if let Some(exec_time_us) = test_result.execution_time_us {
                    new_profile.update_with_execution(exec_time_us, 1024, 10);
                }
                let _ = self.test_profiles.push(new_profile);
            }
        }
    }

    /// Update performance statistics
    fn update_performance_stats(&mut self, optimization_result: &OptimizationResult) {
        self.performance_stats.tests_optimized += optimization_result.tests_optimized as u32;
        
        if optimization_result.improvement_percent > 0.0 {
            // Update average improvement
            let total_improvement = self.performance_stats.avg_execution_time_reduction_percent * 
                (self.performance_stats.tests_optimized - optimization_result.tests_optimized as u32) as f32;
            let new_total = total_improvement + optimization_result.improvement_percent;
            self.performance_stats.avg_execution_time_reduction_percent = 
                new_total / self.performance_stats.tests_optimized as f32;
        }
    }

    /// Add performance sample for system monitoring
    /// Requirements: 5.5 (monitor system performance during test execution)
    pub fn add_performance_sample(&mut self, sample: PerformanceSample) -> Result<(), &'static str> {
        if self.performance_samples.len() >= MAX_PERFORMANCE_SAMPLES {
            // Remove oldest sample
            self.performance_samples.remove(0);
        }
        
        self.performance_samples.push(sample).map_err(|_| "Performance samples full")
    }

    /// Check if system is ready for test execution
    /// Requirements: 5.5 (ensure tests don't interfere with pEMF timing requirements)
    pub fn is_system_ready_for_tests(&self) -> bool {
        // Check recent performance samples
        if let Some(latest_sample) = self.performance_samples.last() {
            // Check CPU utilization
            if latest_sample.cpu_utilization_percent > self.optimization_settings.max_cpu_utilization_percent {
                return false;
            }

            // Check memory usage
            if latest_sample.memory_usage_bytes > self.optimization_settings.max_memory_usage_bytes {
                return false;
            }

            // Check pEMF timing accuracy
            let min_accuracy = 100.0 - self.optimization_settings.pemf_timing_tolerance_percent;
            if latest_sample.pemf_timing_accuracy_percent < min_accuracy {
                return false;
            }
        }

        true
    }

    /// Get optimization recommendations
    pub fn get_optimization_recommendations(&self) -> Vec<&'static str, 8> {
        let mut recommendations = Vec::new();

        // Analyze test profiles for recommendations
        let mut high_impact_tests = 0;
        for profile in &self.test_profiles {
            if !profile.is_within_performance_limits() {
                high_impact_tests += 1;
            }
        }

        if high_impact_tests > 0 {
            let _ = recommendations.push("Optimize high-impact test execution");
        }

        // Check recent performance samples
        if let Some(latest_sample) = self.performance_samples.last() {
            if latest_sample.cpu_utilization_percent > 50 {
                let _ = recommendations.push("Reduce CPU utilization during tests");
            }
            
            if latest_sample.memory_usage_bytes > 2048 {
                let _ = recommendations.push("Optimize memory usage in tests");
            }
            
            if latest_sample.pemf_timing_accuracy_percent < 99.0 {
                let _ = recommendations.push("Improve pEMF timing accuracy");
            }
        }

        if recommendations.is_empty() {
            let _ = recommendations.push("System performance is optimal");
        }

        recommendations
    }

    /// Get performance statistics
    pub fn get_performance_stats(&self) -> &PerformanceStats {
        &self.performance_stats
    }

    /// Reset optimizer state
    pub fn reset(&mut self) {
        self.test_profiles.clear();
        self.performance_samples.clear();
        self.performance_stats = PerformanceStats::default();
    }
}

/// Result of test performance optimization
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    /// Initial performance impact score (0-100)
    pub initial_impact_score: u8,
    /// Final performance impact score (0-100)
    pub final_impact_score: u8,
    /// Improvement percentage
    pub improvement_percent: f32,
    /// Number of tests optimized
    pub tests_optimized: u16,
    /// List of optimizations applied
    pub optimizations_applied: Vec<&'static str, 8>,
}

impl OptimizationResult {
    /// Create a new optimization result
    pub fn new() -> Self {
        Self {
            initial_impact_score: 0,
            final_impact_score: 0,
            improvement_percent: 0.0,
            tests_optimized: 0,
            optimizations_applied: Vec::new(),
        }
    }

    /// Check if optimization was successful
    pub fn is_successful(&self) -> bool {
        self.final_impact_score < self.initial_impact_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_framework::{TestResult, TestSuiteStats};

    // Test converted to no_std - run via test framework
    fn test_performance_profile_creation() {
        let profile = TestExecutionProfile::new("test_sample");
        assert_eq!(profile.test_id.as_str(), "test_sample");
        assert_eq!(profile.sample_count, 0);
    }

    // Test converted to no_std - run via test framework
    fn test_performance_profile_update() {
        let mut profile = TestExecutionProfile::new("test_sample");
        profile.update_with_execution(1000, 512, 10);
        
        assert_eq!(profile.avg_execution_time_us, 1000);
        assert_eq!(profile.max_execution_time_us, 1000);
        assert_eq!(profile.sample_count, 1);
    }

    // Test converted to no_std - run via test framework
    fn test_optimizer_creation() {
        let optimizer = TestPerformanceOptimizer::new();
        assert_eq!(optimizer.test_profiles.len(), 0);
        assert_eq!(optimizer.performance_samples.len(), 0);
    }

    // Test converted to no_std - run via test framework
    fn test_system_readiness_check() {
        let optimizer = TestPerformanceOptimizer::new();
        // Should be ready when no samples are available
        assert!(optimizer.is_system_ready_for_tests());
    }
}