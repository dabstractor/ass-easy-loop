#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaveformConfig {
    pub frequency: u32,
    pub duty_cycle: u8,
    pub waveform_factor: f32,
    pub amplitude: u16,
}
