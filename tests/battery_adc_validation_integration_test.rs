//! Battery ADC Validation Integration Tests
//! 
//! These tests validate the battery ADC validation test functionality,
//! ensuring accurate voltage readings, calibration testing, and state
//! transition validation work correctly with the test processor framework.
//! 
//! Requirements: 9.1, 9.5

#![cfg(test)]

// Note: These tests are designed to run on the host system for validation
// They test the battery ADC validation logic without requiring embedded hardware

/// Test battery ADC parameter creation and validation
/// Requirements: 9.1 (configurable test parameters)
#[test]
fn test_battery_adc_parameter_creation() {
    // Test default parameters
    let default_params = BatteryAdcParameters::default();
    assert_eq_no_std!(default_params.test_duration_ms, 5000);
    assert_eq_no_std!(default_params.reference_voltage_mv, 3300);
    assert_eq_no_std!(default_params.tolerance_percent, 2.0);
    assert_eq_no_std!(default_params.sample_count, 50);
    assert_no_std!(default_params.calibration_enabled);
    assert_no_std!(default_params.state_transition_test);
    assert_eq_no_std!(default_params.expected_adc_value, 1500);
    assert_eq_no_std!(default_params.validation_mode, AdcValidationMode::Accuracy);
    assert_no_std!(default_params.validate().is_ok());

    // Test parameter serialization and parsing
    let serialized = default_params.serialize();
    assert_eq_no_std!(serialized.len(), 20);
    
    let parsed = BatteryAdcParameters::from_payload(&serialized).unwrap();
    assert_eq_no_std!(parsed.test_duration_ms, default_params.test_duration_ms);
    assert_eq_no_std!(parsed.reference_voltage_mv, default_params.reference_voltage_mv);
    assert_eq_no_std!(parsed.tolerance_percent, default_params.tolerance_percent);
    assert_eq_no_std!(parsed.sample_count, default_params.sample_count);
    assert_eq_no_std!(parsed.calibration_enabled, default_params.calibration_enabled);
    assert_eq_no_std!(parsed.state_transition_test, default_params.state_transition_test);
    assert_eq_no_std!(parsed.expected_adc_value, default_params.expected_adc_value);
    assert_eq_no_std!(parsed.validation_mode, default_params.validation_mode);
}

/// Test battery ADC parameter validation
/// Requirements: 9.1 (parameter validation)
#[test]
fn test_battery_adc_parameter_validation() {
    let mut params = BatteryAdcParameters::default();
    
    // Test valid parameters
    assert_no_std!(params.validate().is_ok());
    
    // Test invalid duration (too short)
    params.test_duration_ms = 0;
    assert_no_std!(params.validate().is_err());
    
    // Test invalid duration (too long)
    params.test_duration_ms = 70_000;
    assert_no_std!(params.validate().is_err());
    
    // Reset to valid
    params.test_duration_ms = 5000;
    assert_no_std!(params.validate().is_ok());
    
    // Test invalid reference voltage (too low)
    params.reference_voltage_mv = 500;
    assert_no_std!(params.validate().is_err());
    
    // Test invalid reference voltage (too high)
    params.reference_voltage_mv = 6000;
    assert_no_std!(params.validate().is_err());
    
    // Reset to valid
    params.reference_voltage_mv = 3300;
    assert_no_std!(params.validate().is_ok());
    
    // Test invalid tolerance (too low)
    params.tolerance_percent = 0.05;
    assert_no_std!(params.validate().is_err());
    
    // Test invalid tolerance (too high)
    params.tolerance_percent = 25.0;
    assert_no_std!(params.validate().is_err());
    
    // Reset to valid
    params.tolerance_percent = 2.0;
    assert_no_std!(params.validate().is_ok());
    
    // Test invalid sample count (zero)
    params.sample_count = 0;
    assert_no_std!(params.validate().is_err());
    
    // Test invalid sample count (too high)
    params.sample_count = 2000;
    assert_no_std!(params.validate().is_err());
    
    // Reset to valid
    params.sample_count = 50;
    assert_no_std!(params.validate().is_ok());
    
    // Test invalid expected ADC value
    params.expected_adc_value = 5000; // Above 12-bit ADC range
    assert_no_std!(params.validate().is_err());
}

/// Test battery state enumeration and transitions
/// Requirements: 9.1, 9.5 (battery state transition testing)
#[test]
fn test_battery_state_transitions() {
    // Test state determination from ADC values
    assert_eq_no_std!(BatteryState::from_adc_reading(1000), BatteryState::Low);
    assert_eq_no_std!(BatteryState::from_adc_reading(1425), BatteryState::Low);
    assert_eq_no_std!(BatteryState::from_adc_reading(1426), BatteryState::Normal);
    assert_eq_no_std!(BatteryState::from_adc_reading(1500), BatteryState::Normal);
    assert_eq_no_std!(BatteryState::from_adc_reading(1674), BatteryState::Normal);
    assert_eq_no_std!(BatteryState::from_adc_reading(1675), BatteryState::Charging);
    assert_eq_no_std!(BatteryState::from_adc_reading(2000), BatteryState::Charging);
    
    // Test threshold values
    let (low_min, low_max) = BatteryState::Low.get_thresholds();
    assert_eq_no_std!(low_min, 0);
    assert_eq_no_std!(low_max, 1425);
    
    let (normal_min, normal_max) = BatteryState::Normal.get_thresholds();
    assert_eq_no_std!(normal_min, 1425);
    assert_eq_no_std!(normal_max, 1675);
    
    let (charging_min, charging_max) = BatteryState::Charging.get_thresholds();
    assert_eq_no_std!(charging_min, 1675);
    assert_eq_no_std!(charging_max, u16::MAX);
    
    // Test valid transitions
    assert_no_std!(BatteryState::Low.is_valid_transition(BatteryState::Normal, 1500));
    assert_no_std!(BatteryState::Normal.is_valid_transition(BatteryState::Low, 1400));
    assert_no_std!(BatteryState::Normal.is_valid_transition(BatteryState::Charging, 1700));
    assert_no_std!(BatteryState::Charging.is_valid_transition(BatteryState::Normal, 1600));
    assert_no_std!(BatteryState::Low.is_valid_transition(BatteryState::Charging, 1800));
    assert_no_std!(BatteryState::Charging.is_valid_transition(BatteryState::Low, 1200));
    
    // Test invalid transitions
    assert_no_std!(!BatteryState::Low.is_valid_transition(BatteryState::Normal, 1300)); // ADC too low for Normal
    assert_no_std!(!BatteryState::Normal.is_valid_transition(BatteryState::Charging, 1600)); // ADC too low for Charging
    assert_no_std!(!BatteryState::Charging.is_valid_transition(BatteryState::Low, 1700)); // ADC too high for Low
}

/// Test battery ADC test execution
/// Requirements: 9.1, 9.5 (battery ADC validation test execution)
#[test]
fn test_battery_adc_test_execution() {
    let mut processor = TestCommandProcessor::new();
    let timestamp = 1000;
    
    // Create battery ADC test parameters
    let adc_params = BatteryAdcParameters {
        test_duration_ms: 3000,
        reference_voltage_mv: 3300,
        tolerance_percent: 2.0,
        sample_count: 30,
        calibration_enabled: true,
        state_transition_test: true,
        expected_adc_value: 1500,
        validation_mode: AdcValidationMode::Comprehensive,
    };
    
    // Start battery ADC test
    let test_id = processor.execute_battery_adc_test(adc_params, timestamp).unwrap();
    assert_eq_no_std!(test_id, 1);
    
    // Verify test is active
    let (test_type, status, id) = processor.get_active_test_info().unwrap();
    assert_eq_no_std!(test_type, TestType::BatteryAdcCalibration);
    assert_eq_no_std!(status, TestStatus::Running);
    assert_eq_no_std!(id, test_id);
    
    // Add some ADC samples
    processor.add_battery_adc_sample(1450, timestamp + 100).unwrap();
    processor.add_battery_adc_sample(1500, timestamp + 200).unwrap();
    processor.add_battery_adc_sample(1550, timestamp + 300).unwrap();
    
    // Add state transitions
    processor.add_battery_state_transition(
        BatteryState::Low, BatteryState::Normal, 1450, timestamp + 100
    ).unwrap();
    processor.add_battery_state_transition(
        BatteryState::Normal, BatteryState::Normal, 1500, timestamp + 200
    ).unwrap();
    
    // Get statistics
    let stats = processor.get_battery_adc_statistics().unwrap();
    assert_eq_no_std!(stats.total_samples, 3);
    assert_no_std!(stats.voltage_accuracy_percent > 90.0);
    assert_eq_no_std!(stats.state_transition_count, 2);
    
    // Complete the test
    let result = processor.update_active_test(timestamp + 3100).unwrap();
    assert_eq_no_std!(result.test_type, TestType::BatteryAdcCalibration);
    assert_eq_no_std!(result.status, TestStatus::Completed);
    assert_eq_no_std!(result.test_id, test_id);
}

/// Test battery ADC measurements and calculations
/// Requirements: 9.5 (ADC accuracy measurements)
#[test]
fn test_battery_adc_measurements() {
    let mut measurements = BatteryAdcMeasurements::new();
    
    // Test initial state
    assert_eq_no_std!(measurements.total_samples, 0);
    assert_eq_no_std!(measurements.average_adc_value, 0);
    assert_eq_no_std!(measurements.voltage_accuracy_percent, 0.0);
    assert_eq_no_std!(measurements.calibration_error_percent, 0.0);
    
    // Set test data
    measurements.total_samples = 10;
    measurements.average_adc_value = 1500; // Should be ~3.58V
    
    // Test voltage accuracy calculation
    let accuracy = measurements.calculate_voltage_accuracy(3600); // 3.6V reference
    assert_no_std!(accuracy > 95.0); // Should be very accurate
    assert_eq_no_std!(measurements.measured_voltage_mv, 3579); // Expected calculated voltage
    
    // Test calibration error calculation
    let cal_error = measurements.calculate_calibration_error(1500); // Perfect match
    assert_eq_no_std!(cal_error, 0.0);
    
    let cal_error_2 = measurements.calculate_calibration_error(1450); // 50 ADC units off
    assert_no_std!((cal_error_2 - 3.45).abs() < 0.1); // ~3.45% error
    
    // Test serialization
    let serialized = measurements.serialize();
    assert_eq_no_std!(serialized.len(), 40);
    
    // Verify serialized data
    let serialized_samples = u32::from_le_bytes([
        serialized[0], serialized[1], serialized[2], serialized[3]
    ]);
    assert_eq_no_std!(serialized_samples, measurements.total_samples);
    
    let serialized_adc = u16::from_le_bytes([serialized[4], serialized[5]]);
    assert_eq_no_std!(serialized_adc, measurements.average_adc_value);
}

/// Test battery ADC test result creation and completion
/// Requirements: 9.1, 9.5 (comprehensive ADC validation)
#[test]
fn test_battery_adc_test_result() {
    let adc_params = BatteryAdcParameters {
        test_duration_ms: 2000,
        reference_voltage_mv: 3300,
        tolerance_percent: 3.0,
        sample_count: 20,
        calibration_enabled: true,
        state_transition_test: true,
        expected_adc_value: 1400,
        validation_mode: AdcValidationMode::Comprehensive,
    };
    
    let mut result = BatteryAdcTestResult::new(adc_params, 1000);
    
    // Add ADC samples
    result.add_adc_sample(1380, 1100).unwrap();
    result.add_adc_sample(1400, 1200).unwrap();
    result.add_adc_sample(1420, 1300).unwrap();
    result.add_adc_sample(1410, 1400).unwrap();
    
    // Add state transitions
    result.add_state_transition(BatteryState::Low, BatteryState::Normal, 1430, 1250).unwrap();
    result.add_state_transition(BatteryState::Normal, BatteryState::Normal, 1410, 1350).unwrap();
    
    // Complete the test
    result.complete_test(3000);
    
    // Verify results
    assert_eq_no_std!(result.measurements.total_samples, 4);
    assert_eq_no_std!(result.measurements.average_adc_value, 1402); // Average of samples
    assert_eq_no_std!(result.measurements.state_transition_count, 2);
    assert_eq_no_std!(result.duration_ms(), 2000);
    assert_no_std!(result.test_passed); // Should pass with good data
    
    // Test serialization to command response
    let response = result.serialize_to_response(42).unwrap();
    assert_eq_no_std!(response.command_type, 0x83); // TestResponse::TestResult
    assert_eq_no_std!(response.command_id, 42);
    assert_no_std!(response.payload.len() > 0);
    
    // Verify response payload
    assert_eq_no_std!(response.payload[0], TestType::BatteryAdcCalibration as u8);
    assert_eq_no_std!(response.payload[1], TestStatus::Completed as u8);
    
    let duration = u32::from_le_bytes([
        response.payload[2], response.payload[3], response.payload[4], response.payload[5]
    ]);
    assert_eq_no_std!(duration, 2000);
}

/// Test battery ADC test with invalid data
/// Requirements: 9.5 (error detection and validation)
#[test]
fn test_battery_adc_test_with_errors() {
    let adc_params = BatteryAdcParameters {
        test_duration_ms: 1000,
        reference_voltage_mv: 3300,
        tolerance_percent: 1.0, // Very strict tolerance
        sample_count: 10,
        calibration_enabled: true,
        state_transition_test: true,
        expected_adc_value: 1500,
        validation_mode: AdcValidationMode::Comprehensive,
    };
    
    let mut result = BatteryAdcTestResult::new(adc_params, 1000);
    
    // Add samples with significant error
    result.add_adc_sample(1200, 1100).unwrap(); // Way off from expected 1500
    result.add_adc_sample(1800, 1200).unwrap(); // Way off in other direction
    result.add_adc_sample(5000, 1300).unwrap(); // Invalid ADC value (> 4095)
    
    // Add invalid state transition
    result.add_state_transition(BatteryState::Low, BatteryState::Charging, 1300, 1250).unwrap(); // Invalid: ADC too low for Charging
    
    // Complete the test
    result.complete_test(2000);
    
    // Verify test failed due to errors
    assert_no_std!(!result.test_passed);
    assert_eq_no_std!(result.measurements.invalid_readings_count, 1); // One invalid ADC reading
    assert_eq_no_std!(result.measurements.state_transition_count, 1);
    
    // Check that invalid transition was recorded
    assert_eq_no_std!(result.state_transitions.len(), 1);
    assert_no_std!(!result.state_transitions[0].transition_valid);
}

/// Test battery ADC test parameter conversion to TestParameters
/// Requirements: 9.1 (test parameter integration)
#[test]
fn test_battery_adc_test_parameter_conversion() {
    let adc_params = BatteryAdcParameters {
        test_duration_ms: 4000,
        reference_voltage_mv: 3300,
        tolerance_percent: 2.5,
        sample_count: 40,
        calibration_enabled: true,
        state_transition_test: false,
        expected_adc_value: 1600,
        validation_mode: AdcValidationMode::Accuracy,
    };
    
    // Convert to TestParameters
    let test_params = TestCommandProcessor::create_battery_adc_test_parameters(&adc_params).unwrap();
    
    // Verify conversion
    assert_eq_no_std!(test_params.duration_ms, 4000);
    assert_eq_no_std!(test_params.tolerance_percent, 2.5);
    assert_eq_no_std!(test_params.sample_rate_hz, 10); // 40 samples / 4 seconds = 10 Hz
    assert_eq_no_std!(test_params.validation_criteria.min_success_rate_percent, 97); // 100 - 2.5 = 97.5 -> 97
    assert_eq_no_std!(test_params.resource_limits.max_cpu_usage_percent, 15);
    assert_eq_no_std!(test_params.resource_limits.max_memory_usage_bytes, 1024);
    assert_no_std!(test_params.resource_limits.allow_preemption);
    
    // Verify custom parameters contain serialized ADC parameters
    assert_no_std!(!test_params.custom_parameters.is_empty());
    
    // Parse back the custom parameters
    let parsed_adc_params = TestCommandProcessor::parse_battery_adc_parameters(&test_params.custom_parameters).unwrap();
    assert_eq_no_std!(parsed_adc_params.test_duration_ms, adc_params.test_duration_ms);
    assert_eq_no_std!(parsed_adc_params.reference_voltage_mv, adc_params.reference_voltage_mv);
    assert_eq_no_std!(parsed_adc_params.tolerance_percent, adc_params.tolerance_percent);
    assert_eq_no_std!(parsed_adc_params.sample_count, adc_params.sample_count);
    assert_eq_no_std!(parsed_adc_params.validation_mode, adc_params.validation_mode);
}

/// Test ADC validation modes
/// Requirements: 9.1 (different validation modes)
#[test]
fn test_adc_validation_modes() {
    // Test mode enumeration
    assert_eq_no_std!(AdcValidationMode::Accuracy as u8, 0x00);
    assert_eq_no_std!(AdcValidationMode::Calibration as u8, 0x01);
    assert_eq_no_std!(AdcValidationMode::StateTransition as u8, 0x02);
    assert_eq_no_std!(AdcValidationMode::Comprehensive as u8, 0x03);
    
    // Test mode parsing
    assert_eq_no_std!(AdcValidationMode::from_u8(0x00), Some(AdcValidationMode::Accuracy));
    assert_eq_no_std!(AdcValidationMode::from_u8(0x01), Some(AdcValidationMode::Calibration));
    assert_eq_no_std!(AdcValidationMode::from_u8(0x02), Some(AdcValidationMode::StateTransition));
    assert_eq_no_std!(AdcValidationMode::from_u8(0x03), Some(AdcValidationMode::Comprehensive));
    assert_eq_no_std!(AdcValidationMode::from_u8(0xFF), None);
    
    // Test mode-specific parameter creation
    let accuracy_params = BatteryAdcParameters {
        validation_mode: AdcValidationMode::Accuracy,
        state_transition_test: false,
        calibration_enabled: false,
        ..BatteryAdcParameters::default()
    };
    assert_no_std!(accuracy_params.validate().is_ok());
    
    let comprehensive_params = BatteryAdcParameters {
        validation_mode: AdcValidationMode::Comprehensive,
        state_transition_test: true,
        calibration_enabled: true,
        ..BatteryAdcParameters::default()
    };
    assert_no_std!(comprehensive_params.validate().is_ok());
}

/// Test battery ADC test integration with test processor
/// Requirements: 9.1, 9.5 (full integration test)
#[test]
fn test_battery_adc_integration_with_processor() {
    let mut processor = TestCommandProcessor::new();
    let timestamp = 1000;
    
    // Test that no other test is running initially
    assert_no_std!(processor.get_active_test_info().is_none());
    
    // Create and start battery ADC test
    let adc_params = BatteryAdcParameters::default();
    let test_id = processor.execute_battery_adc_test(adc_params, timestamp).unwrap();
    
    // Verify test is active
    assert_no_std!(processor.get_active_test_info().is_some());
    let (test_type, status, id) = processor.get_active_test_info().unwrap();
    assert_eq_no_std!(test_type, TestType::BatteryAdcCalibration);
    assert_eq_no_std!(status, TestStatus::Running);
    assert_eq_no_std!(id, test_id);
    
    // Try to start another test (should fail)
    let result = processor.execute_battery_adc_test(adc_params, timestamp + 100);
    assert_eq_no_std!(result, Err(TestExecutionError::TestAborted));
    
    // Add test data
    for i in 0..10 {
        let adc_value = 1450 + (i * 10); // Gradually increasing ADC values
        processor.add_battery_adc_sample(adc_value, timestamp + 100 + (i as u32 * 100)).unwrap();
    }
    
    // Add state transition
    processor.add_battery_state_transition(
        BatteryState::Low, BatteryState::Normal, 1450, timestamp + 200
    ).unwrap();
    
    // Update test (should not complete yet)
    let result = processor.update_active_test(timestamp + 2000);
    assert_no_std!(result.is_none());
    
    // Update test after duration (should complete)
    let result = processor.update_active_test(timestamp + 6000).unwrap();
    assert_eq_no_std!(result.test_type, TestType::BatteryAdcCalibration);
    assert_eq_no_std!(result.status, TestStatus::Completed);
    assert_eq_no_std!(result.test_id, test_id);
    
    // Verify test is no longer active
    assert_no_std!(processor.get_active_test_info().is_none());
    
    // Check statistics
    let stats = processor.get_statistics();
    assert_eq_no_std!(stats.total_tests_executed, 1);
    assert_eq_no_std!(stats.total_tests_passed, 1);
    assert_eq_no_std!(stats.total_tests_failed, 0);
    assert_eq_no_std!(stats.success_rate_percent(), 100);
}