#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaveformConfig {
    pub frequency_hz: f32,        // 0.1 to 100Hz
    pub duty_cycle_percent: f32,  // 1.0 to 99.0%
    pub waveform_factor: f32,     // 0.0 to 1.0 (sine to square)
    pub amplitude_percent: f32,   // 1.0 to 100.0% (power level)
}

impl Default for WaveformConfig {
    fn default() -> Self {
        Self {
            frequency_hz: 10.0,         // 10Hz default per PRP
            duty_cycle_percent: 33.0,   // 33% default per PRP
            waveform_factor: 0.5,       // Sawtooth default per PRP
            amplitude_percent: 100.0,   // Full power default
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WaveformSample {
    pub value: f32,      // 0.0 to 1.0 range
    pub timestamp_us: u32,
}

pub const SAMPLE_BUFFER_SIZE: usize = 1000;
pub const SAMPLE_RATE_HZ: u32 = 10000;
pub const PWM_RESOLUTION_BITS: u8 = 12;
pub const PWM_MAX_VALUE: u16 = (1 << PWM_RESOLUTION_BITS) - 1;

/// Circular buffer for waveform samples with efficient generation
pub struct WaveformBuffer {
    samples: [u16; SAMPLE_BUFFER_SIZE],
    current_index: usize,
    samples_per_cycle: usize,
    config: WaveformConfig,
    buffer_valid: bool,
}

impl WaveformBuffer {
    /// Create new waveform buffer with default configuration
    pub fn new() -> Self {
        Self {
            samples: [0; SAMPLE_BUFFER_SIZE],
            current_index: 0,
            samples_per_cycle: SAMPLE_BUFFER_SIZE,
            config: WaveformConfig::default(),
            buffer_valid: false,
        }
    }

    /// Update configuration and regenerate samples if needed
    pub fn update_config(&mut self, new_config: WaveformConfig) -> bool {
        if self.config != new_config {
            self.config = new_config;
            self.regenerate_samples();
            true
        } else {
            false
        }
    }

    /// Regenerate all samples based on current configuration
    pub fn regenerate_samples(&mut self) {
        use crate::utils::waveforms::{generate_waveform_value, waveform_to_pwm};

        // Calculate samples per cycle based on frequency
        let period_samples = (SAMPLE_RATE_HZ as f32 / self.config.frequency_hz) as usize;
        self.samples_per_cycle = period_samples.min(SAMPLE_BUFFER_SIZE);

        // Generate samples for one complete cycle
        for i in 0..self.samples_per_cycle {
            let time_in_cycle = i as f32 / self.samples_per_cycle as f32;
            let duty_cycle_normalized = self.config.duty_cycle_percent / 100.0;
            
            let waveform_value = generate_waveform_value(
                time_in_cycle,
                self.config.waveform_factor,
                duty_cycle_normalized,
            );
            
            self.samples[i] = waveform_to_pwm(waveform_value, self.config.amplitude_percent);
        }

        // Fill remaining buffer with copies of the cycle for seamless looping
        for i in self.samples_per_cycle..SAMPLE_BUFFER_SIZE {
            let cycle_index = i % self.samples_per_cycle;
            self.samples[i] = self.samples[cycle_index];
        }

        self.buffer_valid = true;
        self.current_index = 0;
    }

    /// Get next PWM value from buffer (used by interrupt handler)
    pub fn get_next_sample(&mut self) -> u16 {
        if !self.buffer_valid {
            return 0;
        }

        let sample = self.samples[self.current_index];
        
        // Advance to next sample with wraparound
        self.current_index = (self.current_index + 1) % SAMPLE_BUFFER_SIZE;
        
        sample
    }

    /// Get current configuration
    pub fn config(&self) -> &WaveformConfig {
        &self.config
    }

    /// Check if buffer is valid and ready
    pub fn is_valid(&self) -> bool {
        self.buffer_valid
    }

    /// Reset buffer position to start of cycle
    pub fn reset_position(&mut self) {
        self.current_index = 0;
    }

    /// Get buffer utilization for diagnostics
    pub fn get_diagnostics(&self) -> WaveformBufferDiagnostics {
        WaveformBufferDiagnostics {
            samples_per_cycle: self.samples_per_cycle,
            current_index: self.current_index,
            buffer_valid: self.buffer_valid,
            effective_frequency: if self.samples_per_cycle > 0 {
                SAMPLE_RATE_HZ as f32 / self.samples_per_cycle as f32
            } else {
                0.0
            },
        }
    }
}

impl Default for WaveformBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WaveformBufferDiagnostics {
    pub samples_per_cycle: usize,
    pub current_index: usize,
    pub buffer_valid: bool,
    pub effective_frequency: f32,
}
