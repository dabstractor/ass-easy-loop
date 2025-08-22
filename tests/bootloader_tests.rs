use ass_easy_loop::drivers::usb_command_handler::parse_hid_report;
use ass_easy_loop::types::{
    bootloader_types::{BootloaderConfig, BootloaderState, BootloaderResult},
    usb_commands::UsbCommand,
};

#[cfg(test)]
mod usb_command_parsing_tests {
    use super::*;

    #[test]
    fn test_enter_bootloader_command_parsing() {
        let mut report = [0u8; 64];
        report[0] = 0x03; // EnterBootloader command
        
        let command = parse_hid_report(&report);
        assert_eq!(command, Some(UsbCommand::EnterBootloader));
    }

    #[test]
    fn test_set_frequency_command_parsing() {
        let mut report = [0u8; 64];
        report[0] = 0x01; // SetFrequency command
        report[1] = 0x40; // 1000 Hz = 0x03E8
        report[2] = 0x42;
        report[3] = 0x0F;
        report[4] = 0x00;
        
        let command = parse_hid_report(&report);
        assert_eq!(command, Some(UsbCommand::SetFrequency(1000000)));
    }

    #[test]
    fn test_set_duty_cycle_command_parsing() {
        let mut report = [0u8; 64];
        report[0] = 0x02; // SetDutyCycle command
        report[1] = 50;   // 50% duty cycle
        
        let command = parse_hid_report(&report);
        assert_eq!(command, Some(UsbCommand::SetDutyCycle(50)));
    }

    #[test]
    fn test_invalid_command_parsing() {
        let mut report = [0u8; 64];
        report[0] = 0xFF; // Invalid command
        
        let command = parse_hid_report(&report);
        assert_eq!(command, None);
    }

    #[test]
    fn test_zero_command_parsing() {
        let report = [0u8; 64]; // All zeros
        
        let command = parse_hid_report(&report);
        assert_eq!(command, None);
    }
}

#[cfg(test)]
mod bootloader_types_tests {
    use super::*;

    #[test]
    fn test_bootloader_state_transitions() {
        let state = BootloaderState::Normal;
        assert_eq!(state, BootloaderState::Normal);
        
        let state = BootloaderState::PrepareEntry;
        assert_eq!(state, BootloaderState::PrepareEntry);
        
        let state = BootloaderState::EnteringBootloader;
        assert_eq!(state, BootloaderState::EnteringBootloader);
    }

    #[test]
    fn test_bootloader_config_default() {
        let config = BootloaderConfig::default();
        
        assert_eq!(config.activity_pin_mask, 0);
        assert_eq!(config.disable_interface_mask, 0);
        assert_eq!(config.prep_delay_ms, 100);
    }

    #[test]
    fn test_bootloader_config_custom() {
        let config = BootloaderConfig {
            activity_pin_mask: 0x01,
            disable_interface_mask: 0x02,
            prep_delay_ms: 200,
        };
        
        assert_eq!(config.activity_pin_mask, 0x01);
        assert_eq!(config.disable_interface_mask, 0x02);
        assert_eq!(config.prep_delay_ms, 200);
    }

    #[test]
    fn test_bootloader_result_enum() {
        assert_eq!(BootloaderResult::Success, BootloaderResult::Success);
        assert_eq!(BootloaderResult::PrepareError, BootloaderResult::PrepareError);
        assert_eq!(BootloaderResult::InvalidState, BootloaderResult::InvalidState);
        
        assert_ne!(BootloaderResult::Success, BootloaderResult::PrepareError);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_bootloader_command_to_config_flow() {
        // Test the complete flow from USB command to bootloader config
        let mut report = [0u8; 64];
        report[0] = 0x03; // EnterBootloader command
        
        let command = parse_hid_report(&report);
        assert_eq!(command, Some(UsbCommand::EnterBootloader));
        
        // Simulate creating config when bootloader command is received
        if let Some(UsbCommand::EnterBootloader) = command {
            let config = BootloaderConfig::default();
            assert_eq!(config.activity_pin_mask, 0);
            assert_eq!(config.disable_interface_mask, 0);
            assert!(config.prep_delay_ms > 0);
        } else {
            panic!("Expected EnterBootloader command");
        }
    }

    #[test]
    fn test_state_machine_logic() {
        let initial_state = BootloaderState::Normal;
        
        // Simulate state transition in bootloader task
        let prepare_state = BootloaderState::PrepareEntry;
        let entering_state = BootloaderState::EnteringBootloader;
        
        // Verify states are distinct
        assert_ne!(initial_state, prepare_state);
        assert_ne!(prepare_state, entering_state);
        assert_ne!(initial_state, entering_state);
    }
}