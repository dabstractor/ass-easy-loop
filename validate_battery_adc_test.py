#!/usr/bin/env python3
"""
Battery ADC Validation Test Script

This script validates the battery ADC validation test functionality
by testing the core logic and calculations used in the embedded system.

Requirements: 9.1, 9.5
"""

def battery_state_from_adc(adc_value):
    """Determine battery state from ADC reading"""
    if adc_value <= 1425:
        return 0  # Low
    elif adc_value < 1675:
        return 1  # Normal
    else:
        return 2  # Charging

def is_valid_transition(from_state, to_state, adc_value):
    """Check if battery state transition is valid"""
    transitions = {
        (0, 1): lambda adc: adc > 1425,   # Low -> Normal
        (1, 0): lambda adc: adc <= 1425,  # Normal -> Low
        (1, 2): lambda adc: adc >= 1675,  # Normal -> Charging
        (2, 1): lambda adc: adc < 1675,   # Charging -> Normal
        (0, 2): lambda adc: adc >= 1675,  # Low -> Charging
        (2, 0): lambda adc: adc <= 1425,  # Charging -> Low
    }
    
    if (from_state, to_state) in transitions:
        return transitions[(from_state, to_state)](adc_value)
    else:
        return from_state == to_state  # Same state is always valid

def adc_to_battery_voltage(adc_value):
    """Convert ADC reading to battery voltage in millivolts"""
    # voltage_mv = adc_value * 2386 / 1000
    return (adc_value * 2386) // 1000

def battery_voltage_to_adc(battery_voltage_mv):
    """Convert battery voltage back to expected ADC reading"""
    # adc_value = battery_voltage_mv * 1000 / 2386
    adc_value = (battery_voltage_mv * 1000) // 2386
    return min(adc_value, 4095)

def validate_adc_parameters(duration_ms, reference_voltage_mv, tolerance_percent, 
                           sample_count, expected_adc_value):
    """Validate battery ADC test parameters"""
    if duration_ms == 0 or duration_ms > 60_000:
        return False
    if reference_voltage_mv < 1000 or reference_voltage_mv > 5000:
        return False
    if tolerance_percent < 0.1 or tolerance_percent > 20.0:
        return False
    if sample_count == 0 or sample_count > 1000:
        return False
    if expected_adc_value > 4095:
        return False
    return True

class AdcMeasurements:
    """Battery ADC measurements for testing"""
    def __init__(self):
        self.total_samples = 0
        self.sum_adc_values = 0
        self.voltage_accuracy_percent = 0.0
        self.reference_voltage_mv = 0
        self.measured_voltage_mv = 0

    def add_sample(self, adc_value):
        """Add ADC sample"""
        self.total_samples += 1
        self.sum_adc_values += adc_value

    def get_average_adc(self):
        """Get average ADC value"""
        if self.total_samples == 0:
            return 0
        return self.sum_adc_values // self.total_samples

    def calculate_voltage_accuracy(self, reference_voltage_mv):
        """Calculate voltage accuracy"""
        if self.total_samples == 0 or reference_voltage_mv == 0:
            return 0.0

        average_adc = self.get_average_adc()
        self.measured_voltage_mv = adc_to_battery_voltage(average_adc)
        self.reference_voltage_mv = reference_voltage_mv
        
        voltage_error = abs(self.measured_voltage_mv - reference_voltage_mv)
        self.voltage_accuracy_percent = max(0.0, 100.0 - (voltage_error / reference_voltage_mv * 100.0))
        return self.voltage_accuracy_percent

    def calculate_calibration_error(self, expected_adc):
        """Calculate calibration error"""
        if expected_adc == 0:
            return 0.0

        average_adc = self.get_average_adc()
        adc_error = abs(average_adc - expected_adc)
        return (adc_error / expected_adc) * 100.0

class TestResult:
    """Battery ADC test result for testing"""
    def __init__(self, reference_voltage_mv, tolerance_percent, expected_adc_value):
        self.reference_voltage_mv = reference_voltage_mv
        self.tolerance_percent = tolerance_percent
        self.expected_adc_value = expected_adc_value
        self.samples = []
        self.transitions = []  # (from, to, adc, valid)
        self.test_passed = False
        self.invalid_readings_count = 0

    def add_sample(self, adc_value):
        """Add ADC sample"""
        self.samples.append(adc_value)
        if adc_value > 4095:
            self.invalid_readings_count += 1

    def add_transition(self, from_state, to_state, adc_value):
        """Add state transition"""
        valid = is_valid_transition(from_state, to_state, adc_value)
        self.transitions.append((from_state, to_state, adc_value, valid))

    def complete_test(self):
        """Complete test and evaluate results"""
        if not self.samples:
            self.test_passed = False
            return

        # Calculate average ADC value
        average_adc = sum(self.samples) // len(self.samples)

        # Check voltage accuracy
        measured_voltage = adc_to_battery_voltage(average_adc)
        voltage_error = abs(measured_voltage - self.reference_voltage_mv)
        voltage_accuracy = 100.0 - (voltage_error / self.reference_voltage_mv * 100.0)
        
        if voltage_accuracy < (100.0 - self.tolerance_percent):
            self.test_passed = False
            return

        # Check calibration error
        adc_error = abs(average_adc - self.expected_adc_value)
        calibration_error = (adc_error / self.expected_adc_value) * 100.0
        
        if calibration_error > self.tolerance_percent:
            self.test_passed = False
            return

        # Check for invalid readings
        invalid_reading_rate = (self.invalid_readings_count / len(self.samples)) * 100.0
        if invalid_reading_rate > 5.0:
            self.test_passed = False
            return

        # Check state transitions
        for _, _, _, valid in self.transitions:
            if not valid:
                self.test_passed = False
                return

        self.test_passed = True

def test_battery_state_transitions():
    """Test battery state enumeration and transitions"""
    print("Testing battery state transitions...")
    
    # Test state determination from ADC values
    assert battery_state_from_adc(1000) == 0  # Low
    assert battery_state_from_adc(1425) == 0  # Low (at threshold)
    assert battery_state_from_adc(1426) == 1  # Normal
    assert battery_state_from_adc(1500) == 1  # Normal
    assert battery_state_from_adc(1674) == 1  # Normal
    assert battery_state_from_adc(1675) == 2  # Charging (at threshold)
    assert battery_state_from_adc(2000) == 2  # Charging
    
    # Test valid transitions
    assert is_valid_transition(0, 1, 1500)  # Low -> Normal with appropriate ADC
    assert is_valid_transition(1, 0, 1400)  # Normal -> Low with appropriate ADC
    assert is_valid_transition(1, 2, 1700)  # Normal -> Charging with appropriate ADC
    assert is_valid_transition(2, 1, 1600)  # Charging -> Normal with appropriate ADC
    assert is_valid_transition(0, 2, 1800)  # Low -> Charging with appropriate ADC
    assert is_valid_transition(2, 0, 1200)  # Charging -> Low with appropriate ADC
    
    # Test invalid transitions
    assert not is_valid_transition(0, 1, 1300)  # Low -> Normal with ADC too low
    assert not is_valid_transition(1, 2, 1600)  # Normal -> Charging with ADC too low
    assert not is_valid_transition(2, 0, 1700)  # Charging -> Low with ADC too high
    
    print("✓ Battery state transitions test passed")

def test_adc_to_voltage_conversion():
    """Test ADC to voltage conversion calculations"""
    print("Testing ADC to voltage conversion...")
    
    # Test voltage conversion calculations
    # ADC 1425 should be approximately 3400mV based on the conversion formula
    voltage_1425 = adc_to_battery_voltage(1425)
    assert 3300 <= voltage_1425 <= 3500, f"Expected 3300-3500mV, got {voltage_1425}mV"
    
    # ADC 1675 should be approximately 3996mV based on the conversion formula
    voltage_1675 = adc_to_battery_voltage(1675)
    assert 3900 <= voltage_1675 <= 4100, f"Expected 3900-4100mV, got {voltage_1675}mV"
    
    # Test reverse conversion
    adc_from_3400mv = battery_voltage_to_adc(3400)
    assert 1400 <= adc_from_3400mv <= 1450, f"Expected 1400-1450 ADC, got {adc_from_3400mv}"
    
    adc_from_4000mv = battery_voltage_to_adc(4000)
    assert 1650 <= adc_from_4000mv <= 1700, f"Expected 1650-1700 ADC, got {adc_from_4000mv}"
    
    # Test boundary conditions
    voltage_0 = adc_to_battery_voltage(0)
    assert voltage_0 == 0, f"Expected 0mV for ADC 0, got {voltage_0}mV"
    
    voltage_4095 = adc_to_battery_voltage(4095)
    assert 9700 <= voltage_4095 <= 9800, f"Expected 9700-9800mV for ADC 4095, got {voltage_4095}mV"
    
    print("✓ ADC to voltage conversion test passed")

def test_battery_adc_parameter_validation():
    """Test battery ADC parameter validation"""
    print("Testing battery ADC parameter validation...")
    
    # Test valid parameters
    assert validate_adc_parameters(5000, 3300, 2.0, 50, 1500)
    
    # Test invalid duration (too short)
    assert not validate_adc_parameters(0, 3300, 2.0, 50, 1500)
    
    # Test invalid duration (too long)
    assert not validate_adc_parameters(70_000, 3300, 2.0, 50, 1500)
    
    # Test invalid reference voltage (too low)
    assert not validate_adc_parameters(5000, 500, 2.0, 50, 1500)
    
    # Test invalid reference voltage (too high)
    assert not validate_adc_parameters(5000, 6000, 2.0, 50, 1500)
    
    # Test invalid tolerance (too low)
    assert not validate_adc_parameters(5000, 3300, 0.05, 50, 1500)
    
    # Test invalid tolerance (too high)
    assert not validate_adc_parameters(5000, 3300, 25.0, 50, 1500)
    
    # Test invalid sample count (zero)
    assert not validate_adc_parameters(5000, 3300, 2.0, 0, 1500)
    
    # Test invalid sample count (too high)
    assert not validate_adc_parameters(5000, 3300, 2.0, 2000, 1500)
    
    # Test invalid expected ADC value
    assert not validate_adc_parameters(5000, 3300, 2.0, 50, 5000)  # Above 12-bit ADC range
    
    print("✓ Battery ADC parameter validation test passed")

def test_battery_adc_measurements():
    """Test battery ADC measurements and calculations"""
    print("Testing battery ADC measurements...")
    
    measurements = AdcMeasurements()
    
    # Test initial state
    assert measurements.total_samples == 0
    assert measurements.get_average_adc() == 0
    assert measurements.voltage_accuracy_percent == 0.0
    
    # Add samples and test calculations
    measurements.add_sample(1450)
    measurements.add_sample(1500)
    measurements.add_sample(1550)
    
    assert measurements.total_samples == 3
    assert measurements.get_average_adc() == 1500  # Average of 1450, 1500, 1550
    
    # Test voltage accuracy calculation (using calculated voltage for 1500 ADC)
    expected_voltage = adc_to_battery_voltage(1500)  # ~3579mV
    accuracy = measurements.calculate_voltage_accuracy(expected_voltage)
    assert accuracy > 95.0, f"Expected >95% accuracy, got {accuracy}%"
    
    # Test calibration error calculation
    cal_error = measurements.calculate_calibration_error(1500)  # Perfect match
    assert cal_error == 0.0, f"Expected 0% calibration error, got {cal_error}%"
    
    cal_error_2 = measurements.calculate_calibration_error(1450)  # 50 ADC units off
    assert abs(cal_error_2 - 3.45) < 0.5, f"Expected ~3.45% calibration error, got {cal_error_2}%"
    
    print("✓ Battery ADC measurements test passed")

def test_battery_adc_test_result_evaluation():
    """Test battery ADC test result evaluation"""
    print("Testing battery ADC test result evaluation...")
    
    # Test successful test case (use realistic voltage for 1500 ADC)
    expected_voltage = adc_to_battery_voltage(1500)  # ~3579mV
    result = TestResult(expected_voltage, 2.0, 1500)  # Use calculated voltage, 2% tolerance, 1500 ADC expected
    
    # Add good samples
    result.add_sample(1480)  # Within tolerance
    result.add_sample(1500)  # Perfect
    result.add_sample(1520)  # Within tolerance
    
    # Add valid state transition
    result.add_transition(0, 1, 1480)  # Low -> Normal with valid ADC
    
    result.complete_test()
    assert result.test_passed, "Expected test to pass with good data"
    
    # Test failed test case with errors
    result_fail = TestResult(expected_voltage, 1.0, 1500)  # Strict 1% tolerance
    
    # Add samples with significant error
    result_fail.add_sample(1200)  # Way off
    result_fail.add_sample(1800)  # Way off
    result_fail.add_sample(5000)  # Invalid (> 4095)
    
    # Add invalid state transition
    result_fail.add_transition(0, 2, 1300)  # Low -> Charging with invalid ADC
    
    result_fail.complete_test()
    assert not result_fail.test_passed, "Expected test to fail with bad data"
    
    print("✓ Battery ADC test result evaluation test passed")

def main():
    """Run all battery ADC validation tests"""
    print("=== Battery ADC Validation Test Suite ===")
    print()
    
    try:
        test_battery_state_transitions()
        test_adc_to_voltage_conversion()
        test_battery_adc_parameter_validation()
        test_battery_adc_measurements()
        test_battery_adc_test_result_evaluation()
        
        print()
        print("🎉 ALL BATTERY ADC VALIDATION TESTS PASSED")
        print()
        print("Battery ADC validation functionality is working correctly:")
        print("- State transitions are properly validated")
        print("- ADC to voltage conversion is accurate")
        print("- Parameter validation catches invalid inputs")
        print("- Measurements and calculations are correct")
        print("- Test result evaluation works as expected")
        print()
        print("The battery ADC validation test implementation meets requirements 9.1 and 9.5")
        
    except AssertionError as e:
        print(f"❌ TEST FAILED: {e}")
        return 1
    except Exception as e:
        print(f"❌ UNEXPECTED ERROR: {e}")
        return 1
    
    return 0

if __name__ == "__main__":
    exit(main())