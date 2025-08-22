use crate::types::waveform::WaveformConfig;

pub const DEFAULT_WAVEFORM_CONFIG: WaveformConfig = WaveformConfig {
    frequency_hz: 10.0,        // 10Hz default per PRP
    duty_cycle_percent: 33.0,  // 33% default per PRP
    waveform_factor: 0.5,      // Sawtooth default per PRP
    amplitude_percent: 100.0,  // Full power default
};
