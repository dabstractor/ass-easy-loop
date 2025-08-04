//! Integration tests module
//!
//! This module contains host-side integration tests that test component
//! interactions and system behavior using std and comprehensive mocks.

// Import common test utilities
use crate::common;

// Re-export common utilities for integration tests
pub use common::*;

#[cfg(test)]
mod test_infrastructure {
    use super::*;

    #[test]
    fn test_mock_test_environment() {
        let env = MockTestEnvironment::new();

        // Test that all components are available
        env.battery.set_voltage(3500);
        env.usb_hid.send_message(vec![1, 2, 3]);
        env.system_state.set_state("test", "value");
        env.bootloader.enter_bootloader_mode();

        // Verify state
        assert_eq!(env.battery.get_voltage(), 3500);
        assert_eq!(env.usb_hid.get_sent_messages().len(), 1);
        assert_eq!(
            env.system_state.get_state("test"),
            Some("value".to_string())
        );
        let _ = env.bootloader.prepare_for_bootloader_entry();
        assert!(env.bootloader.is_in_bootloader_mode());

        // Test reset functionality
        env.reset();
        assert_eq!(env.battery.get_readings().len(), 0);
        assert_eq!(env.usb_hid.get_sent_messages().len(), 0);
        assert_eq!(env.system_state.get_state("test"), None);
        assert_eq!(env.bootloader.get_flash_operations().len(), 0);
    }

    #[test]
    fn test_system_simulation() {
        let env = MockTestEnvironment::new();

        // Simulate a battery discharge scenario
        let voltages = test_data::battery::discharge_sequence(4000, 3200, 10);
        for voltage in voltages {
            env.battery.set_voltage(voltage);

            // Simulate system response to battery level
            if voltage < 3300 {
                env.system_state.set_state("power_mode", "low_power");
            } else {
                env.system_state.set_state("power_mode", "normal");
            }
        }

        // Verify final state
        assert_eq!(env.battery.get_voltage(), 3280);
        assert_eq!(
            env.system_state.get_state("power_mode"),
            Some("low_power".to_string())
        );
        assert_eq!(env.battery.get_readings().len(), 10);
    }

    #[test]
    fn test_usb_communication_flow() {
        let env = MockTestEnvironment::new();

        // Simulate USB command sequence
        let commands = test_data::usb_hid::config_messages();
        for command in commands {
            env.usb_hid.send_message(command);
        }

        // Verify all commands were sent
        let sent_messages = env.usb_hid.get_sent_messages();
        assert_eq!(sent_messages.len(), 7); // Number of config messages

        // Verify command types
        assert_eq!(sent_messages[0].data[0], 0x01); // GetConfig
        assert_eq!(sent_messages[1].data[0], 0x02); // SetConfig
        assert_eq!(sent_messages[6].data[0], 0x07); // GetStats
    }
}
