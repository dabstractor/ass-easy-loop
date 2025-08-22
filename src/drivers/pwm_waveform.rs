use crate::types::waveform::WaveformConfig;
use rp2040_hal::pwm::{FreeRunning, Slice, SliceId};

pub struct WaveformGenerator<Id: SliceId> {
    pwm: Slice<Id, FreeRunning>,
}

impl<Id: SliceId> WaveformGenerator<Id> {
    pub fn new(pwm: Slice<Id, FreeRunning>) -> Self {
        Self { pwm }
    }

    pub fn set_config(&mut self, config: &WaveformConfig) {
        // Implementation to be added
    }
}
