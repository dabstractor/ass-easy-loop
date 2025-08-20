use crate::types::waveform::WaveformConfig;

pub const DEFAULT_WAVEFORM_CONFIG: WaveformConfig = WaveformConfig {
    frequency: 10,
    duty_cycle: 33,
    waveform_factor: 0.5,
    amplitude: 4095,
};