//! Performance Profiler Module
//!
//! This module provides basic performance profiling types for the pEMF device system.
//! Most actual performance monitoring is handled by the logging module's PerformanceMonitor.
//! This module only provides the minimal types that are imported by main.rs.
//!
//! Requirements: 2.3, 3.5, 4.4

use crate::log_info;
use heapless::Vec;

// Constants referenced in the implementation
#[allow(dead_code)]
const PROFILING_SAMPLE_SIZE: usize = 100;
#[allow(dead_code)]
const TIMING_TOLERANCE_PERCENT: f32 = 0.01;
#[allow(dead_code)]
const PEMF_TARGET_FREQUENCY_HZ: f32 = 2.0;
#[allow(dead_code)]
const PEMF_HIGH_DURATION_MS: u64 = 2;
#[allow(dead_code)]
const PEMF_LOW_DURATION_MS: u64 = 498;
#[allow(dead_code)]
const BATTERY_MONITOR_INTERVAL_MS: u64 = 100;
#[allow(dead_code)]
const LED_RESPONSE_TIMEOUT_MS: u64 = 500;

/// Task execution time measurements
#[derive(Clone, Copy, Debug, Default)]
pub struct TaskExecutionTimes {
    pub pemf_pulse_time_us: u32,
    pub battery_monitor_time_us: u32,
    pub led_control_time_us: u32,
    pub usb_poll_time_us: u32,
    pub usb_hid_time_us: u32,
}

/// Timing accuracy measurements
#[derive(Clone, Copy, Debug, Default)]
pub struct TimingAccuracy {
    pub pemf_high_accuracy_percent: f32,
    pub pemf_low_accuracy_percent: f32,
    pub pemf_frequency_accuracy_percent: f32,
    pub battery_sampling_accuracy_percent: f32,
    pub led_response_accuracy_percent: f32,
}

/// System jitter measurements
#[derive(Clone, Copy, Debug, Default)]
pub struct JitterMeasurements {
    pub pemf_pulse_jitter_us: u32,
    pub battery_monitor_jitter_us: u32,
    pub led_control_jitter_us: u32,
    pub max_system_jitter_us: u32,
}

/// Performance profiling results
#[derive(Clone, Copy, Debug, Default)]
pub struct ProfilingResults {
    pub task_execution_times: TaskExecutionTimes,
    pub timing_accuracy: TimingAccuracy,
    pub jitter_measurements: JitterMeasurements,
    pub cpu_utilization_percent: u8,
    pub memory_utilization_percent: u8,
    pub overall_performance_score: u8, // 0-100 score
}

/// Performance profiler for system-wide monitoring
#[allow(dead_code)]
pub struct PerformanceProfiler {
    sample_count: usize,
    execution_time_samples: Vec<TaskExecutionTimes, PROFILING_SAMPLE_SIZE>,
    timing_samples: Vec<TimingMeasurement, PROFILING_SAMPLE_SIZE>,
    jitter_samples: Vec<JitterMeasurements, PROFILING_SAMPLE_SIZE>,
    start_time: Option<u32>, // Placeholder for start timestamp
}

/// Individual timing measurement
#[derive(Clone, Copy, Debug, Default)]
pub struct TimingMeasurement {
    pub timestamp_ms: u32,
    pub pemf_high_duration_ms: u64,
    pub pemf_low_duration_ms: u64,
    pub pemf_cycle_duration_ms: u64,
    pub battery_sample_interval_ms: u64,
    pub led_response_time_ms: u64,
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub const fn new() -> Self {
        Self {
            sample_count: 0,
            execution_time_samples: Vec::new(),
            timing_samples: Vec::new(),
            jitter_samples: Vec::new(),
            start_time: None,
        }
    }

    /// Start profiling session
    pub fn start_profiling(&mut self) {
        self.start_time = Some(0); // Placeholder timestamp
        self.sample_count = 0;
        self.execution_time_samples.clear();
        self.timing_samples.clear();
        self.jitter_samples.clear();

        log_info!("Performance profiling started");
        log_info!("Target metrics:");
        log_info!(
            "- pEMF frequency: {:.1}Hz (±{:.1}%)",
            PEMF_TARGET_FREQUENCY_HZ,
            TIMING_TOLERANCE_PERCENT * 100.0
        );
        log_info!(
            "- pEMF HIGH duration: {}ms (±{:.1}%)",
            PEMF_HIGH_DURATION_MS,
            TIMING_TOLERANCE_PERCENT * 100.0
        );
        log_info!(
            "- pEMF LOW duration: {}ms (±{:.1}%)",
            PEMF_LOW_DURATION_MS,
            TIMING_TOLERANCE_PERCENT * 100.0
        );
        log_info!(
            "- Battery sampling: {}ms intervals",
            BATTERY_MONITOR_INTERVAL_MS
        );
        log_info!("- LED response: <{}ms", LED_RESPONSE_TIMEOUT_MS);
    }

    /// Record task execution time measurement
    pub fn record_execution_times(&mut self, times: TaskExecutionTimes) -> bool {
        if self.execution_time_samples.len() < PROFILING_SAMPLE_SIZE {
            let _ = self.execution_time_samples.push(times);
            true
        } else {
            false
        }
    }

    /// Record timing measurement
    pub fn record_timing_measurement(&mut self, measurement: TimingMeasurement) -> bool {
        if self.timing_samples.len() < PROFILING_SAMPLE_SIZE {
            let _ = self.timing_samples.push(measurement);
            true
        } else {
            false
        }
    }

    /// Record jitter measurement
    pub fn record_jitter_measurement(&mut self, jitter: JitterMeasurements) -> bool {
        if self.jitter_samples.len() < PROFILING_SAMPLE_SIZE {
            let _ = self.jitter_samples.push(jitter);
            true
        } else {
            false
        }
    }

    /// Calculate comprehensive profiling results
    pub fn calculate_results(&self) -> ProfilingResults {
        let mut results = ProfilingResults::default();

        // Calculate average execution times
        if !self.execution_time_samples.is_empty() {
            let mut total_times = TaskExecutionTimes::default();
            for sample in &self.execution_time_samples {
                total_times.pemf_pulse_time_us += sample.pemf_pulse_time_us;
                total_times.battery_monitor_time_us += sample.battery_monitor_time_us;
                total_times.led_control_time_us += sample.led_control_time_us;
                total_times.usb_poll_time_us += sample.usb_poll_time_us;
                total_times.usb_hid_time_us += sample.usb_hid_time_us;
            }

            let sample_count = self.execution_time_samples.len() as u32;
            results.task_execution_times = TaskExecutionTimes {
                pemf_pulse_time_us: total_times.pemf_pulse_time_us / sample_count,
                battery_monitor_time_us: total_times.battery_monitor_time_us / sample_count,
                led_control_time_us: total_times.led_control_time_us / sample_count,
                usb_poll_time_us: total_times.usb_poll_time_us / sample_count,
                usb_hid_time_us: total_times.usb_hid_time_us / sample_count,
            };
        }

        // Calculate timing accuracy
        results.timing_accuracy = self.calculate_timing_accuracy();

        // Calculate jitter measurements
        results.jitter_measurements = self.calculate_jitter_measurements();

        // Calculate CPU utilization
        results.cpu_utilization_percent = self.calculate_cpu_utilization();

        // Calculate memory utilization
        results.memory_utilization_percent = self.calculate_memory_utilization();

        // Calculate overall performance score
        results.overall_performance_score = self.calculate_performance_score(&results);

        results
    }

    /// Calculate timing accuracy percentages
    fn calculate_timing_accuracy(&self) -> TimingAccuracy {
        let mut accuracy = TimingAccuracy::default();

        if self.timing_samples.is_empty() {
            return accuracy;
        }

        let mut pemf_high_deviations = 0u32;
        let mut pemf_low_deviations = 0u32;
        let mut pemf_frequency_deviations = 0u32;
        let mut battery_sampling_deviations = 0u32;
        let mut led_response_deviations = 0u32;

        let tolerance_ms = ((PEMF_HIGH_DURATION_MS as f32) * TIMING_TOLERANCE_PERCENT) as u64;

        for sample in &self.timing_samples {
            // Check pEMF HIGH duration accuracy
            let high_deviation = sample.pemf_high_duration_ms.abs_diff(PEMF_HIGH_DURATION_MS);

            if high_deviation <= tolerance_ms {
                pemf_high_deviations += 1;
            }

            // Check pEMF LOW duration accuracy
            let low_deviation = sample.pemf_low_duration_ms.abs_diff(PEMF_LOW_DURATION_MS);

            if low_deviation <= tolerance_ms {
                pemf_low_deviations += 1;
            }

            // Check pEMF frequency accuracy
            let expected_cycle_ms = PEMF_HIGH_DURATION_MS + PEMF_LOW_DURATION_MS;
            let cycle_deviation = sample.pemf_cycle_duration_ms.abs_diff(expected_cycle_ms);

            if cycle_deviation <= tolerance_ms {
                pemf_frequency_deviations += 1;
            }

            // Check battery sampling accuracy
            let battery_deviation = sample
                .battery_sample_interval_ms
                .abs_diff(BATTERY_MONITOR_INTERVAL_MS);

            if battery_deviation <= tolerance_ms {
                battery_sampling_deviations += 1;
            }

            // Check LED response time
            if sample.led_response_time_ms <= LED_RESPONSE_TIMEOUT_MS {
                led_response_deviations += 1;
            }
        }

        let sample_count = self.timing_samples.len() as f32;
        accuracy.pemf_high_accuracy_percent = (pemf_high_deviations as f32 / sample_count) * 100.0;
        accuracy.pemf_low_accuracy_percent = (pemf_low_deviations as f32 / sample_count) * 100.0;
        accuracy.pemf_frequency_accuracy_percent =
            (pemf_frequency_deviations as f32 / sample_count) * 100.0;
        accuracy.battery_sampling_accuracy_percent =
            (battery_sampling_deviations as f32 / sample_count) * 100.0;
        accuracy.led_response_accuracy_percent =
            (led_response_deviations as f32 / sample_count) * 100.0;

        accuracy
    }

    /// Calculate jitter measurements
    fn calculate_jitter_measurements(&self) -> JitterMeasurements {
        let mut jitter = JitterMeasurements::default();

        if self.jitter_samples.is_empty() {
            return jitter;
        }

        let mut max_pemf_jitter = 0u32;
        let mut max_battery_jitter = 0u32;
        let mut max_led_jitter = 0u32;
        let mut max_system_jitter = 0u32;

        for sample in &self.jitter_samples {
            if sample.pemf_pulse_jitter_us > max_pemf_jitter {
                max_pemf_jitter = sample.pemf_pulse_jitter_us;
            }
            if sample.battery_monitor_jitter_us > max_battery_jitter {
                max_battery_jitter = sample.battery_monitor_jitter_us;
            }
            if sample.led_control_jitter_us > max_led_jitter {
                max_led_jitter = sample.led_control_jitter_us;
            }
            if sample.max_system_jitter_us > max_system_jitter {
                max_system_jitter = sample.max_system_jitter_us;
            }
        }

        jitter.pemf_pulse_jitter_us = max_pemf_jitter;
        jitter.battery_monitor_jitter_us = max_battery_jitter;
        jitter.led_control_jitter_us = max_led_jitter;
        jitter.max_system_jitter_us = max_system_jitter;

        jitter
    }

    /// Calculate CPU utilization percentage
    fn calculate_cpu_utilization(&self) -> u8 {
        if self.execution_time_samples.is_empty() {
            return 0;
        }

        // Calculate total execution time per cycle
        let mut total_execution_time_us = 0u32;
        for sample in &self.execution_time_samples {
            total_execution_time_us += sample.pemf_pulse_time_us;
            total_execution_time_us += sample.battery_monitor_time_us;
            total_execution_time_us += sample.led_control_time_us;
            total_execution_time_us += sample.usb_poll_time_us;
            total_execution_time_us += sample.usb_hid_time_us;
        }

        let avg_execution_time_us =
            total_execution_time_us / (self.execution_time_samples.len() as u32);

        // Assume 500ms cycle time (pEMF period)
        let cycle_time_us = 500_000u32;
        let cpu_utilization = (avg_execution_time_us * 100) / cycle_time_us;

        core::cmp::min(cpu_utilization, 100) as u8
    }

    /// Calculate memory utilization percentage
    fn calculate_memory_utilization(&self) -> u8 {
        // Estimate memory usage based on queue sizes and buffers
        const LOG_QUEUE_SIZE: usize = 32 * 64; // 32 messages * 64 bytes each
        const USB_BUFFER_SIZE: usize = 1024; // Estimated USB buffer size
        const PROFILER_SIZE: usize = core::mem::size_of::<PerformanceProfiler>();
        const TOTAL_ESTIMATED_USAGE: usize = LOG_QUEUE_SIZE + USB_BUFFER_SIZE + PROFILER_SIZE;

        // RP2040 has 264KB of RAM
        const TOTAL_RAM: usize = 264 * 1024;

        let utilization = (TOTAL_ESTIMATED_USAGE * 100) / TOTAL_RAM;
        core::cmp::min(utilization, 100) as u8
    }

    /// Calculate overall performance score (0-100)
    fn calculate_performance_score(&self, results: &ProfilingResults) -> u8 {
        let mut score = 100u8;

        // Deduct points for timing inaccuracy
        let avg_timing_accuracy = (results.timing_accuracy.pemf_high_accuracy_percent
            + results.timing_accuracy.pemf_low_accuracy_percent
            + results.timing_accuracy.pemf_frequency_accuracy_percent
            + results.timing_accuracy.battery_sampling_accuracy_percent
            + results.timing_accuracy.led_response_accuracy_percent)
            / 5.0;

        if avg_timing_accuracy < 99.0 {
            score = score.saturating_sub((100.0 - avg_timing_accuracy) as u8);
        }

        // Deduct points for high CPU utilization
        if results.cpu_utilization_percent > 50 {
            score = score.saturating_sub(results.cpu_utilization_percent - 50);
        }

        // Deduct points for high memory utilization
        if results.memory_utilization_percent > 25 {
            score = score.saturating_sub(results.memory_utilization_percent - 25);
        }

        // Deduct points for excessive jitter
        const MAX_ACCEPTABLE_JITTER_US: u32 = 1000; // 1ms
        if results.jitter_measurements.max_system_jitter_us > MAX_ACCEPTABLE_JITTER_US {
            let jitter_penalty = (results.jitter_measurements.max_system_jitter_us / 1000) as u8;
            score = score.saturating_sub(jitter_penalty);
        }

        score
    }

    /// Generate comprehensive performance report
    pub fn generate_report(&self, results: &ProfilingResults) -> PerformanceReport {
        PerformanceReport {
            profiling_duration_ms: self.get_profiling_duration_ms(),
            sample_count: self.timing_samples.len(),
            results: *results,
            recommendations: self.generate_recommendations(results),
        }
    }

    /// Get profiling duration in milliseconds
    fn get_profiling_duration_ms(&self) -> u32 {
        if let Some(_start_time) = self.start_time {
            // Placeholder - would calculate actual duration using timer
            1000 // Return 1 second as placeholder
        } else {
            0
        }
    }

    /// Generate performance recommendations
    fn generate_recommendations(&self, results: &ProfilingResults) -> Vec<&'static str, 8> {
        let mut recommendations = Vec::new();

        if results.timing_accuracy.pemf_high_accuracy_percent < 99.0 {
            let _ = recommendations.push("Optimize pEMF HIGH phase timing");
        }

        if results.timing_accuracy.pemf_low_accuracy_percent < 99.0 {
            let _ = recommendations.push("Optimize pEMF LOW phase timing");
        }

        if results.cpu_utilization_percent > 75 {
            let _ = recommendations.push("Reduce CPU utilization");
        }

        if results.memory_utilization_percent > 50 {
            let _ = recommendations.push("Optimize memory usage");
        }

        if results.jitter_measurements.max_system_jitter_us > 2000 {
            let _ = recommendations.push("Reduce system jitter");
        }

        if results.timing_accuracy.battery_sampling_accuracy_percent < 95.0 {
            let _ = recommendations.push("Improve battery sampling timing");
        }

        if results.timing_accuracy.led_response_accuracy_percent < 90.0 {
            let _ = recommendations.push("Optimize LED response time");
        }

        if results.overall_performance_score < 80 {
            let _ = recommendations.push("Overall system optimization needed");
        }

        recommendations
    }
}

/// Performance report structure
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct PerformanceReport {
    pub profiling_duration_ms: u32,
    pub sample_count: usize,
    pub results: ProfilingResults,
    pub recommendations: Vec<&'static str, 8>,
}

impl PerformanceReport {
    /// Log the performance report
    pub fn log_report(&self) {
        log_info!("=== PERFORMANCE PROFILING REPORT ===");
        log_info!("Profiling duration: {}ms", self.profiling_duration_ms);
        log_info!("Sample count: {}", self.sample_count);
        log_info!("");

        log_info!("Task Execution Times (average):");
        log_info!(
            "- pEMF pulse: {}μs",
            self.results.task_execution_times.pemf_pulse_time_us
        );
        log_info!(
            "- Battery monitor: {}μs",
            self.results.task_execution_times.battery_monitor_time_us
        );
        log_info!(
            "- LED control: {}μs",
            self.results.task_execution_times.led_control_time_us
        );
        log_info!(
            "- USB poll: {}μs",
            self.results.task_execution_times.usb_poll_time_us
        );
        log_info!(
            "- USB HID: {}μs",
            self.results.task_execution_times.usb_hid_time_us
        );
        log_info!("");

        log_info!("Timing Accuracy:");
        log_info!(
            "- pEMF HIGH: {:.1}%",
            self.results.timing_accuracy.pemf_high_accuracy_percent
        );
        log_info!(
            "- pEMF LOW: {:.1}%",
            self.results.timing_accuracy.pemf_low_accuracy_percent
        );
        log_info!(
            "- pEMF frequency: {:.1}%",
            self.results.timing_accuracy.pemf_frequency_accuracy_percent
        );
        log_info!(
            "- Battery sampling: {:.1}%",
            self.results
                .timing_accuracy
                .battery_sampling_accuracy_percent
        );
        log_info!(
            "- LED response: {:.1}%",
            self.results.timing_accuracy.led_response_accuracy_percent
        );
        log_info!("");

        log_info!("Jitter Measurements:");
        log_info!(
            "- pEMF pulse: {}μs",
            self.results.jitter_measurements.pemf_pulse_jitter_us
        );
        log_info!(
            "- Battery monitor: {}μs",
            self.results.jitter_measurements.battery_monitor_jitter_us
        );
        log_info!(
            "- LED control: {}μs",
            self.results.jitter_measurements.led_control_jitter_us
        );
        log_info!(
            "- Max system: {}μs",
            self.results.jitter_measurements.max_system_jitter_us
        );
        log_info!("");

        log_info!("System Utilization:");
        log_info!("- CPU: {}%", self.results.cpu_utilization_percent);
        log_info!("- Memory: {}%", self.results.memory_utilization_percent);
        log_info!(
            "- Overall score: {}/100",
            self.results.overall_performance_score
        );
        log_info!("");

        if !self.recommendations.is_empty() {
            log_info!("Recommendations:");
            for (i, recommendation) in self.recommendations.iter().enumerate() {
                log_info!("{}. {}", i + 1, recommendation);
            }
        } else {
            log_info!("No recommendations - system performance is optimal");
        }

        log_info!("=== END PERFORMANCE REPORT ===");
    }
}

/// Global performance profiler instance
#[allow(dead_code)]
static mut GLOBAL_PROFILER: Option<PerformanceProfiler> = None;

/// Initialize global performance profiler
pub fn init_global_profiler() {
    unsafe {
        GLOBAL_PROFILER = Some(PerformanceProfiler::new());
    }
}

/// Get mutable reference to global profiler
pub fn get_global_profiler() -> Option<&'static mut PerformanceProfiler> {
    unsafe { (*core::ptr::addr_of_mut!(GLOBAL_PROFILER)).as_mut() }
}

/// Convenience macros for performance measurement
#[macro_export]
macro_rules! measure_task_execution {
    ($task_name:expr, $code:block) => {{
        // Placeholder implementation - would use actual timer in real system
        let result = $code;
        let execution_time_us = 100u32; // Placeholder execution time

        log_debug!(
            "Task {} execution time: {}μs",
            $task_name,
            execution_time_us
        );

        (result, execution_time_us)
    }};
}

#[macro_export]
macro_rules! measure_timing_accuracy {
    ($expected_duration_ms:expr, $actual_duration_ms:expr) => {{
        let deviation = if $actual_duration_ms > $expected_duration_ms {
            $actual_duration_ms - $expected_duration_ms
        } else {
            $expected_duration_ms - $actual_duration_ms
        };

        let accuracy_percent = if $expected_duration_ms > 0 {
            100.0 - ((deviation as f32 / $expected_duration_ms as f32) * 100.0)
        } else {
            0.0
        };

        accuracy_percent
    }};
}
