# Waveform Generator Requirements

## Overview

The waveform generator is the core functionality of the pEMF device, producing configurable electromagnetic field patterns. **Key change from existing implementation**: The system must support **configurable waveform types, frequency, and duty cycle on-the-fly**, moving away from the fixed 2Hz square wave to a flexible, runtime-configurable system.

## Critical Requirements from Project Specification

**User-Specified Defaults** (per project requirements):
- **Default Waveform**: Sawtooth (not square wave)
- **Default Frequency**: 10Hz (not 2Hz)
- **Default Duty Cycle**: 33% (not 0.4%)

**Waveform Type Specification**:
- **Waveform Parameter**: Float value 0.0 to 1.0
  - **0.0** = Perfectly sinusoidal wave
  - **1.0** = Perfectly square wave
  - **0.5** = Sawtooth wave (default)
  - **Intermediate values** = Blend between waveform types

**Configurability Requirement**:
- All parameters (waveform type, frequency, duty cycle) MUST be configurable **on-the-fly**
- Configuration changes take effect immediately without device restart
- Settings persist across power cycles (stored in non-volatile memory)

## Core Waveform Generation Requirements

### Configurable Parameters

**1. Frequency Control**
- **Range**: 0.1Hz to 100Hz
- **Default**: 10Hz (100ms period)
- **Resolution**: 0.1Hz steps
- **Accuracy**: ±1% frequency tolerance maintained
- **Real-time**: Frequency changes apply to next complete cycle

**2. Duty Cycle Control**
- **Range**: 1% to 99%
- **Default**: 33% (33ms HIGH, 67ms LOW at 10Hz)
- **Resolution**: 1% steps
- **Accuracy**: ±1% duty cycle tolerance
- **Real-time**: Duty cycle changes apply immediately within current cycle

**3. Waveform Type Control**
- **Parameter**: Float 0.0 to 1.0 (waveform_factor)
- **Default**: 0.5 (sawtooth)
- **Resolution**: 0.01 steps (100 discrete levels)
- **Waveform Generation**:
  ```rust
  enum WaveformType {
      Sinusoidal(f32),  // waveform_factor = 0.0
      Sawtooth(f32),    // waveform_factor = 0.5 (default)
      Square(f32),      // waveform_factor = 1.0
      Blended(f32),     // intermediate values
  }
  ```

### Advanced Waveform Synthesis

**Waveform Generation Algorithm**:
```rust
fn generate_waveform_value(
    time_in_cycle: f32,      // 0.0 to 1.0 within cycle
    waveform_factor: f32,    // 0.0 (sine) to 1.0 (square)
    duty_cycle: f32          // 0.0 to 1.0
) -> f32 {
    match waveform_factor {
        0.0 => sine_wave(time_in_cycle),
        0.5 => sawtooth_wave(time_in_cycle, duty_cycle),
        1.0 => square_wave(time_in_cycle, duty_cycle),
        factor => blend_waveforms(time_in_cycle, factor, duty_cycle),
    }
}
```

**Waveform Types Implementation**:

1. **Sinusoidal Wave** (waveform_factor = 0.0):
   ```rust
   fn sine_wave(t: f32) -> f32 {
       (2.0 * PI * t).sin() * 0.5 + 0.5  // 0 to 1 range
   }
   ```

2. **Sawtooth Wave** (waveform_factor = 0.5, default):
   ```rust
   fn sawtooth_wave(t: f32, duty_cycle: f32) -> f32 {
       if t < duty_cycle {
           t / duty_cycle  // Rising edge 0 to 1
       } else {
           1.0 - ((t - duty_cycle) / (1.0 - duty_cycle))  // Falling edge 1 to 0
       }
   }
   ```

3. **Square Wave** (waveform_factor = 1.0):
   ```rust
   fn square_wave(t: f32, duty_cycle: f32) -> f32 {
       if t < duty_cycle { 1.0 } else { 0.0 }
   }
   ```

4. **Blended Waveforms** (intermediate values):
   ```rust
   fn blend_waveforms(t: f32, factor: f32, duty_cycle: f32) -> f32 {
       let sine_val = sine_wave(t);
       let square_val = square_wave(t, duty_cycle);
       let sawtooth_val = sawtooth_wave(t, duty_cycle);

       // Blend based on factor value
       interpolate_waveforms(sine_val, sawtooth_val, square_val, factor)
   }
   ```

### Hardware Output Generation

**GPIO Control** (existing: GPIO 15 for MOSFET):
- **PWM Generation**: Hardware PWM for smooth waveform output
- **Resolution**: 12-bit PWM resolution (4096 levels)
- **Update Rate**: 1000Hz PWM frequency (inaudible, above pEMF range)
- **Output Range**: 0V to 3.3V (driving MOSFET gate)

**Timing Generation**:
- **Base Timer**: Hardware timer with microsecond precision
- **Sample Rate**: 10kHz sampling for smooth waveform generation
- **Buffer Size**: 1000 samples (100ms at 10kHz) for one complete cycle storage
- **Real-time**: RTIC highest priority task for sample generation

**MOSFET Drive Circuit**:
```
RP2040 GPIO15 (PWM) → Gate Driver → MOSFET → Electromagnetic Coil
                   ↓
Smooth analog waveform drives electromagnetic field strength
```

### Real-Time Configuration Interface

**Configuration Structure**:
```rust
#[derive(Clone, Copy)]
pub struct WaveformConfig {
    pub frequency_hz: f32,        // 0.1 to 100Hz
    pub duty_cycle_percent: f32,  // 1.0 to 99.0%
    pub waveform_factor: f32,     // 0.0 to 1.0
    pub amplitude_percent: f32,   // 1.0 to 100.0% (power level)
}

impl Default for WaveformConfig {
    fn default() -> Self {
        Self {
            frequency_hz: 10.0,      // 10Hz default
            duty_cycle_percent: 33.0, // 33% default
            waveform_factor: 0.5,    // Sawtooth default
            amplitude_percent: 100.0, // Full power default
        }
    }
}
```

**Configuration Commands** (via USB HID):
```
SET_FREQUENCY <freq_hz>     # e.g., SET_FREQUENCY 15.5
SET_DUTY_CYCLE <percent>    # e.g., SET_DUTY_CYCLE 25
SET_WAVEFORM <factor>       # e.g., SET_WAVEFORM 0.75
SET_AMPLITUDE <percent>     # e.g., SET_AMPLITUDE 80
GET_CONFIG                  # Returns current configuration
SAVE_CONFIG                 # Save to non-volatile memory
LOAD_CONFIG                 # Load from non-volatile memory
RESET_CONFIG                # Reset to defaults
```

### Performance Requirements

**Timing Accuracy** (enhanced from original ±1%):
- **Frequency Accuracy**: ±0.5% frequency tolerance
- **Duty Cycle Accuracy**: ±0.5% duty cycle tolerance
- **Waveform Fidelity**: <5% total harmonic distortion
- **Phase Accuracy**: ±1° phase accuracy for waveform transitions

**Real-Time Constraints**:
- **Priority**: Highest RTIC priority (priority 1)
- **Interrupt Latency**: <10μs maximum interrupt response
- **Sample Generation**: 10kHz sample rate maintained under all system loads
- **Configuration Changes**: Applied within 1 complete waveform cycle

**Resource Management**:
- **Memory Usage**: <4KB RAM for waveform buffers and state
- **CPU Usage**: <25% CPU utilization at 10Hz with complex waveforms
- **Power Consumption**: <500mA additional current for waveform generation
- **Thermal**: No thermal throttling under continuous operation

### Advanced Features

**Waveform Modulation**:
- **Amplitude Modulation**: Variable output power (1-100%)
- **Frequency Sweep**: Gradual frequency changes over time
- **Burst Mode**: On/off patterns with configurable timing
- **Phase Control**: Configurable phase offset for multi-device synchronization

**Synchronization**:
- **Multi-Device**: USB commands for synchronized operation across devices
- **External Trigger**: GPIO input for external synchronization signal
- **Phase Lock**: Maintain phase relationship between multiple devices
- **Network Sync**: Future capability for wireless synchronization

**Safety Features**:
- **Thermal Protection**: Automatic amplitude reduction if overheating detected
- **Overcurrent Protection**: MOSFET current monitoring and limiting
- **Emergency Stop**: Immediate waveform generation halt via USB command
- **Safe Startup**: Gradual amplitude ramp on power-on to prevent transients

### Configuration Storage

**Non-Volatile Storage**:
- **Technology**: RP2040 internal flash memory (XIP region)
- **Storage Size**: 4KB sector for configuration data
- **Wear Leveling**: Configuration changes distributed across multiple sectors
- **Backup**: Redundant storage with checksum validation

**Configuration Management**:
```rust
pub struct ConfigManager {
    current_config: WaveformConfig,
    default_config: WaveformConfig,
    storage_address: u32,
    checksum: u32,
}

impl ConfigManager {
    fn save_config(&mut self) -> Result<(), StorageError>;
    fn load_config(&mut self) -> Result<WaveformConfig, StorageError>;
    fn validate_checksum(&self) -> bool;
    fn reset_to_defaults(&mut self);
}
```

### Integration with Existing System

**RTIC Task Structure**:
```rust
#[rtic::app]
mod app {
    #[shared]
    struct Shared {
        waveform_config: WaveformConfig,
        sample_buffer: WaveformBuffer<1000>,
    }

    #[local]
    struct Local {
        pwm_channel: PWM<RP2040_PWM>,
        waveform_generator: WaveformGenerator,
        config_manager: ConfigManager,
    }

    // Highest priority - waveform sample generation
    #[task(priority = 1, local = [waveform_generator], shared = [sample_buffer])]
    fn generate_samples(ctx: generate_samples::Context);

    // Medium priority - configuration updates
    #[task(priority = 2, shared = [waveform_config])]
    fn update_config(ctx: update_config::Context, new_config: WaveformConfig);

    // Low priority - configuration storage
    #[task(priority = 3, local = [config_manager])]
    fn save_config(ctx: save_config::Context);
}
```

**Coexistence with Battery Monitoring**:
- **Priority Hierarchy**: Waveform generation (P1) > Battery monitoring (P2) > LED control (P3)
- **Resource Sharing**: Shared resources protected by RTIC resource locks
- **Performance Impact**: Battery monitoring timing unaffected by waveform changes
- **Power Management**: Waveform amplitude automatically reduced during low battery

### Testing and Validation Requirements

**Waveform Accuracy Testing**:
- **Oscilloscope Verification**: All waveform types measured for accuracy
- **Frequency Analysis**: Spectrum analyzer verification of harmonic content
- **Timing Validation**: Phase accuracy and frequency stability measurement
- **Load Testing**: Waveform accuracy under varying electromagnetic loads

**Configuration Testing**:
- **Parameter Range**: All combinations of frequency, duty cycle, waveform type
- **Transition Testing**: Smooth transitions between different configurations
- **Storage Testing**: Configuration persistence across power cycles
- **Edge Case Testing**: Boundary conditions and invalid parameter handling

**Integration Testing**:
- **System Integration**: Waveform generation with battery monitoring and USB logging
- **Performance Testing**: System performance under maximum waveform complexity
- **Stress Testing**: Extended operation (24+ hours) with various waveform patterns
- **Error Recovery**: System behavior during waveform generation failures

**Hardware Testing**:
- **MOSFET Drive**: Gate drive signal quality and switching characteristics
- **Thermal Testing**: Temperature rise during continuous operation
- **EMI Testing**: Electromagnetic interference from switching waveforms
- **Safety Testing**: Overcurrent and thermal protection validation

### Electromagnetic Field Characteristics

**Field Generation Parameters**:
- **Coil Specifications**: Optimized for target frequency range (0.1-100Hz)
- **Inductance**: Coil inductance affects frequency response and power
- **Field Strength**: Variable based on waveform amplitude and coil parameters
- **Field Pattern**: Uniform field within 5cm of coil center

**Therapeutic Waveform Profiles** (common pEMF therapy patterns):
```rust
// Schumann Resonance (7.83Hz sinusoidal)
WaveformConfig::schumann() -> WaveformConfig {
    WaveformConfig {
        frequency_hz: 7.83,
        duty_cycle_percent: 50.0,
        waveform_factor: 0.0,  // Pure sine wave
        amplitude_percent: 80.0,
    }
}

// Traditional Square Wave (10Hz, 33% duty cycle) - DEFAULT
WaveformConfig::default() -> WaveformConfig {
    WaveformConfig {
        frequency_hz: 10.0,
        duty_cycle_percent: 33.0,
        waveform_factor: 0.5,  // Sawtooth
        amplitude_percent: 100.0,
    }
}

// High Frequency Burst (50Hz square pulses)
WaveformConfig::burst_mode() -> WaveformConfig {
    WaveformConfig {
        frequency_hz: 50.0,
        duty_cycle_percent: 10.0,
        waveform_factor: 1.0,  // Square wave
        amplitude_percent: 100.0,
    }
}
```

## Implementation Timeline

**Phase 1: Core Waveform Engine** (Week 1-2)
- Waveform generation algorithms (sine, sawtooth, square, blended)
- PWM output implementation with 12-bit resolution
- Basic frequency and duty cycle control
- RTIC task structure for real-time operation

**Phase 2: Configuration System** (Week 3-4)
- USB HID command interface for parameter control
- Non-volatile configuration storage and management
- Real-time parameter updates without system restart
- Configuration validation and error handling

**Phase 3: Advanced Features** (Week 5-6)
- Amplitude modulation and power control
- Waveform blending and intermediate waveform types
- Thermal and overcurrent protection integration
- Multi-device synchronization capabilities

**Phase 4: Integration and Testing** (Week 7-8)
- Integration with existing battery monitoring and USB logging
- Comprehensive testing with oscilloscope and spectrum analyzer
- Performance optimization and resource usage minimization
- Documentation and user interface development

## Success Criteria

1. **Flexibility**: Support for all specified waveform types (sine, sawtooth, square, blended)
2. **Accuracy**: Frequency and duty cycle within ±0.5% tolerance
3. **Real-time**: Configuration changes applied within 1 waveform cycle
4. **Performance**: No impact on existing system functionality (battery, USB, LED)
5. **Persistence**: Configuration settings survive power cycles
6. **Safety**: Thermal and overcurrent protection prevent damage
7. **User Experience**: Intuitive USB command interface for configuration
8. **Integration**: Seamless coexistence with all existing system features

## Test-Driven Development Requirements

**Implementation Order** (per project requirements):
1. **Waveform Algorithm Tests**: Unit tests for all waveform generation functions
2. **Hardware PWM Tests**: Validate PWM output accuracy and resolution
3. **Configuration Tests**: Parameter validation and storage functionality
4. **Integration Tests**: Real-time performance with complete system
5. **Hardware Validation**: Oscilloscope verification of all waveform outputs

**Critical Test Categories**:
- Mathematical accuracy of waveform algorithms
- Real-time performance under system load
- Hardware output signal quality and timing
- Configuration persistence and recovery
- Integration with existing system components
- Safety protection and error recovery mechanisms

**Performance Benchmarks**:
- Frequency accuracy: ±0.5% measured with frequency counter
- Duty cycle accuracy: ±0.5% measured with oscilloscope
- Waveform fidelity: <5% THD measured with spectrum analyzer
- System impact: No degradation of existing ±1% battery/LED timing requirements
- Resource usage: <25% CPU, <4KB RAM for waveform subsystem