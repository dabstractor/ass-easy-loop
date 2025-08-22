/// Battery Performance Validation Tests
/// 
/// CRITICAL TIMING VALIDATION FRAMEWORK
/// These tests ensure battery charging does not degrade pEMF timing accuracy
/// Required tolerance: ±1% pEMF timing accuracy during all charging phases

#[cfg(test)]
mod performance_validation {
    use core::time::Duration;
    
    /// Performance test configuration
    pub struct PerformanceTestConfig {
        pub pemf_target_frequency_hz: u32,      // Target pEMF frequency
        pub pemf_timing_tolerance_percent: f32,  // ±1% timing tolerance
        pub adc_sampling_rate_hz: u16,          // ADC sampling during charging
        pub test_duration_minutes: u16,         // Test duration
        pub charge_phases_to_test: Vec<ChargePhase>, // Which charge phases to validate
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum ChargePhase {
        NotCharging,
        PreCharge,       // < 3.0V, 100mA
        ConstantCurrent, // 3.0V-4.2V, 1A
        ConstantVoltage, // 4.2V, decreasing current
        ChargingComplete,
    }

    impl Default for PerformanceTestConfig {
        fn default() -> Self {
            Self {
                pemf_target_frequency_hz: 1000,      // 1kHz example
                pemf_timing_tolerance_percent: 1.0,   // ±1% as specified
                adc_sampling_rate_hz: 10,            // Baseline 10Hz
                test_duration_minutes: 5,            // 5 minute test cycles
                charge_phases_to_test: vec![
                    ChargePhase::NotCharging,
                    ChargePhase::ConstantCurrent,
                    ChargePhase::ConstantVoltage,
                ],
            }
        }
    }

    /// Timing measurement result
    #[derive(Debug, Clone)]
    pub struct TimingMeasurement {
        pub timestamp_ms: u32,
        pub measured_frequency_hz: f32,
        pub timing_error_percent: f32,
        pub charge_phase: ChargePhase,
        pub battery_voltage_mv: u16,
        pub adc_value: u16,
    }

    impl TimingMeasurement {
        pub fn new(
            timestamp_ms: u32,
            measured_frequency_hz: f32,
            target_frequency_hz: u32,
            charge_phase: ChargePhase,
            battery_voltage_mv: u16,
            adc_value: u16,
        ) -> Self {
            let timing_error_percent = 
                ((measured_frequency_hz - target_frequency_hz as f32) / target_frequency_hz as f32) * 100.0;
            
            Self {
                timestamp_ms,
                measured_frequency_hz,
                timing_error_percent,
                charge_phase,
                battery_voltage_mv,
                adc_value,
            }
        }

        pub fn is_within_tolerance(&self, tolerance_percent: f32) -> bool {
            self.timing_error_percent.abs() <= tolerance_percent
        }
    }

    /// Performance test result aggregation
    #[derive(Debug)]
    pub struct PerformanceTestResult {
        pub test_name: String,
        pub passed: bool,
        pub measurements: Vec<TimingMeasurement>,
        pub max_timing_error_percent: f32,
        pub average_timing_error_percent: f32,
        pub out_of_tolerance_count: u32,
        pub test_duration_ms: u64,
        pub charge_phase_coverage: Vec<ChargePhase>,
    }

    impl PerformanceTestResult {
        pub fn new(test_name: &str) -> Self {
            Self {
                test_name: test_name.to_string(),
                passed: false,
                measurements: Vec::new(),
                max_timing_error_percent: 0.0,
                average_timing_error_percent: 0.0,
                out_of_tolerance_count: 0,
                test_duration_ms: 0,
                charge_phase_coverage: Vec::new(),
            }
        }

        pub fn add_measurement(&mut self, measurement: TimingMeasurement) {
            self.measurements.push(measurement);
        }

        pub fn analyze_results(&mut self, tolerance_percent: f32) {
            if self.measurements.is_empty() {
                return;
            }

            // Calculate max and average timing error
            self.max_timing_error_percent = self.measurements.iter()
                .map(|m| m.timing_error_percent.abs())
                .fold(0.0, f32::max);

            let sum_error: f32 = self.measurements.iter()
                .map(|m| m.timing_error_percent)
                .sum();
            self.average_timing_error_percent = sum_error / self.measurements.len() as f32;

            // Count out-of-tolerance measurements
            self.out_of_tolerance_count = self.measurements.iter()
                .filter(|m| !m.is_within_tolerance(tolerance_percent))
                .count() as u32;

            // Determine pass/fail
            self.passed = self.out_of_tolerance_count == 0 && 
                         self.max_timing_error_percent <= tolerance_percent;

            // Track charge phase coverage
            let mut covered_phases = Vec::new();
            for measurement in &self.measurements {
                if !covered_phases.contains(&measurement.charge_phase) {
                    covered_phases.push(measurement.charge_phase);
                }
            }
            self.charge_phase_coverage = covered_phases;
        }

        pub fn get_statistics_by_phase(&self) -> Vec<(ChargePhase, f32, f32)> {
            let mut phase_stats = Vec::new();
            
            for &phase in &self.charge_phase_coverage {
                let phase_measurements: Vec<&TimingMeasurement> = self.measurements.iter()
                    .filter(|m| m.charge_phase == phase)
                    .collect();

                if !phase_measurements.is_empty() {
                    let avg_error: f32 = phase_measurements.iter()
                        .map(|m| m.timing_error_percent)
                        .sum::<f32>() / phase_measurements.len() as f32;

                    let max_error: f32 = phase_measurements.iter()
                        .map(|m| m.timing_error_percent.abs())
                        .fold(0.0, f32::max);

                    phase_stats.push((phase, avg_error, max_error));
                }
            }
            
            phase_stats
        }
    }

    // Performance test implementations

    /// CRITICAL TEST: pEMF timing during constant current charging
    pub fn test_pemf_timing_constant_current_phase() -> PerformanceTestResult {
        let mut result = PerformanceTestResult::new("pEMF Timing - Constant Current Phase");
        let config = PerformanceTestConfig::default();
        
        // Simulate measurements during constant current charging (3.0V - 4.2V)
        // In real implementation, this would:
        // 1. Start pEMF generation at target frequency
        // 2. Begin constant current charging
        // 3. Measure actual pEMF timing via external measurement or internal counters
        // 4. Record timing accuracy throughout the charge phase
        
        let test_measurements = simulate_cc_phase_measurements(&config);
        
        for measurement in test_measurements {
            result.add_measurement(measurement);
        }
        
        result.analyze_results(config.pemf_timing_tolerance_percent);
        result
    }

    /// CRITICAL TEST: pEMF timing during constant voltage charging
    pub fn test_pemf_timing_constant_voltage_phase() -> PerformanceTestResult {
        let mut result = PerformanceTestResult::new("pEMF Timing - Constant Voltage Phase");
        let config = PerformanceTestConfig::default();
        
        // Simulate measurements during constant voltage charging (4.2V, decreasing current)
        let test_measurements = simulate_cv_phase_measurements(&config);
        
        for measurement in test_measurements {
            result.add_measurement(measurement);
        }
        
        result.analyze_results(config.pemf_timing_tolerance_percent);
        result
    }

    /// CRITICAL TEST: ADC sampling impact on pEMF timing
    pub fn test_adc_sampling_timing_impact() -> PerformanceTestResult {
        let mut result = PerformanceTestResult::new("ADC Sampling Impact on pEMF Timing");
        let mut config = PerformanceTestConfig::default();
        
        // Test different ADC sampling rates during charging
        let sampling_rates = [10, 50, 100]; // Hz
        
        for &rate in &sampling_rates {
            config.adc_sampling_rate_hz = rate;
            let measurements = simulate_adc_impact_measurements(&config);
            
            for measurement in measurements {
                result.add_measurement(measurement);
            }
        }
        
        result.analyze_results(config.pemf_timing_tolerance_percent);
        result
    }

    /// CRITICAL TEST: Task priority interference during charging
    pub fn test_task_priority_interference() -> PerformanceTestResult {
        let mut result = PerformanceTestResult::new("Task Priority Interference");
        let config = PerformanceTestConfig::default();
        
        // Test pEMF timing with various system loads during charging
        // Simulates high-priority interrupts, USB activity, logging, etc.
        
        let measurements = simulate_task_interference_measurements(&config);
        
        for measurement in measurements {
            result.add_measurement(measurement);
        }
        
        result.analyze_results(config.pemf_timing_tolerance_percent);
        result
    }

    /// STRESS TEST: Long-term timing stability during extended charging
    pub fn test_long_term_timing_stability() -> PerformanceTestResult {
        let mut result = PerformanceTestResult::new("Long-term Timing Stability");
        let mut config = PerformanceTestConfig::default();
        config.test_duration_minutes = 60; // 1 hour test
        
        let measurements = simulate_long_term_measurements(&config);
        
        for measurement in measurements {
            result.add_measurement(measurement);
        }
        
        result.analyze_results(config.pemf_timing_tolerance_percent);
        result
    }

    /// INTEGRATION TEST: Complete charge cycle timing validation
    pub fn test_complete_charge_cycle_timing() -> PerformanceTestResult {
        let mut result = PerformanceTestResult::new("Complete Charge Cycle Timing");
        let config = PerformanceTestConfig::default();
        
        // Simulate entire charge cycle: Pre-charge -> CC -> CV -> Complete
        let mut all_measurements = Vec::new();
        
        all_measurements.extend(simulate_precharge_measurements(&config));
        all_measurements.extend(simulate_cc_phase_measurements(&config));
        all_measurements.extend(simulate_cv_phase_measurements(&config));
        all_measurements.extend(simulate_charge_complete_measurements(&config));
        
        for measurement in all_measurements {
            result.add_measurement(measurement);
        }
        
        result.analyze_results(config.pemf_timing_tolerance_percent);
        result
    }

    // Simulation functions (replace with real measurements in actual implementation)
    
    fn simulate_cc_phase_measurements(config: &PerformanceTestConfig) -> Vec<TimingMeasurement> {
        let mut measurements = Vec::new();
        let test_points = 50; // 50 measurement points
        
        for i in 0..test_points {
            let timestamp_ms = i * 1000; // 1 second intervals
            let voltage_mv = 3000 + (i * 24); // 3.0V to 4.2V progression
            let adc_value = (voltage_mv as f32 * 0.337 * 4095.0 / 3300.0) as u16;
            
            // Simulate slight timing variation during charging (should be < 1%)
            let timing_variation = (i as f32 * 0.01) % 0.8; // Max 0.8% variation
            let measured_freq = config.pemf_target_frequency_hz as f32 * (1.0 + timing_variation / 100.0);
            
            let measurement = TimingMeasurement::new(
                timestamp_ms,
                measured_freq,
                config.pemf_target_frequency_hz,
                ChargePhase::ConstantCurrent,
                voltage_mv,
                adc_value,
            );
            
            measurements.push(measurement);
        }
        
        measurements
    }

    fn simulate_cv_phase_measurements(config: &PerformanceTestConfig) -> Vec<TimingMeasurement> {
        let mut measurements = Vec::new();
        let test_points = 30; // 30 measurement points
        
        for i in 0..test_points {
            let timestamp_ms = i * 2000; // 2 second intervals
            let voltage_mv = 4200; // Constant voltage phase
            let adc_value = (voltage_mv as f32 * 0.337 * 4095.0 / 3300.0) as u16;
            
            // CV phase might have slightly different timing characteristics
            let timing_variation = (i as f32 * 0.005) % 0.6; // Max 0.6% variation
            let measured_freq = config.pemf_target_frequency_hz as f32 * (1.0 - timing_variation / 100.0);
            
            let measurement = TimingMeasurement::new(
                timestamp_ms,
                measured_freq,
                config.pemf_target_frequency_hz,
                ChargePhase::ConstantVoltage,
                voltage_mv,
                adc_value,
            );
            
            measurements.push(measurement);
        }
        
        measurements
    }

    fn simulate_adc_impact_measurements(config: &PerformanceTestConfig) -> Vec<TimingMeasurement> {
        let mut measurements = Vec::new();
        let test_points = 20;
        
        // Higher ADC sampling rates might affect timing slightly
        let adc_impact = match config.adc_sampling_rate_hz {
            10 => 0.1,   // 0.1% impact at 10Hz
            50 => 0.3,   // 0.3% impact at 50Hz  
            100 => 0.5,  // 0.5% impact at 100Hz
            _ => 0.2,    // Default impact
        };
        
        for i in 0..test_points {
            let timestamp_ms = i * 500; // 500ms intervals
            let voltage_mv = 3600; // Mid-charge voltage
            let adc_value = (voltage_mv as f32 * 0.337 * 4095.0 / 3300.0) as u16;
            
            let measured_freq = config.pemf_target_frequency_hz as f32 * (1.0 - adc_impact / 100.0);
            
            let measurement = TimingMeasurement::new(
                timestamp_ms,
                measured_freq,
                config.pemf_target_frequency_hz,
                ChargePhase::ConstantCurrent,
                voltage_mv,
                adc_value,
            );
            
            measurements.push(measurement);
        }
        
        measurements
    }

    fn simulate_task_interference_measurements(config: &PerformanceTestConfig) -> Vec<TimingMeasurement> {
        let mut measurements = Vec::new();
        let test_points = 40;
        
        for i in 0..test_points {
            let timestamp_ms = i * 750; // 750ms intervals
            let voltage_mv = 3800; // Charging voltage
            let adc_value = (voltage_mv as f32 * 0.337 * 4095.0 / 3300.0) as u16;
            
            // Simulate task interference spikes
            let interference = if i % 10 == 0 { 0.7 } else { 0.2 }; // Periodic interference
            let measured_freq = config.pemf_target_frequency_hz as f32 * (1.0 - interference / 100.0);
            
            let measurement = TimingMeasurement::new(
                timestamp_ms,
                measured_freq,
                config.pemf_target_frequency_hz,
                ChargePhase::ConstantCurrent,
                voltage_mv,
                adc_value,
            );
            
            measurements.push(measurement);
        }
        
        measurements
    }

    fn simulate_long_term_measurements(config: &PerformanceTestConfig) -> Vec<TimingMeasurement> {
        let mut measurements = Vec::new();
        let test_points = (config.test_duration_minutes * 60 / 10) as u32; // Every 10 seconds
        
        for i in 0..test_points {
            let timestamp_ms = i * 10000; // 10 second intervals
            let voltage_mv = 3000 + ((i * 1200) / test_points); // Gradual voltage rise
            let adc_value = (voltage_mv as f32 * 0.337 * 4095.0 / 3300.0) as u16;
            
            // Long-term drift simulation (should remain < 1%)
            let drift = ((i as f32 / test_points as f32) * 0.5).sin() * 0.3; // Sinusoidal drift
            let measured_freq = config.pemf_target_frequency_hz as f32 * (1.0 + drift / 100.0);
            
            let phase = if voltage_mv < 4200 { 
                ChargePhase::ConstantCurrent 
            } else { 
                ChargePhase::ConstantVoltage 
            };
            
            let measurement = TimingMeasurement::new(
                timestamp_ms,
                measured_freq,
                config.pemf_target_frequency_hz,
                phase,
                voltage_mv,
                adc_value,
            );
            
            measurements.push(measurement);
        }
        
        measurements
    }

    fn simulate_precharge_measurements(config: &PerformanceTestConfig) -> Vec<TimingMeasurement> {
        let mut measurements = Vec::new();
        let test_points = 10;
        
        for i in 0..test_points {
            let timestamp_ms = i * 3000; // 3 second intervals (pre-charge is short)
            let voltage_mv = 2800 + (i * 20); // 2.8V to 3.0V progression
            let adc_value = (voltage_mv as f32 * 0.337 * 4095.0 / 3300.0) as u16;
            
            // Pre-charge should have minimal timing impact
            let measured_freq = config.pemf_target_frequency_hz as f32 * 1.001; // 0.1% variation
            
            let measurement = TimingMeasurement::new(
                timestamp_ms,
                measured_freq,
                config.pemf_target_frequency_hz,
                ChargePhase::PreCharge,
                voltage_mv,
                adc_value,
            );
            
            measurements.push(measurement);
        }
        
        measurements
    }

    fn simulate_charge_complete_measurements(config: &PerformanceTestConfig) -> Vec<TimingMeasurement> {
        let mut measurements = Vec::new();
        let test_points = 5;
        
        for i in 0..test_points {
            let timestamp_ms = i * 5000; // 5 second intervals
            let voltage_mv = 4200; // Full charge voltage
            let adc_value = (voltage_mv as f32 * 0.337 * 4095.0 / 3300.0) as u16;
            
            // Charging complete - should return to baseline timing
            let measured_freq = config.pemf_target_frequency_hz as f32; // Exact timing
            
            let measurement = TimingMeasurement::new(
                timestamp_ms,
                measured_freq,
                config.pemf_target_frequency_hz,
                ChargePhase::ChargingComplete,
                voltage_mv,
                adc_value,
            );
            
            measurements.push(measurement);
        }
        
        measurements
    }
}

/// Unit tests for performance validation framework
#[cfg(test)]
mod performance_unit_tests {
    use super::performance_validation::*;

    #[test]
    fn test_timing_measurement_creation() {
        let measurement = TimingMeasurement::new(
            1000,    // timestamp_ms
            999.5,   // measured_frequency_hz 
            1000,    // target_frequency_hz
            ChargePhase::ConstantCurrent,
            3600,    // battery_voltage_mv
            1675,    // adc_value
        );
        
        assert_eq!(measurement.timestamp_ms, 1000);
        assert!((measurement.timing_error_percent - (-0.05)).abs() < 0.001);
        assert_eq!(measurement.charge_phase, ChargePhase::ConstantCurrent);
        assert!(measurement.is_within_tolerance(1.0));
    }

    #[test]
    fn test_performance_result_analysis() {
        let mut result = PerformanceTestResult::new("Test Analysis");
        
        // Add measurements within tolerance
        result.add_measurement(TimingMeasurement::new(
            1000, 1000.5, 1000, ChargePhase::ConstantCurrent, 3600, 1675
        ));
        result.add_measurement(TimingMeasurement::new(
            2000, 999.2, 1000, ChargePhase::ConstantCurrent, 3700, 1690  
        ));
        
        result.analyze_results(1.0);
        
        assert!(result.passed);
        assert_eq!(result.out_of_tolerance_count, 0);
        assert!(result.max_timing_error_percent < 1.0);
    }

    #[test] 
    fn test_out_of_tolerance_detection() {
        let mut result = PerformanceTestResult::new("Out of Tolerance Test");
        
        // Add measurement outside tolerance
        result.add_measurement(TimingMeasurement::new(
            1000, 1015.0, 1000, ChargePhase::ConstantCurrent, 3600, 1675 // 1.5% error
        ));
        
        result.analyze_results(1.0);
        
        assert!(!result.passed);
        assert_eq!(result.out_of_tolerance_count, 1);
        assert!(result.max_timing_error_percent > 1.0);
    }

    #[test]
    fn test_phase_statistics() {
        let mut result = PerformanceTestResult::new("Phase Statistics Test");
        
        // Add measurements for different phases
        result.add_measurement(TimingMeasurement::new(
            1000, 1000.2, 1000, ChargePhase::ConstantCurrent, 3600, 1675
        ));
        result.add_measurement(TimingMeasurement::new(
            2000, 999.8, 1000, ChargePhase::ConstantVoltage, 4200, 1770
        ));
        
        result.analyze_results(1.0);
        let stats = result.get_statistics_by_phase();
        
        assert_eq!(stats.len(), 2);
        assert!(stats.iter().any(|(phase, _, _)| *phase == ChargePhase::ConstantCurrent));
        assert!(stats.iter().any(|(phase, _, _)| *phase == ChargePhase::ConstantVoltage));
    }
}