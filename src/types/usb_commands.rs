#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UsbCommand {
    SetFrequency(u32),
    SetDutyCycle(u8),
    EnterBootloader,
}