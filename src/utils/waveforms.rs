use libm::sinf;

const PI: f32 = 3.14159265358979323846;

/// Generate sinusoidal waveform value (waveform_factor = 0.0)
/// Returns value in range 0.0 to 1.0
pub fn sine_wave(time_in_cycle: f32) -> f32 {
    sinf(2.0 * PI * time_in_cycle) * 0.5 + 0.5
}

/// Generate sawtooth waveform value (waveform_factor = 0.5, default)
/// Returns value in range 0.0 to 1.0
pub fn sawtooth_wave(time_in_cycle: f32, duty_cycle: f32) -> f32 {
    if time_in_cycle < duty_cycle {
        time_in_cycle / duty_cycle  // Rising edge 0 to 1
    } else {
        1.0 - ((time_in_cycle - duty_cycle) / (1.0 - duty_cycle))  // Falling edge 1 to 0
    }
}

/// Generate square waveform value (waveform_factor = 1.0)
/// Returns value in range 0.0 to 1.0
pub fn square_wave(time_in_cycle: f32, duty_cycle: f32) -> f32 {
    if time_in_cycle < duty_cycle { 
        1.0 
    } else { 
        0.0 
    }
}

/// Linear interpolation between three values
fn interpolate_waveforms(sine_val: f32, sawtooth_val: f32, square_val: f32, factor: f32) -> f32 {
    if factor <= 0.5 {
        // Interpolate between sine and sawtooth
        let t = factor * 2.0; // 0.0 to 1.0
        sine_val * (1.0 - t) + sawtooth_val * t
    } else {
        // Interpolate between sawtooth and square
        let t = (factor - 0.5) * 2.0; // 0.0 to 1.0
        sawtooth_val * (1.0 - t) + square_val * t
    }
}

/// Generate blended waveform for intermediate waveform_factor values
/// Returns value in range 0.0 to 1.0
pub fn blend_waveforms(time_in_cycle: f32, factor: f32, duty_cycle: f32) -> f32 {
    let sine_val = sine_wave(time_in_cycle);
    let square_val = square_wave(time_in_cycle, duty_cycle);
    let sawtooth_val = sawtooth_wave(time_in_cycle, duty_cycle);

    interpolate_waveforms(sine_val, sawtooth_val, square_val, factor)
}

/// Main waveform generation function per PRP specification
/// Returns value in range 0.0 to 1.0
pub fn generate_waveform_value(
    time_in_cycle: f32,      // 0.0 to 1.0 within cycle
    waveform_factor: f32,    // 0.0 (sine) to 1.0 (square)
    duty_cycle: f32          // 0.0 to 1.0
) -> f32 {
    // Clamp inputs to valid ranges
    let time = time_in_cycle.max(0.0).min(1.0);
    let factor = waveform_factor.max(0.0).min(1.0);
    let duty = duty_cycle.max(0.01).min(0.99);

    match factor {
        f if f <= 0.01 => sine_wave(time),
        f if f >= 0.99 => square_wave(time, duty),
        f if (f - 0.5).abs() <= 0.01 => sawtooth_wave(time, duty),
        _ => blend_waveforms(time, factor, duty),
    }
}

/// Calculate time position within waveform cycle
/// Returns value in range 0.0 to 1.0
pub fn calculate_cycle_time(
    timestamp_us: u32,
    frequency_hz: f32
) -> f32 {
    let period_us = (1_000_000.0 / frequency_hz) as u32;
    let phase_us = timestamp_us % period_us;
    phase_us as f32 / period_us as f32
}

/// Convert waveform value to PWM duty cycle
/// Input: 0.0 to 1.0 waveform value
/// Output: 0 to PWM_MAX_VALUE PWM counts
pub fn waveform_to_pwm(waveform_value: f32, amplitude_percent: f32) -> u16 {
    use crate::types::waveform::PWM_MAX_VALUE;
    
    let amplitude_factor = (amplitude_percent / 100.0).max(0.0).min(1.0);
    let scaled_value = waveform_value * amplitude_factor;
    (scaled_value * PWM_MAX_VALUE as f32) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sine_wave_bounds() {
        assert!((0.0..=1.0).contains(&sine_wave(0.0)));
        assert!((0.0..=1.0).contains(&sine_wave(0.25)));
        assert!((0.0..=1.0).contains(&sine_wave(0.5)));
        assert!((0.0..=1.0).contains(&sine_wave(0.75)));
        assert!((0.0..=1.0).contains(&sine_wave(1.0)));
    }

    #[test]
    fn test_square_wave_values() {
        assert_eq!(square_wave(0.0, 0.5), 1.0);
        assert_eq!(square_wave(0.25, 0.5), 1.0);
        assert_eq!(square_wave(0.5, 0.5), 0.0);
        assert_eq!(square_wave(0.75, 0.5), 0.0);
    }

    #[test]
    fn test_sawtooth_wave_bounds() {
        assert!((0.0..=1.0).contains(&sawtooth_wave(0.0, 0.5)));
        assert!((0.0..=1.0).contains(&sawtooth_wave(0.25, 0.5)));
        assert!((0.0..=1.0).contains(&sawtooth_wave(0.5, 0.5)));
        assert!((0.0..=1.0).contains(&sawtooth_wave(0.75, 0.5)));
        assert!((0.0..=1.0).contains(&sawtooth_wave(1.0, 0.5)));
    }

    #[test]
    fn test_generate_waveform_value() {
        // Test sine wave (factor = 0.0)
        let sine_val = generate_waveform_value(0.25, 0.0, 0.5);
        assert!((0.0..=1.0).contains(&sine_val));

        // Test square wave (factor = 1.0)
        let square_val = generate_waveform_value(0.25, 1.0, 0.5);
        assert_eq!(square_val, 1.0);

        // Test sawtooth wave (factor = 0.5)
        let sawtooth_val = generate_waveform_value(0.25, 0.5, 0.5);
        assert!((0.0..=1.0).contains(&sawtooth_val));
    }
}