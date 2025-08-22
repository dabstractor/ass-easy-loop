#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootloaderState {
    Normal,
    PrepareEntry,
    EnteringBootloader,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BootloaderConfig {
    pub activity_pin_mask: u32,
    pub disable_interface_mask: u32,
    pub prep_delay_ms: u32,
}

impl Default for BootloaderConfig {
    fn default() -> Self {
        Self {
            activity_pin_mask: 0,
            disable_interface_mask: 0,
            prep_delay_ms: 100,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootloaderResult {
    Success,
    PrepareError,
    InvalidState,
}
