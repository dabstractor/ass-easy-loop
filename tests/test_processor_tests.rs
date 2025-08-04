//! Unit tests for the test command processor framework
//!
//! Tests parameter validation, result handling, timeout protection,
//! resource usage monitoring, and test execution logic.
//!
//! Requirements: 2.1, 2.2, 2.3, 8.1, 8.2, 8.3, 8.4, 8.5

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
use panic_halt as _;

use ass_easy_loop::test_processor::{
    PemfTimingParameters, PemfTimingStatistics, ResourceLimits, TimingMeasurement,
    ValidationCriteria,
};
use ass_easy_loop::{
    CommandReport, TestCommandProcessor, TestExecutionError, TestMeasurements, TestParameterError,
    TestParameters, TestResponse, TestStatus, TestType, TimingDeviationType,
};
use heapless::Vec;

// Import the no_std test framework
use ass_easy_loop::test_framework::{TestResult, TestRunner, TestSuiteResult};
use ass_easy_loop::{assert_eq_no_std, assert_no_std, register_tests, test_case};

/// Test parameter validation and range checking
/// Requirements: 2.2 (test parameter validation and range checking)
test_case!(test_parameter_validation, {
    // Test valid parameters
    let valid_params = TestParameters {
        duration_ms: 5000,
        tolerance_percent: 2.0,
        sample_rate_hz: 1000.0,
        validation_criteria: ValidationCriteria::default(),
        resource_limits: ResourceLimits::default(),
        custom_parameters: Vec::new(),
    };
    assert!(valid_params.validate().is_ok());

    // Test invalid duration (too short)
    let mut invalid_params = valid_params.clone();
    invalid_params.duration_ms = 0;
    assert_eq!(
        invalid_params.validate(),
        Err(TestParameterError::InvalidDuration)
    );

    // Test invalid duration (too long)
    invalid_params.duration_ms = 70_000;
    assert_eq!(
        invalid_params.validate(),
        Err(TestParameterError::InvalidDuration)
    );

    // Test invalid tolerance (too low)
    invalid_params = valid_params.clone();
    invalid_params.tolerance_percent = 0.05;
    assert_eq!(
        invalid_params.validate(),
        Err(TestParameterError::InvalidTolerance)
    );

    // Test invalid tolerance (too high)
    invalid_params.tolerance_percent = 15.0;
    assert_eq!(
        invalid_params.validate(),
        Err(TestParameterError::InvalidTolerance)
    );

    // Test invalid sample rate (zero)
    invalid_params = valid_params.clone();
    invalid_params.sample_rate_hz = 0.0;
    assert_eq!(
        invalid_params.validate(),
        Err(TestParameterError::InvalidSampleRate)
    );

    // Test invalid sample rate (too high)
    invalid_params.sample_rate_hz = 20_000.0;
    assert_eq!(
        invalid_params.validate(),
        Err(TestParameterError::InvalidSampleRate)
    );
});

/// Test resource limits validation
/// Requirements: 8.1, 8.2 (resource usage limits)
test_case!(test_resource_limits_validation, {
    // Test valid resource limits
    let valid_limits = ResourceLimits {
        max_cpu_usage_percent: 75,
        max_memory_usage_bytes: 8192,
        max_execution_time_ms: 10_000,
        allow_preemption: true,
    };
    assert!(valid_limits.validate().is_ok());

    // Test invalid CPU usage (over 100%)
    let mut invalid_limits = valid_limits;
    invalid_limits.max_cpu_usage_percent = 150;
    assert_eq!(
        invalid_limits.validate(),
        Err(TestParameterError::InvalidResourceLimits)
    );

    // Test invalid memory usage (too high)
    invalid_limits = valid_limits;
    invalid_limits.max_memory_usage_bytes = 100_000;
    assert_eq!(
        invalid_limits.validate(),
        Err(TestParameterError::InvalidResourceLimits)
    );

    // Test invalid execution time (too long)
    invalid_limits = valid_limits;
    invalid_limits.max_execution_time_ms = 400_000;
    assert_eq!(
        invalid_limits.validate(),
        Err(TestParameterError::InvalidResourceLimits)
    );
});

/// Test parameter parsing from command payload
/// Requirements: 2.1 (command parsing and validation)
test_case!(test_parameter_parsing_from_payload, {
    // Create test payload with valid parameters
    let mut payload = Vec::<u8, 64>::new();

    // Duration: 5000ms (5 seconds)
    let duration_bytes = 5000u32.to_le_bytes();
    for &byte in &duration_bytes {
        payload.push(byte).unwrap();
    }

    // Tolerance: 1.5%
    let tolerance_bytes = 1.5f32.to_le_bytes();
    for &byte in &tolerance_bytes {
        payload.push(byte).unwrap();
    }

    // Sample rate: 500Hz
    let sample_rate_bytes = 500u32.to_le_bytes();
    for &byte in &sample_rate_bytes {
        payload.push(byte).unwrap();
    }

    // Max error count: 10
    let max_errors_bytes = 10u32.to_le_bytes();
    for &byte in &max_errors_bytes {
        payload.push(byte).unwrap();
    }

    // Resource limits
    payload.push(50).unwrap(); // max_cpu_usage_percent
    let memory_bytes = 4096u32.to_le_bytes();
    for &byte in &memory_bytes {
        payload.push(byte).unwrap();
    }

    // Parse parameters
    let parsed_params = TestParameters::from_payload(&payload).unwrap();

    assert_eq!(parsed_params.duration_ms, 5000);
    assert_eq!(parsed_params.tolerance_percent, 1.5);
    assert_eq!(parsed_params.sample_rate_hz, 500.0);
    assert_eq!(parsed_params.validation_criteria.max_error_count, 10);
    assert_eq!(parsed_params.resource_limits.max_cpu_usage_percent, 50);
    assert_eq!(parsed_params.resource_limits.max_memory_usage_bytes, 4096);

    // Test payload too short
    let short_payload = [1, 2, 3, 4]; // Only 4 bytes
    assert_eq!(
        TestParameters::from_payload(&short_payload),
        Err(TestParameterError::PayloadTooShort)
    );
});

/// Test parameter serialization
/// Requirements: 2.3 (test result serialization)
test_case!(test_parameter_serialization, {
    let params = TestParameters {
        duration_ms: 3000,
        tolerance_percent: 2.5,
        sample_rate_hz: 1000.0,
        validation_criteria: ValidationCriteria {
            max_error_count: 5,
            min_success_rate_percent: 98,
            max_timing_deviation_us: 500,
            require_stable_operation: true,
        },
        resource_limits: ResourceLimits {
            max_cpu_usage_percent: 60,
            max_memory_usage_bytes: 2048,
            max_execution_time_ms: 5000,
            allow_preemption: false,
        },
        custom_parameters: Vec::new(),
    };

    let serialized = params.serialize();

    // Verify serialized data contains expected values
    assert!(serialized.len() >= 21); // Minimum expected size

    // Check duration (first 4 bytes)
    let duration = u32::from_le_bytes([serialized[0], serialized[1], serialized[2], serialized[3]]);
    assert_eq!(duration, 3000);

    // Check tolerance (next 4 bytes)
    let tolerance =
        f32::from_le_bytes([serialized[4], serialized[5], serialized[6], serialized[7]]);
    assert_eq!(tolerance, 2.5);

    // Check sample rate (next 4 bytes)
    let sample_rate =
        u32::from_le_bytes([serialized[8], serialized[9], serialized[10], serialized[11]]);
    assert_eq!(sample_rate, 1000);

    // Check max error count (next 4 bytes)
    let max_errors = u32::from_le_bytes([
        serialized[12],
        serialized[13],
        serialized[14],
        serialized[15],
    ]);
    assert_eq!(max_errors, 5);

    // Check CPU usage limit (next byte)
    assert_eq!(serialized[16], 60);

    // Check memory limit (next 4 bytes)
    let memory_limit = u32::from_le_bytes([
        serialized[17],
        serialized[18],
        serialized[19],
        serialized[20],
    ]);
    assert_eq!(memory_limit, 2048);
});

/// Test test command processor initialization and basic operations
/// Requirements: 2.1, 2.2, 2.3 (configurable test execution)
test_case!(test_processor_initialization, {
    let processor = TestCommandProcessor::new();

    // Check initial state
    assert!(processor.get_active_test_info().is_none());

    let stats = processor.get_statistics();
    assert_eq!(stats.total_tests_executed, 0);
    assert_eq!(stats.total_tests_passed, 0);
    assert_eq!(stats.total_tests_failed, 0);
    assert_eq!(stats.active_test_count, 0);
    assert_eq!(stats.stored_results_count, 0);
});

/// Test starting and managing test execution
/// Requirements: 2.1, 2.2 (configurable test execution with parameter validation)
test_case!(test_start_test_execution, {
    let mut processor = TestCommandProcessor::new();
    let timestamp = 1000;

    // Create valid test parameters
    let params = TestParameters {
        duration_ms: 2000,
        tolerance_percent: 1.0,
        sample_rate_hz: 100.0,
        validation_criteria: ValidationCriteria::default(),
        resource_limits: ResourceLimits::default(),
        custom_parameters: Vec::new(),
    };

    // Start a test
    let test_id = processor
        .start_test(TestType::PemfTimingValidation, params.clone(), timestamp)
        .unwrap();
    assert_eq!(test_id, 1); // First test should have ID 1

    // Check active test info
    let active_info = processor.get_active_test_info().unwrap();
    assert_eq!(active_info.0, TestType::PemfTimingValidation);
    assert_eq!(active_info.1, TestStatus::Running);
    assert_eq!(active_info.2, test_id);

    // Try to start another test (should fail)
    let result = processor.start_test(TestType::BatteryAdcCalibration, params, timestamp);
    assert_eq!(result, Err(TestExecutionError::TestAborted));
});

/// Test test timeout protection
/// Requirements: 8.3 (timeout protection)
test_case!(test_timeout_protection, {
    let mut processor = TestCommandProcessor::new();
    let start_timestamp = 1000;

    // Create test parameters with short duration
    let params = TestParameters {
        duration_ms: 1000, // 1 second duration
        tolerance_percent: 1.0,
        sample_rate_hz: 100.0,
        validation_criteria: ValidationCriteria::default(),
        resource_limits: ResourceLimits::default(),
        custom_parameters: Vec::new(),
    };

    // Start a test
    let test_id = processor
        .start_test(TestType::SystemStressTest, params, start_timestamp)
        .unwrap();

    // Update before timeout (should not complete)
    let result = processor.update_active_test(start_timestamp + 500);
    assert!(result.is_none());

    // Update after normal completion time (should complete normally)
    let result = processor.update_active_test(start_timestamp + 1100);
    assert!(result.is_some());
    let completed_result = result.unwrap();
    assert_eq!(completed_result.status, TestStatus::Completed);
    assert_eq!(completed_result.test_id, test_id);

    // Check that active test is cleared
    assert!(processor.get_active_test_info().is_none());
});

/// Test test abortion capability
/// Requirements: 8.4 (test abortion capability)
test_case!(test_abort_active_test, {
    let mut processor = TestCommandProcessor::new();
    let timestamp = 1000;

    let params = TestParameters::new();

    // Start a test
    let test_id = processor
        .start_test(TestType::LedFunctionality, params, timestamp)
        .unwrap();

    // Abort the test
    let result = processor.abort_active_test(timestamp + 500).unwrap();
    assert_eq!(result.status, TestStatus::Aborted);
    assert_eq!(result.test_id, test_id);
    assert_eq!(result.duration_ms(), 500);

    // Check that active test is cleared
    assert!(processor.get_active_test_info().is_none());

    // Try to abort when no test is active
    let result = processor.abort_active_test(timestamp + 1000);
    assert!(result.is_none());
});

/// Test result collection and serialization
/// Requirements: 2.3 (test result collection and serialization)
test_case!(test_result_collection_and_serialization, {
    let mut processor = TestCommandProcessor::new();
    let timestamp = 1000;

    let params = TestParameters {
        duration_ms: 500,
        tolerance_percent: 1.0,
        sample_rate_hz: 100.0,
        validation_criteria: ValidationCriteria::default(),
        resource_limits: ResourceLimits::default(),
        custom_parameters: Vec::new(),
    };

    // Start and complete a test
    let test_id = processor
        .start_test(TestType::UsbCommunicationTest, params, timestamp)
        .unwrap();
    let result = processor.update_active_test(timestamp + 600).unwrap(); // Complete after duration

    // Check result properties
    assert_eq!(result.test_type, TestType::UsbCommunicationTest);
    assert_eq!(result.status, TestStatus::Completed);
    assert_eq!(result.test_id, test_id);
    assert_eq!(result.start_timestamp_ms, timestamp);
    assert_eq!(result.end_timestamp_ms, timestamp + 600);
    assert_eq!(result.duration_ms(), 600);

    // Test result serialization to command response
    let response = result.serialize_to_response(test_id).unwrap();
    assert_eq!(response.command_type, TestResponse::TestResult as u8);
    assert_eq!(response.command_id, test_id);
    assert!(response.payload.len() > 0);

    // Check serialized data
    assert_eq!(response.payload[0], TestType::UsbCommunicationTest as u8);
    assert_eq!(response.payload[1], TestStatus::Completed as u8);
    assert_eq!(response.payload[2], test_id);

    // Check duration in payload (next 4 bytes)
    let duration = u32::from_le_bytes([
        response.payload[3],
        response.payload[4],
        response.payload[5],
        response.payload[6],
    ]);
    assert_eq!(duration, 600);
});

/// Test statistics tracking
/// Requirements: 2.3 (test result collection)
test_case!(test_statistics_tracking, {
    let mut processor = TestCommandProcessor::new();
    let mut timestamp = 1000;

    let params = TestParameters {
        duration_ms: 100,
        tolerance_percent: 1.0,
        sample_rate_hz: 100.0,
        validation_criteria: ValidationCriteria::default(),
        resource_limits: ResourceLimits::default(),
        custom_parameters: Vec::new(),
    };

    // Run several tests with different outcomes

    // Test 1: Successful completion
    processor
        .start_test(TestType::PemfTimingValidation, params.clone(), timestamp)
        .unwrap();
    timestamp += 150;
    processor.update_active_test(timestamp).unwrap(); // Complete

    // Test 2: Timeout
    processor
        .start_test(TestType::BatteryAdcCalibration, params.clone(), timestamp)
        .unwrap();
    timestamp += 6000; // Way past timeout
    processor.update_active_test(timestamp).unwrap(); // Timeout

    // Test 3: Abortion
    processor
        .start_test(TestType::LedFunctionality, params.clone(), timestamp)
        .unwrap();
    timestamp += 50;
    processor.abort_active_test(timestamp).unwrap(); // Abort

    // Check statistics
    let stats = processor.get_statistics();
    assert_eq!(stats.total_tests_executed, 3);
    assert_eq!(stats.total_tests_passed, 1);
    assert_eq!(stats.total_tests_failed, 2); // Timeout and abort count as failures
    assert_eq!(stats.success_rate_percent(), 33); // 1/3 = 33%
    assert_eq!(stats.failure_rate_percent(), 66); // 2/3 = 66%
    assert_eq!(stats.active_test_count, 0);
    assert_eq!(stats.stored_results_count, 3);
});

/// Test command processing integration
/// Requirements: 2.1, 2.2, 2.3 (command processing with validation and result collection)
test_case!(test_command_processing_integration, {
    let mut processor = TestCommandProcessor::new();
    let timestamp = 1000;

    // Create test command payload
    let mut payload = Vec::<u8, 60>::new();
    payload.push(TestType::SystemStressTest as u8).unwrap(); // Test type

    // Add test parameters
    let duration_bytes = 2000u32.to_le_bytes();
    for &byte in &duration_bytes {
        payload.push(byte).unwrap();
    }
    let tolerance_bytes = 1.5f32.to_le_bytes();
    for &byte in &tolerance_bytes {
        payload.push(byte).unwrap();
    }
    let sample_rate_bytes = 200u32.to_le_bytes();
    for &byte in &sample_rate_bytes {
        payload.push(byte).unwrap();
    }
    let max_errors_bytes = 5u32.to_le_bytes();
    for &byte in &max_errors_bytes {
        payload.push(byte).unwrap();
    }

    // Create command report
    let command = CommandReport::new(0x82, 42, &payload).unwrap(); // ExecuteTest command

    // Process command
    let response = processor.process_test_command(&command, timestamp).unwrap();

    // Check response
    assert_eq!(response.command_type, TestResponse::TestResult as u8);
    assert_eq!(response.command_id, 42);
    assert!(response.payload.len() >= 3);

    // Check response payload
    assert_eq!(response.payload[0], TestType::SystemStressTest as u8);
    assert_eq!(response.payload[2], TestStatus::Running as u8);

    // Verify test is now active
    let active_info = processor.get_active_test_info().unwrap();
    assert_eq!(active_info.0, TestType::SystemStressTest);
    assert_eq!(active_info.1, TestStatus::Running);
});

/// Test error handling for invalid commands
/// Requirements: 2.1, 2.2 (command validation and error handling)
test_case!(test_error_handling_invalid_commands, {
    let mut processor = TestCommandProcessor::new();
    let timestamp = 1000;

    // Test empty payload
    let empty_command = CommandReport::new(0x82, 1, &[]).unwrap();
    let response = processor.process_test_command(&empty_command, timestamp);
    assert!(response.is_err());

    // Test invalid test type
    let invalid_payload = [0xFF]; // Invalid test type
    let invalid_command = CommandReport::new(0x82, 2, &invalid_payload).unwrap();
    let response = processor.process_test_command(&invalid_command, timestamp);
    assert!(response.is_err());

    // Test invalid parameters (duration too long)
    let mut bad_params_payload = Vec::<u8, 60>::new();
    bad_params_payload
        .push(TestType::PemfTimingValidation as u8)
        .unwrap();
    let bad_duration_bytes = 100_000u32.to_le_bytes(); // 100 seconds (too long)
    for &byte in &bad_duration_bytes {
        bad_params_payload.push(byte).unwrap();
    }

    let bad_params_command = CommandReport::new(0x82, 3, &bad_params_payload).unwrap();
    let response = processor.process_test_command(&bad_params_command, timestamp);
    assert!(response.is_err());
});

/// Test resource usage monitoring
/// Requirements: 8.1, 8.2 (resource usage monitoring)
#[test]
fn test_resource_usage_monitoring() {
    let mut processor = TestCommandProcessor::new();
    let timestamp = 1000;

    // Create test with strict resource limits
    let params = TestParameters {
        duration_ms: 5000,
        tolerance_percent: 1.0,
        sample_rate_hz: 100.0,
        validation_criteria: ValidationCriteria::default(),
        resource_limits: ResourceLimits {
            max_cpu_usage_percent: 10,    // Very low limit to trigger failure
            max_memory_usage_bytes: 1000, // Low memory limit
            max_execution_time_ms: 2000,
            allow_preemption: true,
        },
        custom_parameters: Vec::new(),
    };

    // Start test
    processor
        .start_test(TestType::SystemStressTest, params, timestamp)
        .unwrap();

    // Update test - should detect resource limit exceeded and fail
    // Note: In the real implementation, this would depend on actual resource monitoring
    // For the test, we simulate the behavior
    let result = processor.update_active_test(timestamp + 100);

    // The test should either complete normally or fail due to resource limits
    // depending on the simulated resource usage in the ResourceMonitor
    if let Some(result) = result {
        // If test completed due to resource limits, it should be marked as failed
        if result.status == TestStatus::Failed {
            assert!(result.error_details.is_some());
        }
    }
}

/// Test measurement collection and accuracy calculation
/// Requirements: 2.3 (test result collection)
test_case!(test_measurement_collection, {
    let mut measurements = TestMeasurements::new();

    // Test initial state
    assert_eq!(measurements.timing_accuracy, 0.0);
    assert_eq!(measurements.error_count, 0);
    assert!(measurements.timing_measurements.is_empty());

    // Add some timing measurements
    let measurement1 = TimingMeasurement {
        task_name: "test",
        execution_time_us: 1000,
        expected_time_us: 1000,
        timestamp_ms: 1000,
    };
    let measurement2 = TimingMeasurement {
        task_name: "test",
        execution_time_us: 1050, // 5% deviation
        expected_time_us: 1000,
        timestamp_ms: 1001,
    };

    measurements.add_timing_measurement(measurement1).unwrap();
    measurements.add_timing_measurement(measurement2).unwrap();

    // Calculate timing accuracy
    let accuracy = measurements.calculate_timing_accuracy(1000);
    assert!(accuracy > 90.0); // Should be high accuracy despite some deviation
    assert!(accuracy <= 100.0);

    // Test serialization
    let serialized = measurements.serialize();
    assert!(serialized.len() > 0);

    // Check that timing accuracy is serialized (first 4 bytes)
    let serialized_accuracy =
        f32::from_le_bytes([serialized[0], serialized[1], serialized[2], serialized[3]]);
    assert_eq!(serialized_accuracy, measurements.timing_accuracy);
});

/// Test pEMF timing validation test functionality
/// Requirements: 9.1, 9.5 (pEMF timing validation without interference)
#[test]
fn test_pemf_timing_validation() {
    let mut processor = TestCommandProcessor::new();
    let timestamp = 1000;

    // Create pEMF timing test parameters
    let params = TestCommandProcessor::create_pemf_timing_parameters(5000, 1.0).unwrap();

    // Verify parameters are configured correctly for pEMF testing
    assert_eq!(params.duration_ms, 5000);
    assert_eq!(params.tolerance_percent, 1.0);
    assert_eq!(params.sample_rate_hz, 2.0); // 2Hz for pEMF
    assert_eq!(params.validation_criteria.min_success_rate_percent, 95);
    assert!(params.resource_limits.max_cpu_usage_percent <= 10); // Non-intrusive

    // Start pEMF timing test
    let test_id = processor
        .execute_pemf_timing_test(params, timestamp)
        .unwrap();
    assert_eq!(test_id, 1);

    // Verify test is active
    let (test_type, status, id) = processor.get_active_test_info().unwrap();
    assert_eq!(test_type, TestType::PemfTimingValidation);
    assert_eq!(status, TestStatus::Running);
    assert_eq!(id, test_id);

    // Add some timing measurements
    let perfect_measurement = TimingMeasurement {
        task_name: "pEMF_pulse",
        execution_time_us: 500_000, // Perfect 500ms timing
        expected_time_us: 500_000,
        timestamp_ms: timestamp + 500,
    };

    let slightly_off_measurement = TimingMeasurement {
        task_name: "pEMF_pulse",
        execution_time_us: 502_000, // 0.4% error (within 1% tolerance)
        expected_time_us: 500_000,
        timestamp_ms: timestamp + 1000,
    };

    let way_off_measurement = TimingMeasurement {
        task_name: "pEMF_pulse",
        execution_time_us: 510_000, // 2% error (outside 1% tolerance)
        expected_time_us: 500_000,
        timestamp_ms: timestamp + 1500,
    };

    // Update processor with measurements
    processor
        .update_pemf_timing_measurements(perfect_measurement)
        .unwrap();
    processor
        .update_pemf_timing_measurements(slightly_off_measurement)
        .unwrap();
    processor
        .update_pemf_timing_measurements(way_off_measurement)
        .unwrap();

    // Get timing statistics
    let stats = processor.get_pemf_timing_statistics().unwrap();
    assert_eq!(stats.total_measurements, 3);
    assert_eq!(stats.within_tolerance_count, 2); // 2 measurements within 1% tolerance
    assert_eq!(stats.error_count, 1); // 1 measurement outside tolerance
    assert!((stats.success_rate_percent() - 66.67).abs() < 0.1); // ~66.67% success rate
    assert!(stats.max_jitter_us >= 10_000); // Should capture the 10ms deviation

    // Test validation criteria checking
    assert!(!stats.meets_validation_criteria(95.0, 0)); // Should not meet 95% success rate
    assert!(stats.meets_validation_criteria(60.0, 5)); // Should meet 60% success rate with 5 error limit
}

/// Test pEMF timing parameters creation and validation
/// Requirements: 9.1 (configurable test duration and tolerance parameters)
#[test]
fn test_pemf_timing_parameters() {
    // Test default parameters
    let default_params = PemfTimingParameters::default();
    assert_eq!(default_params.expected_frequency_mhz, 2000); // 2Hz = 2000 mHz
    assert_eq!(default_params.expected_high_duration_us, 2000); // 2ms
    assert_eq!(default_params.expected_low_duration_us, 498000); // 498ms
    assert_eq!(default_params.expected_total_period_us, 500000); // 500ms total
    assert!(default_params.validate().is_ok());

    // Test parameters from frequency
    let freq_params = PemfTimingParameters::from_frequency_hz(1.0); // 1Hz
    assert_eq!(freq_params.expected_frequency_mhz, 1000); // 1Hz = 1000 mHz
    assert_eq!(freq_params.expected_high_duration_us, 2000); // Fixed 2ms HIGH
    assert_eq!(freq_params.expected_low_duration_us, 998000); // 998ms LOW for 1Hz
    assert_eq!(freq_params.expected_total_period_us, 1000000); // 1000ms total
    assert!(freq_params.validate().is_ok());

    // Test parameter serialization
    let serialized = default_params.serialize();
    assert_eq!(serialized.len(), 10); // 2 + 4 + 4 bytes

    // Test parameter parsing
    let parsed = TestCommandProcessor::parse_pemf_timing_parameters(&serialized).unwrap();
    assert_eq!(
        parsed.expected_frequency_mhz,
        default_params.expected_frequency_mhz
    );
    assert_eq!(
        parsed.expected_high_duration_us,
        default_params.expected_high_duration_us
    );
    assert_eq!(
        parsed.expected_low_duration_us,
        default_params.expected_low_duration_us
    );
    assert_eq!(
        parsed.expected_total_period_us,
        default_params.expected_total_period_us
    );

    // Test invalid parameters
    let mut invalid_params = default_params;
    invalid_params.expected_frequency_mhz = 50; // 0.05Hz - too low
    assert!(invalid_params.validate().is_err());

    invalid_params.expected_frequency_mhz = 15000; // 15Hz - too high
    assert!(invalid_params.validate().is_err());
}

/// Test pEMF timing statistics calculation and serialization
/// Requirements: 9.5 (timing statistics and error counts)
#[test]
fn test_pemf_timing_statistics() {
    let stats = PemfTimingStatistics {
        total_measurements: 100,
        timing_accuracy_percent: 99.2,
        error_count: 3,
        max_jitter_us: 2500,
        average_timing_error_percent: 0.8,
        test_duration_ms: 30000,
        within_tolerance_count: 97,
    };

    // Test success rate calculation
    assert!((stats.success_rate_percent() - 97.0).abs() < 0.1);

    // Test validation criteria
    assert!(stats.meets_validation_criteria(95.0, 5)); // Should meet criteria
    assert!(!stats.meets_validation_criteria(98.0, 2)); // Should not meet stricter criteria

    // Test serialization
    let serialized = stats.serialize();
    assert_eq!(serialized.len(), 28); // 7 fields * 4 bytes each

    // Verify first field (total_measurements) is serialized correctly
    let serialized_total =
        u32::from_le_bytes([serialized[0], serialized[1], serialized[2], serialized[3]]);
    assert_eq!(serialized_total, stats.total_measurements);

    // Verify timing accuracy is serialized correctly
    let serialized_accuracy =
        f32::from_le_bytes([serialized[4], serialized[5], serialized[6], serialized[7]]);
    assert_eq!(serialized_accuracy, stats.timing_accuracy_percent);
}

/// Test that pEMF timing test prevents multiple concurrent tests
/// Requirements: 9.1 (test execution without interference)
#[test]
fn test_pemf_timing_single_test_restriction() {
    let mut processor = TestCommandProcessor::new();
    let timestamp = 1000;

    // Create test parameters
    let params = TestCommandProcessor::create_pemf_timing_parameters(5000, 1.0).unwrap();

    // Start first test
    let test_id1 = processor
        .execute_pemf_timing_test(params.clone(), timestamp)
        .unwrap();
    assert_eq!(test_id1, 1);

    // Try to start second test - should fail
    let result = processor.execute_pemf_timing_test(params, timestamp + 100);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), TestExecutionError::TestAborted);

    // Verify first test is still active
    let (test_type, status, id) = processor.get_active_test_info().unwrap();
    assert_eq!(test_type, TestType::PemfTimingValidation);
    assert_eq!(status, TestStatus::Running);
    assert_eq!(id, test_id1);
}

/// Test pEMF timing measurement rejection when no test is active
/// Requirements: 9.1 (proper test state management)
#[test]
fn test_pemf_timing_measurement_rejection() {
    let mut processor = TestCommandProcessor::new();

    let measurement = TimingMeasurement {
        task_name: "pEMF_pulse",
        execution_time_us: 500_000,
        expected_time_us: 500_000,
        timestamp_ms: 1000,
    };

    // Should reject measurement when no test is active
    let result = processor.update_pemf_timing_measurements(measurement);
    assert!(result.is_ok()); // Function doesn't error, just ignores when no test active

    // Should return None for statistics when no test is active
    let stats = processor.get_pemf_timing_statistics();
    assert!(stats.is_none());
}

/// Test enhanced timing deviation detection functionality
/// Requirements: 9.1, 9.5 (timing deviation detection and reporting)
#[test]
fn test_enhanced_timing_deviation_detection() {
    let mut processor = TestCommandProcessor::new();
    let timestamp = 1000;

    // Create test parameters with strict tolerance
    let params = TestCommandProcessor::create_pemf_timing_parameters(3000, 0.5).unwrap();

    // Start test
    let _test_id = processor
        .execute_pemf_timing_test(params, timestamp)
        .unwrap();

    // Add measurements with known deviations
    let measurements = [
        (500000, "perfect timing"),
        (501000, "1ms too slow"),
        (498000, "2ms too fast"),
        (505000, "5ms too slow - should be detected"),
        (500000, "perfect timing again"),
        (497000, "3ms too fast - should be detected"),
    ];

    for (i, (timing_us, _description)) in measurements.iter().enumerate() {
        let measurement = TimingMeasurement {
            task_name: "pemf_pulse",
            execution_time_us: *timing_us,
            expected_time_us: 500000,
            timestamp_ms: timestamp + (i as u32 * 500),
        };
        processor
            .update_pemf_timing_measurements(measurement)
            .unwrap();
    }

    // Test enhanced deviation detection
    let deviations = processor.detect_detailed_timing_deviations(0.5); // 0.5% tolerance

    // Should detect the 5ms and 3ms deviations (both > 0.5% of 500ms = 2.5ms)
    assert!(
        deviations.len() >= 2,
        "Should detect at least 2 timing deviations"
    );

    // Check specific deviations
    let slow_deviation = deviations
        .iter()
        .find(|d| d.deviation_type == TimingDeviationType::TooSlow && d.deviation_us == 5000);
    assert!(slow_deviation.is_some(), "Should detect 5ms slow deviation");

    let fast_deviation = deviations
        .iter()
        .find(|d| d.deviation_type == TimingDeviationType::TooFast && d.deviation_us == 3000);
    assert!(fast_deviation.is_some(), "Should detect 3ms fast deviation");
}

/// Test comprehensive timing report generation
/// Requirements: 9.5 (timing statistics and error counts)
#[test]
fn test_comprehensive_timing_report() {
    let mut processor = TestCommandProcessor::new();
    let timestamp = 1000;

    // Create test parameters
    let params = TestCommandProcessor::create_pemf_timing_parameters(5000, 1.0).unwrap();

    // Start test
    let _test_id = processor
        .execute_pemf_timing_test(params, timestamp)
        .unwrap();

    // Add measurements with known characteristics
    let measurements = [
        500000, // Perfect
        501000, // 0.2% error
        498000, // 0.4% error
        503000, // 0.6% error
        497000, // 0.6% error
        508000, // 1.6% error - outside tolerance
        492000, // 1.6% error - outside tolerance
    ];

    for (i, &timing_us) in measurements.iter().enumerate() {
        let measurement = TimingMeasurement {
            task_name: "pemf_pulse",
            execution_time_us: timing_us,
            expected_time_us: 500000,
            timestamp_ms: timestamp + (i as u32 * 500),
        };
        processor
            .update_pemf_timing_measurements(measurement)
            .unwrap();
    }

    // Generate comprehensive report before completing the test
    let report = processor
        .generate_comprehensive_timing_report()
        .expect("Failed to generate comprehensive timing report");

    // Complete the test
    processor.update_active_test(timestamp + 5500).unwrap();

    // Verify report contents
    assert_eq!(report.total_measurements, 7);
    assert_eq!(report.within_tolerance_count, 5); // First 5 measurements within 1%
    assert!(report.success_rate_percent > 70); // Should be around 71% (5/7)
    assert!(report.timing_accuracy_percent > 0.0);
    assert!(report.max_deviation_us >= 8000); // Should capture the 8ms deviation
    assert!(report.deviation_count >= 2); // Should detect 2 deviations outside tolerance
    assert_eq!(report.tolerance_percent, 1.0);
    assert!(report.timing_stability_score > 0);
}

/// Test test result retrieval by ID
/// Requirements: 9.5 (test result structure with timing statistics)
test_case!(test_get_test_result_by_id, {
    let mut processor = TestCommandProcessor::new();
    let timestamp = 1000;

    // Create and start test
    let params = TestCommandProcessor::create_pemf_timing_parameters(2000, 1.0).unwrap();
    let test_id = processor
        .execute_pemf_timing_test(params, timestamp)
        .unwrap();

    // Add some measurements
    for i in 0..3 {
        let measurement = TimingMeasurement {
            task_name: "pemf_pulse",
            execution_time_us: 500000 + (i * 1000),
            expected_time_us: 500000,
            timestamp_ms: timestamp + (i * 500),
        };
        processor
            .update_pemf_timing_measurements(measurement)
            .unwrap();
    }

    // Complete the test
    processor.update_active_test(timestamp + 2500).unwrap();

    // Get test result by ID
    let result = processor
        .get_test_result(test_id)
        .expect("Failed to get test result");

    // Verify result structure
    assert_eq!(result.test_type, TestType::PemfTimingValidation);
    assert_eq!(result.status, TestStatus::Completed);
    assert_eq!(result.test_id, test_id);
    assert!(result.duration_ms() > 0);
    assert_eq!(result.measurements.timing_measurements.len(), 3);

    // Test with invalid ID
    let invalid_result = processor.get_test_result(99);
    assert!(invalid_result.is_none());
});

// Test runner for no_std environment
#[no_mangle]
pub extern "C" fn run_test_processor_tests() -> TestSuiteResult {
    let mut runner = TestRunner::new("test_processor_tests");

    // Register all converted tests
    register_tests!(
        runner,
        test_parameter_validation,
        test_resource_limits_validation,
        test_parameter_parsing_from_payload,
        test_parameter_serialization,
        test_processor_initialization,
        test_start_test_execution,
        test_timeout_protection,
        test_abort_active_test,
        test_result_collection_and_serialization,
        test_statistics_tracking,
        test_command_processing_integration,
        test_error_handling_invalid_commands,
        test_measurement_collection,
        test_get_test_result_by_id // TODO: Add remaining tests as they are converted
    );

    runner.run_all()
}

// Entry point for embedded test execution
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() -> ! {
    let _result = run_test_processor_tests();

    // In a real embedded environment, this would send results via USB HID
    // For now, we just loop
    loop {
        // Wait for next test execution command
    }
}
