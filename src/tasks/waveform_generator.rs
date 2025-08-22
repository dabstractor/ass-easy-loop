use crate::types::waveform::{WaveformConfig, WaveformBuffer};
use crate::drivers::pwm_waveform::WaveformGenerator;
use systick_monotonic::fugit::Duration;
use rp2040_hal::pwm::{SliceId, FreeRunning, Slice};

/// Waveform generator task implementation
/// 
/// This task runs at 10kHz (every 100 microseconds) to provide smooth waveform generation
/// Priority 1 matches USB polling to ensure real-time performance per PRP requirements
pub fn waveform_sample_task<Id: SliceId>(
    waveform_gen: &mut WaveformGenerator<Id>
) {
    // Update PWM output with next sample from buffer
    waveform_gen.update_output();
}

/// Task to handle waveform configuration updates
/// Priority 2 to allow real-time sample generation to take precedence
pub fn waveform_config_task<Id: SliceId>(
    waveform_gen: &mut WaveformGenerator<Id>,
    new_config: WaveformConfig
) {
    // Update waveform configuration
    let config_changed = waveform_gen.set_config(&new_config);
    
    if config_changed {
        // Configuration successfully updated
        // Log the change if logging is enabled
        #[cfg(feature = "system-logs")]
        {
            use crate::drivers::logging;
            logging::log_system_event("Waveform config updated");
        }
    }
}
