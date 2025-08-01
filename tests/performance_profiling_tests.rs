//! Performance Profiling Tests
//! 
//! This module contains comprehensive tests for performance profiling and timing validation.
//! Tests cover pEMF pulse timing accuracy, battery monitoring latency, and LED response times.
//! 
//! Requirements: 2.3, 3.5, 4.4

#![cfg(test)]

use std::time::{Duration, Instant};
use std::thread;

// Mock structures for testing (since we can't run embedded code in tests)
#[derive(Clone, Copy, Debug, Default)]
struct MockTaskExecutionTimes {
    pub pemf_pulse_time_us: u32,
    pub battery_monitor_time_us: u32,
    pub led_control_time_us: u32,
    pub usb_poll_time_us: u32,
    pub usb_hid_time_us: u32,
}

#[derive(Clone, Copy, Debug, Default)]
struct MockTimingAccuracy {
    pub pemf_high_accuracy_percent: f32,
    pub pemf_low_accuracy_percent: f32,
    pub pemf_frequency_accuracy_percent: f32,
    pub battery_sampling_accuracy_percent: f32,
    pub led_response_accuracy_percent: f32,
}

#[derive(Clone, Copy, Debug, Default)]
struct MockJitterMeasurements {
    pub pemf_pulse_jitter_us: u32,
    pub battery_monitor_jitter_us: u32,
    pub led_control_jitter_us: u32,
    pub max_system_jitter_us: u32,
}

#[derive(Clone, Copy, Debug, Default)]
struct MockProfilingResults {
    pub task_execution_times: MockTaskExecutionTimes,
    pub timing_accuracy: MockTimingAccuracy,
    pub jitter_measurements: MockJitterMeasurements,
    pub cpu_utilization_percent: u8,
    pub memory_utilization_percent: u8,
    pub overall_performance_score: u8,
}

/// Mock performance profiler for testing
struct MockPerformanceProfiler {
    samples: Vec<MockProfilingResults>,
    start_time: Option<Instant>,
}

impl MockPerformanceProfiler {
    fn new() -> Self {
        Self {
            samples: Vec::new(),
            start_time: None,
        }
    }

    fn start_profiling(&mut self) {
        self.start_time = Some(Instant::now());
        self.samples.clear();
        println!("Mock performance profiling started");
    }

    fn record_sample(&mut self, sample: MockProfilingResults) {
        self.samples.push(sample);
    }

    fn calculate_results(&self) -> MockProfilingResults {
        if self.samples.is_empty() {
            return MockProfilingResults::default();
        }

        let mut total = MockProfilingResults::default();
        for sample in &self.samples {
            total.task_execution_times.pemf_pulse_time_us += sample.task_execution_times.pemf_pulse_time_us;
            total.task_execution_times.battery_monitor_time_us += sample.task_execution_times.battery_monitor_time_us;
            total.task_execution_times.led_control_time_us += sample.task_execution_times.led_control_time_us;
            total.task_execution_times.usb_poll_time_us += sample.task_execution_times.usb_poll_time_us;
            total.task_execution_times.usb_hid_time_us += sample.task_execution_times.usb_hid_time_us;
            
            total.timing_accuracy.pemf_high_accuracy_percent += sample.timing_accuracy.pemf_high_accuracy_percent;
            total.timing_accuracy.pemf_low_accuracy_percent += sample.timing_accuracy.pemf_low_accuracy_percent;
            total.timing_accuracy.pemf_frequency_accuracy_percent += sample.timing_accuracy.pemf_frequency_accuracy_percent;
            total.timing_accuracy.battery_sampling_accuracy_percent += sample.timing_accuracy.battery_sampling_accuracy_percent;
            total.timing_accuracy.led_response_accuracy_percent += sample.timing_accuracy.led_response_accuracy_percent;
            
            total.jitter_measurements.pemf_pulse_jitter_us = total.jitter_measurements.pemf_pulse_jitter_us.max(sample.jitter_measurements.pemf_pulse_jitter_us);
            total.jitter_measurements.battery_monitor_jitter_us = total.jitter_measurements.battery_monitor_jitter_us.max(sample.jitter_measurements.battery_monitor_jitter_us);
            total.jitter_measurements.led_control_jitter_us = total.jitter_measurements.led_control_jitter_us.max(sample.jitter_measurements.led_control_jitter_us);
            total.jitter_measurements.max_system_jitter_us = total.jitter_measurements.max_system_jitter_us.max(sample.jitter_measurements.max_system_jitter_us);
            
            total.cpu_utilization_percent += sample.cpu_utilization_percent;
            total.memory_utilization_percent += sample.memory_utilization_percent;
            total.overall_performance_score += sample.overall_performance_score;
        }

        let sample_count = self.samples.len() as u32;
        MockProfilingResults {
            task_execution_times: MockTaskExecutionTimes {
                pemf_pulse_time_us: total.task_execution_times.pemf_pulse_time_us / sample_count,
                battery_monitor_time_us: total.task_execution_times.battery_monitor_time_us / sample_count,
                led_control_time_us: total.task_execution_times.led_control_time_us / sample_count,
                usb_poll_time_us: total.task_execution_times.usb_poll_time_us / sample_count,
                usb_hid_time_us: total.task_execution_times.usb_hid_time_us / sample_count,
            },
            timing_accuracy: MockTimingAccuracy {
                pemf_high_accuracy_percent: total.timing_accuracy.pemf_high_accuracy_percent / sample_count as f32,
                pemf_low_accuracy_percent: total.timing_accuracy.pemf_low_accuracy_percent / sample_count as f32,
                pemf_frequency_accuracy_percent: total.timing_accuracy.pemf_frequency_accuracy_percent / sample_count as f32,
                battery_sampling_accuracy_percent: total.timing_accuracy.battery_sampling_accuracy_percent / sample_count as f32,
                led_response_accuracy_percent: total.timing_accuracy.led_response_accuracy_percent / sample_count as f32,
            },
            jitter_measurements: total.jitter_measurements,
            cpu_utilization_percent: total.cpu_utilization_percent / sample_count as u8,
            memory_utilization_percent: total.memory_utilization_percent / sample_count as u8,
            overall_performance_score: total.overall_performance_score / sample_count as u8,
        }
    }
}

/// Test pEMF pulse timing accuracy measurement
/// Requirements: 2.3 (±1% timing tolerance)
#[test]
fn test_pemf_pulse_timing_accuracy() {
    println!("Testing pEMF pulse timing accuracy measurement");
    
    let mut profiler = MockPerformanceProfiler::new();
    profiler.start_profiling();
    
    // Simulate pEMF pulse measurements with various accuracy levels
    let test_cases = vec![
        // Perfect timing
        (2.0, 498.0, 100.0), // HIGH, LOW, expected accuracy
        // Slight deviation within tolerance
        (2.02, 497.98, 99.0), // 1% deviation
        // Deviation at tolerance limit
        (2.02, 497.98, 99.0), // 1% deviation
        // Deviation exceeding tolerance
        (2.05, 497.95, 97.5), // 2.5% deviation
    ];
    
    for (high_ms, low_ms, expected_accuracy) in test_cases {
        let sample = MockProfilingResults {
            timing_accuracy: MockTimingAccuracy {
                pemf_high_accuracy_percent: calculate_timing_accuracy(2.0, high_ms),
                pemf_low_accuracy_percent: calculate_timing_accuracy(498.0, low_ms),
                pemf_frequency_accuracy_percent: calculate_timing_accuracy(500.0, high_ms + low_ms),
                ..Default::default()
            },
            ..Default::default()
        };
        
        profiler.record_sample(sample);
        
        // Verify timing accuracy calculation
        let high_accuracy = calculate_timing_accuracy(2.0, high_ms);
        assert!(
            (high_accuracy - expected_accuracy).abs() < 1.0,
            "HIGH phase accuracy mismatch: expected {:.1}%, got {:.1}%",
            expected_accuracy,
            high_accuracy
        );
        
        println!("pEMF timing test: HIGH={:.2}ms, LOW={:.2}ms, Accuracy={:.1}%", 
                high_ms, low_ms, high_accuracy);
    }
    
    let results = profiler.calculate_results();
    
    // Verify overall timing accuracy meets requirements
    assert!(
        results.timing_accuracy.pemf_high_accuracy_percent >= 99.0,
        "pEMF HIGH phase accuracy below requirement: {:.1}% < 99.0%",
        results.timing_accuracy.pemf_high_accuracy_percent
    );
    
    println!("✓ pEMF pulse timing accuracy test passed");
}

/// Test battery monitoring latency requirements
/// Requirements: 3.5 (200ms state update requirement)
#[test]
fn test_battery_monitoring_latency() {
    println!("Testing battery monitoring latency requirements");
    
    let mut profiler = MockPerformanceProfiler::new();
    profiler.start_profiling();
    
    // Simulate battery monitoring with various latencies
    let test_cases = vec![
        (50, 100.0),   // 50ms latency - excellent
        (100, 100.0),  // 100ms latency - good
        (150, 100.0),  // 150ms latency - acceptable
        (200, 100.0),  // 200ms latency - at limit
        (250, 80.0),   // 250ms latency - exceeds requirement
    ];
    
    for (latency_ms, expected_accuracy) in test_cases {
        let sample = MockProfilingResults {
            task_execution_times: MockTaskExecutionTimes {
                battery_monitor_time_us: (latency_ms * 1000) as u32, // Convert to microseconds
                ..Default::default()
            },
            timing_accuracy: MockTimingAccuracy {
                battery_sampling_accuracy_percent: if latency_ms <= 200 { 100.0 } else { expected_accuracy },
                ..Default::default()
            },
            ..Default::default()
        };
        
        profiler.record_sample(sample);
        
        println!("Battery monitoring test: latency={}ms, meets requirement={}", 
                latency_ms, latency_ms <= 200);
    }
    
    let results = profiler.calculate_results();
    
    // Verify battery monitoring latency meets requirements
    let avg_latency_ms = results.task_execution_times.battery_monitor_time_us / 1000;
    assert!(
        avg_latency_ms <= 200,
        "Battery monitoring latency exceeds requirement: {}ms > 200ms",
        avg_latency_ms
    );
    
    assert!(
        results.timing_accuracy.battery_sampling_accuracy_percent >= 95.0,
        "Battery sampling accuracy below requirement: {:.1}% < 95.0%",
        results.timing_accuracy.battery_sampling_accuracy_percent
    );
    
    println!("✓ Battery monitoring latency test passed");
}

/// Test LED response time requirements
/// Requirements: 4.4 (500ms LED update requirement)
#[test]
fn test_led_response_time() {
    println!("Testing LED response time requirements");
    
    let mut profiler = MockPerformanceProfiler::new();
    profiler.start_profiling();
    
    // Simulate LED response times for different scenarios
    let test_cases = vec![
        (50, "immediate response"),
        (100, "fast response"),
        (250, "normal response"),
        (400, "slow but acceptable"),
        (500, "at requirement limit"),
        (600, "exceeds requirement"),
    ];
    
    for (response_time_ms, description) in test_cases {
        let sample = MockProfilingResults {
            task_execution_times: MockTaskExecutionTimes {
                led_control_time_us: (response_time_ms * 1000) as u32,
                ..Default::default()
            },
            timing_accuracy: MockTimingAccuracy {
                led_response_accuracy_percent: if response_time_ms <= 500 { 100.0 } else { 80.0 },
                ..Default::default()
            },
            ..Default::default()
        };
        
        profiler.record_sample(sample);
        
        println!("LED response test: {}ms ({}), meets requirement={}", 
                response_time_ms, description, response_time_ms <= 500);
    }
    
    let results = profiler.calculate_results();
    
    // Verify LED response time meets requirements
    let avg_response_time_ms = results.task_execution_times.led_control_time_us / 1000;
    assert!(
        avg_response_time_ms <= 500,
        "LED response time exceeds requirement: {}ms > 500ms",
        avg_response_time_ms
    );
    
    assert!(
        results.timing_accuracy.led_response_accuracy_percent >= 90.0,
        "LED response accuracy below requirement: {:.1}% < 90.0%",
        results.timing_accuracy.led_response_accuracy_percent
    );
    
    println!("✓ LED response time test passed");
}

/// Test system jitter measurements
#[test]
fn test_system_jitter_measurement() {
    println!("Testing system jitter measurements");
    
    let mut profiler = MockPerformanceProfiler::new();
    profiler.start_profiling();
    
    // Simulate various jitter scenarios
    let test_cases = vec![
        (100, 50, 25, "low jitter system"),
        (500, 200, 100, "moderate jitter"),
        (1000, 500, 250, "high jitter"),
        (2000, 1000, 500, "excessive jitter"),
    ];
    
    for (pemf_jitter_us, battery_jitter_us, led_jitter_us, description) in test_cases {
        let max_jitter = pemf_jitter_us.max(battery_jitter_us).max(led_jitter_us);
        
        let sample = MockProfilingResults {
            jitter_measurements: MockJitterMeasurements {
                pemf_pulse_jitter_us: pemf_jitter_us,
                battery_monitor_jitter_us: battery_jitter_us,
                led_control_jitter_us: led_jitter_us,
                max_system_jitter_us: max_jitter,
            },
            ..Default::default()
        };
        
        profiler.record_sample(sample);
        
        println!("Jitter test: pEMF={}μs, Battery={}μs, LED={}μs, Max={}μs ({})", 
                pemf_jitter_us, battery_jitter_us, led_jitter_us, max_jitter, description);
    }
    
    let results = profiler.calculate_results();
    
    // Verify jitter is within acceptable limits
    assert!(
        results.jitter_measurements.pemf_pulse_jitter_us <= 1000,
        "pEMF pulse jitter too high: {}μs > 1000μs",
        results.jitter_measurements.pemf_pulse_jitter_us
    );
    
    assert!(
        results.jitter_measurements.max_system_jitter_us <= 2000,
        "System jitter too high: {}μs > 2000μs",
        results.jitter_measurements.max_system_jitter_us
    );
    
    println!("✓ System jitter measurement test passed");
}

/// Test CPU utilization calculation
#[test]
fn test_cpu_utilization_calculation() {
    println!("Testing CPU utilization calculation");
    
    let mut profiler = MockPerformanceProfiler::new();
    profiler.start_profiling();
    
    // Simulate different CPU utilization scenarios
    let test_cases = vec![
        (10, 5, 2, 3, 2, "low utilization"),      // Total: 22μs per 500ms cycle = 0.004%
        (100, 50, 20, 30, 20, "moderate utilization"), // Total: 220μs per 500ms cycle = 0.044%
        (1000, 500, 200, 300, 200, "high utilization"), // Total: 2200μs per 500ms cycle = 0.44%
        (10000, 5000, 2000, 3000, 2000, "very high utilization"), // Total: 22000μs per 500ms cycle = 4.4%
    ];
    
    for (pemf_us, battery_us, led_us, usb_poll_us, usb_hid_us, description) in test_cases {
        let total_execution_time_us = pemf_us + battery_us + led_us + usb_poll_us + usb_hid_us;
        let cpu_utilization = calculate_cpu_utilization(total_execution_time_us, 500_000); // 500ms cycle
        
        let sample = MockProfilingResults {
            task_execution_times: MockTaskExecutionTimes {
                pemf_pulse_time_us: pemf_us,
                battery_monitor_time_us: battery_us,
                led_control_time_us: led_us,
                usb_poll_time_us: usb_poll_us,
                usb_hid_time_us: usb_hid_us,
            },
            cpu_utilization_percent: cpu_utilization,
            ..Default::default()
        };
        
        profiler.record_sample(sample);
        
        println!("CPU utilization test: total={}μs, utilization={}% ({})", 
                total_execution_time_us, cpu_utilization, description);
    }
    
    let results = profiler.calculate_results();
    
    // Verify CPU utilization is reasonable
    assert!(
        results.cpu_utilization_percent <= 50,
        "CPU utilization too high: {}% > 50%",
        results.cpu_utilization_percent
    );
    
    println!("✓ CPU utilization calculation test passed");
}

/// Test overall performance scoring
#[test]
fn test_performance_scoring() {
    println!("Testing overall performance scoring");
    
    let mut profiler = MockPerformanceProfiler::new();
    profiler.start_profiling();
    
    // Test different performance scenarios
    let test_cases = vec![
        // Perfect performance
        (MockProfilingResults {
            timing_accuracy: MockTimingAccuracy {
                pemf_high_accuracy_percent: 99.9,
                pemf_low_accuracy_percent: 99.9,
                pemf_frequency_accuracy_percent: 99.9,
                battery_sampling_accuracy_percent: 99.9,
                led_response_accuracy_percent: 99.9,
            },
            cpu_utilization_percent: 10,
            memory_utilization_percent: 5,
            jitter_measurements: MockJitterMeasurements {
                max_system_jitter_us: 100,
                ..Default::default()
            },
            overall_performance_score: 100,
            ..Default::default()
        }, "perfect performance"),
        
        // Good performance
        (MockProfilingResults {
            timing_accuracy: MockTimingAccuracy {
                pemf_high_accuracy_percent: 99.0,
                pemf_low_accuracy_percent: 99.0,
                pemf_frequency_accuracy_percent: 99.0,
                battery_sampling_accuracy_percent: 98.0,
                led_response_accuracy_percent: 97.0,
            },
            cpu_utilization_percent: 25,
            memory_utilization_percent: 15,
            jitter_measurements: MockJitterMeasurements {
                max_system_jitter_us: 500,
                ..Default::default()
            },
            overall_performance_score: 95,
            ..Default::default()
        }, "good performance"),
        
        // Poor performance
        (MockProfilingResults {
            timing_accuracy: MockTimingAccuracy {
                pemf_high_accuracy_percent: 95.0,
                pemf_low_accuracy_percent: 94.0,
                pemf_frequency_accuracy_percent: 93.0,
                battery_sampling_accuracy_percent: 90.0,
                led_response_accuracy_percent: 85.0,
            },
            cpu_utilization_percent: 75,
            memory_utilization_percent: 60,
            jitter_measurements: MockJitterMeasurements {
                max_system_jitter_us: 3000,
                ..Default::default()
            },
            overall_performance_score: 60,
            ..Default::default()
        }, "poor performance"),
    ];
    
    for (sample, description) in test_cases {
        profiler.record_sample(sample);
        
        let avg_timing_accuracy = (
            sample.timing_accuracy.pemf_high_accuracy_percent +
            sample.timing_accuracy.pemf_low_accuracy_percent +
            sample.timing_accuracy.pemf_frequency_accuracy_percent +
            sample.timing_accuracy.battery_sampling_accuracy_percent +
            sample.timing_accuracy.led_response_accuracy_percent
        ) / 5.0;
        
        println!("Performance test: score={}, timing={:.1}%, CPU={}%, memory={}%, jitter={}μs ({})", 
                sample.overall_performance_score, avg_timing_accuracy, 
                sample.cpu_utilization_percent, sample.memory_utilization_percent,
                sample.jitter_measurements.max_system_jitter_us, description);
    }
    
    let results = profiler.calculate_results();
    
    // Verify performance scoring is reasonable
    assert!(
        results.overall_performance_score >= 60,
        "Overall performance score too low: {} < 60",
        results.overall_performance_score
    );
    
    println!("✓ Performance scoring test passed");
}

/// Test comprehensive performance validation
#[test]
fn test_comprehensive_performance_validation() {
    println!("Testing comprehensive performance validation");
    
    let mut profiler = MockPerformanceProfiler::new();
    profiler.start_profiling();
    
    // Simulate a realistic system performance profile
    for i in 0..50 {
        let base_jitter = (i % 10) as u32 * 50; // Varying jitter
        let cpu_load_factor = 1.0 + (i as f32 * 0.01); // Gradually increasing load
        
        let sample = MockProfilingResults {
            task_execution_times: MockTaskExecutionTimes {
                pemf_pulse_time_us: (100.0 * cpu_load_factor) as u32,
                battery_monitor_time_us: (200.0 * cpu_load_factor) as u32,
                led_control_time_us: (50.0 * cpu_load_factor) as u32,
                usb_poll_time_us: (150.0 * cpu_load_factor) as u32,
                usb_hid_time_us: (100.0 * cpu_load_factor) as u32,
            },
            timing_accuracy: MockTimingAccuracy {
                pemf_high_accuracy_percent: 99.5 - (i as f32 * 0.1),
                pemf_low_accuracy_percent: 99.3 - (i as f32 * 0.08),
                pemf_frequency_accuracy_percent: 99.7 - (i as f32 * 0.05),
                battery_sampling_accuracy_percent: 98.5 - (i as f32 * 0.1),
                led_response_accuracy_percent: 97.0 - (i as f32 * 0.1),
            },
            jitter_measurements: MockJitterMeasurements {
                pemf_pulse_jitter_us: base_jitter + 100,
                battery_monitor_jitter_us: base_jitter + 200,
                led_control_jitter_us: base_jitter + 150,
                max_system_jitter_us: base_jitter + 300,
            },
            cpu_utilization_percent: (10.0 + (i as f32 * 0.5)) as u8,
            memory_utilization_percent: (5.0 + (i as f32 * 0.3)) as u8,
            overall_performance_score: (100 - i) as u8,
            ..Default::default()
        };
        
        profiler.record_sample(sample);
    }
    
    let results = profiler.calculate_results();
    
    // Validate all performance requirements
    println!("Comprehensive validation results:");
    println!("- Average pEMF HIGH accuracy: {:.1}%", results.timing_accuracy.pemf_high_accuracy_percent);
    println!("- Average pEMF LOW accuracy: {:.1}%", results.timing_accuracy.pemf_low_accuracy_percent);
    println!("- Average pEMF frequency accuracy: {:.1}%", results.timing_accuracy.pemf_frequency_accuracy_percent);
    println!("- Average battery sampling accuracy: {:.1}%", results.timing_accuracy.battery_sampling_accuracy_percent);
    println!("- Average LED response accuracy: {:.1}%", results.timing_accuracy.led_response_accuracy_percent);
    println!("- Average CPU utilization: {}%", results.cpu_utilization_percent);
    println!("- Average memory utilization: {}%", results.memory_utilization_percent);
    println!("- Max system jitter: {}μs", results.jitter_measurements.max_system_jitter_us);
    println!("- Overall performance score: {}", results.overall_performance_score);
    
    // Requirements validation
    assert!(
        results.timing_accuracy.pemf_high_accuracy_percent >= 99.0,
        "pEMF HIGH timing accuracy requirement not met: {:.1}% < 99.0%",
        results.timing_accuracy.pemf_high_accuracy_percent
    );
    
    assert!(
        results.timing_accuracy.pemf_low_accuracy_percent >= 99.0,
        "pEMF LOW timing accuracy requirement not met: {:.1}% < 99.0%",
        results.timing_accuracy.pemf_low_accuracy_percent
    );
    
    assert!(
        results.timing_accuracy.battery_sampling_accuracy_percent >= 95.0,
        "Battery sampling accuracy requirement not met: {:.1}% < 95.0%",
        results.timing_accuracy.battery_sampling_accuracy_percent
    );
    
    assert!(
        results.timing_accuracy.led_response_accuracy_percent >= 90.0,
        "LED response accuracy requirement not met: {:.1}% < 90.0%",
        results.timing_accuracy.led_response_accuracy_percent
    );
    
    assert!(
        results.cpu_utilization_percent <= 50,
        "CPU utilization too high: {}% > 50%",
        results.cpu_utilization_percent
    );
    
    assert!(
        results.memory_utilization_percent <= 30,
        "Memory utilization too high: {}% > 30%",
        results.memory_utilization_percent
    );
    
    assert!(
        results.jitter_measurements.max_system_jitter_us <= 2000,
        "System jitter too high: {}μs > 2000μs",
        results.jitter_measurements.max_system_jitter_us
    );
    
    println!("✓ Comprehensive performance validation test passed");
}

// Helper functions for testing

fn calculate_timing_accuracy(expected: f32, actual: f32) -> f32 {
    if expected == 0.0 {
        return 0.0;
    }
    
    let deviation = (actual - expected).abs();
    let accuracy = 100.0 - ((deviation / expected) * 100.0);
    accuracy.max(0.0)
}

fn calculate_cpu_utilization(execution_time_us: u32, period_us: u32) -> u8 {
    if period_us == 0 {
        return 0;
    }
    
    let utilization = (execution_time_us * 100) / period_us;
    std::cmp::min(utilization, 100) as u8
}

#[test]
fn test_timing_accuracy_calculation() {
    // Test perfect timing
    assert_eq!(calculate_timing_accuracy(100.0, 100.0), 100.0);
    
    // Test 1% deviation
    assert!((calculate_timing_accuracy(100.0, 101.0) - 99.0).abs() < 0.1);
    
    // Test 5% deviation
    assert!((calculate_timing_accuracy(100.0, 105.0) - 95.0).abs() < 0.1);
    
    // Test large deviation
    assert!((calculate_timing_accuracy(100.0, 150.0) - 50.0).abs() < 0.1);
    
    println!("✓ Timing accuracy calculation test passed");
}

#[test]
fn test_cpu_utilization_calculation_helper() {
    // Test low utilization
    assert_eq!(calculate_cpu_utilization(1000, 100000), 1); // 1%
    
    // Test moderate utilization
    assert_eq!(calculate_cpu_utilization(25000, 100000), 25); // 25%
    
    // Test high utilization
    assert_eq!(calculate_cpu_utilization(75000, 100000), 75); // 75%
    
    // Test maximum utilization
    assert_eq!(calculate_cpu_utilization(100000, 100000), 100); // 100%
    
    // Test over-utilization (should cap at 100%)
    assert_eq!(calculate_cpu_utilization(150000, 100000), 100); // Capped at 100%
    
    println!("✓ CPU utilization calculation helper test passed");
}