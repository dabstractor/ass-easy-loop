#[cfg(feature = "usb-logs")]
use crate::types::logging::{LogCategory, LogLevel, LoggingUsbCommand};
use crate::types::usb_commands::UsbCommand;

pub fn parse_hid_report(report: &[u8; 64]) -> Option<UsbCommand> {
    match report[0] {
        0x01 => Some(UsbCommand::SetFrequency(u32::from_le_bytes([
            report[1], report[2], report[3], report[4],
        ]))),
        0x02 => Some(UsbCommand::SetDutyCycle(report[1])),
        0x03 => Some(UsbCommand::EnterBootloader),
        _ => None,
    }
}

#[cfg(feature = "usb-logs")]
pub fn parse_logging_hid_report(report: &[u8; 64]) -> Option<LoggingUsbCommand> {
    match report[0] {
        0x10 => Some(LoggingUsbCommand::SetLogLevel(match report[1] {
            0 => LogLevel::Debug,
            1 => LogLevel::Info,
            2 => LogLevel::Warn,
            3 => LogLevel::Error,
            _ => LogLevel::Info,
        })),
        0x11 => Some(LoggingUsbCommand::EnableCategory(match report[1] {
            0 => LogCategory::Battery,
            1 => LogCategory::Pemf,
            2 => LogCategory::System,
            3 => LogCategory::Usb,
            _ => LogCategory::System,
        })),
        0x12 => Some(LoggingUsbCommand::DisableCategory(match report[1] {
            0 => LogCategory::Battery,
            1 => LogCategory::Pemf,
            2 => LogCategory::System,
            3 => LogCategory::Usb,
            _ => LogCategory::System,
        })),
        0x13 => {
            let config = crate::types::logging::LoggingConfig {
                enabled_categories: report[1],
                verbosity_level: match report[2] {
                    0 => LogLevel::Debug,
                    1 => LogLevel::Info,
                    2 => LogLevel::Warn,
                    3 => LogLevel::Error,
                    _ => LogLevel::Info,
                },
                enabled: report[3] != 0,
            };
            Some(LoggingUsbCommand::SetConfig(config))
        }
        0x14 => Some(LoggingUsbCommand::GetConfig),
        _ => None,
    }
}
