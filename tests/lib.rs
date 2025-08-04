//! Test library for ass-easy-loop
//!
//! This module provides the main entry point for the restructured test suite.
//! It organizes tests into unit, integration, and embedded categories with
//! shared common utilities.

// Enable std for host-side testing
extern crate std;

// Import the main library
extern crate ass_easy_loop;

// Common test utilities (always available for host-side tests)
pub mod common;

// Test categories
pub mod integration;
pub mod unit;

// Embedded tests (only available with hardware-tests feature)
#[cfg(feature = "hardware-tests")]
pub mod embedded;

// Re-export commonly used items for convenience
pub use common::*;

#[cfg(test)]
mod test_suite_validation {
    use super::*;

    #[test]
    fn test_suite_structure_validation() {
        // Validate that the test suite structure is properly set up

        // Test that common utilities are available
        let _env = MockTestEnvironment::new();

        // Test that we can create test data
        let _battery_data = test_data::battery::discharge_sequence(4000, 3000, 5);
        let _usb_data = test_data::usb_hid::create_test_message(0x01, &[1, 2, 3]);

        // Test that assertions work
        assert_in_range!(50, 0, 100);
        assert_approx_eq!(3.14159, 3.14159, 1e-5);

        // Test timing utilities
        // use std::time::Duration;
        let (result, _duration) = timing::measure_time(|| 42);
        assert_eq!(result, 42);

        println!("✓ Test suite structure validation passed");
    }

    #[test]
    fn test_mock_infrastructure_validation() {
        let env = MockTestEnvironment::new();

        // Test battery mock
        env.battery.set_voltage(3500);
        assert_eq!(env.battery.get_voltage(), 3500);

        // Test USB HID mock
        env.usb_hid.send_message(vec![1, 2, 3]);
        assert_eq!(env.usb_hid.get_sent_messages().len(), 1);

        // Test system state mock
        env.system_state.set_state("test", "value");
        assert_eq!(
            env.system_state.get_state("test"),
            Some("value".to_string())
        );

        // Test bootloader mock
        let _ = env.bootloader.prepare_for_bootloader_entry();
        assert!(env.bootloader.is_in_bootloader_mode());
        let result = env.bootloader.enter_bootloader_mode();
        assert!(result.is_ok());

        println!("✓ Mock infrastructure validation passed");
    }

    #[test]
    fn test_data_generators_validation() {
        // Test battery data generators
        let discharge = test_data::battery::discharge_sequence(4000, 3000, 5);
        assert_eq!(discharge.len(), 5);
        assert!(discharge[0] >= discharge[4]); // Should be decreasing

        let charge = test_data::battery::charge_sequence(3000, 4000, 5);
        assert_eq!(charge.len(), 5);
        assert!(charge[0] <= charge[4]); // Should be increasing

        // Test USB HID data generators
        let messages = test_data::usb_hid::config_messages();
        assert_eq!(messages.len(), 7); // Should have 7 config message types

        // Test timing data generators
        let timestamps = test_data::timing::timestamp_sequence(0, 100, 5);
        assert_eq!(timestamps.len(), 5);
        assert_eq!(timestamps[0], 0);
        assert_eq!(timestamps[4], 400);

        println!("✓ Test data generators validation passed");
    }

    #[test]
    fn test_assertion_macros_validation() {
        // Test range assertions
        assert_in_range!(50, 0, 100);

        // Test approximate equality
        assert_approx_eq!(3.14159, 3.14159, 1e-5);

        // Test duration assertions
        use std::time::Duration;
        let duration = Duration::from_millis(100);
        assert_duration_in_range!(
            duration,
            Duration::from_millis(50),
            Duration::from_millis(150)
        );

        // Test collection assertions
        let vec = vec![1, 2, 3, 4, 5];
        assert_len!(vec, 5);
        assert_contains_all!(vec, [1, 3, 5]);

        // Test result assertions
        let ok_result: Result<i32, &str> = Ok(42);
        let value = assert_ok!(ok_result);
        assert_eq!(value, 42);

        let err_result: Result<i32, &str> = Err("test error");
        let error = assert_err!(err_result);
        assert_eq!(error, "test error");

        println!("✓ Assertion macros validation passed");
    }
}
