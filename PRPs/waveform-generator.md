## Goal

**Feature Goal**: Implement a fully configurable waveform generator that replaces the fixed 2Hz square wave with runtime-configurable electromagnetic field patterns supporting sine, sawtooth, square, and blended waveforms with frequency (0.1Hz-100Hz), duty cycle (1%-99%), and amplitude (1%-100%) control.

**Deliverable**: A complete waveform generation subsystem integrated into the existing RTIC framework with USB HID command interface, non-volatile configuration storage, and 12-bit PWM output to GPIO15 for MOSFET control.

**Success Definition**: The device generates clean, accurate waveforms of all types within ±0.5% frequency and duty cycle tolerance, responds to real-time USB configuration changes, persists settings across power cycles, and maintains all existing system functionality including USB enumeration, battery monitoring, and bootloader entry.

## User Persona (if applicable)

**Target User**: Researchers and developers working with pEMF (pulsed electromagnetic field) therapy applications who need flexible waveform generation for experimentation.

**Use Case**: Configuring electromagnetic field patterns for biological research, testing different waveform types and parameters for therapeutic applications, and validating waveform accuracy with laboratory equipment.

**User Journey**: 
1. Connect device via USB
2. Use host tools to send waveform configuration commands
3. Observe generated waveforms on oscilloscope connected to GPIO15
4. Adjust parameters in real-time via USB commands
5. Save configurations for future use
6. Verify settings persist after power cycling

**Pain Points Addressed**: 
- Fixed 2Hz square wave limiting research possibilities
- Manual hardware changes required for different waveforms
- No way to validate waveform accuracy without specialized equipment
- Configuration changes requiring device restart

## Why

- **Research Flexibility**: Enables comprehensive pEMF research with configurable waveforms instead of fixed output
- **System Integration**: Builds on existing USB HID, RTIC, and flash storage infrastructure
- **Scientific Validation**: Provides precise, measurable electromagnetic field patterns for research
- **User Experience**: Real-time configuration changes without device restart or manual intervention

## What

**Core Requirements**:
- Generate sine (0.0), sawtooth (0.5), square (1.0), and blended (0.0-1.0) waveforms
- Frequency range: 0.1Hz to 100Hz with 0.1Hz resolution
- Duty cycle range: 1% to 99% with 1% resolution
- Amplitude control: 1% to 100% with 1% resolution
- 12-bit PWM output (4096 levels) at 1000Hz update rate
- Real-time configuration via USB HID commands
- Non-volatile storage of settings
- Integration with existing RTIC task structure

### Success Criteria

- [ ] All waveform types generate clean, accurate signals within ±0.5% tolerance
- [ ] USB enumeration maintained: `lsusb | grep fade` shows device
- [ ] Bootloader entry works flawlessly with new firmware
- [ ] Battery monitoring timing accuracy preserved (±1%)
- [ ] Configuration changes applied within one complete waveform cycle
- [ ] Settings persist across power cycles
- [ ] All existing system features continue working perfectly

## All Needed Context

### Context Completeness Check

_Before writing this PRP, validate: "If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"_

### Documentation & References

```yaml
# MUST READ - Include these in your context window
- file: src/main.rs
  why: Main RTIC application structure and task definitions
  pattern: Existing USB polling and command handling tasks
  gotcha: USB polling must run every 10ms or device disappears from USB

- file: src/drivers/pwm_waveform.rs
  why: Existing PWM waveform driver skeleton to extend
  pattern: Hardware abstraction layer usage for RP2040 PWM
  gotcha: Need to implement actual waveform generation logic

- file: src/types/waveform.rs
  why: Existing WaveformConfig structure to extend
  pattern: Current simple config structure needs expansion
  gotcha: Current u32 frequency needs to become f32 for 0.1Hz resolution

- file: src/config/defaults.rs
  why: Default configuration values
  pattern: Current defaults need updating to match PRD requirements
  gotcha: Default should be 10Hz sawtooth (0.5) not 2Hz square

- file: src/drivers/usb_command_handler.rs
  why: USB command parsing and handling
  pattern: Existing command structure to extend with new waveform commands
  gotcha: Command IDs must not conflict with existing commands

- file: src/config/flash_storage.rs
  why: Non-volatile storage implementation
  pattern: Existing storage skeleton to complete
  gotcha: Need proper error handling and wear leveling considerations
```

### Current Codebase tree (run `tree` in the root of the project) to get an overview of the codebase

```bash
src/
├── config/
│   ├── defaults.rs
│   ├── flash_storage.rs
│   ├── mod.rs
│   ├── usb.rs
│   └── validation.rs
├── drivers/
│   ├── adc_battery.rs
│   ├── battery_safety.rs
│   ├── led_control.rs
│   ├── logging.rs
│   ├── mod.rs
│   ├── pwm_waveform.rs
│   ├── usb_command_handler.rs
│   └── usb_hid.rs
├── lib.rs
├── main.rs
├── tasks/
│   ├── battery_monitor.rs
│   ├── led_manager.rs
│   ├── mod.rs
│   ├── usb_handler.rs
│   └── waveform_generator.rs
├── types/
│   ├── battery.rs
│   ├── bootloader_types.rs
│   ├── errors.rs
│   ├── logging.rs
│   ├── mod.rs
│   ├── usb_commands.rs
│   └── waveform.rs
└── utils/
    └── mod.rs
```

### Desired Codebase tree with files to be added and responsibility of file

```bash
src/
├── config/
│   ├── defaults.rs              # UPDATE: New default waveform config
│   ├── flash_storage.rs         # COMPLETE: Full flash storage implementation
│   └── validation.rs            # UPDATE: Enhanced validation for new parameters
├── drivers/
│   ├── pwm_waveform.rs          # COMPLETE: Full waveform generation implementation
│   └── usb_command_handler.rs   # UPDATE: New waveform command handlers
├── tasks/
│   └── waveform_generator.rs    # CREATE: Main waveform generation RTIC task
├── types/
│   ├── usb_commands.rs          # UPDATE: New waveform command enums
│   └── waveform.rs              # UPDATE: Enhanced WaveformConfig structure
└── main.rs                      # UPDATE: Task spawning and shared resource integration
```

### Known Gotchas of our codebase & Library Quirks

```rust
// CRITICAL: USB polling task MUST run every 10ms or device disappears from lsusb
// CRITICAL: Battery monitoring timing accuracy must remain ±1% 
// CRITICAL: RTIC task priorities must be maintained (waveform = 1, battery = 2, logging = 3)
// CRITICAL: Flash storage writes must be wear-leveled and error-checked
// CRITICAL: PWM frequency must be 1000Hz for smooth analog output
// CRITICAL: Waveform generation must use 12-bit resolution (0-4095)
```

## Implementation Blueprint

### Data models and structure

Create the core data models, we ensure type safety and consistency.

```rust
// Enhanced WaveformConfig in src/types/waveform.rs
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaveformConfig {
    pub frequency_hz: f32,        // 0.1 to 100Hz
    pub duty_cycle_percent: f32,  // 1.0 to 99.0%
    pub waveform_factor: f32,     // 0.0 to 1.0
    pub amplitude_percent: f32,   // 1.0 to 100.0%
}

// Enhanced USB commands in src/types/usb_commands.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UsbCommand {
    SetFrequency(u32),      // Already exists
    SetDutyCycle(u8),       // Already exists
    SetWaveformFactor(u8),  // NEW: 0-255 mapped to 0.0-1.0
    SetAmplitude(u8),       // NEW: 1-100%
    GetConfig,              // NEW: Retrieve current configuration
    SaveConfig,             // NEW: Save to flash storage
    LoadConfig,             // NEW: Load from flash storage
    ResetConfig,            // NEW: Reset to defaults
    EnterBootloader,        // Already exists
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: UPDATE src/types/waveform.rs
  - IMPLEMENT: Enhanced WaveformConfig with f32 fields
  - IMPLEMENT: Default implementation matching PRD requirements
  - FOLLOW pattern: Existing struct definition with Clone, Copy, Debug
  - NAMING: frequency_hz, duty_cycle_percent, waveform_factor, amplitude_percent
  - PLACEMENT: Replace existing simple struct

Task 2: UPDATE src/config/defaults.rs
  - IMPLEMENT: New DEFAULT_WAVEFORM_CONFIG with PRD values
  - VALUES: frequency=10.0Hz, duty_cycle=33.0%, waveform_factor=0.5, amplitude=100.0%
  - FOLLOW pattern: Existing constant definition
  - PLACEMENT: Update existing constant

Task 3: UPDATE src/types/usb_commands.rs
  - IMPLEMENT: New USB command variants for waveform control
  - IMPLEMENT: Enhanced CommandReport structure if needed
  - FOLLOW pattern: Existing enum variants and report descriptor
  - NAMING: SetWaveformFactor, SetAmplitude, GetConfig, SaveConfig, LoadConfig, ResetConfig
  - PLACEMENT: Extend existing UsbCommand enum

Task 4: UPDATE src/drivers/usb_command_handler.rs
  - IMPLEMENT: Parsing for new waveform commands
  - IMPLEMENT: Response handling for GetConfig command
  - FOLLOW pattern: Existing parse_hid_report function structure
  - NAMING: Command IDs 0x04-0x09 for new commands
  - PLACEMENT: Extend existing parsing logic

Task 5: COMPLETE src/config/flash_storage.rs
  - IMPLEMENT: Full save_config and load_config functionality
  - IMPLEMENT: Proper error handling and validation
  - FOLLOW pattern: Embedded storage crate usage
  - NAMING: Standard flash storage patterns
  - PLACEMENT: Complete existing skeleton implementation

Task 6: COMPLETE src/drivers/pwm_waveform.rs
  - IMPLEMENT: Full waveform generation algorithms
  - IMPLEMENT: PWM hardware configuration and control
  - IMPLEMENT: GPIO15 setup for MOSFET output
  - FOLLOW pattern: RP2040 HAL PWM usage
  - NAMING: sine_wave, sawtooth_wave, square_wave, blend_waveforms functions
  - PLACEMENT: Complete existing skeleton implementation

Task 7: CREATE src/tasks/waveform_generator.rs
  - IMPLEMENT: Main RTIC task for waveform generation
  - IMPLEMENT: Sample buffer management and timing
  - IMPLEMENT: Real-time configuration updates
  - FOLLOW pattern: Existing RTIC task structure (battery_monitor.rs)
  - NAMING: generate_waveform_samples, update_waveform_config
  - PLACEMENT: New file in tasks module

Task 8: UPDATE src/main.rs
  - INTEGRATE: New waveform generator task
  - INTEGRATE: Shared resource management for waveform config
  - INTEGRATE: Task spawning with proper timing
  - FIND pattern: Existing task spawning and shared resource usage
  - ADD: waveform_config shared resource
  - ADD: waveform generator task spawn
  - PRESERVE: Existing USB, battery, and logging task structure

Task 9: CREATE tests/unit_tests/test_waveform.rs
  - IMPLEMENT: Unit tests for waveform generation functions
  - IMPLEMENT: Configuration validation tests
  - IMPLEMENT: USB command parsing tests
  - FOLLOW pattern: Existing unit test structure
  - NAMING: test_sine_wave, test_sawtooth_wave, test_square_wave, etc.
  - PLACEMENT: New file in tests/unit_tests/
```

### Implementation Patterns & Key Details

```rust
// Waveform generation core functions
fn generate_waveform_value(time_in_cycle: f32, waveform_factor: f32, duty_cycle: f32) -> f32 {
    match waveform_factor {
        0.0 => sine_wave(time_in_cycle),
        0.5 => sawtooth_wave(time_in_cycle, duty_cycle),
        1.0 => square_wave(time_in_cycle, duty_cycle),
        factor => blend_waveforms(time_in_cycle, factor, duty_cycle),
    }
}

fn sine_wave(t: f32) -> f32 {
    (2.0 * core::f32::consts::PI * t).sin() * 0.5 + 0.5
}

fn sawtooth_wave(t: f32, duty_cycle: f32) -> f32 {
    if t < duty_cycle {
        t / duty_cycle
    } else {
        1.0 - ((t - duty_cycle) / (1.0 - duty_cycle))
    }
}

fn square_wave(t: f32, duty_cycle: f32) -> f32 {
    if t < duty_cycle { 1.0 } else { 0.0 }
}

// RTIC task structure pattern
#[task(
    shared = [waveform_config],
    local = [waveform_generator],
    priority = 1
)]
fn generate_waveform_samples(ctx: generate_waveform_samples::Context) {
    // Lock shared resources
    ctx.shared.waveform_config.lock(|config| {
        // Generate next sample based on current config
        // Update PWM output
    });
    
    // Schedule next sample generation
    generate_waveform_samples::spawn_after(Duration::<u64, 1, 1000>::micros(100)).unwrap();
}

// USB command parsing pattern
pub fn parse_hid_report(report: &[u8; 64]) -> Option<UsbCommand> {
    match report[0] {
        0x01 => Some(UsbCommand::SetFrequency(u32::from_le_bytes([
            report[1], report[2], report[3], report[4],
        ]))),
        0x02 => Some(UsbCommand::SetDutyCycle(report[1])),
        0x04 => Some(UsbCommand::SetWaveformFactor(report[1])),  // NEW
        0x05 => Some(UsbCommand::SetAmplitude(report[1])),       // NEW
        0x06 => Some(UsbCommand::GetConfig),                     // NEW
        0x07 => Some(UsbCommand::SaveConfig),                    // NEW
        0x08 => Some(UsbCommand::LoadConfig),                    // NEW
        0x09 => Some(UsbCommand::ResetConfig),                   // NEW
        0x03 => Some(UsbCommand::EnterBootloader),
        _ => None,
    }
}
```

### Integration Points

```yaml
RTIC_TASKS:
  - add: waveform_generator task with priority 1
  - shared: waveform_config resource
  - timing: 100μs sample generation (10kHz rate)

USB_COMMANDS:
  - add: SetWaveformFactor (0x04)
  - add: SetAmplitude (0x05)
  - add: GetConfig (0x06)
  - add: SaveConfig (0x07)
  - add: LoadConfig (0x08)
  - add: ResetConfig (0x09)

FLASH_STORAGE:
  - address: Dedicated flash sector for waveform config
  - backup: Redundant storage with checksum validation
  - wear_leveling: Distribution across multiple sectors

GPIO:
  - pin: GPIO15 for PWM output
  - function: FunctionPwm
  - slice: PWM slice appropriate for 1000Hz carrier frequency
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation - fix before proceeding
cargo check                              # Syntax checking
cargo fmt -- --check                     # Formatting validation
cargo clippy                             # Linting and best practices

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test waveform generation functions
cargo test test_waveform                 # Waveform algorithm tests
cargo test test_usb_commands             # USB command parsing tests
cargo test test_config_validation        # Configuration validation tests
cargo test test_flash_storage            # Flash storage tests

# Full test suite for affected areas
cargo test

# Expected: All tests pass. If failing, debug root cause and fix implementation.
```

### Level 3: Integration Testing (System Validation)

```bash
# Build and flash firmware
cargo run

# Verify USB enumeration
lsusb | grep fade                        # Device should appear as "dabstractor Ass-Easy Loop"

# Test bootloader entry
python host_tools/bootloader_entry.py    # Should enter bootloader mode successfully

# Test waveform commands
python3 -c "
import hid
device = hid.Device(0xfade, 0x1212)
# Test sine wave (factor 0)
device.write([0, 0x04, 0, 0, 0, 0] + [0] * 58)
# Test 10Hz frequency
device.write([0, 0x01, 10, 0, 0, 0] + [0] * 58)
# Test 50% duty cycle
device.write([0, 0x02, 50] + [0] * 61)
device.close()
"

# Monitor logs for system health
python host_tools/log_monitor.py --duration 30

# Expected: All integrations working, proper responses, no USB enumeration errors
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Hardware validation with oscilloscope
# 1. Connect oscilloscope to GPIO15
# 2. Generate sine wave and verify clean sinusoidal output
# 3. Generate square wave and verify sharp transitions
# 4. Generate sawtooth and verify linear ramps
# 5. Test frequency accuracy with function generator reference
# 6. Verify amplitude control with variable output levels

# Long-term stability test
# 1. Run continuous waveform generation for 24 hours
# 2. Monitor for timing drift or performance degradation
# 3. Verify USB enumeration maintained throughout
# 4. Check battery monitoring accuracy unaffected

# Configuration persistence test
# 1. Set custom waveform configuration
# 2. Power cycle device
# 3. Verify configuration restored from flash
# 4. Test Save/Load/Reset commands

# Expected: Hardware signals accurate within ±0.5%, no system degradation, persistent storage working
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test`
- [ ] No compilation errors: `cargo check`
- [ ] No clippy warnings: `cargo clippy`
- [ ] No formatting issues: `cargo fmt -- --check`

### Feature Validation

- [ ] USB enumeration: Device appears as "dabstractor Ass-Easy Loop" (VID: 0xfade, PID: 0x1212)
- [ ] Bootloader entry: Seamless transition to/from bootloader mode
- [ ] Battery monitoring: ±1% accuracy maintained during waveform generation
- [ ] Waveform generation: All types (sine, square, sawtooth, blended) produce clean signals
- [ ] Frequency range: 0.1Hz to 100Hz accurate within ±0.5%
- [ ] Duty cycle range: 1% to 99% accurate within ±0.5%
- [ ] Amplitude control: 1% to 100% with linear response
- [ ] Real-time configuration: Changes take effect within 100ms
- [ ] Non-volatile storage: Settings survive power cycles
- [ ] USB command interface: All waveform commands process correctly

### Code Quality Validation

- [ ] Follows existing codebase patterns and naming conventions
- [ ] File placement matches desired codebase tree structure
- [ ] Anti-patterns avoided (check against Anti-Patterns section)
- [ ] Dependencies properly managed and imported
- [ ] Configuration changes properly integrated
- [ ] Proper error handling throughout implementation
- [ ] Memory usage within specified limits (<4KB RAM)

### Documentation & Deployment

- [ ] Code is self-documenting with clear variable/function names
- [ ] Logs are informative but not verbose
- [ ] Host tools updated to support new waveform commands
- [ ] README updated with new feature documentation

---

## Anti-Patterns to Avoid

- ❌ Don't break existing USB enumeration - test `lsusb | grep fade` after every change
- ❌ Don't compromise battery monitoring timing accuracy
- ❌ Don't use blocking operations in high-priority RTIC tasks
- ❌ Don't ignore flash storage wear leveling requirements
- ❌ Don't hardcode GPIO pins - use proper hardware abstraction
- ❌ Don't create new RTIC task priority conflicts
- ❌ Don't skip validation because "it should work"