use crate::types::waveform::{WaveformConfig, WaveformBuffer, PWM_MAX_VALUE};
use rp2040_hal::pwm::{FreeRunning, Slice, SliceId};
use embedded_hal::PwmPin;

pub struct WaveformGenerator<Id: SliceId> {
    pwm_slice: Slice<Id, FreeRunning>,
    channel_a_enabled: bool,
    channel_b_enabled: bool,
    buffer: WaveformBuffer,
    enabled: bool,
}

impl<Id: SliceId> WaveformGenerator<Id> {
    /// Create new waveform generator with PWM slice
    /// GPIO 15 should be used for MOSFET gate control per PRP specification
    pub fn new(mut pwm_slice: Slice<Id, FreeRunning>) -> Self {
        // Configure PWM for 12-bit resolution (4096 levels)
        // Set PWM frequency to 1000Hz (inaudible, above pEMF range per PRP)
        let pwm_freq = 1000u32; // 1kHz PWM frequency
        let system_freq = 125_000_000u32; // 125MHz system clock
        
        // Calculate divider for desired PWM frequency
        // Target: 1000Hz with 12-bit resolution (4096 counts)
        let target_count = PWM_MAX_VALUE as u32 + 1; // 4096
        let required_div = system_freq / (pwm_freq * target_count);
        
        // Set clock divider (8-bit integer + 4-bit fractional)
        let div_int = (required_div as u8).max(1);
        let div_frac = ((required_div % 16) as u8) & 0x0F;
        
        pwm_slice.set_div_int(div_int);
        pwm_slice.set_div_frac(div_frac);
        
        // Set top value for 12-bit resolution
        pwm_slice.set_top(PWM_MAX_VALUE);
        
        // Initialize with zero duty cycle
        pwm_slice.channel_a.set_duty(0);
        pwm_slice.channel_b.set_duty(0);
        
        // Enable PWM slice
        pwm_slice.enable();

        let mut buffer = WaveformBuffer::new();
        buffer.regenerate_samples(); // Initialize with default waveform

        Self {
            pwm_slice,
            channel_a_enabled: false, // GPIO 15 is typically Channel A
            channel_b_enabled: false,
            buffer,
            enabled: false,
        }
    }

    /// Update waveform configuration and regenerate samples
    pub fn set_config(&mut self, config: &WaveformConfig) -> bool {
        self.buffer.update_config(*config)
    }

    /// Get current waveform configuration
    pub fn get_config(&self) -> &WaveformConfig {
        self.buffer.config()
    }

    /// Enable waveform generation on Channel A (GPIO 15)
    pub fn enable_channel_a(&mut self) {
        self.channel_a_enabled = true;
        self.enabled = true;
        // Reset buffer position for clean start
        self.buffer.reset_position();
    }

    /// Enable waveform generation on Channel B (GPIO 14)
    pub fn enable_channel_b(&mut self) {
        self.channel_b_enabled = true;
        self.enabled = true;
        // Reset buffer position for clean start
        self.buffer.reset_position();
    }

    /// Disable waveform generation (set output to zero)
    pub fn disable(&mut self) {
        self.enabled = false;
        self.channel_a_enabled = false;
        self.channel_b_enabled = false;
        // Set duty cycle to zero immediately
        self.pwm_slice.channel_a.set_duty(0);
        self.pwm_slice.channel_b.set_duty(0);
    }

    /// Update PWM output with next sample from buffer
    /// This should be called from high-priority timer interrupt at 10kHz
    pub fn update_output(&mut self) {
        if !self.enabled {
            return;
        }

        let sample = self.buffer.get_next_sample();
        
        // Update PWM duty cycle for enabled channels
        if self.channel_a_enabled {
            self.pwm_slice.channel_a.set_duty(sample);
        }
        if self.channel_b_enabled {
            self.pwm_slice.channel_b.set_duty(sample);
        }
    }

    /// Check if waveform generation is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get buffer diagnostics for monitoring
    pub fn get_diagnostics(&self) -> crate::types::waveform::WaveformBufferDiagnostics {
        self.buffer.get_diagnostics()
    }

    /// Emergency stop - immediately disable output
    pub fn emergency_stop(&mut self) {
        self.disable();
        // Additional safety: force PWM slice disable if needed
        // self.pwm_slice.disable(); // Commented out to allow quick re-enable
    }

    /// Get current PWM duty cycle for Channel A
    pub fn get_current_duty_a(&self) -> u16 {
        self.pwm_slice.channel_a.get_duty()
    }

    /// Get current PWM duty cycle for Channel B
    pub fn get_current_duty_b(&self) -> u16 {
        self.pwm_slice.channel_b.get_duty()
    }

    /// Set duty cycle directly on Channel A (for testing/calibration)
    pub fn set_direct_duty_a(&mut self, duty: u16) {
        let clamped_duty = duty.min(PWM_MAX_VALUE);
        self.pwm_slice.channel_a.set_duty(clamped_duty);
    }

    /// Set duty cycle directly on Channel B (for testing/calibration)
    pub fn set_direct_duty_b(&mut self, duty: u16) {
        let clamped_duty = duty.min(PWM_MAX_VALUE);
        self.pwm_slice.channel_b.set_duty(clamped_duty);
    }

    /// Validate PWM configuration
    pub fn validate_config(&self) -> Result<(), PwmError> {
        let diagnostics = self.get_diagnostics();
        
        if !diagnostics.buffer_valid {
            return Err(PwmError::InvalidBuffer);
        }
        
        if diagnostics.effective_frequency < 0.1 || diagnostics.effective_frequency > 100.0 {
            return Err(PwmError::FrequencyOutOfRange);
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PwmError {
    InvalidBuffer,
    FrequencyOutOfRange,
    HardwareError,
}

impl core::fmt::Display for PwmError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PwmError::InvalidBuffer => write!(f, "Invalid waveform buffer"),
            PwmError::FrequencyOutOfRange => write!(f, "Frequency out of range (0.1-100Hz)"),
            PwmError::HardwareError => write!(f, "PWM hardware error"),
        }
    }
}
