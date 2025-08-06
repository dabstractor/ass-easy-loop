//! pEMF Timing Validation Integration Tests
//! 
//! This module implements comprehensive integration tests for pEMF timing validation
//! that measure pulse accuracy without interfering with normal operation.
//! 
//! Requirements: 9.1, 9.5 (timing deviation detection and reporting)

#![cfg(test)]

use ass_easy_loop::test_processor::{
    TestCommandProcessor, TestType, TestStatus, TestParameters, TestResult,
    TestMeasurements, TimingMeasurement, PemfTimingStatistics, PemfTimingParameters,
    TimingDeviation, TimingDeviationReport, TimingDeviationType, TestExecutionError
};
use ass_easy_loop::error_handling::SystemError;
use heapless::Vec;

/// Test constants for pEMF timing validation
const PEMF_TARGET_FREQUENCY_HZ: f32 = 2.0;
const PEMF_HIGH_DURATION_US: u32 = 2000;  // 2ms
const PEMF_LOW_DURATION_US: u32 = 498000; // 498ms
const PEMF_TOTAL_PERIOD_US: u32 = 500000; // 500ms
const DEFAULT_TOLERANCE_PERCENT: f32 = 1.0; // ±1%

/// Helper function to create a timing measurement
fn create_timing_measurement(
    execution_time_us: u32,
    expected_time_us: u32,
    timestamp_ms: u32,
) -> TimingMeasurement {
    TimingMeasurement {
        task_name: "pemf_pulse",
        execution_time_us,
        expected_time_us,
        timestamp_ms,
    }
}

/// Helper function to create test parameters for pEMF timing validation
fn create_pemf_test_parameters(duration_ms: u32, tolerance_percent: f32) -> TestParameters {
    TestCommandProcessor::create_pemf_timing_parameters(duration_ms, tolerance_percent)
        .expect("Failed to create pEMF test parameters")
}

#[test]
fn test_pemf_timing_validation_basic_functionality() {
    // Test basic pEMF timing validation functionality
    // Requirements: 9.1 (measure pulse accuracy without interfering)
    
    let mut processor = TestCommandProcessor::new();
    let timestamp = 1000;
    
    // Create test parameters for 5-second test with 1% tolerance
    let params = create_pemf_test_parameters(5000, 1.0);
    
    // Start pEMF timing validation test
    let test_id = processor.execute_pemf_timing_test(params, timestamp)
        .expect("Failed to start pEMF timing test");
    
    assert_eq_no_std!(test_id, 1);
    
    // Verify test is active
    let (test_type, status, id) = processor.get_active_test_info()
        .expect("No active test found");
    assert_eq_no_std!(test_type, TestType::PemfTimingValidation);
    assert_eq_no_std!(status, TestStatus::Running);
    assert_eq_no_std!(id, test_id);
    
    // Simulate perfect timing measurements
    for i in 0..10 {
        let measurement = create_timing_measurement(
            PEMF_TOTAL_PERIOD_US,
            PEMF_TOTAL_PERIOD_US,
            timestamp + (i * 500),
        );
        processor.update_pemf_timing_measurements(measurement)
            .expect("Failed to update timing measurements");
    }
    
    // Get timing statistics
    let stats = processor.get_pemf_timing_statistics()
        .expect("Failed to get timing statistics");
    
    assert_eq_no_std!(stats.total_measurements, 10);
    assert_eq_no_std!(stats.within_tolerance_count, 10);
    assert_no_std!(stats.timing_accuracy_percent > 99.0);
    assert_eq_no_std!(stats.error_count, 0);
    
    // Complete the test
    processor.update_active_test(timestamp + 5500)
        .expect("Failed to complete test");
    
    // Verify test completed successfully
    let (_, status, _) = processor.get_active_test_info()
        .expect("No active test found");
    assert_eq_no_std!(status, TestStatus::Completed);
}

#[test]
fn test_pemf_timing_deviation_detection() {
    // Test timing deviation detection and reporting
    // Requirements: 9.5 (timing deviation detection and reporting)
    
    let mut processor = TestCommandProcessor::new();
    let timestamp = 1000;
    
    // Create test parameters with strict tolerance
    let params = create_pemf_test_parameters(3000, 0.5);
    
    // Start test
    let _test_id = processor.execute_pemf_timing_test(params, timestamp)
        .expect("Failed to start pEMF timing test");
    
    // Add measurements with known deviations
    let measurements = [
        (PEMF_TOTAL_PERIOD_US, "perfect timing"),
        (PEMF_TOTAL_PERIOD_US + 1000, "1ms too slow"),
        (PEMF_TOTAL_PERIOD_US - 2000, "2ms too fast"),
        (PEMF_TOTAL_PERIOD_US + 5000, "5ms too slow - should be detected"),
        (PEMF_TOTAL_PERIOD_US, "perfect timing again"),
        (PEMF_TOTAL_PERIOD_US - 3000, "3ms too fast - should be detected"),
    ];
    
    for (i, (timing_us, description)) in measurements.iter().enumerate() {
        let measurement = create_timing_measurement(
            *timing_us,
            PEMF_TOTAL_PERIOD_US,
            timestamp + (i as u32 * 500),
        );
        processor.update_pemf_timing_measurements(measurement)
            .expect(&format!("Failed to update measurement: {}", description));
    }
    
    // Detect timing deviations
    let deviations = processor.detect_timing_deviations(0.5); // 0.5% tolerance
    
    // Should detect the 5ms and 3ms deviations (both > 0.5% of 500ms = 2.5ms)
    assert_no_std!(deviations.len() >= 2, "Should detect at least 2 timing deviations");
    
    // Check specific deviations
    let slow_deviation = deviations.iter()
        .find(|d| d.deviation_type == TimingDeviationType::TooSlow && d.deviation_us == 5000);
    assert_no_std!(slow_deviation.is_some(), "Should detect 5ms slow deviation");
    
    let fast_deviation = deviations.iter()
        .find(|d| d.deviation_type == TimingDeviationType::TooFast && d.deviation_us == 3000);
    assert_no_std!(fast_deviation.is_some(), "Should detect 3ms fast deviation");
    
    // Generate timing deviation report
    let report = processor.generate_timing_deviation_report()
        .expect("Failed to generate timing deviation report");
    
    assert_eq_no_std!(report.total_measurements, 6);
    assert_no_std!(report.total_deviations >= 2);
    assert_no_std!(report.max_deviation_us >= 5000);
    assert_no_std!(report.too_slow_count >= 1);
    assert_no_std!(report.too_fast_count >= 1);
    assert_eq_no_std!(report.tolerance_percent, 0.5);
}

#[test]
fn test_pemf_timing_configurable_parameters() {
    // Test configurable test duration and tolerance parameters
    // Requirements: 9.1 (configurable test duration and tolerance parameters)
    
    let mut processor = TestCommandProcessor::new();
    let timestamp = 1000;
    
    // Test different parameter configurations
    let test_configs = [
        (1000, 0.5),  // 1 second, 0.5% tolerance
        (10000, 1.0), // 10 seconds, 1% tolerance
        (30000, 2.0), // 30 seconds, 2% tolerance
    ];
    
    for (duration_ms, tolerance_percent) in test_configs.iter() {
        // Create parameters
        let params = create_pemf_test_parameters(*duration_ms, *tolerance_percent);
        
        // Verify parameters are configured correctly
        assert_eq_no_std!(params.duration_ms, *duration_ms);
        assert_eq_no_std!(params.tolerance_percent, *tolerance_percent);
        assert_eq_no_std!(params.sample_rate_hz, 2); // Should match pEMF frequency
        
        // Verify resource limits are set for non-intrusive testing
        assert_no_std!(params.resource_limits.max_cpu_usage_percent <= 10);
        assert_no_std!(params.resource_limits.max_memory_usage_bytes <= 2048);
        assert_no_std!(params.resource_limits.allow_preemption);
        
        // Verify validation criteria
        assert_no_std!(params.validation_criteria.min_success_rate_percent >= 95);
        assert_no_std!(params.validation_criteria.require_stable_operation);
        
        // Start and immediately stop test to verify configuration
        let test_id = processor.execute_pemf_timing_test(params, timestamp)
            .expect("Failed to start test with custom parameters");
        
        // Verify test started with correct parameters
        let (test_type, status, id) = processor.get_active_test_info()
            .expect("No active test found");
        assert_eq_no_std!(test_type, TestType::PemfTimingValidation);
        assert_eq_no_std!(status, TestStatus::Running);
        assert_eq_no_std!(id, test_id);
        
        // Complete the test
        processor.update_active_test(timestamp + duration_ms + 100)
            .expect("Failed to complete test");
        
        // Reset for next test
        processor = TestCommandProcessor::new();
    }
}

#[test]
fn test_pemf_timing_statistics_calculation() {
    // Test timing statistics calculation and error counting
    // Requirements: 9.5 (timing statistics and error counts)
    
    let mut processor = TestCommandProcessor::new();
    let timestamp = 1000;
    
    // Create test parameters
    let params = create_pemf_test_parameters(5000, 1.0);
    
    // Start test
    let _test_id = processor.execute_pemf_timing_test(params, timestamp)
        .expect("Failed to start pEMF timing test");
    
    // Add measurements with known characteristics
    let measurements = [
        // Perfect measurements (within tolerance)
        PEMF_TOTAL_PERIOD_US,      // 0% error
        PEMF_TOTAL_PERIOD_US + 1000, // 0.2% error
        PEMF_TOTAL_PERIOD_US - 2000, // 0.4% error
        PEMF_TOTAL_PERIOD_US + 3000, // 0.6% error
        PEMF_TOTAL_PERIOD_US - 4000, // 0.8% error
        
        // Measurements outside tolerance (> 1%)
        PEMF_TOTAL_PERIOD_US + 8000, // 1.6% error
        PEMF_TOTAL_PERIOD_US - 10000, // 2.0% error
        PEMF_TOTAL_PERIOD_US + 15000, // 3.0% error
    ];
    
    for (i, &timing_us) in measurements.iter().enumerate() {
        let measurement = create_timing_measurement(
            timing_us,
            PEMF_TOTAL_PERIOD_US,
            timestamp + (i as u32 * 500),
        );
        processor.update_pemf_timing_measurements(measurement)
            .expect("Failed to update timing measurement");
    }
    
    // Get timing statistics
    let stats = processor.get_pemf_timing_statistics()
        .expect("Failed to get timing statistics");
    
    // Verify statistics
    assert_eq_no_std!(stats.total_measurements, 8);
    assert_eq_no_std!(stats.within_tolerance_count, 5); // First 5 measurements within 1%
    assert_eq_no_std!(stats.error_count, 3); // Last 3 measurements outside tolerance
    
    // Verify timing accuracy calculation
    assert_no_std!(stats.timing_accuracy_percent > 0.0);
    assert_no_std!(stats.timing_accuracy_percent < 100.0);
    
    // Verify jitter measurement
    assert_no_std!(stats.max_jitter_us >= 15000); // Should capture the 15ms deviation
    
    // Verify average timing error
    assert_no_std!(stats.average_timing_error_percent > 0.0);
    assert_no_std!(stats.average_timing_error_percent < 5.0); // Should be reasonable average
}

#[test]
fn test_pemf_timing_non_interference() {
    // Test that pEMF timing validation doesn't interfere with normal operation
    // Requirements: 9.1 (measure pulse accuracy without interfering)
    
    let mut processor = TestCommandProcessor::new();
    let timestamp = 1000;
    
    // Create parameters with minimal resource usage
    let params = create_pemf_test_parameters(2000, 1.0);
    
    // Verify resource limits are set for non-interference
    assert_no_std!(params.resource_limits.max_cpu_usage_percent <= 10);
    assert_no_std!(params.resource_limits.max_memory_usage_bytes <= 2048);
    assert_no_std!(params.resource_limits.allow_preemption);
    
    // Start test
    let test_id = processor.execute_pemf_timing_test(params, timestamp)
        .expect("Failed to start pEMF timing test");
    
    // Simulate measurements during normal operation
    for i in 0..4 {
        let measurement = create_timing_measurement(
            PEMF_TOTAL_PERIOD_US + (i * 100), // Small variations
            PEMF_TOTAL_PERIOD_US,
            timestamp + (i * 500),
        );
        
        // Update measurements should not fail or interfere
        processor.update_pemf_timing_measurements(measurement)
            .expect("Timing measurement update should not interfere");
        
        // Update active test should not fail
        processor.update_active_test(timestamp + (i * 500) + 100)
            .expect("Active test update should not interfere");
    }
    
    // Verify test completed successfully without interference
    let stats = processor.get_pemf_timing_statistics()
        .expect("Should be able to get statistics");
    
    assert_eq_no_std!(stats.total_measurements, 4);
    assert_no_std!(stats.within_tolerance_count >= 3); // Most measurements should be good
}

#[test]
fn test_pemf_timing_concurrent_test_prevention() {
    // Test that only one pEMF timing test can run at a time
    // Requirements: 9.1 (prevent interference between tests)
    
    let mut processor = TestCommandProcessor::new();
    let timestamp = 1000;
    
    // Create test parameters
    let params = create_pemf_test_parameters(5000, 1.0);
    
    // Start first test
    let test_id1 = processor.execute_pemf_timing_test(params.clone(), timestamp)
        .expect("Failed to start first pEMF timing test");
    
    assert_eq_no_std!(test_id1, 1);
    
    // Try to start second test - should fail
    let result = processor.execute_pemf_timing_test(params, timestamp + 100);
    assert_no_std!(result.is_err());
    assert_eq_no_std!(result.unwrap_err(), TestExecutionError::TestAborted);
    
    // Verify first test is still active
    let (test_type, status, id) = processor.get_active_test_info()
        .expect("No active test found");
    assert_eq_no_std!(test_type, TestType::PemfTimingValidation);
    assert_eq_no_std!(status, TestStatus::Running);
    assert_eq_no_std!(id, test_id1);
    
    // Complete first test
    processor.update_active_test(timestamp + 5500)
        .expect("Failed to complete first test");
    
    // Now should be able to start second test
    let test_id2 = processor.execute_pemf_timing_test(
        create_pemf_test_parameters(3000, 1.0),
        timestamp + 6000
    ).expect("Failed to start second test after first completed");
    
    assert_eq_no_std!(test_id2, 2);
}

#[test]
fn test_pemf_timing_parameter_validation() {
    // Test parameter validation for pEMF timing tests
    // Requirements: 9.1 (configurable test duration and tolerance parameters)
    
    // Test valid parameters
    let valid_params = [
        (1000, 0.1),   // Minimum duration, minimum tolerance
        (5000, 1.0),   // Standard parameters
        (60000, 10.0), // Maximum duration, maximum tolerance
    ];
    
    for (duration_ms, tolerance_percent) in valid_params.iter() {
        let params = create_pemf_test_parameters(*duration_ms, *tolerance_percent);
        assert_no_std!(params.validate().is_ok(), 
                "Valid parameters should pass validation: {}ms, {}%", 
                duration_ms, tolerance_percent);
    }
    
    // Test invalid parameters - these should be caught by create_pemf_test_parameters
    let invalid_params = [
        (0, 1.0),      // Zero duration
        (100000, 1.0), // Duration too long
        (5000, 0.0),   // Zero tolerance
        (5000, 20.0),  // Tolerance too high
    ];
    
    for (duration_ms, tolerance_percent) in invalid_params.iter() {
        let result = TestCommandProcessor::create_pemf_timing_parameters(*duration_ms, *tolerance_percent);
        assert_no_std!(result.is_err(), 
                "Invalid parameters should fail validation: {}ms, {}%", 
                duration_ms, tolerance_percent);
    }
}

#[test]
fn test_pemf_timing_measurement_rejection() {
    // Test that timing measurements are rejected when no test is active
    // Requirements: 9.1 (proper test state management)
    
    let mut processor = TestCommandProcessor::new();
    
    // Create a timing measurement
    let measurement = create_timing_measurement(
        PEMF_TOTAL_PERIOD_US,
        PEMF_TOTAL_PERIOD_US,
        1000,
    );
    
    // Should reject measurement when no test is active
    let result = processor.update_pemf_timing_measurements(measurement);
    assert_no_std!(result.is_ok()); // Function doesn't error, just ignores
    
    // Should return None for statistics when no test is active
    let stats = processor.get_pemf_timing_statistics();
    assert_no_std!(stats.is_none());
    
    // Should return empty deviations when no test is active
    let deviations = processor.detect_timing_deviations(1.0);
    assert_no_std!(deviations.is_empty());
    
    // Should return None for deviation report when no test is active
    let report = processor.generate_timing_deviation_report();
    assert_no_std!(report.is_none());
}

#[test]
fn test_pemf_timing_result_serialization() {
    // Test that timing test results can be properly serialized
    // Requirements: 9.5 (test result structure with timing statistics)
    
    let mut processor = TestCommandProcessor::new();
    let timestamp = 1000;
    
    // Create and start test
    let params = create_pemf_test_parameters(2000, 1.0);
    let test_id = processor.execute_pemf_timing_test(params, timestamp)
        .expect("Failed to start pEMF timing test");
    
    // Add some measurements
    for i in 0..5 {
        let measurement = create_timing_measurement(
            PEMF_TOTAL_PERIOD_US + (i * 1000),
            PEMF_TOTAL_PERIOD_US,
            timestamp + (i * 500),
        );
        processor.update_pemf_timing_measurements(measurement)
            .expect("Failed to update timing measurement");
    }
    
    // Complete the test
    processor.update_active_test(timestamp + 2500)
        .expect("Failed to complete test");
    
    // Get test result
    let result = processor.get_test_result(test_id)
        .expect("Failed to get test result");
    
    // Verify result structure
    assert_eq_no_std!(result.test_type, TestType::PemfTimingValidation);
    assert_eq_no_std!(result.status, TestStatus::Completed);
    assert_eq_no_std!(result.test_id, test_id);
    assert_no_std!(result.duration_ms() > 0);
    
    // Verify measurements are included
    assert_eq_no_std!(result.measurements.timing_measurements.len(), 5);
    assert_no_std!(result.measurements.timing_accuracy > 0.0);
    
    // Test serialization to command response
    let response = result.serialize_to_response(0x42)
        .expect("Failed to serialize test result");
    
    assert_eq_no_std!(response.command_id, 0x42);
    assert_no_std!(!response.payload.is_empty());
    
    // Verify payload contains test type and status
    assert_eq_no_std!(response.payload[0], TestType::PemfTimingValidation as u8);
    assert_eq_no_std!(response.payload[1], TestStatus::Completed as u8);
    assert_eq_no_std!(response.payload[2], test_id);
}

#[test]
fn test_pemf_timing_parameters_serialization() {
    // Test pEMF timing parameters serialization and parsing
    // Requirements: 9.1 (configurable test duration and tolerance parameters)
    
    // Create default parameters
    let default_params = PemfTimingParameters::default();
    
    // Verify default values
    assert_eq_no_std!(default_params.expected_frequency_mhz, 2000); // 2Hz = 2000 mHz
    assert_eq_no_std!(default_params.expected_high_duration_us, 2000); // 2ms
    assert_eq_no_std!(default_params.expected_low_duration_us, 498000); // 498ms
    assert_eq_no_std!(default_params.expected_total_period_us, 500000); // 500ms
    
    // Test serialization
    let serialized = default_params.serialize();
    assert_eq_no_std!(serialized.len(), 10); // 2 + 4 + 4 bytes
    
    // Test parsing
    let parsed = TestCommandProcessor::parse_pemf_timing_parameters(&serialized)
        .expect("Failed to parse pEMF timing parameters");
    
    assert_eq_no_std!(parsed.expected_frequency_mhz, default_params.expected_frequency_mhz);
    assert_eq_no_std!(parsed.expected_high_duration_us, default_params.expected_high_duration_us);
    assert_eq_no_std!(parsed.expected_low_duration_us, default_params.expected_low_duration_us);
    assert_eq_no_std!(parsed.expected_total_period_us, default_params.expected_total_period_us);
    
    // Test validation
    assert_no_std!(parsed.validate().is_ok());
    
    // Test custom frequency parameters
    let custom_params = PemfTimingParameters::from_frequency_hz(1.0); // 1Hz
    assert_eq_no_std!(custom_params.expected_frequency_mhz, 1000); // 1Hz = 1000 mHz
    assert_eq_no_std!(custom_params.expected_high_duration_us, 2000); // Fixed 2ms HIGH
    assert_eq_no_std!(custom_params.expected_low_duration_us, 998000); // 998ms LOW
    assert_eq_no_std!(custom_params.expected_total_period_us, 1000000); // 1000ms total
    
    assert_no_std!(custom_params.validate().is_ok());
}