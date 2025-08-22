## Goal

**Feature Goal**: Implement USB HID logging system with build-time and runtime configuration that provides real-time debug information from the RP2040 device to host computers without requiring special drivers, while maintaining zero performance impact on core functionality and preserving ±1% timing accuracy for pEMF pulse generation.

**Deliverable**: A fully functional USB HID logging system with conditional compilation feature flags (usb-logs, battery-logs, pemf-logs, system-logs) and runtime control via USB commands, integrated with the existing RTIC task structure.

**Success Definition**: USB HID logging system successfully enumerates as a standard HID device with VID 0x1234/PID 0x5678, transmits log messages in real-time with proper formatting, maintains zero performance impact when disabled, preserves ±1% timing accuracy for pEMF pulses, and passes all validation tests including 24+ hour stability testing.

## User Persona (if applicable)

**Target User**: Embedded firmware developers and system integrators working with the Ass-Easy Loop device.

**Use Case**: Debugging and monitoring device behavior during development, testing, and field diagnostics without requiring special drivers or additional hardware.

**User Journey**: 
1. Developer enables logging features in Cargo.toml or uses runtime commands
2. Builds firmware with desired logging configuration
3. Connects device via USB
4. Runs host-side monitoring tool
5. Views real-time log messages with timestamps and categories
6. Adjusts logging verbosity or categories via USB commands
7. Analyzes system behavior and performance metrics

**Pain Points Addressed**: 
- No existing debug capability without special hardware
- No visibility into real-time system behavior
- No standardized logging format for diagnostics
- No build-time or runtime control over logging features
- No ability to adjust logging without reflashing firmware

## Why

- **Developer Productivity**: Enable real-time debugging without special hardware or drivers
- **System Integration**: Provide standardized logging interface compatible with existing USB HID infrastructure
- **Performance Optimization**: Maintain critical timing requirements while adding debug capabilities
- **Production Safety**: Ensure zero impact on production builds when logging is disabled
- **Operational Flexibility**: Allow runtime control of logging without reflashing firmware
- **Testing Validation**: Support comprehensive automated testing with detailed logging

## What

### Success Criteria

- [ ] Build-time configuration with feature flags works correctly
- [ ] Runtime control via USB commands functions properly
- [ ] Zero performance impact when logging disabled in production builds
- [ ] ±1% timing accuracy maintained for pEMF pulse generation with logging enabled
- [ ] USB HID device enumerates correctly with standard descriptors (VID 0x1234, PID 0x5678)
- [ ] Log messages transmitted in proper format with timestamps and categories
- [ ] Queue management handles overflow with FIFO behavior (32 message capacity)
- [ ] System continues operating normally during USB disconnect/reconnect
- [ ] Host-side utilities successfully receive and display log messages
- [ ] All automated tests pass including 24+ hour stability test

## All Needed Context

### Context Completeness Check

_Passed: "If someone knew nothing about this codebase, they would have everything needed to implement this successfully"_

### Documentation & References

```yaml
# MUST READ - Include these in your context window
- url: https://docs.rs/usbd-hid/0.8.2/usbd_hid/
  why: Core HID implementation patterns and report descriptor structures
  critical: Custom report descriptors must maintain 64-byte format for compatibility

- url: https://docs.rs/heapless/0.7.16/heapless/
  why: Memory-safe queue implementations without dynamic allocation
  critical: Queue operations must be non-blocking in RTIC context

- file: /home/dustin/projects/asseasyloop/src/main.rs
  why: Existing RTIC app structure and USB implementation patterns
  pattern: Follow existing shared resource locking and task spawning patterns
  gotcha: Tasks defined within main RTIC app, not in separate modules

- file: /home/dustin/projects/asseasyloop/src/types/usb_commands.rs
  why: Current HID report structure and descriptor implementation
  pattern: 64-byte vendor-defined HID reports with existing descriptor format
  gotcha: Must maintain existing enumeration compatibility

- file: /home/dustin/projects/asseasyloop/src/config/usb.rs
  why: USB device configuration and descriptor constants
  pattern: VID/PID and string descriptor management
  gotcha: Development VID/PID (0x1234:0x5678) for logging device

- docfile: /home/dustin/projects/asseasyloop/PRPs/ai_docs/battery_adc_mapping.md
  why: Battery ADC voltage mapping for accurate battery logging
  section: ADC Voltage Mapping specifications

- docfile: /home/dustin/projects/asseasyloop/PRPs/ai_docs/pemf_timing_specs.md
  why: pEMF timing requirements for performance validation
  section: Target frequency and timing tolerance specifications
```

### Current Codebase tree (run `tree` in the root of the project) to get an overview of the codebase

```bash
src/
├── main.rs              # RTIC app with USB device implementation  
├── lib.rs               # Library exports for testing
├── config/
│   ├── defaults.rs      # Default configurations
│   ├── flash_storage.rs # Flash storage utilities
│   ├── usb.rs          # USB device configuration (VID/PID/descriptors)
│   └── validation.rs    # Configuration validation
├── drivers/
│   ├── adc_battery.rs   # Battery monitoring (placeholder)
│   ├── led_control.rs   # LED driver (placeholder)
│   ├── pwm_waveform.rs  # PWM waveform generation (placeholder)
│   ├── usb_command_handler.rs # USB HID command parsing
│   └── usb_hid.rs      # USB HID driver (placeholder)
├── tasks/
│   ├── battery_monitor.rs    # Battery monitoring task (placeholder)
│   ├── led_manager.rs       # LED management task (placeholder)
│   ├── usb_handler.rs       # USB handling task (placeholder)
│   └── waveform_generator.rs # Waveform generation task (placeholder)
├── types/
│   ├── battery.rs       # Battery state types
│   ├── bootloader_types.rs # Bootloader state management
│   ├── errors.rs        # System error types
│   ├── usb_commands.rs  # USB command and HID report structures
│   └── waveform.rs      # Waveform configuration types
└── utils/
    ├── math.rs          # Mathematical utilities
    └── timing.rs        # Timing utilities
```

### Desired Codebase tree with files to be added and responsibility of file

```bash
src/
├── main.rs              # RTIC app with USB device implementation + logging tasks
├── lib.rs               # Library exports for testing
├── config/
│   ├── defaults.rs      # Default configurations
│   ├── flash_storage.rs # Flash storage utilities
│   ├── usb.rs          # USB device configuration (VID/PID/descriptors) + logging config
│   └── validation.rs    # Configuration validation
├── drivers/
│   ├── adc_battery.rs   # Battery monitoring (placeholder)
│   ├── led_control.rs   # LED driver (placeholder)
│   ├── pwm_waveform.rs  # PWM waveform generation (placeholder)
│   ├── usb_command_handler.rs # USB HID command parsing + logging command handling
│   ├── usb_hid.rs      # USB HID driver (placeholder)
│   └── logging.rs      # NEW: USB HID logging driver and message management
├── tasks/
│   ├── battery_monitor.rs    # Battery monitoring task (placeholder)
│   ├── led_manager.rs       # LED management task (placeholder)
│   ├── usb_handler.rs       # USB handling task (placeholder)
│   ├── waveform_generator.rs # Waveform generation task (placeholder)
│   └── logging.rs           # NEW: RTIC logging task for message transmission
├── types/
│   ├── battery.rs       # Battery state types
│   ├── bootloader_types.rs # Bootloader state management
│   ├── errors.rs        # System error types + logging errors
│   ├── usb_commands.rs  # USB command and HID report structures + log message types
│   ├── logging.rs       # NEW: Logging message types and structures
│   └── waveform.rs      # Waveform configuration types
└── utils/
    ├── math.rs          # Mathematical utilities
    └── timing.rs        # Timing utilities + timestamp functions
```

### Known Gotchas of our codebase & Library Quirks

```rust
// CRITICAL: RTIC 1.1.4 requires tasks to be defined within main.rs, not separate modules
// CRITICAL: USB polling must occur every 10ms or device disappears from enumeration
// CRITICAL: All operations must use stack-only allocation, no dynamic allocation
// CRITICAL: Logging tasks must use priority 3 (lower than critical USB tasks)
// CRITICAL: Queue operations must be non-blocking to prevent task blocking
// CRITICAL: Message queue size must be exactly 32 entries as per requirements
// CRITICAL: Battery ADC mappings: Low ≤ 1425 (≈ 3.1V), Normal 1425-1675, Charging ≥ 1675
// CRITICAL: pEMF timing: 2.0 Hz (500ms period) with ±1% tolerance
```

## Implementation Blueprint

### Data models and structure

Create the core data models for logging messages, queue management, and configuration.

```rust
// In src/types/logging.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LogCategory {
    Battery = 0,
    Pemf = 1,
    System = 2,
    Usb = 3,
}

#[derive(Clone, Copy, Debug)]
pub struct LogMessage {
    pub timestamp_ms: u32,
    pub level: LogLevel,
    pub category: LogCategory,
    pub content: [u8; 52], // 64 - 12 bytes for header
    pub content_len: u8,
}

// Configuration structure for runtime control
#[derive(Clone, Copy, Debug)]
pub struct LoggingConfig {
    pub enabled_categories: u8, // Bitmask for enabled categories
    pub verbosity_level: LogLevel,
    pub enabled: bool,
}

// In src/types/usb_commands.rs - extend existing structure
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LogReport {
    pub data: [u8; 64],  // 64-byte reports for HID compatibility
}

// New USB commands for logging control
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoggingUsbCommand {
    SetLogLevel(LogLevel),
    EnableCategory(LogCategory),
    DisableCategory(LogCategory),
    SetConfig(LoggingConfig),
    GetConfig,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: MODIFY Cargo.toml
  - ADD feature flags: usb-logs, battery-logs, pemf-logs, system-logs
  - ADD build configurations: production, development, testing
  - DEFINE feature dependencies and defaults
  - FOLLOW pattern: conditional compilation with feature dependencies
  - NAMING: Standard Rust feature naming conventions
  - PLACEMENT: [features] section in Cargo.toml

Task 2: CREATE src/types/logging.rs
  - IMPLEMENT: LogMessage, LogLevel, LogCategory, LoggingConfig data structures
  - IMPLEMENT: LogReport HID structure for USB transmission
  - FOLLOW pattern: Existing type definitions in src/types/
  - NAMING: PascalCase for structs, snake_case for fields
  - PLACEMENT: New file in src/types/ module

Task 3: MODIFY src/types/usb_commands.rs
  - EXTEND: Add LogReport structure for HID logging
  - ADD: LoggingUsbCommand enum for runtime control
  - EXTEND: Update HID descriptor for logging reports if needed
  - PRESERVE: Existing CommandReport and UsbCommand functionality
  - FOLLOW pattern: Existing HID report structure patterns
  - PLACEMENT: Extension of existing file

Task 4: CREATE src/drivers/logging.rs
  - IMPLEMENT: LogMessageQueue using heapless::spsc::Queue with 32 message capacity
  - IMPLEMENT: Message formatting and serialization functions
  - IMPLEMENT: Conditional compilation for feature-specific logging
  - IMPLEMENT: Runtime configuration management
  - FOLLOW pattern: Existing driver structure in src/drivers/
  - NAMING: Module-level functions for queue operations
  - DEPENDENCIES: Import types from Task 2
  - PLACEMENT: New driver file in src/drivers/

Task 5: MODIFY src/drivers/usb_command_handler.rs
  - EXTEND: Add parsing logic for LoggingUsbCommand
  - IMPLEMENT: Runtime configuration update handlers
  - PRESERVE: Existing USB command handling functionality
  - FOLLOW pattern: Existing command parsing structure
  - DEPENDENCIES: Import logging types and commands
  - PLACEMENT: Extension of existing file

Task 6: MODIFY src/main.rs
  - ADD: Logging task implementation with RTIC task structure
  - ADD: Shared resource for log message queue and logging configuration
  - ADD: Conditional compilation for logging tasks and resources
  - IMPLEMENT: Error handling for USB transmission with retry logic
  - FOLLOW pattern: Existing RTIC task and shared resource patterns
  - NAMING: logging_transmit_task with priority 3
  - DEPENDENCIES: Import drivers from Task 4
  - PLACEMENT: Within existing RTIC app structure

Task 7: MODIFY src/config/usb.rs
  - ADD: Logging-specific configuration constants (VID 0x1234, PID 0x5678)
  - EXTEND: Add logging configuration defaults
  - PRESERVE: Existing USB device configuration
  - FOLLOW pattern: Existing configuration structure
  - PLACEMENT: Extension of existing module

Task 8: CREATE host-side logging utilities
  - IMPLEMENT: Python utility using hidapi for log monitoring
  - IMPLEMENT: Command-line interface for log control
  - IMPLEMENT: Log file saving and analysis capabilities
  - FOLLOW pattern: Standard hidapi usage for cross-platform compatibility
  - PLACEMENT: New directory for host utilities
```

### Implementation Patterns & Key Details

```rust
// Show critical patterns and gotchas - keep concise, focus on non-obvious details

// Example: Log message formatting pattern with proper byte ordering
fn format_log_message(msg: &LogMessage) -> LogReport {
    let mut report = LogReport { data: [0u8; 64] };
    
    // PATTERN: Proper byte ordering for multi-byte values (little-endian)
    report.data[0] = (msg.timestamp_ms & 0xFF) as u8;
    report.data[1] = ((msg.timestamp_ms >> 8) & 0xFF) as u8;
    report.data[2] = ((msg.timestamp_ms >> 16) & 0xFF) as u8;
    report.data[3] = ((msg.timestamp_ms >> 24) & 0xFF) as u8;
    report.data[4] = msg.level as u8;
    report.data[5] = msg.category as u8;
    report.data[6] = msg.content_len;
    report.data[7] = 0; // Reserved/padding
    
    // PATTERN: Content with truncation and "..." indicator
    let copy_len = core::cmp::min(msg.content_len as usize, 51);
    report.data[8..(8 + copy_len)].copy_from_slice(&msg.content[..copy_len]);
    
    // PATTERN: Add "..." indicator for truncated messages
    if msg.content_len as usize > 51 {
        report.data[59] = b'.';
        report.data[60] = b'.';
        report.data[61] = b'.';
    }
    
    // CRITICAL: Message must fit exactly in 64-byte HID report
    report
}

// Example: RTIC logging task pattern with error handling
#[task(
    shared = [hid_class, log_queue, logging_config],
    priority = 3  // Lower than critical USB tasks
)]
fn logging_transmit_task(mut ctx: logging_transmit_task::Context) {
    // PATTERN: Check if logging is enabled before processing
    let is_enabled = ctx.shared.logging_config.lock(|config| config.enabled);
    if !is_enabled {
        logging_transmit_task::spawn_after(Duration::<u64, 1, 1000>::millis(100)).unwrap();
        return;
    }
    
    // PATTERN: Non-blocking queue operations to prevent task blocking
    // GOTCHA: Must check queue before locking shared resources
    if let Some(message) = ctx.shared.log_queue.lock(|queue| queue.dequeue()) {
        let report = format_log_message(&message);
        
        // PATTERN: Error handling for USB transmission with retry logic
        let mut retry_count = 0;
        loop {
            match ctx.shared.hid_class.lock(|hid_class| {
                hid_class.push_raw_input(&report.data)
            }) {
                Ok(_) => break, // Success
                Err(_) if retry_count < 3 => {
                    retry_count += 1;
                    // Exponential backoff: 10ms, 20ms, 40ms
                    cortex_m::asm::delay(10000 * (1 << retry_count));
                }
                Err(_) => {
                    // Log transmission failed after 3 retries
                    // Continue with next message to prevent blocking
                    break;
                }
            }
        }
    }
    
    // PATTERN: Reschedule task for periodic checking
    logging_transmit_task::spawn_after(Duration::<u64, 1, 1000>::millis(50)).unwrap();
}

// Example: Conditional compilation pattern with feature-specific logging
#[cfg(feature = "battery-logs")]
pub fn log_battery_status(state: BatteryState, adc_value: u16, voltage: f32) {
    // PATTERN: Feature-specific logging implementation
    // Only compiled when battery-logs feature is enabled
    let msg = LogMessage {
        timestamp_ms: get_timestamp_ms(),
        level: LogLevel::Info,
        category: LogCategory::Battery,
        content: format_battery_content(state, adc_value, voltage),
        content_len: content_length,
    };
    
    // PATTERN: Safe queue enqueue with overflow handling (FIFO)
    LOG_QUEUE.lock(|queue| {
        if queue.enqueue(msg).is_err() {
            // FIFO behavior - oldest message automatically discarded
            // This is the required behavior per specifications
        }
    });
}

// Example: Runtime configuration update pattern
fn handle_logging_command(command: LoggingUsbCommand) {
    match command {
        LoggingUsbCommand::SetLogLevel(level) => {
            LOGGING_CONFIG.lock(|config| {
                config.verbosity_level = level;
            });
        }
        LoggingUsbCommand::EnableCategory(category) => {
            LOGGING_CONFIG.lock(|config| {
                config.enabled_categories |= 1 << (category as u8);
            });
        }
        LoggingUsbCommand::DisableCategory(category) => {
            LOGGING_CONFIG.lock(|config| {
                config.enabled_categories &= !(1 << (category as u8));
            });
        }
        _ => {}
    }
}
```

### Integration Points

```yaml
DATABASE:
  - none: No database integration required for embedded logging

CONFIG:
  - add to: Cargo.toml
  - pattern: |
      [features]
      default = []
      usb-logs = []
      battery-logs = []
      pemf-logs = []
      system-logs = []
      development = ["usb-logs", "battery-logs", "pemf-logs", "system-logs"]
      testing = ["development"]
      production = []

ROUTES:
  - none: No HTTP routes required for embedded USB HID logging

USB:
  - extend: src/types/usb_commands.rs
  - pattern: Additional HID report descriptor for logging if needed
  - integration: Existing USB device framework in src/main.rs
  - config: Separate VID/PID (0x1234:0x5678) for logging device

HOST:
  - create: Python utilities directory for host-side tools
  - deps: hidapi library for cross-platform HID access
  - pattern: Command-line interface for log monitoring and control
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation - fix before proceeding
cargo check --features development  # Check with all logging features enabled
cargo fmt -- --check                # Ensure consistent formatting
cargo clippy --features development # Linting with all features

# Project-wide validation
cargo check
cargo fmt -- --check
cargo clippy

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test each component as it's created
cargo test --lib types::logging     # Test logging type implementations
cargo test --lib drivers::logging   # Test logging driver functionality
cargo test --lib drivers::usb_command_handler  # Test USB command parsing

# Feature-specific testing
cargo test --features usb-logs      # Test USB logging functionality
cargo test --features battery-logs  # Test battery logging functionality
cargo test --features development   # Test all logging features together

# Queue behavior testing
cargo test --lib drivers::logging::queue  # Test FIFO queue behavior

# Expected: All tests pass. If failing, debug root cause and fix implementation.
```

### Level 3: Integration Testing (System Validation)

```bash
# Build with different configurations
cargo build --features production   # Should have zero logging overhead
cargo build --features development  # Should include all logging features
cargo build --features testing      # Should include test commands

# Binary size analysis for zero-overhead verification
cargo size --features production -- -A
cargo size --features development -- -A

# Firmware flashing and basic USB enumeration
cargo run --features development    # Flash and test USB enumeration

# Host-side integration testing
python tests/tools/usb_log_test.py --features development

# Runtime control testing
python tests/tools/runtime_control_test.py --features development

# Expected: All integrations working, proper responses, no connection errors
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Real-time constraint validation
# Test pEMF timing accuracy with logging enabled
cargo run --features testing --example pemf_timing_test

# Long-term stability testing (24+ hours)
python tests/tools/long_term_log_test.py --duration 86400

# Memory usage validation
# Monitor stack usage during extended logging operations
cargo run --features testing --example memory_usage_test

# USB disconnect/reconnect testing
# Test system behavior during USB cable manipulation
python tests/tools/usb_disconnect_test.py

# Queue overflow behavior testing
# Verify FIFO behavior with high logging volume
cargo run --features testing --example queue_overflow_test

# Multi-device testing scenarios
# Test multiple logging devices simultaneously
python tests/tools/multi_device_test.py

# Error condition testing
# Test USB transmission failures and retry logic
python tests/tools/error_handling_test.py

# Cross-platform host tool validation
# Test on Linux, Windows, macOS
python tests/tools/cross_platform_test.py

# Expected: All creative validations pass, performance meets requirements
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --features development`
- [ ] No linting errors: `cargo clippy --features development`
- [ ] No formatting issues: `cargo fmt -- --check`
- [ ] Binary size analysis shows zero overhead in production builds
- [ ] Queue overflow behavior verified (FIFO with 32 message capacity)

### Feature Validation

- [ ] All success criteria from "What" section met
- [ ] Manual testing successful: USB enumeration and log transmission
- [ ] Runtime control via USB commands functions correctly
- [ ] Error cases handled gracefully with proper error messages
- [ ] Integration points work as specified
- [ ] User persona requirements satisfied

### Code Quality Validation

- [ ] Follows existing codebase patterns and naming conventions
- [ ] File placement matches desired codebase tree structure
- [ ] Anti-patterns avoided (check against Anti-Patterns section)
- [ ] Dependencies properly managed and imported
- [ ] Configuration changes properly integrated

### Documentation & Deployment

- [ ] Code is self-documenting with clear variable/function names
- [ ] Logs are informative but not verbose
- [ ] Environment variables documented if new ones added
- [ ] Host-side utilities created for log monitoring
- [ ] Build configurations properly documented

---
## Anti-Patterns to Avoid

- ❌ Don't create new RTIC task modules - integrate tasks in main.rs
- ❌ Don't use dynamic allocation - stick to heapless collections
- ❌ Don't skip validation because "it should work"
- ❌ Don't ignore failing tests - fix them
- ❌ Don't block RTIC tasks with queue operations
- ❌ Don't hardcode values that should be config
- ❌ Don't catch all exceptions - be specific
- ❌ Don't exceed 32 message queue capacity
- ❌ Don't violate pEMF timing requirements