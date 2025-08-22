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
