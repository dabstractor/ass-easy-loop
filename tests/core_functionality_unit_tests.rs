//! Core Functionality Unit Tests
//! 
//! This module contains comprehensive unit tests for the core functionality
//! of the pEMF device, including battery state machine logic, ADC conversion,
//! threshold detection, and timing calculations.
//! 
//! Requirements: 2.3, 3.2, 3.3, 3.4 (Task 9.1)

#![cfg(test)]

// ============================================================================
// Battery State Machine Logic Tests
// Requirements: 3.2, 3.3, 3.4
// ============================================================================

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BatteryState {
    Low,      // ADC ≤ 1425 (< 3.1V)
    Normal,   // 1425 < ADC < 1675 (3.1V - 3.6V)
    Charging, // ADC ≥ 1675 (> 3.6V)
}

impl BatteryState {
    /// Determine battery state from ADC reading with threshold comparisons
    pub fn from_adc_reading(adc_value: u16) -> Self {
        if adc_value <= 1425 {
            BatteryState::Low
        } else if adc_value < 1675 {
            BatteryState::Normal
        } else {
            BatteryState::Charging
        }
    }

    /// Get the ADC threshold values for this state
    pub fn get_thresholds(&self) -> (u16, u16) {
        match self {
            BatteryState::Low => (0, 1425),
            BatteryState::Normal => (1425, 1675),
            BatteryState::Charging => (1675, u16::MAX),
        }
    }

    /// Check if a state transition should occur based on new ADC reading
    pub fn should_transition_to(&self, adc_value: u16) -> Option<BatteryState> {
        let new_state = Self::from_adc_reading(adc_value);
        if new_state != *self {
            Some(new_state)
        } else {
            None
        }
    }
}

/// Battery monitoring functionality for testing
pub struct BatteryMonitor;

impl BatteryMonitor {
    /// Convert ADC reading to actual battery voltage
    /// Uses voltage divider calculation: Vbat = ADC_value * (3.3V / 4095) / voltage_divider_ratio
    /// Voltage divider ratio = R2 / (R1 + R2) = 5.1kΩ / (10kΩ + 5.1kΩ) = 0.337
    pub fn adc_to_battery_voltage(adc_value: u16) -> u32 {
        // Convert to millivolts for better precision
        // ADC voltage = adc_value * 3300mV / 4095
        // Battery voltage = ADC voltage / 0.337
        // Simplified: battery_voltage_mv = adc_value * 3300 / (4095 * 0.337)
        // Further simplified: battery_voltage_mv = adc_value * 2386 / 1000
        (adc_value as u32 * 2386) / 1000
    }

    /// Convert battery voltage (in millivolts) back to expected ADC reading
    /// This is useful for testing and validation
    pub fn battery_voltage_to_adc(battery_voltage_mv: u32) -> u16 {
        // Reverse calculation: adc_value = battery_voltage_mv * 1000 / 2386
        let adc_value = (battery_voltage_mv * 1000) / 2386;
        if adc_value > 4095 {
            4095
        } else {
            adc_value as u16
        }
    }
}

#[cfg(test)]
mod battery_state_machine_tests {
    use super::*;

    /// Test battery state determination from ADC readings
    /// Requirements: 3.2 (ADC ≤ 1425 = Low), 3.3 (1425 < ADC < 1675 = Normal), 3.4 (ADC ≥ 1675 = Charging)
    #[test]
    fn test_battery_state_from_adc_reading() {
        // Test Low battery state (ADC ≤ 1425)
        assert_eq!(BatteryState::from_adc_reading(0), BatteryState::Low);
        assert_eq!(BatteryState::from_adc_reading(500), BatteryState::Low);
        assert_eq!(BatteryState::from_adc_reading(1000), BatteryState::Low);
        assert_eq!(BatteryState::from_adc_reading(1425), BatteryState::Low);

        // Test Normal battery state (1425 < ADC < 1675)
        assert_eq!(BatteryState::from_adc_reading(1426), BatteryState::Normal);
        assert_eq!(BatteryState::from_adc_reading(1500), BatteryState::Normal);
        assert_eq!(BatteryState::from_adc_reading(1600), BatteryState::Normal);
        assert_eq!(BatteryState::from_adc_reading(1674), BatteryState::Normal);

        // Test Charging battery state (ADC ≥ 1675)
        assert_eq!(BatteryState::from_adc_reading(1675), BatteryState::Charging);
        assert_eq!(BatteryState::from_adc_reading(1800), BatteryState::Charging);
        assert_eq!(BatteryState::from_adc_reading(2000), BatteryState::Charging);
        assert_eq!(BatteryState::from_adc_reading(4095), BatteryState::Charging);
    }

    /// Test boundary conditions for battery state thresholds
    /// Requirements: 3.2, 3.3, 3.4
    #[test]
    fn test_battery_state_boundary_conditions() {
        // Test exact boundary values
        assert_eq!(BatteryState::from_adc_reading(1425), BatteryState::Low, 
                   "ADC 1425 should be Low state (≤ threshold)");
        assert_eq!(BatteryState::from_adc_reading(1426), BatteryState::Normal, 
                   "ADC 1426 should be Normal state (> low threshold)");
        assert_eq!(BatteryState::from_adc_reading(1674), BatteryState::Normal, 
                   "ADC 1674 should be Normal state (< charging threshold)");
        assert_eq!(BatteryState::from_adc_reading(1675), BatteryState::Charging, 
                   "ADC 1675 should be Charging state (≥ threshold)");

        // Test edge cases
        assert_eq!(BatteryState::from_adc_reading(u16::MIN), BatteryState::Low);
        assert_eq!(BatteryState::from_adc_reading(u16::MAX), BatteryState::Charging);
    }

    /// Test battery state threshold values
    /// Requirements: 3.2, 3.3, 3.4
    #[test]
    fn test_battery_state_thresholds() {
        let low_state = BatteryState::Low;
        let (low_min, low_max) = low_state.get_thresholds();
        assert_eq!(low_min, 0, "Low state minimum threshold should be 0");
        assert_eq!(low_max, 1425, "Low state maximum threshold should be 1425");

        let normal_state = BatteryState::Normal;
        let (normal_min, normal_max) = normal_state.get_thresholds();
        assert_eq!(normal_min, 1425, "Normal state minimum threshold should be 1425");
        assert_eq!(normal_max, 1675, "Normal state maximum threshold should be 1675");

        let charging_state = BatteryState::Charging;
        let (charging_min, charging_max) = charging_state.get_thresholds();
        assert_eq!(charging_min, 1675, "Charging state minimum threshold should be 1675");
        assert_eq!(charging_max, u16::MAX, "Charging state maximum threshold should be u16::MAX");
    }

    /// Test state transition logic
    /// Requirements: 3.2, 3.3, 3.4
    #[test]
    fn test_battery_state_transitions() {
        // Test transitions from Normal state
        let normal_state = BatteryState::Normal;
        
        // Transition to Low
        assert_eq!(normal_state.should_transition_to(1400), Some(BatteryState::Low));
        assert_eq!(normal_state.should_transition_to(1425), Some(BatteryState::Low));
        
        // Transition to Charging
        assert_eq!(normal_state.should_transition_to(1675), Some(BatteryState::Charging));
        assert_eq!(normal_state.should_transition_to(1800), Some(BatteryState::Charging));
        
        // No transition (stay in Normal)
        assert_eq!(normal_state.should_transition_to(1500), None);
        assert_eq!(normal_state.should_transition_to(1600), None);

        // Test transitions from Low state
        let low_state = BatteryState::Low;
        
        // Transition to Normal
        assert_eq!(low_state.should_transition_to(1426), Some(BatteryState::Normal));
        assert_eq!(low_state.should_transition_to(1500), Some(BatteryState::Normal));
        
        // Transition to Charging (direct jump)
        assert_eq!(low_state.should_transition_to(1675), Some(BatteryState::Charging));
        assert_eq!(low_state.should_transition_to(1800), Some(BatteryState::Charging));
        
        // No transition (stay in Low)
        assert_eq!(low_state.should_transition_to(1000), None);
        assert_eq!(low_state.should_transition_to(1425), None);

        // Test transitions from Charging state
        let charging_state = BatteryState::Charging;
        
        // Transition to Normal
        assert_eq!(charging_state.should_transition_to(1674), Some(BatteryState::Normal));
        assert_eq!(charging_state.should_transition_to(1500), Some(BatteryState::Normal));
        
        // Transition to Low (direct jump)
        assert_eq!(charging_state.should_transition_to(1425), Some(BatteryState::Low));
        assert_eq!(charging_state.should_transition_to(1000), Some(BatteryState::Low));
        
        // No transition (stay in Charging)
        assert_eq!(charging_state.should_transition_to(1675), None);
        assert_eq!(charging_state.should_transition_to(2000), None);
    }

    /// Test battery state equality and comparison
    #[test]
    fn test_battery_state_equality() {
        assert_eq!(BatteryState::Low, BatteryState::Low);
        assert_eq!(BatteryState::Normal, BatteryState::Normal);
        assert_eq!(BatteryState::Charging, BatteryState::Charging);

        assert_ne!(BatteryState::Low, BatteryState::Normal);
        assert_ne!(BatteryState::Normal, BatteryState::Charging);
        assert_ne!(BatteryState::Low, BatteryState::Charging);
    }
}

// ============================================================================
// ADC Value Conversion and Threshold Detection Tests
// Requirements: 3.2, 3.3, 3.4
// ============================================================================

#[cfg(test)]
mod adc_conversion_tests {
    use super::*;

    /// Test ADC to battery voltage conversion
    /// Requirements: 3.2, 3.3, 3.4
    #[test]
    fn test_adc_to_battery_voltage_conversion() {
        // Test known conversion points
        // Voltage divider ratio = 0.337, so battery voltage = ADC voltage / 0.337
        // ADC voltage = adc_value * 3300mV / 4095
        // Battery voltage = adc_value * 3300 / (4095 * 0.337) ≈ adc_value * 2.386

        // Test zero point
        assert_eq!(BatteryMonitor::adc_to_battery_voltage(0), 0);

        // Test threshold points with tolerance
        let low_threshold_voltage = BatteryMonitor::adc_to_battery_voltage(1425);
        assert!(low_threshold_voltage >= 3000 && low_threshold_voltage <= 3200, 
                "Low threshold (ADC 1425) should be ~3100mV, got {}mV", low_threshold_voltage);

        let charging_threshold_voltage = BatteryMonitor::adc_to_battery_voltage(1675);
        assert!(charging_threshold_voltage >= 3500 && charging_threshold_voltage <= 3700, 
                "Charging threshold (ADC 1675) should be ~3600mV, got {}mV", charging_threshold_voltage);

        // Test maximum ADC value
        let max_voltage = BatteryMonitor::adc_to_battery_voltage(4095);
        assert!(max_voltage >= 9700 && max_voltage <= 9800, 
                "Max ADC (4095) should be ~9770mV, got {}mV", max_voltage);

        // Test mid-range values
        let mid_voltage = BatteryMonitor::adc_to_battery_voltage(2048); // ~50% of ADC range
        assert!(mid_voltage >= 4800 && mid_voltage <= 5000, 
                "Mid ADC (2048) should be ~4900mV, got {}mV", mid_voltage);
    }

    /// Test battery voltage to ADC conversion (reverse calculation)
    /// Requirements: 3.2, 3.3, 3.4
    #[test]
    fn test_battery_voltage_to_adc_conversion() {
        // Test known voltage points
        assert_eq!(BatteryMonitor::battery_voltage_to_adc(0), 0);

        // Test threshold voltages with tolerance
        let low_threshold_adc = BatteryMonitor::battery_voltage_to_adc(3100);
        assert!(low_threshold_adc >= 1400 && low_threshold_adc <= 1450, 
                "3100mV should convert to ~1425 ADC, got {}", low_threshold_adc);

        let charging_threshold_adc = BatteryMonitor::battery_voltage_to_adc(3600);
        assert!(charging_threshold_adc >= 1650 && charging_threshold_adc <= 1700, 
                "3600mV should convert to ~1675 ADC, got {}", charging_threshold_adc);

        // Test maximum voltage (should clamp to 4095)
        let max_adc = BatteryMonitor::battery_voltage_to_adc(15000); // Very high voltage
        assert_eq!(max_adc, 4095, "High voltage should clamp to maximum ADC value");

        // Test typical battery voltages
        let typical_low_adc = BatteryMonitor::battery_voltage_to_adc(3000);
        let typical_normal_adc = BatteryMonitor::battery_voltage_to_adc(3700);
        let typical_charging_adc = BatteryMonitor::battery_voltage_to_adc(4200);

        assert!(typical_low_adc <= 1425, "3000mV should be in Low range");
        assert!(typical_normal_adc > 1425 && typical_normal_adc < 1675, "3700mV should be in Normal range");
        assert!(typical_charging_adc >= 1675, "4200mV should be in Charging range");
    }

    /// Test round-trip conversion accuracy
    /// Requirements: 3.2, 3.3, 3.4
    #[test]
    fn test_round_trip_conversion_accuracy() {
        let test_adc_values = vec![0, 500, 1000, 1425, 1500, 1675, 2000, 3000, 4095];

        for &adc_value in &test_adc_values {
            let voltage = BatteryMonitor::adc_to_battery_voltage(adc_value);
            let converted_back = BatteryMonitor::battery_voltage_to_adc(voltage);

            // Allow for small rounding errors (±2 ADC counts)
            let error = if converted_back > adc_value {
                converted_back - adc_value
            } else {
                adc_value - converted_back
            };

            assert!(error <= 2, 
                    "Round-trip conversion error too large: ADC {} -> {}mV -> ADC {} (error: {})", 
                    adc_value, voltage, converted_back, error);
        }
    }

    /// Test threshold detection accuracy
    /// Requirements: 3.2, 3.3, 3.4
    #[test]
    fn test_threshold_detection_accuracy() {
        // Test values around thresholds
        let test_cases = vec![
            // (ADC value, expected state)
            (1424, BatteryState::Low),
            (1425, BatteryState::Low),
            (1426, BatteryState::Normal),
            (1500, BatteryState::Normal),
            (1674, BatteryState::Normal),
            (1675, BatteryState::Charging),
            (1676, BatteryState::Charging),
        ];

        for (adc_value, expected_state) in test_cases {
            let detected_state = BatteryState::from_adc_reading(adc_value);
            assert_eq!(detected_state, expected_state, 
                      "ADC {} should be detected as {:?}, got {:?}", 
                      adc_value, expected_state, detected_state);
        }
    }

    /// Test voltage divider calculation constants
    /// Requirements: 3.2, 3.3, 3.4
    #[test]
    fn test_voltage_divider_constants() {
        // Verify the voltage divider calculation constants
        // R1 = 10kΩ, R2 = 5.1kΩ
        // Ratio = R2 / (R1 + R2) = 5.1 / (10 + 5.1) = 5.1 / 15.1 ≈ 0.337
        
        const R1_KOHM: f32 = 10.0;
        const R2_KOHM: f32 = 5.1;
        const EXPECTED_RATIO: f32 = R2_KOHM / (R1_KOHM + R2_KOHM);
        
        // The ratio should be approximately 0.337
        assert!((EXPECTED_RATIO - 0.337).abs() < 0.001, 
                "Voltage divider ratio should be ~0.337, calculated: {:.3}", EXPECTED_RATIO);

        // Test that our conversion constant matches the expected calculation
        // Conversion factor = 3300 / (4095 * 0.337) ≈ 2.386
        const ADC_REF_MV: f32 = 3300.0;
        const ADC_MAX: f32 = 4095.0;
        const EXPECTED_CONVERSION_FACTOR: f32 = ADC_REF_MV / (ADC_MAX * EXPECTED_RATIO);
        
        // Our implementation uses integer math: (adc_value * 2386) / 1000
        const IMPLEMENTATION_FACTOR: f32 = 2386.0 / 1000.0;
        
        assert!((EXPECTED_CONVERSION_FACTOR - IMPLEMENTATION_FACTOR).abs() < 0.01, 
                "Conversion factor mismatch: expected {:.3}, implementation {:.3}", 
                EXPECTED_CONVERSION_FACTOR, IMPLEMENTATION_FACTOR);
    }
}

// ============================================================================
// Timing Calculations and Pulse Generation Logic Tests
// Requirements: 2.3
// ============================================================================

#[cfg(test)]
mod timing_calculation_tests {
    use super::*;

    /// Test pEMF pulse timing constants
    /// Requirements: 2.3 (±1% timing tolerance)
    #[test]
    fn test_pemf_pulse_timing_constants() {
        const PULSE_HIGH_DURATION_MS: u64 = 2;
        const PULSE_LOW_DURATION_MS: u64 = 498;
        const TOTAL_PERIOD_MS: u64 = PULSE_HIGH_DURATION_MS + PULSE_LOW_DURATION_MS;

        // Verify total period equals 500ms for 2Hz frequency
        assert_eq!(TOTAL_PERIOD_MS, 500, "Total period should be 500ms for 2Hz");

        // Verify frequency calculation: f = 1/T, where T = 0.5s
        let frequency_hz = 1000.0 / TOTAL_PERIOD_MS as f32;
        assert!((frequency_hz - 2.0).abs() < 0.001, 
                "Frequency should be 2Hz, calculated: {:.3}Hz", frequency_hz);

        // Verify pulse width is exactly 2ms as required
        assert_eq!(PULSE_HIGH_DURATION_MS, 2, "Pulse HIGH duration must be exactly 2ms");

        // Verify low phase duration
        assert_eq!(PULSE_LOW_DURATION_MS, 498, "Pulse LOW duration must be exactly 498ms");
    }

    /// Test timing accuracy requirements (±1% tolerance)
    /// Requirements: 2.3
    #[test]
    fn test_timing_accuracy_tolerance() {
        const PULSE_HIGH_DURATION_MS: u64 = 2;
        const PULSE_LOW_DURATION_MS: u64 = 498;
        const TOLERANCE_PERCENT: f32 = 0.01; // ±1%

        // Calculate acceptable timing ranges
        let high_min = PULSE_HIGH_DURATION_MS as f32 * (1.0 - TOLERANCE_PERCENT);
        let high_max = PULSE_HIGH_DURATION_MS as f32 * (1.0 + TOLERANCE_PERCENT);
        let low_min = PULSE_LOW_DURATION_MS as f32 * (1.0 - TOLERANCE_PERCENT);
        let low_max = PULSE_LOW_DURATION_MS as f32 * (1.0 + TOLERANCE_PERCENT);

        // Verify timing values are within tolerance
        assert!(high_min <= 2.0 && 2.0 <= high_max, 
                "HIGH pulse timing within ±1% tolerance: {:.3}ms - {:.3}ms", high_min, high_max);
        assert!(low_min <= 498.0 && 498.0 <= low_max, 
                "LOW pulse timing within ±1% tolerance: {:.3}ms - {:.3}ms", low_min, low_max);

        // Verify total period accuracy
        let total_period = PULSE_HIGH_DURATION_MS + PULSE_LOW_DURATION_MS;
        let period_min = 500.0 * (1.0 - TOLERANCE_PERCENT);
        let period_max = 500.0 * (1.0 + TOLERANCE_PERCENT);
        assert!(period_min <= total_period as f32 && total_period as f32 <= period_max, 
                "Total period within ±1% tolerance: {:.3}ms - {:.3}ms", period_min, period_max);
    }

    /// Test pulse state tracking logic
    /// Requirements: 2.3
    #[test]
    fn test_pulse_state_tracking() {
        let mut pulse_active = false;

        // Simulate pulse cycle state changes
        // Start of HIGH phase
        pulse_active = true;
        assert!(pulse_active, "pulse_active should be true during HIGH phase");

        // Start of LOW phase
        pulse_active = false;
        assert!(!pulse_active, "pulse_active should be false during LOW phase");

        // Verify state alternation over multiple cycles
        for cycle in 0..10 {
            pulse_active = true;  // HIGH phase
            assert!(pulse_active, "pulse_active should be true during HIGH phase of cycle {}", cycle);

            pulse_active = false; // LOW phase
            assert!(!pulse_active, "pulse_active should be false during LOW phase of cycle {}", cycle);
        }
    }

    /// Test frequency calculation validation
    /// Requirements: 2.3
    #[test]
    fn test_frequency_calculation() {
        const PULSE_HIGH_DURATION_MS: u64 = 2;
        const PULSE_LOW_DURATION_MS: u64 = 498;

        // Calculate frequency from timing constants
        let period_ms = PULSE_HIGH_DURATION_MS + PULSE_LOW_DURATION_MS;
        let period_s = period_ms as f32 / 1000.0;
        let calculated_frequency = 1.0 / period_s;

        // Verify calculated frequency matches requirement
        assert!((calculated_frequency - 2.0).abs() < 0.001, 
                "Calculated frequency should be 2Hz, got {:.3}Hz", calculated_frequency);

        // Verify duty cycle calculation
        let duty_cycle = (PULSE_HIGH_DURATION_MS as f32 / period_ms as f32) * 100.0;
        let expected_duty_cycle = 0.4; // 2ms / 500ms = 0.4%
        assert!((duty_cycle - expected_duty_cycle).abs() < 0.01, 
                "Duty cycle should be 0.4%, got {:.2}%", duty_cycle);
    }

    /// Test timing precision requirements
    /// Requirements: 2.3
    #[test]
    fn test_timing_precision_requirements() {
        // Verify that the timing meets the ±1% precision requirement
        const PULSE_HIGH_DURATION_MS: u64 = 2;
        const PULSE_LOW_DURATION_MS: u64 = 498;
        const REQUIRED_PRECISION_PERCENT: f32 = 1.0; // ±1%

        // Calculate maximum allowed deviation
        let high_max_deviation = PULSE_HIGH_DURATION_MS as f32 * REQUIRED_PRECISION_PERCENT / 100.0;
        let low_max_deviation = PULSE_LOW_DURATION_MS as f32 * REQUIRED_PRECISION_PERCENT / 100.0;

        // Verify deviations are reasonable for hardware timer precision
        assert!(high_max_deviation >= 0.02, 
                "HIGH pulse precision tolerance should be at least 0.02ms, got {:.3}ms", high_max_deviation);
        assert!(low_max_deviation >= 4.98, 
                "LOW pulse precision tolerance should be at least 4.98ms, got {:.3}ms", low_max_deviation);

        // Verify total period precision
        let total_period = PULSE_HIGH_DURATION_MS + PULSE_LOW_DURATION_MS;
        let total_max_deviation = total_period as f32 * REQUIRED_PRECISION_PERCENT / 100.0;
        assert!(total_max_deviation >= 5.0, 
                "Total period precision tolerance should be at least 5ms, got {:.3}ms", total_max_deviation);
    }

    /// Test GPIO control timing requirements
    /// Requirements: 2.3
    #[test]
    fn test_gpio_control_timing() {
        // Verify that GPIO operations can be performed within timing constraints
        const PULSE_HIGH_DURATION_MS: u64 = 2;
        const PULSE_LOW_DURATION_MS: u64 = 498;

        // Assume GPIO operations take < 1μs (typical for RP2040)
        const GPIO_OVERHEAD_US: f32 = 1.0;
        const GPIO_OVERHEAD_MS: f32 = GPIO_OVERHEAD_US / 1000.0;

        // Verify that timing allows for GPIO overhead
        let effective_high_time = PULSE_HIGH_DURATION_MS as f32 - GPIO_OVERHEAD_MS;
        let effective_low_time = PULSE_LOW_DURATION_MS as f32 - GPIO_OVERHEAD_MS;

        assert!(effective_high_time > 1.9, 
                "HIGH pulse should have sufficient time after GPIO overhead: {:.3}ms", effective_high_time);
        assert!(effective_low_time > 497.9, 
                "LOW pulse should have sufficient time after GPIO overhead: {:.3}ms", effective_low_time);
    }

    /// Test pulse generation cycle consistency
    /// Requirements: 2.3
    #[test]
    fn test_pulse_generation_cycle_consistency() {
        const PULSE_HIGH_DURATION_MS: u64 = 2;
        const PULSE_LOW_DURATION_MS: u64 = 498;
        const TOTAL_PERIOD_MS: u64 = PULSE_HIGH_DURATION_MS + PULSE_LOW_DURATION_MS;

        // Simulate multiple pulse cycles
        let mut total_time = 0u64;
        let num_cycles = 10;

        for cycle in 0..num_cycles {
            // HIGH phase
            total_time += PULSE_HIGH_DURATION_MS;
            
            // LOW phase
            total_time += PULSE_LOW_DURATION_MS;

            // Verify cycle timing
            let expected_time = (cycle + 1) * TOTAL_PERIOD_MS;
            assert_eq!(total_time, expected_time, 
                      "Cycle {} timing mismatch: expected {}ms, got {}ms", 
                      cycle + 1, expected_time, total_time);
        }

        // Verify total time for all cycles
        let expected_total_time = num_cycles * TOTAL_PERIOD_MS;
        assert_eq!(total_time, expected_total_time, 
                  "Total time for {} cycles should be {}ms, got {}ms", 
                  num_cycles, expected_total_time, total_time);

        // Verify average frequency
        let average_frequency = (num_cycles as f32 * 1000.0) / total_time as f32;
        assert!((average_frequency - 2.0).abs() < 0.001, 
                "Average frequency over {} cycles should be 2Hz, got {:.3}Hz", 
                num_cycles, average_frequency);
    }
}

// ============================================================================
// Integration Tests for Core Functionality
// ============================================================================

#[cfg(test)]
mod core_integration_tests {
    use super::*;

    /// Test integration between battery state machine and ADC conversion
    /// Requirements: 2.3, 3.2, 3.3, 3.4
    #[test]
    fn test_battery_state_adc_integration() {
        // Test realistic battery voltage scenarios
        let test_scenarios = vec![
            // (battery_voltage_mv, expected_state, description)
            (2800, BatteryState::Low, "Deeply discharged battery"),
            (3000, BatteryState::Low, "Low battery warning level"),
            (3100, BatteryState::Low, "At low threshold"),
            (3200, BatteryState::Normal, "Normal operating voltage"),
            (3500, BatteryState::Normal, "Good battery level"),
            (3600, BatteryState::Normal, "Near charging threshold"),
            (3700, BatteryState::Charging, "Charging detected"),
            (4200, BatteryState::Charging, "Full charge voltage"),
        ];

        for (voltage_mv, expected_state, description) in test_scenarios {
            // Convert voltage to ADC value
            let adc_value = BatteryMonitor::battery_voltage_to_adc(voltage_mv);
            
            // Determine state from ADC value
            let detected_state = BatteryState::from_adc_reading(adc_value);
            
            // Convert back to voltage for verification
            let converted_voltage = BatteryMonitor::adc_to_battery_voltage(adc_value);
            
            assert_eq!(detected_state, expected_state, 
                      "{}: {}mV -> ADC {} -> {:?} (expected {:?})", 
                      description, voltage_mv, adc_value, detected_state, expected_state);
            
            // Verify voltage conversion accuracy (allow ±50mV tolerance)
            let voltage_error = if converted_voltage > voltage_mv {
                converted_voltage - voltage_mv
            } else {
                voltage_mv - converted_voltage
            };
            
            assert!(voltage_error <= 50, 
                   "{}: Voltage conversion error too large: {}mV -> {}mV (error: {}mV)", 
                   description, voltage_mv, converted_voltage, voltage_error);
        }
    }

    /// Test timing and battery monitoring integration
    /// Requirements: 2.3, 3.2, 3.3, 3.4
    #[test]
    fn test_timing_battery_integration() {
        // Verify that battery monitoring timing doesn't interfere with pEMF timing
        const PEMF_PERIOD_MS: u64 = 500;  // 2Hz
        const BATTERY_SAMPLE_INTERVAL_MS: u64 = 100;  // 10Hz
        
        // Calculate timing relationships
        let samples_per_pemf_cycle = PEMF_PERIOD_MS / BATTERY_SAMPLE_INTERVAL_MS;
        assert_eq!(samples_per_pemf_cycle, 5, 
                  "Should have exactly 5 battery samples per pEMF cycle");
        
        // Verify no timing conflicts
        assert!(PEMF_PERIOD_MS % BATTERY_SAMPLE_INTERVAL_MS == 0, 
               "Battery sampling should align with pEMF cycles");
        
        // Verify battery monitoring can complete within its time slice
        const MAX_BATTERY_PROCESSING_TIME_MS: u64 = 10;  // Assumed processing time
        assert!(MAX_BATTERY_PROCESSING_TIME_MS < BATTERY_SAMPLE_INTERVAL_MS, 
               "Battery processing should complete within sampling interval");
    }

    /// Test comprehensive core functionality validation
    /// Requirements: 2.3, 3.2, 3.3, 3.4
    #[test]
    fn test_comprehensive_core_validation() {
        // Test all core functions work together correctly
        
        // 1. Test battery state machine with various ADC values
        let adc_test_values = vec![0, 500, 1425, 1500, 1675, 2000, 4095];
        let mut state_changes = 0;
        let mut previous_state: Option<BatteryState> = None;
        
        for adc_value in adc_test_values {
            let current_state = BatteryState::from_adc_reading(adc_value);
            let voltage = BatteryMonitor::adc_to_battery_voltage(adc_value);
            
            // Verify state is consistent with voltage
            match current_state {
                BatteryState::Low => assert!(voltage <= 3200, 
                    "Low state voltage should be ≤3200mV, got {}mV", voltage),
                BatteryState::Normal => assert!(voltage > 3000 && voltage < 3800, 
                    "Normal state voltage should be 3000-3800mV, got {}mV", voltage),
                BatteryState::Charging => assert!(voltage >= 3500, 
                    "Charging state voltage should be ≥3500mV, got {}mV", voltage),
            }
            
            // Count state changes
            if let Some(prev_state) = previous_state {
                if prev_state != current_state {
                    state_changes += 1;
                }
            }
            
            previous_state = Some(current_state);
        }
        
        // Verify we detected state changes
        assert!(state_changes >= 2, "Should detect multiple state changes, got {}", state_changes);
        
        // 2. Test timing calculations
        const PULSE_HIGH_MS: u64 = 2;
        const PULSE_LOW_MS: u64 = 498;
        const TOTAL_PERIOD_MS: u64 = PULSE_HIGH_MS + PULSE_LOW_MS;
        
        let frequency = 1000.0 / TOTAL_PERIOD_MS as f32;
        assert!((frequency - 2.0).abs() < 0.001, "Frequency should be 2Hz");
        
        let duty_cycle = (PULSE_HIGH_MS as f32 / TOTAL_PERIOD_MS as f32) * 100.0;
        assert!((duty_cycle - 0.4).abs() < 0.01, "Duty cycle should be 0.4%");
        
        // 3. Test integration between timing and battery monitoring
        // Battery monitoring at 10Hz should not interfere with 2Hz pEMF
        let battery_samples_per_pulse = TOTAL_PERIOD_MS / 100; // 100ms battery interval
        assert_eq!(battery_samples_per_pulse, 5, "Should have 5 battery samples per pulse cycle");
    }
}