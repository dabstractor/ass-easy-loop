//! Type aliases for consistent numeric types throughout the codebase
//!
//! This module provides type aliases to ensure consistent usage of numeric types
//! across the entire codebase, reducing type conversion errors and improving
//! code clarity.

/// ADC reading value (0-4095 for 12-bit ADC)
pub type AdcValue = u16;

/// Battery voltage in millivolts for precision
pub type VoltageMillivolts = u32;

/// Timestamp in milliseconds since boot
pub type TimestampMs = u32;

/// Percentage value (0.0-100.0)
pub type Percentage = f32;

/// Frequency in Hz
pub type FrequencyHz = f32;

/// Duration in milliseconds
pub type DurationMs = u64;

/// Duration in microseconds for high-precision timing
pub type DurationUs = u32;

/// Memory size in bytes
pub type MemoryBytes = u32;

/// Count of items/operations
pub type Count = u32;

/// Index for arrays and collections
pub type Index = usize;

/// Priority level (0-255)
pub type Priority = u8;

/// GPIO pin number
pub type GpioPin = u8;

/// USB Vendor/Product ID
pub type UsbId = u16;

/// Conversion utilities for type-safe conversions
pub mod conversions {
    use super::*;

    /// ADC to voltage conversions
    pub mod adc {
        use super::*;

        /// Convert ADC value to voltage in millivolts
        /// Uses the standard RP2040 ADC reference voltage of 3.3V
        pub fn adc_to_millivolts(adc_value: AdcValue) -> VoltageMillivolts {
            // ADC is 12-bit (0-4095), reference voltage is 3.3V
            (adc_value as u32 * 3300) / 4095
        }

        /// Convert voltage in millivolts to expected ADC value
        pub fn millivolts_to_adc(voltage_mv: VoltageMillivolts) -> AdcValue {
            let adc_value = (voltage_mv * 4095) / 3300;
            if adc_value > 4095 {
                4095
            } else {
                adc_value as AdcValue
            }
        }

        /// Convert ADC value to battery voltage with calibration
        pub fn battery_voltage_to_adc(voltage_mv: VoltageMillivolts) -> AdcValue {
            // Battery voltage divider: 2:1 ratio, so ADC sees half the battery voltage
            let adc_voltage_mv = voltage_mv / 2;
            millivolts_to_adc(adc_voltage_mv)
        }

        /// Convert battery ADC reading to actual battery voltage
        pub fn adc_to_battery_voltage(adc_value: AdcValue) -> VoltageMillivolts {
            // Battery voltage divider: 2:1 ratio, so multiply by 2
            adc_to_millivolts(adc_value) * 2
        }
    }

    /// Time unit conversions
    pub mod time {
        use super::*;

        /// Convert milliseconds to microseconds
        pub fn ms_to_us(duration_ms: DurationMs) -> DurationUs {
            (duration_ms * 1000) as DurationUs
        }

        /// Convert microseconds to milliseconds
        pub fn us_to_ms(duration_us: DurationUs) -> DurationMs {
            (duration_us / 1000) as DurationMs
        }

        /// Convert seconds to milliseconds
        pub fn seconds_to_ms(seconds: f32) -> DurationMs {
            (seconds * 1000.0) as DurationMs
        }

        /// Convert milliseconds to seconds
        pub fn ms_to_seconds(duration_ms: DurationMs) -> f32 {
            duration_ms as f32 / 1000.0
        }

        /// Convert frequency to period in milliseconds
        pub fn frequency_to_period_ms(frequency_hz: FrequencyHz) -> DurationMs {
            if frequency_hz > 0.0 {
                (1000.0 / frequency_hz) as DurationMs
            } else {
                0
            }
        }

        /// Convert period in milliseconds to frequency
        pub fn period_ms_to_frequency(period_ms: DurationMs) -> FrequencyHz {
            if period_ms > 0 {
                1000.0 / (period_ms as f32)
            } else {
                0.0
            }
        }
    }

    /// Percentage calculations
    pub mod percentage {
        use super::*;

        /// Convert count to percentage of total
        pub fn count_to_percentage(count: Count, total: Count) -> Percentage {
            if total > 0 {
                (count as f32 * 100.0) / total as f32
            } else {
                0.0
            }
        }

        /// Convert ratio to percentage
        pub fn ratio_to_percentage(ratio: f32) -> Percentage {
            ratio * 100.0
        }

        /// Convert percentage to ratio
        pub fn percentage_to_ratio(percentage: Percentage) -> f32 {
            percentage / 100.0
        }

        /// Calculate percentage error between expected and actual values
        pub fn calculate_error_percentage(expected: f32, actual: f32) -> Percentage {
            if expected != 0.0 {
                ((actual - expected) / expected * 100.0).abs()
            } else {
                0.0
            }
        }

        /// Calculate accuracy percentage (100% - error%)
        pub fn calculate_accuracy_percentage(expected: f32, actual: f32) -> Percentage {
            100.0 - calculate_error_percentage(expected, actual)
        }
    }

    /// Type-safe conversions between numeric types
    pub mod numeric {

        /// Safe conversion from f64 to f32 with bounds checking
        pub fn f64_to_f32_safe(value: f64) -> f32 {
            if value > f32::MAX as f64 {
                f32::MAX
            } else if value < f32::MIN as f64 {
                f32::MIN
            } else {
                value as f32
            }
        }

        /// Safe conversion from f32 to f64
        pub fn f32_to_f64(value: f32) -> f64 {
            value as f64
        }

        /// Safe conversion from usize to u8 with bounds checking
        pub fn usize_to_u8_safe(value: usize) -> u8 {
            if value > u8::MAX as usize {
                u8::MAX
            } else {
                value as u8
            }
        }

        /// Safe conversion from u8 to usize
        pub fn u8_to_usize(value: u8) -> usize {
            value as usize
        }

        /// Safe conversion from u32 to u16 with bounds checking
        pub fn u32_to_u16_safe(value: u32) -> u16 {
            if value > u16::MAX as u32 {
                u16::MAX
            } else {
                value as u16
            }
        }

        /// Safe conversion from u16 to u32
        pub fn u16_to_u32(value: u16) -> u32 {
            value as u32
        }

        /// Convert boolean to u8 (0 or 1)
        pub fn bool_to_u8(value: bool) -> u8 {
            if value {
                1
            } else {
                0
            }
        }

        /// Convert u8 to boolean (0 = false, non-zero = true)
        pub fn u8_to_bool(value: u8) -> bool {
            value != 0
        }

        /// Safe conversion from u64 to u32 with bounds checking
        pub fn u64_to_u32_safe(value: u64) -> u32 {
            if value > u32::MAX as u64 {
                u32::MAX
            } else {
                value as u32
            }
        }

        /// Safe conversion from u32 to u8 with bounds checking
        pub fn u32_to_u8_safe(value: u32) -> u8 {
            if value > u8::MAX as u32 {
                u8::MAX
            } else {
                value as u8
            }
        }

        /// Safe conversion from i32 to u32 (negative values become 0)
        pub fn i32_to_u32_safe(value: i32) -> u32 {
            if value < 0 {
                0
            } else {
                value as u32
            }
        }

        /// Convert enum to u8 (for serialization)
        pub fn enum_to_u8<T>(value: T) -> u8
        where
            T: Into<u8>,
        {
            value.into()
        }

        /// Safe division with zero check, returns 0 if divisor is 0
        pub fn safe_divide_u32(dividend: u32, divisor: u32) -> u32 {
            if divisor == 0 {
                0
            } else {
                dividend / divisor
            }
        }

        /// Safe division with zero check, returns 0.0 if divisor is 0
        pub fn safe_divide_f32(dividend: f32, divisor: f32) -> f32 {
            if divisor == 0.0 {
                0.0
            } else {
                dividend / divisor
            }
        }

        /// Calculate percentage with safe division
        pub fn safe_percentage(numerator: u32, denominator: u32) -> f32 {
            if denominator == 0 {
                0.0
            } else {
                (numerator as f32 / denominator as f32) * 100.0
            }
        }

        /// Clamp value between min and max
        pub fn clamp_f32(value: f32, min: f32, max: f32) -> f32 {
            if value < min {
                min
            } else if value > max {
                max
            } else {
                value
            }
        }

        /// Clamp value between min and max
        pub fn clamp_u32(value: u32, min: u32, max: u32) -> u32 {
            if value < min {
                min
            } else if value > max {
                max
            } else {
                value
            }
        }
    }

    // Re-export commonly used conversions at the module level for backward compatibility
    pub use adc::{adc_to_millivolts, millivolts_to_adc};
    pub use percentage::count_to_percentage;
    pub use time::{frequency_to_period_ms, ms_to_us, period_ms_to_frequency, us_to_ms};
}

#[cfg(test)]
mod tests {
    use super::conversions::*;

    #[test]
    fn test_adc_voltage_conversion() {
        // Test basic ADC conversions
        assert_eq!(adc::adc_to_millivolts(0), 0);
        assert_eq!(adc::adc_to_millivolts(4095), 3300);
        assert_eq!(adc::adc_to_millivolts(2048), 1650); // Approximately half

        // Test reverse conversion
        assert_eq!(adc::millivolts_to_adc(0), 0);
        assert_eq!(adc::millivolts_to_adc(3300), 4095);
        assert_eq!(adc::millivolts_to_adc(1650), 2047); // Close to half

        // Test battery voltage conversions (with voltage divider)
        assert_eq!(adc::battery_voltage_to_adc(6600), 4095); // 6.6V battery -> 4095 ADC (3.3V at ADC)
        assert_eq!(adc::adc_to_battery_voltage(2048), 3300); // ADC 2048 -> 3.3V battery
    }

    #[test]
    fn test_time_conversion() {
        // Basic time conversions
        assert_eq!(time::ms_to_us(1), 1000);
        assert_eq!(time::ms_to_us(100), 100_000);
        assert_eq!(time::us_to_ms(1000), 1);
        assert_eq!(time::us_to_ms(100_000), 100);

        // Seconds conversions
        assert_eq!(time::seconds_to_ms(1.0), 1000);
        assert_eq!(time::seconds_to_ms(0.5), 500);
        assert_eq!(time::ms_to_seconds(1000), 1.0);
        assert_eq!(time::ms_to_seconds(500), 0.5);
    }

    #[test]
    fn test_frequency_conversion() {
        assert_eq!(time::frequency_to_period_ms(1.0), 1000);
        assert_eq!(time::frequency_to_period_ms(2.0), 500);
        assert_eq!(time::frequency_to_period_ms(10.0), 100);

        assert_eq!(time::period_ms_to_frequency(1000), 1.0);
        assert_eq!(time::period_ms_to_frequency(500), 2.0);
        assert_eq!(time::period_ms_to_frequency(100), 10.0);

        // Edge cases
        assert_eq!(time::frequency_to_period_ms(0.0), 0);
        assert_eq!(time::period_ms_to_frequency(0), 0.0);
    }

    #[test]
    fn test_percentage_conversion() {
        // Basic percentage calculations
        assert_eq!(percentage::count_to_percentage(50, 100), 50.0);
        assert_eq!(percentage::count_to_percentage(25, 100), 25.0);
        assert_eq!(percentage::count_to_percentage(0, 100), 0.0);
        assert_eq!(percentage::count_to_percentage(100, 100), 100.0);

        // Edge case: division by zero
        assert_eq!(percentage::count_to_percentage(50, 0), 0.0);

        // Ratio conversions
        assert_eq!(percentage::ratio_to_percentage(0.5), 50.0);
        assert_eq!(percentage::ratio_to_percentage(1.0), 100.0);
        assert_eq!(percentage::percentage_to_ratio(50.0), 0.5);
        assert_eq!(percentage::percentage_to_ratio(100.0), 1.0);

        // Error calculations
        assert_eq!(percentage::calculate_error_percentage(100.0, 105.0), 5.0);
        assert_eq!(percentage::calculate_error_percentage(100.0, 95.0), 5.0);
        assert_eq!(
            percentage::calculate_accuracy_percentage(100.0, 105.0),
            95.0
        );
        assert_eq!(percentage::calculate_accuracy_percentage(100.0, 95.0), 95.0);

        // Edge case: expected is zero
        assert_eq!(percentage::calculate_error_percentage(0.0, 5.0), 0.0);
    }

    #[test]
    fn test_numeric_conversions() {
        // f64 to f32 conversions
        assert_eq!(numeric::f64_to_f32_safe(1.5), 1.5f32);
        assert_eq!(numeric::f64_to_f32_safe(f64::MAX), f32::MAX);
        assert_eq!(numeric::f64_to_f32_safe(f64::MIN), f32::MIN);
        assert_eq!(numeric::f32_to_f64(1.5f32), 1.5f64);

        // usize to u8 conversions
        assert_eq!(numeric::usize_to_u8_safe(100), 100u8);
        assert_eq!(numeric::usize_to_u8_safe(300), u8::MAX); // Clamped
        assert_eq!(numeric::u8_to_usize(100u8), 100usize);

        // u32 to u16 conversions
        assert_eq!(numeric::u32_to_u16_safe(1000), 1000u16);
        assert_eq!(numeric::u32_to_u16_safe(70000), u16::MAX); // Clamped
        assert_eq!(numeric::u16_to_u32(1000u16), 1000u32);

        // u64 to u32 conversions
        assert_eq!(numeric::u64_to_u32_safe(1000), 1000u32);
        assert_eq!(numeric::u64_to_u32_safe(u64::MAX), u32::MAX); // Clamped

        // u32 to u8 conversions
        assert_eq!(numeric::u32_to_u8_safe(100), 100u8);
        assert_eq!(numeric::u32_to_u8_safe(300), u8::MAX); // Clamped

        // i32 to u32 conversions
        assert_eq!(numeric::i32_to_u32_safe(100), 100u32);
        assert_eq!(numeric::i32_to_u32_safe(-50), 0u32); // Negative becomes 0

        // Boolean conversions
        assert_eq!(numeric::bool_to_u8(true), 1);
        assert_eq!(numeric::bool_to_u8(false), 0);
        assert_eq!(numeric::u8_to_bool(1), true);
        assert_eq!(numeric::u8_to_bool(0), false);
        assert_eq!(numeric::u8_to_bool(255), true); // Non-zero is true

        // Safe division
        assert_eq!(numeric::safe_divide_u32(100, 10), 10);
        assert_eq!(numeric::safe_divide_u32(100, 0), 0); // Division by zero
        assert_eq!(numeric::safe_divide_f32(100.0, 10.0), 10.0);
        assert_eq!(numeric::safe_divide_f32(100.0, 0.0), 0.0); // Division by zero

        // Safe percentage
        assert_eq!(numeric::safe_percentage(50, 100), 50.0);
        assert_eq!(numeric::safe_percentage(50, 0), 0.0); // Division by zero

        // Clamping
        assert_eq!(numeric::clamp_f32(5.0, 0.0, 10.0), 5.0);
        assert_eq!(numeric::clamp_f32(-5.0, 0.0, 10.0), 0.0); // Clamped to min
        assert_eq!(numeric::clamp_f32(15.0, 0.0, 10.0), 10.0); // Clamped to max

        assert_eq!(numeric::clamp_u32(5, 0, 10), 5);
        assert_eq!(numeric::clamp_u32(15, 0, 10), 10); // Clamped to max
    }

    #[test]
    fn test_backward_compatibility() {
        // Test that the re-exported functions still work
        assert_eq!(adc_to_millivolts(2048), 1650);
        assert_eq!(millivolts_to_adc(1650), 2047);
        assert_eq!(ms_to_us(1), 1000);
        assert_eq!(us_to_ms(1000), 1);
        assert_eq!(frequency_to_period_ms(2.0), 500);
        assert_eq!(period_ms_to_frequency(500), 2.0);
        assert_eq!(count_to_percentage(50, 100), 50.0);
    }
}
