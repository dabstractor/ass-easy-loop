use crate::types::waveform::WaveformConfig;

pub fn validate_config(config: &WaveformConfig) -> bool {
    // Validate frequency range per PRP: 0.1Hz to 100Hz
    config.frequency_hz >= 0.1 && config.frequency_hz <= 100.0 &&
    // Validate duty cycle range per PRP: 1% to 99%
    config.duty_cycle_percent >= 1.0 && config.duty_cycle_percent <= 99.0 &&
    // Validate waveform factor range: 0.0 to 1.0 (sine to square)
    config.waveform_factor >= 0.0 && config.waveform_factor <= 1.0 &&
    // Validate amplitude range: 1% to 100%
    config.amplitude_percent >= 1.0 && config.amplitude_percent <= 100.0
}

pub fn validate_frequency(freq_hz: f32) -> bool {
    freq_hz >= 0.1 && freq_hz <= 100.0
}

pub fn validate_duty_cycle(duty_percent: f32) -> bool {
    duty_percent >= 1.0 && duty_percent <= 99.0
}

pub fn validate_waveform_factor(factor: f32) -> bool {
    factor >= 0.0 && factor <= 1.0
}

pub fn validate_amplitude(amplitude_percent: f32) -> bool {
    amplitude_percent >= 1.0 && amplitude_percent <= 100.0
}
