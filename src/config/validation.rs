use crate::types::waveform::WaveformConfig;

pub fn validate_config(config: &WaveformConfig) -> bool {
    config.frequency > 0 && config.duty_cycle <= 100
}