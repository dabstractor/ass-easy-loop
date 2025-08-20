# Base RP2040 Rust Project Structure PRP

## Goal

**Feature Goal**: Create a comprehensive base Rust/Cargo project structure targeting RP2040 as the sole embedded platform, optimized for developer experience with easy flashing via `cargo run`.

**Deliverable**: Complete Rust project foundation with RTIC 2.0 framework, supporting all PRD requirements (pEMF generation, battery monitoring, USB HID logging, configuration management, and bootloader automation).

**Success Definition**: A developer can clone the repository, run `cargo run` to flash the device, and have a fully functional base system ready for feature implementation.

## User Persona

**Target User**: Embedded Rust developer working on the ass-easy-loop pEMF device

**Use Case**: Setting up the development environment and base project structure for implementing the complete pEMF system with multiple subsystems (waveform generation, battery monitoring, USB communications, configuration management).

**User Journey**: 
1. Clone repository
2. Install Rust toolchain and dependencies
3. Run `cargo run` to flash device
4. Verify basic functionality (LED blinking, USB enumeration)
5. Begin implementing specific features using established patterns

**Pain Points Addressed**: 
- Complex embedded Rust toolchain setup
- RP2040-specific configuration challenges
- RTIC 2.0 project structure decisions
- Multi-subsystem architecture organization
- Testing strategy for embedded systems

## Why

- **Foundation First**: All PRD features require a solid, well-architected base project structure
- **Developer Experience**: Easy flashing with `cargo run` reduces development friction significantly
- **Future-Proof Architecture**: RTIC 2.0 provides the real-time guarantees needed for precise pEMF timing (±1% tolerance)
- **Scalability**: Multi-subsystem design supports complex interactions between PWM, ADC, USB, and configuration systems
- **Testing Strategy**: Embedded testing framework enables Test-Driven Development as required by project specifications

## What

A complete Rust embedded project targeting RP2040 exclusively, with:
- RTIC 2.0 real-time framework for deterministic task scheduling
- Probe-rs integration for seamless flashing via `cargo run`
- Multi-subsystem architecture (PWM/waveform, ADC/battery, USB/HID, flash/config)
- Conditional compilation for production/development/testing builds
- Comprehensive testing strategy (unit tests, hardware-in-the-loop, integration tests)
- Flash-based configuration management with wear leveling
- USB HID infrastructure with build-time feature flags
- Project structure optimized for the specific requirements in all PRDs

### Success Criteria

- [ ] `cargo run` successfully flashes RP2040 device without manual intervention
- [ ] RTIC 2.0 application starts with proper hardware initialization (12MHz crystal, PLL settings)
- [ ] All GPIO pins properly configured (GPIO 15 PWM output, GPIO 25 LED, GPIO 26 ADC)
- [ ] USB HID enumeration successful (development build only)
- [ ] Basic tasks running at correct priorities (PWM highest, battery medium, LED lowest)
- [ ] Test suite passes with both host tests and target tests
- [ ] Project structure supports all PRD requirements with clear module separation
- [ ] Build configurations work correctly (production/development/testing)
- [ ] Flash-based configuration system functional with defaults
- [ ] Documentation complete for setup and development workflow

## All Needed Context

### Context Completeness Check

This PRP provides comprehensive context for implementing a complete RP2040 Rust base project. It includes specific crate versions, configuration files, project structure, hardware specifications, and implementation patterns based on extensive research of the 2024/2025 embedded Rust ecosystem.

### Documentation & References

```yaml
# MUST READ - Include these in your context window
- url: https://rtic.rs/2/book/en/
  why: RTIC 2.0 official documentation for app structure, task management, resource sharing
  critical: Priority hierarchy, shared vs local resources, task communication patterns

- url: https://docs.rs/rp2040-hal/0.11.0/rp2040_hal/
  why: RP2040 HAL API documentation for GPIO, PWM, ADC, USB, clock configuration
  critical: Hardware abstraction patterns, peripheral initialization, clock setup

- url: https://probe.rs/docs/
  why: Probe-rs configuration for seamless flashing and debugging
  critical: Embed.toml configuration, chip selection, flashing workflows

- url: https://docs.rs/usb-device/0.3.2/usb_device/
  why: USB device framework for HID implementation
  critical: Descriptor creation, enumeration process, composite devices

- url: https://docs.rs/embedded-storage/0.3.1/embedded_storage/
  why: Flash storage abstraction for configuration management
  critical: Sector-based operations, atomic updates, wear leveling patterns

- file: PRPs/PRDs/device-requirements.md
  why: Core hardware and timing requirements for the pEMF system
  pattern: RTIC priority hierarchy, GPIO assignments, timing tolerances
  gotcha: ±1% timing accuracy requirement affects task priority design

- file: PRPs/PRDs/waveform-generator.md
  why: PWM and waveform generation requirements with configurable parameters
  pattern: Real-time waveform synthesis, parameter updates, flash storage
  gotcha: 10Hz default frequency, 33% duty cycle, sawtooth waveform, 0.5 waveform factor

- file: PRPs/PRDs/battery-charging-circuit.md
  why: ADC monitoring requirements and voltage thresholds
  pattern: Voltage divider calculations, state detection logic, charging algorithms
  gotcha: Specific ADC thresholds (≤1425 Low, 1426-1674 Normal, ≥1675 Charging)

- file: PRPs/PRDs/usb-hid-logging.md
  why: USB HID implementation with build-time configuration
  pattern: Conditional compilation, HID descriptors, message queuing
  gotcha: Must be disabled by default in production builds, feature flag system

- file: PRPs/PRDs/automatic-bootloader-entry.md
  why: Bootloader automation and test command interface requirements
  pattern: USB command processing, authentication, graceful shutdown
  gotcha: 500ms maximum bootloader entry time, authentication required

- docfile: PRPs/ai_docs/rtic2-patterns.md
  why: RTIC 2.0 implementation patterns and multi-subsystem architecture
  section: Task Communication, Resource Management, Priority Design
```

### Current Codebase tree

```bash
asseasyloop/
├── PRPs/
│   ├── PRDs/
│   │   ├── automatic-bootloader-entry.md
│   │   ├── battery-charging-circuit.md
│   │   ├── device-requirements.md
│   │   ├── usb-hid-logging.md
│   │   └── waveform-generator.md
│   ├── README.md
│   ├── scripts/
│   │   └── prp_runner.py
│   └── templates/
│       ├── prp_base.md
│       ├── prp_base_typescript.md
│       ├── prp_planning.md
│       ├── prp_poc_react.md
│       ├── prp_spec.md
│       └── prp_task.md
└── (empty - no existing Rust code)
```

### Desired Codebase tree with files to be added and responsibility of file

```bash
asseasyloop/
├── .cargo/
│   └── config.toml                    # Rust target configuration and runner setup
├── .vscode/
│   ├── launch.json                    # VS Code debugging configuration
│   └── tasks.json                     # VS Code build tasks
├── src/
│   ├── main.rs                        # RTIC app entry point and task definitions
│   ├── lib.rs                         # Library crate root (for testing)
│   ├── config/
│   │   ├── mod.rs                     # Configuration module exports
│   │   ├── defaults.rs                # Default configuration values
│   │   ├── flash_storage.rs           # Flash-based configuration persistence
│   │   └── validation.rs              # Configuration validation logic
│   ├── drivers/
│   │   ├── mod.rs                     # Driver module exports
│   │   ├── adc_battery.rs             # Battery ADC monitoring driver
│   │   ├── led_control.rs             # LED control driver
│   │   ├── pwm_waveform.rs            # PWM-based waveform generation driver
│   │   └── usb_hid.rs                 # USB HID communication driver
│   ├── tasks/
│   │   ├── mod.rs                     # Task module exports
│   │   ├── battery_monitor.rs         # Battery monitoring task (priority 2)
│   │   ├── led_manager.rs             # LED control task (priority 3)
│   │   ├── usb_handler.rs             # USB communication task (priority 3)
│   │   └── waveform_generator.rs      # Waveform generation task (priority 1)
│   ├── types/
│   │   ├── mod.rs                     # Type definitions module
│   │   ├── battery.rs                 # Battery-related types and enums
│   │   ├── errors.rs                  # System error types
│   │   ├── usb_commands.rs            # USB command/response types
│   │   └── waveform.rs                # Waveform configuration types
│   └── utils/
│       ├── mod.rs                     # Utility module exports
│       ├── math.rs                    # Mathematical utilities (waveform synthesis)
│       └── timing.rs                  # Timing and delay utilities
├── tests/
│   ├── integration_tests.rs           # Integration tests for complete system
│   ├── hardware_tests.rs              # Hardware-in-the-loop tests
│   └── unit_tests/
│       ├── mod.rs                     # Unit test module organization
│       ├── test_battery.rs            # Battery monitoring unit tests
│       ├── test_config.rs             # Configuration management tests
│       ├── test_math.rs               # Mathematical utility tests
│       └── test_waveform.rs           # Waveform generation tests
├── host_tools/
│   ├── device_control.py              # Host-side device control utility
│   ├── log_monitor.py                 # USB HID log monitoring utility
│   └── test_runner.py                 # Automated test execution utility
├── memory.x                           # Linker script for RP2040 memory layout
├── Embed.toml                         # Probe-rs embedding configuration
├── Cargo.toml                         # Project dependencies and metadata
├── build.rs                           # Build script for compile-time setup
├── README.md                          # Project documentation and setup guide
└── CLAUDE.md                          # AI context and project information
```

### Known Gotchas of our codebase & Library Quirks

```rust
// CRITICAL: RP2040 requires thumbv6m-none-eabi target, not thumbv7em
// Example: Must use correct target in .cargo/config.toml

// CRITICAL: RTIC 2.0 syntax has changed significantly from 1.0
// Example: #[rtic::app] instead of #[rtfm::app], different resource syntax

// CRITICAL: rp2040-hal clock configuration requires specific PLL setup
// Example: 12MHz external crystal must be configured correctly for USB

// CRITICAL: USB HID requires proper timing and cannot be interrupted
// Example: USB enumeration fails if high-priority tasks block too long

// CRITICAL: Flash operations must be atomic and protect against power loss
// Example: Configuration updates require sector erase + write with checksum

// CRITICAL: PWM frequency and resolution are inversely related on RP2040
// Example: Higher frequencies reduce available duty cycle resolution

// CRITICAL: ADC readings require proper reference voltage and calibration
// Example: Voltage divider calculations affect battery state detection accuracy

// CRITICAL: Probe-rs requires specific chip identifier for RP2040
// Example: Must use "RP2040" exactly in Embed.toml, not variant names

// CRITICAL: RTIC shared resources require explicit mutex locking
// Example: Shared resources accessed from multiple priorities need lock() calls

// CRITICAL: USB device must handle suspend/resume properly
// Example: Host sleep/wake cycles can cause enumeration issues if not handled
```

## Implementation Blueprint

### Data models and structure

Create the core data models to ensure type safety and consistency across all subsystems.

```rust
// Configuration management types
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SystemConfig {
    pub waveform: WaveformConfig,
    pub battery: BatteryConfig, 
    pub usb: UsbConfig,
    pub system: SystemSettings,
}

// Waveform generation types
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WaveformConfig {
    pub frequency_hz: f32,           // 0.1 to 100Hz
    pub duty_cycle_percent: f32,     // 1.0 to 99.0%
    pub waveform_factor: f32,        // 0.0 to 1.0 (0.0=sine, 0.5=sawtooth, 1.0=square)
    pub amplitude_percent: f32,      // 1.0 to 100.0%
}

// Battery monitoring types
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BatteryState {
    Low,      // ADC ≤ 1425
    Normal,   // 1426-1674
    Charging, // ≥ 1675
}

// USB command types
#[derive(Serialize, Deserialize, Debug)]
pub enum UsbCommand {
    SetFrequency(f32),
    SetDutyCycle(f32),
    SetWaveform(f32),
    GetSystemStatus,
    EnterBootloader,
    SaveConfig,
}

// Error handling types
#[derive(Debug, Clone, Copy)]
pub enum SystemError {
    ConfigurationInvalid,
    FlashOperationFailed,
    UsbCommunicationError,
    HardwareInitializationFailed,
    TimingViolation,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE .cargo/config.toml and Embed.toml
  - IMPLEMENT: Rust target configuration for thumbv6m-none-eabi
  - SET: probe-rs as runner with "cargo run" command
  - CONFIGURE: RP2040 chip selection and flashing parameters
  - FOLLOW pattern: Standard RP2040 embedded Rust setup
  - PLACEMENT: Root directory configuration files

Task 2: CREATE Cargo.toml with dependencies
  - IMPLEMENT: Project metadata and dependencies
  - ADD: rp2040-hal, rtic, usb-device, usbd-hid, embedded-storage, serde, postcard
  - CONFIGURE: Feature flags for usb-logs, battery-logs, pemf-logs, system-logs
  - SET: Profile optimizations for embedded targets
  - PLACEMENT: Root directory project configuration

Task 3: CREATE memory.x linker script
  - IMPLEMENT: RP2040 memory layout definition
  - SET: Flash origin at 0x10000000, RAM at 0x20000000
  - CONFIGURE: Stack size and memory regions
  - FOLLOW pattern: Standard RP2040 memory layout
  - PLACEMENT: Root directory linker configuration

Task 4: CREATE src/types/ module
  - IMPLEMENT: All data types and enums (SystemConfig, WaveformConfig, BatteryState, UsbCommand, SystemError)
  - FOLLOW pattern: Derive Serialize/Deserialize for configuration types
  - NAMING: CamelCase for types, snake_case for fields
  - VALIDATION: Include bounds checking and validation methods
  - PLACEMENT: src/types/ directory structure

Task 5: CREATE src/config/ module  
  - IMPLEMENT: Configuration management with flash storage
  - IMPLEMENT: Default values matching PRD specifications (10Hz, 33%, sawtooth)
  - IMPLEMENT: Atomic configuration updates with checksums
  - FOLLOW pattern: embedded-storage trait for flash operations
  - DEPENDENCIES: Types from Task 4
  - PLACEMENT: src/config/ directory structure

Task 6: CREATE src/drivers/ module
  - IMPLEMENT: Hardware abstraction drivers for ADC, PWM, LED, USB HID
  - IMPLEMENT: Non-blocking driver interfaces for RTIC integration
  - FOLLOW pattern: embedded-hal trait implementations
  - CONFIGURE: GPIO assignments (15=PWM, 25=LED, 26=ADC)
  - DEPENDENCIES: Types from Task 4, Config from Task 5
  - PLACEMENT: src/drivers/ directory structure

Task 7: CREATE src/tasks/ module
  - IMPLEMENT: RTIC task implementations with proper priorities
  - IMPLEMENT: Waveform generation (priority 1), battery monitoring (priority 2), LED/USB (priority 3)
  - IMPLEMENT: Inter-task communication and resource sharing
  - FOLLOW pattern: RTIC 2.0 task structure and resource management
  - DEPENDENCIES: Drivers from Task 6, Types from Task 4
  - PLACEMENT: src/tasks/ directory structure

Task 8: CREATE src/main.rs RTIC application
  - IMPLEMENT: RTIC app structure with shared/local resources
  - IMPLEMENT: Hardware initialization (clocks, GPIO, peripherals)
  - IMPLEMENT: Task scheduling and resource allocation
  - INTEGRATE: All modules from previous tasks
  - FOLLOW pattern: RTIC 2.0 app macro and structure
  - PLACEMENT: src/main.rs application entry point

Task 9: CREATE testing infrastructure
  - IMPLEMENT: Unit tests for all modules (mathematics, configuration, drivers)
  - IMPLEMENT: Hardware-in-the-loop test framework
  - IMPLEMENT: Integration tests for complete system
  - FOLLOW pattern: defmt-test for embedded testing
  - CONFIGURE: Test features and conditional compilation
  - PLACEMENT: tests/ directory structure

Task 10: CREATE host tools and utilities
  - IMPLEMENT: Python scripts for device control and log monitoring
  - IMPLEMENT: USB HID communication libraries
  - IMPLEMENT: Automated test execution tools
  - FOLLOW pattern: hidapi for cross-platform USB HID access
  - INTEGRATE: With embedded USB HID implementation
  - PLACEMENT: host_tools/ directory structure

Task 11: CREATE build system and documentation
  - IMPLEMENT: build.rs for compile-time configuration
  - CREATE: README.md with setup and usage instructions
  - CREATE: CLAUDE.md with AI context and architecture decisions
  - CONFIGURE: VS Code integration with launch.json and tasks.json
  - DOCUMENT: Development workflow and testing procedures
  - PLACEMENT: Root directory and .vscode/ configuration
```

### Implementation Patterns & Key Details

```rust
// RTIC 2.0 application structure pattern
#[rtic::app(device = rp2040_hal::pac, peripherals = true)]
mod app {
    use super::*;
    
    // PATTERN: Shared resources with mutex protection
    #[shared]
    struct Shared {
        waveform_config: WaveformConfig,
        battery_state: BatteryState,
        usb_command_queue: heapless::spsc::Queue<UsbCommand, 8>,
    }

    // PATTERN: Local resources (no sharing)
    #[local]
    struct Local {
        pwm_channel: pwm::Channel<rp2040_hal::pwm::Pwm0, pwm::A>,
        adc: adc::Adc,
        led_pin: gpio::Pin<gpio::bank0::Gpio25, gpio::Output<gpio::PushPull>>,
        usb_device: usb_device::UsbDevice<'static, rp2040_hal::usb::UsbBus>,
    }

    // CRITICAL: Priority 1 (highest) for waveform generation - timing critical
    #[task(priority = 1, shared = [waveform_config], local = [pwm_channel])]
    fn waveform_update(mut ctx: waveform_update::Context) {
        // PATTERN: Lock shared resource for read
        let config = ctx.shared.waveform_config.lock(|cfg| *cfg);
        
        // PATTERN: Calculate waveform value based on configuration
        let duty_value = calculate_waveform_duty_cycle(&config, get_current_phase());
        
        // CRITICAL: Non-blocking PWM update to maintain timing
        ctx.local.pwm_channel.set_duty(duty_value);
    }

    // PATTERN: Medium priority for battery monitoring
    #[task(priority = 2, shared = [battery_state], local = [adc])]
    fn battery_monitor(mut ctx: battery_monitor::Context) {
        // PATTERN: ADC reading with error handling
        let adc_reading: u16 = ctx.local.adc.read(&mut adc_pin).unwrap_or(0);
        
        // PATTERN: State calculation based on PRD thresholds
        let new_state = match adc_reading {
            0..=1425 => BatteryState::Low,
            1426..=1674 => BatteryState::Normal,
            1675.. => BatteryState::Charging,
        };
        
        // PATTERN: Update shared state
        ctx.shared.battery_state.lock(|state| *state = new_state);
    }
}

// Flash configuration storage pattern
impl SystemConfig {
    // PATTERN: Atomic configuration update with error handling
    async fn save_to_flash(&self, flash: &mut impl Flash) -> Result<(), SystemError> {
        // CRITICAL: Serialize configuration with size limits
        let mut buffer = [0u8; CONFIG_MAX_SIZE];
        let serialized = postcard::to_slice(&self, &mut buffer)
            .map_err(|_| SystemError::ConfigurationInvalid)?;
        
        // PATTERN: Atomic sector erase and write
        flash.erase(CONFIG_SECTOR_ADDRESS, CONFIG_SECTOR_SIZE)
            .await.map_err(|_| SystemError::FlashOperationFailed)?;
        
        flash.write(CONFIG_SECTOR_ADDRESS, serialized)
            .await.map_err(|_| SystemError::FlashOperationFailed)?;
        
        Ok(())
    }
}

// USB HID implementation pattern
#[cfg(feature = "usb-logs")]
impl UsbHidHandler {
    // PATTERN: Non-blocking HID report sending
    fn send_log_message(&mut self, message: &LogMessage) -> Result<(), UsbError> {
        let report = self.format_log_report(message);
        
        // CRITICAL: Non-blocking send to prevent task blocking
        match self.hid_class.push_input(&report) {
            Ok(_) => Ok(()),
            Err(UsbError::WouldBlock) => {
                // PATTERN: Queue message for later retry
                self.pending_messages.enqueue(message.clone()).ok();
                Err(UsbError::WouldBlock)
            }
            Err(e) => Err(e),
        }
    }
}

// Waveform generation mathematical pattern
fn calculate_waveform_duty_cycle(config: &WaveformConfig, phase: f32) -> u16 {
    let normalized_phase = phase % 1.0; // 0.0 to 1.0
    
    // PATTERN: Waveform type blending based on factor
    let waveform_value = match config.waveform_factor {
        f if f <= 0.1 => sine_wave(normalized_phase),
        f if f >= 0.9 => square_wave(normalized_phase, config.duty_cycle_percent / 100.0),
        f if (0.4..=0.6).contains(&f) => sawtooth_wave(normalized_phase, config.duty_cycle_percent / 100.0),
        f => blend_waveforms(normalized_phase, f, config.duty_cycle_percent / 100.0),
    };
    
    // PATTERN: Apply amplitude scaling and convert to PWM duty cycle
    let scaled_value = waveform_value * (config.amplitude_percent / 100.0);
    (scaled_value * PWM_MAX_DUTY as f32) as u16
}
```

### Integration Points

```yaml
HARDWARE:
  - gpio_assignments: "GPIO 15 (PWM), GPIO 25 (LED), GPIO 26 (ADC)"
  - clock_config: "12MHz external crystal, 133MHz system clock via PLL"
  - usb_config: "USB device enumeration with HID class"

BUILD_SYSTEM:
  - target: "thumbv6m-none-eabi"
  - runner: "probe-rs run --chip RP2040" 
  - features: "usb-logs, battery-logs, pemf-logs, system-logs"

FLASH_LAYOUT:
  - application: "0x10000000 - 0x10180000 (1.5MB)"
  - configuration: "0x10180000 - 0x10200000 (512KB)"
  - wear_leveling: "Rotate across multiple sectors"

RTIC_PRIORITIES:
  - priority_1: "Waveform generation (highest)"
  - priority_2: "Battery monitoring (medium)" 
  - priority_3: "LED control, USB communication (lowest)"
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation - fix before proceeding
cargo check                          # Basic compilation check
cargo clippy -- -D warnings         # Linting with error escalation
cargo fmt                           # Code formatting

# Target-specific validation
cargo check --target thumbv6m-none-eabi --all-features
cargo clippy --target thumbv6m-none-eabi --all-features -- -D warnings

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Host-based unit tests (mathematical functions, configuration logic)
cargo test --lib                    # Library unit tests
cargo test --test unit_tests        # Organized unit test modules

# Feature-specific testing
cargo test --features usb-logs       # USB logging tests
cargo test --features battery-logs   # Battery monitoring tests  
cargo test --features pemf-logs     # Waveform generation tests

# Coverage validation (if available)
cargo tarpaulin --out html           # Code coverage report

# Expected: All tests pass. If failing, debug root cause and fix implementation.
```

### Level 3: Integration Testing (Hardware Validation)

```bash
# Build for target platform
cargo build --target thumbv6m-none-eabi --release

# Flash device and verify basic operation
cargo run --target thumbv6m-none-eabi --bin main

# Hardware-in-the-loop testing (requires connected RP2040)
probe-rs list                        # Verify probe connection
probe-rs info --chip RP2040         # Verify chip detection

# USB enumeration testing (development build)
lsusb | grep -i "pico\|rp2040"      # Check USB device enumeration

# GPIO functionality testing
# Connect oscilloscope to GPIO 15 to verify PWM output
# Connect multimeter to GPIO 26 to verify ADC operation
# Observe GPIO 25 LED behavior

# Flash configuration testing
python host_tools/device_control.py --test-config
python host_tools/log_monitor.py --verify-logs

# Expected: Hardware responds correctly, USB enumerates, PWM generates correct waveforms
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Waveform accuracy testing (requires oscilloscope)
python host_tools/waveform_validator.py --frequency 10 --duty-cycle 33 --waveform sawtooth

# Battery monitoring accuracy testing (requires variable power supply)
python host_tools/battery_tester.py --test-thresholds

# Real-time constraint validation (requires logic analyzer)
python host_tools/timing_validator.py --check-priorities --duration 60s

# USB HID communication testing
python host_tools/usb_stress_test.py --commands 1000 --concurrent

# Configuration persistence testing (power cycle testing)
python host_tools/config_persistence_test.py --cycles 100

# Multi-subsystem integration testing
python host_tools/integration_test.py --full-system --duration 3600s

# Performance profiling (if profiling tools available)
probe-rs trace collect --chip RP2040 --duration 30s

# Memory usage validation
cargo bloat --target thumbv6m-none-eabi --release --crates

# Power consumption measurement (requires current measurement setup)
# Expected: <500mA during normal operation, specific waveform characteristics verified
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --all-features`
- [ ] No compilation errors: `cargo check --target thumbv6m-none-eabi --all-features`
- [ ] No linting warnings: `cargo clippy --target thumbv6m-none-eabi --all-features -- -D warnings`
- [ ] Code formatting correct: `cargo fmt --check`
- [ ] Memory usage within constraints: `cargo bloat --target thumbv6m-none-eabi --release`

### Feature Validation

- [ ] Basic PWM waveform generation functional (GPIO 15)
- [ ] ADC battery monitoring operational (GPIO 26) with correct thresholds
- [ ] LED control working (GPIO 25) with battery state indication
- [ ] USB HID enumeration successful in development builds
- [ ] Flash configuration system functional with defaults
- [ ] `cargo run` flashes device successfully without manual intervention
- [ ] All PRD timing requirements met (±1% PWM accuracy)
- [ ] Multi-priority RTIC task scheduling working correctly

### Code Quality Validation

- [ ] RTIC 2.0 patterns followed consistently
- [ ] Resource sharing properly implemented with mutex protection
- [ ] Error handling comprehensive and graceful
- [ ] Feature flags work correctly for conditional compilation
- [ ] Hardware abstraction layer properly structured
- [ ] Configuration system atomic and power-loss safe
- [ ] USB HID implementation follows standard patterns
- [ ] Mathematical algorithms accurate and efficient

### Documentation & Deployment

- [ ] README.md complete with setup instructions
- [ ] CLAUDE.md documents architecture decisions
- [ ] Code comments explain complex algorithms and hardware interactions
- [ ] VS Code configuration functional for debugging
- [ ] Host tools properly documented and functional
- [ ] Build system optimized for embedded target

---

## Anti-Patterns to Avoid

- ❌ Don't use blocking operations in high-priority RTIC tasks
- ❌ Don't ignore flash sector alignment and wear leveling requirements  
- ❌ Don't assume USB is always connected - handle disconnection gracefully
- ❌ Don't use floating-point arithmetic in interrupt contexts without consideration
- ❌ Don't hardcode GPIO pin numbers - use constants and configuration
- ❌ Don't skip proper clock configuration for USB functionality
- ❌ Don't use dynamic memory allocation in embedded contexts
- ❌ Don't ignore real-time timing constraints specified in PRDs
- ❌ Don't mix sync and async patterns within RTIC tasks
- ❌ Don't assume probe-rs will work without proper udev rules on Linux

## Confidence Score: 9/10

This PRP provides comprehensive, implementation-ready context with:
- Extensive research backing all technology choices
- Specific crate versions and configurations
- Complete project structure with clear responsibilities  
- Hardware-specific implementation details
- Comprehensive testing strategy
- Real-world validation commands
- Clear integration with all PRD requirements

The high confidence score reflects the depth of research conducted, specific technical details provided, and alignment with proven embedded Rust patterns for 2024/2025.