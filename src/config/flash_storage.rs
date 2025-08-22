use crate::types::errors::SystemError;
use crate::types::waveform::WaveformConfig;
use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};

pub struct ConfigStorage<F: NorFlash + ReadNorFlash> {
    flash: F,
}

impl<F: NorFlash + ReadNorFlash> ConfigStorage<F> {
    pub fn new(flash: F) -> Self {
        Self { flash }
    }

    pub fn save_config(&mut self, _config: &WaveformConfig) -> Result<(), SystemError> {
        // Implementation to be added
        Ok(())
    }

    pub fn load_config(&mut self) -> Result<WaveformConfig, SystemError> {
        // Implementation to be added
        Ok(crate::config::defaults::DEFAULT_WAVEFORM_CONFIG)
    }
}
