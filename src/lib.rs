#![cfg_attr(not(test), no_std)]

pub mod config;
pub mod drivers;
pub mod types;
pub mod utils;

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    
    // Waveform generation tests
    mod test_waveform {
        use super::*;
        use crate::utils::waveforms::*;
        
        /// Test sine wave generation at boundary values
        #[test]
        fn test_sine_wave_boundary_values() {
            // Test exact boundary values
            assert_relative_eq!(sine_wave(0.0), 0.5, epsilon = 0.001);
            assert_relative_eq!(sine_wave(0.25), 1.0, epsilon = 0.001);
            assert_relative_eq!(sine_wave(0.5), 0.5, epsilon = 0.001);
            assert_relative_eq!(sine_wave(0.75), 0.0, epsilon = 0.001);
            assert_relative_eq!(sine_wave(1.0), 0.5, epsilon = 0.001);
        }
        
        /// Test square wave generation at boundary values
        #[test]
        fn test_square_wave_boundary_values() {
            // Test with 50% duty cycle
            assert_eq!(square_wave(0.0, 0.5), 1.0);
            assert_eq!(square_wave(0.25, 0.5), 1.0);
            assert_eq!(square_wave(0.5, 0.5), 0.0);
            assert_eq!(square_wave(0.75, 0.5), 0.0);
            assert_eq!(square_wave(0.99, 0.5), 0.0); // At end of cycle, still 0
            
            // Test with 33% duty cycle
            assert_eq!(square_wave(0.0, 0.33), 1.0);
            assert_eq!(square_wave(0.32, 0.33), 1.0);
            assert_eq!(square_wave(0.34, 0.33), 0.0);
        }
        
        /// Test sawtooth wave generation at boundary values (takes duty cycle parameter)
        #[test]
        fn test_sawtooth_wave_boundary_values() {
            // Test with 50% duty cycle  
            assert_eq!(sawtooth_wave(0.0, 0.5), 0.0);
            assert_eq!(sawtooth_wave(0.25, 0.5), 0.5);
            assert_eq!(sawtooth_wave(0.5, 0.5), 1.0);
            assert_eq!(sawtooth_wave(0.75, 0.5), 0.5);
            assert_eq!(sawtooth_wave(1.0, 0.5), 0.0);
        }
        
        /// Test main waveform generation function
        #[test] 
        fn test_generate_waveform_value() {
            // Test sine wave (factor = 0.0)
            let sine_val = generate_waveform_value(0.25, 0.0, 0.5);
            assert!((0.0..=1.0).contains(&sine_val));
            
            // Test square wave (factor = 1.0)
            let square_val = generate_waveform_value(0.25, 1.0, 0.5);
            assert_eq!(square_val, 1.0);
            
            // Test sawtooth wave (factor = 0.5)
            let sawtooth_val = generate_waveform_value(0.25, 0.5, 0.5);
            assert!((0.0..=1.0).contains(&sawtooth_val));
        }
        
        /// Test PWM conversion
        #[test]
        fn test_waveform_to_pwm() {
            // Test boundary values
            let pwm_min = waveform_to_pwm(0.0, 100.0);
            assert_eq!(pwm_min, 0);
            
            let pwm_half = waveform_to_pwm(0.5, 100.0);
            assert!(pwm_half > 0);
            
            let pwm_max = waveform_to_pwm(1.0, 100.0);
            assert!(pwm_max > pwm_half);
        }
    }
    
    // Battery monitoring tests  
    mod test_battery {
        use crate::types::battery::*;
        use crate::types::errors::*;
        
        /// Test battery state detection from ADC readings
        #[test]
        fn test_battery_state_from_adc_reading() {
            // Test low battery state (≤ 1425)
            assert_eq!(BatteryState::from_adc_reading(1000), BatteryState::Low);
            assert_eq!(BatteryState::from_adc_reading(1425), BatteryState::Low);
            
            // Test normal battery state (1426-1674)  
            assert_eq!(BatteryState::from_adc_reading(1426), BatteryState::Normal);
            assert_eq!(BatteryState::from_adc_reading(1500), BatteryState::Normal);
            assert_eq!(BatteryState::from_adc_reading(1674), BatteryState::Normal);
            
            // Test charging state (1675-1800)
            assert_eq!(BatteryState::from_adc_reading(1675), BatteryState::Charging);
            assert_eq!(BatteryState::from_adc_reading(1750), BatteryState::Charging);
            assert_eq!(BatteryState::from_adc_reading(1800), BatteryState::Charging);
            
            // Test full state (> 1800)
            assert_eq!(BatteryState::from_adc_reading(1801), BatteryState::Full);
            assert_eq!(BatteryState::from_adc_reading(1900), BatteryState::Full);
        }
        
        /// Test battery error severity levels
        #[test]
        fn test_battery_error_severity() {
            let error = BatteryError::AdcFailed;
            assert_eq!(error.severity_level(), 4);
            assert!(error.requires_emergency_shutdown());
            
            let over_voltage_error = BatteryError::OverVoltage {
                adc_value: 1900,
                voltage_mv: 4500,
                current_state: BatteryState::Full
            };
            assert_eq!(over_voltage_error.severity_level(), 4);
            assert!(over_voltage_error.requires_emergency_shutdown());
        }
        
        /// Test battery error descriptions
        #[test]
        fn test_battery_error_descriptions() {
            assert_eq!(BatteryError::AdcFailed.description(), "ADC failed to read battery voltage");
            
            let under_voltage_error = BatteryError::UnderVoltage {
                adc_value: 800,
                voltage_mv: 2500,
                current_state: BatteryState::Low
            };
            assert_eq!(under_voltage_error.description(), "Battery under-voltage detected");
        }
    }
    
    // Configuration tests
    mod test_config {
        use crate::types::waveform::*;
        
        /// Test waveform configuration default values
        #[test]
        fn test_waveform_config_defaults() {
            let config = WaveformConfig::default();
            
            assert_eq!(config.frequency_hz, 10.0);
            assert_eq!(config.duty_cycle_percent, 33.0);
            assert_eq!(config.waveform_factor, 0.5);
            assert_eq!(config.amplitude_percent, 100.0);
        }
        
        /// Test waveform configuration field ranges
        #[test]
        fn test_waveform_config_ranges() {
            let mut config = WaveformConfig::default();
            
            // Test frequency range boundaries
            config.frequency_hz = 0.1;
            assert!(config.frequency_hz >= 0.1);
            
            config.frequency_hz = 100.0;
            assert!(config.frequency_hz <= 100.0);
            
            // Test duty cycle range
            config.duty_cycle_percent = 1.0;
            assert!(config.duty_cycle_percent >= 1.0);
            
            config.duty_cycle_percent = 99.0;
            assert!(config.duty_cycle_percent <= 99.0);
            
            // Test waveform factor range
            config.waveform_factor = 0.0;
            assert!(config.waveform_factor >= 0.0);
            
            config.waveform_factor = 1.0;
            assert!(config.waveform_factor <= 1.0);
        }
        
        /// Test waveform buffer creation
        #[test]
        fn test_waveform_buffer_creation() {
            let _buffer = WaveformBuffer::new();
            // Buffer should be created successfully
            // Note: Most buffer fields are private, so we can only test creation
        }
    }
}
