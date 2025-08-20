# Automatic Bootloader Entry PRP

---

## Goal

**Feature Goal**: Implement automatic bootloader entry functionality that enables fully automated testing workflows by allowing host systems to remotely trigger the RP2040 device to enter bootloader mode without manual BOOTSEL button intervention.

**Deliverable**: Complete bootloader entry system with:
- Safe bootloader entry command processing via USB HID interface
- Coordinated task shutdown sequence in priority order
- Hardware state validation and safety mechanisms
- Host-side Python automation framework for CI/CD integration
- Authentication and security validation for bootloader commands

**Success Definition**: 
- 100% automated firmware flashing without manual intervention
- <1% failure rate for bootloader entry under normal conditions
- Zero impact on normal operation when test infrastructure inactive
- Complete bootloader entry within 500ms from command reception
- Graceful recovery from all error conditions within 5 seconds

## User Persona

**Target User**: Test automation engineers and CI/CD pipeline maintainers

**Use Case**: Automated firmware flashing and testing in build farm environments

**User Journey**: 
1. CI/CD pipeline triggers test execution
2. Python test framework discovers connected RP2040 devices
3. Framework sends authenticated bootloader entry command via USB HID
4. Device safely shuts down tasks and enters bootloader mode
5. Framework flashes new firmware and validates functionality
6. Process repeats for regression testing

**Pain Points Addressed**: 
- Eliminates manual BOOTSEL button intervention requirement
- Enables 24/7 automated testing without physical access
- Reduces testing cycle time from hours to minutes
- Prevents firmware flashing errors from incomplete shutdowns

## Why

- **Business value**: Enables continuous integration for embedded firmware development
- **Integration**: Builds on existing USB HID logging infrastructure and RTIC task system
- **Problems solved**: Manual testing bottlenecks, unreliable firmware updates, lack of automated regression testing

## What

The system provides remote bootloader entry capabilities through authenticated USB HID commands with comprehensive safety mechanisms:

### Core Functionality
- USB HID command interface for bootloader entry requests
- Multi-level authentication with checksum validation
- Hardware state validation before bootloader entry
- Coordinated RTIC task shutdown in reverse priority order
- Safe hardware state enforcement (MOSFET OFF, pEMF inactive)
- Automatic return from bootloader after configurable timeout

### Host-Side Automation
- Python framework for device discovery and management
- Parallel testing support across multiple devices
- CI/CD integration with standard test result formats
- Comprehensive logging and error reporting

### Success Criteria

- [ ] Bootloader entry completes within 500ms of command reception
- [ ] Hardware safety validation prevents unsafe bootloader entry
- [ ] Task shutdown coordination maintains system integrity
- [ ] Authentication prevents unauthorized bootloader commands
- [ ] Host framework supports multi-device testing scenarios
- [ ] CI/CD integration works with GitHub Actions and Jenkins
- [ ] Zero impact on normal operation timing (<1% tolerance deviation)
- [ ] Recovery mechanisms handle all error conditions gracefully

## All Needed Context

### Context Completeness Check

_This PRP provides everything needed for implementation: working example codebase patterns, specific file structures, authentication mechanisms, RTIC coordination patterns, hardware safety requirements, and host-side automation frameworks._

### Documentation & References

```yaml
# MUST READ - Essential implementation references
- url: https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf#section=2.8.2
  why: Official RP2040 bootloader entry mechanisms and magic values
  critical: Memory layout 0x20041FFC for bootloader magic, reset vector configuration

- url: https://rtic.rs/2/book/en/by-example/app_priorities.html
  why: RTIC v2 priority-based task coordination and scheduling patterns
  critical: Priority levels 1-3, task shutdown coordination, resource management

- url: https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-124r2.pdf
  why: USB HID security best practices and authentication patterns
  critical: Command validation, rate limiting, cryptographic authentication

- file: /home/dustin/projects/ass-easy-loop/src/bootloader.rs
  why: Complete bootloader entry implementation with safety mechanisms
  pattern: BootloaderEntryManager, TaskShutdownSequence, HardwareState validation
  gotcha: Hardware safety requires MOSFET OFF and pEMF inactive before entry

- file: /home/dustin/projects/ass-easy-loop/src/command/parsing.rs
  why: USB HID 64-byte command report format and authentication patterns
  pattern: CommandReport structure, XOR checksum validation, error handling
  gotcha: Must use exactly 64-byte reports, checksum all header and payload bytes

- file: /home/dustin/projects/ass-easy-loop/src/main.rs
  why: RTIC app structure, task priorities, and resource management patterns
  pattern: 3-level priority hierarchy, shared/local resource allocation, task spawning
  gotcha: USB bus requires static allocation, high priority tasks cannot be preempted

- file: /home/dustin/projects/ass-easy-loop/test_framework/device_manager.py
  why: Host-side device discovery, connection management, and parallel testing
  pattern: USB HID device enumeration, multi-device coordination, CI/CD integration
  gotcha: Requires libhidapi system dependencies, device identification by VID/PID

- docfile: PRPs/ai_docs/rtic2-patterns.md
  why: RTIC v2 task coordination patterns and shutdown mechanisms
  section: Task shutdown sequences and priority-based resource management
```

### Current Codebase Tree

```bash
asseasyloop/
├── src/
│   ├── main.rs                    # Basic RTIC app stub - needs full implementation
│   ├── drivers/
│   │   ├── usb_hid.rs            # Placeholder - needs complete USB HID implementation
│   │   └── mod.rs
│   ├── tasks/
│   │   ├── usb_handler.rs        # Placeholder - needs bootloader command handling
│   │   └── mod.rs
│   ├── types/
│   │   ├── usb_commands.rs       # Basic EnterBootloader command - needs expansion
│   │   ├── errors.rs             # Basic SystemError - needs bootloader error types
│   │   └── mod.rs
│   └── utils/
├── host_tools/                   # Basic Python scripts - needs full framework
│   ├── device_control.py         # Placeholder
│   └── requirements.txt
├── Cargo.toml                   # Basic dependencies - needs USB HID and RTIC additions
└── tests/                       # Basic test structure - needs bootloader tests
```

### Desired Codebase Tree

```bash
asseasyloop/
├── src/
│   ├── main.rs                           # Complete RTIC app with bootloader tasks
│   ├── bootloader/
│   │   ├── mod.rs                        # Bootloader module exports
│   │   ├── entry_manager.rs              # BootloaderEntryManager with state machine
│   │   ├── task_shutdown.rs              # TaskShutdownSequence coordination
│   │   ├── hardware_safety.rs            # HardwareSafetyManager validation
│   │   └── reset_handler.rs              # Magic value and reset implementation
│   ├── drivers/
│   │   ├── usb_hid.rs                   # Complete USB HID implementation
│   │   └── usb_command_handler.rs       # Command processing and validation
│   ├── tasks/
│   │   ├── bootloader_task.rs           # High-priority bootloader entry task
│   │   ├── usb_command_task.rs          # Command reception and processing
│   │   └── system_monitor_task.rs       # Hardware state monitoring
│   ├── types/
│   │   ├── bootloader_types.rs          # BootloaderError, EntryState, HardwareState
│   │   ├── usb_commands.rs              # Complete command types and responses
│   │   └── authentication.rs            # Security validation types
│   └── security/
│       ├── mod.rs                       # Security module exports
│       ├── command_validator.rs         # Authentication and rate limiting
│       └── checksum.rs                  # XOR checksum implementation
├── host_tools/
│   ├── bootloader_framework/
│   │   ├── __init__.py                  # Framework package
│   │   ├── device_manager.py            # Device discovery and connection
│   │   ├── command_handler.py           # Command transmission and responses
│   │   ├── bootloader_manager.py        # Bootloader entry coordination
│   │   ├── test_sequencer.py            # Test execution orchestration
│   │   ├── result_collector.py          # Test result aggregation
│   │   └── ci_integration.py            # CI/CD pipeline integration
│   ├── examples/
│   │   ├── basic_bootloader_entry.py    # Simple bootloader entry example
│   │   ├── multi_device_testing.py      # Parallel device testing
│   │   └── ci_pipeline_example.py       # CI/CD integration example
│   └── configs/
│       ├── device_profiles.json         # Device configuration profiles
│       └── test_scenarios.yaml          # Test scenario definitions
├── tests/
│   ├── hardware/
│   │   ├── test_bootloader_entry.rs     # Hardware bootloader entry tests
│   │   └── test_safety_mechanisms.rs    # Hardware safety validation tests
│   ├── integration/
│   │   ├── test_usb_commands.rs         # USB command integration tests
│   │   └── test_task_coordination.rs    # RTIC task coordination tests
│   └── host_tools/
│       ├── test_device_manager.py       # Python framework tests
│       └── test_ci_integration.py       # CI/CD integration tests
└── ci/
    ├── github_actions.yml               # GitHub Actions workflow
    └── jenkins_pipeline.groovy          # Jenkins pipeline definition
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: RP2040 bootloader magic value and memory location
const BOOTLOADER_MAGIC: u32 = 0xB007C0DE;
const BOOTLOADER_MAGIC_ADDR: *mut u32 = 0x20041FFC as *mut u32; // End of 264KB SRAM

// CRITICAL: USB HID requires static bus allocator allocation
static mut USB_BUS: Option<UsbBusAllocator<UsbBus>> = None;

// CRITICAL: RTIC v2 requires cortex-m-rtic = "2.2.0" with thumbv6-backend feature
// rtic-monotonics = { version = "2.1.0", features = ["rp2040"] }

// CRITICAL: Hardware safety - MOSFET GPIO15 must be LOW before bootloader entry
const MOSFET_GPIO_PIN: u8 = 15; // GPIO15 controls MOSFET, must be OFF for safety

// CRITICAL: Task shutdown sequence must be reverse priority order
// High (3) -> Medium (2) -> Low (1) with 5000ms timeout

// CRITICAL: USB HID reports must be exactly 64 bytes
// Format: [Command Type:1][Command ID:1][Payload Length:1][Auth Token:1][Payload:60]

// CRITICAL: Authentication uses XOR checksum of all header and payload bytes
let checksum = command_type ^ command_id ^ payload_length ^ payload_bytes_xor;

// CRITICAL: Reset sequence requires memory barriers and specific SCB register write
cortex_m::asm::dsb(); cortex_m::asm::isb(); // Ensure write completion
scb.aircr.write(0x05FA0004); // VECTKEY | SYSRESETREQ

// CRITICAL: Python host framework requires libhidapi system dependency
// Ubuntu/Debian: sudo apt-get install libhidapi-dev
// Device identification: VID=0x2E8A, PID=0x000A (normal), PID=0x0003 (bootloader)
```

## Implementation Blueprint

### Data Models and Structure

```rust
// Core bootloader data models for type safety and consistency

// Bootloader entry state machine
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BootloaderEntryState {
    Normal,                  // Normal operation
    EntryRequested,         // Entry requested, starting validation
    ValidatingHardware,     // Validating hardware state
    ShuttingDownTasks,      // Shutting down tasks in sequence
    FinalSafetyCheck,       // Final hardware safety verification
    ReadyForBootloader,     // Ready to enter bootloader
    EntryFailed,           // Entry failed, return to normal
}

// Hardware state validation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HardwareState {
    pub mosfet_state: bool,        // Must be false (LOW) for safe entry
    pub led_state: bool,           // Can be any state (will be reset)
    pub adc_active: bool,          // Can be active (non-critical)
    pub usb_transmitting: bool,    // Can be transmitting (non-critical)
    pub pemf_pulse_active: bool,   // Must be false (INACTIVE) for safety
}

// Task priority hierarchy for shutdown coordination
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum TaskPriority {
    Low = 1,     // LED control, USB polling, diagnostics
    Medium = 2,  // Battery monitoring, USB HID transmission
    High = 3,    // pEMF pulse generation (timing-critical)
}

// USB command report structure (64-byte standardized format)
#[derive(Clone, Debug, PartialEq)]
pub struct CommandReport {
    pub command_type: u8,    // Command type identifier
    pub command_id: u8,      // Sequence number for tracking
    pub payload_length: u8,  // Payload size (0-60 bytes)
    pub auth_token: u8,      // XOR checksum for validation
    pub payload: Vec<u8, 60>, // Command-specific data
}

// Bootloader error types for comprehensive error handling
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BootloaderError {
    UnsafeHardwareState,     // Hardware not safe for entry
    TaskShutdownFailed,      // Task shutdown failed/timeout
    HardwareValidationFailed, // Hardware validation failed
    SystemBusy,              // System busy with critical operations
    EntryInterrupted,        // Bootloader entry was interrupted
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/bootloader/mod.rs
  - IMPLEMENT: Module exports and public interface
  - FOLLOW pattern: src/types/mod.rs (module structure)
  - NAMING: snake_case module names, clear public exports
  - PLACEMENT: Core bootloader functionality module

Task 2: CREATE src/bootloader/hardware_safety.rs  
  - IMPLEMENT: HardwareState, HardwareSafetyManager structs
  - FOLLOW pattern: /home/dustin/projects/ass-easy-loop/src/bootloader.rs:85-350 (safety validation)
  - NAMING: HardwareState fields match hardware components, validation methods return Results
  - DEPENDENCIES: Import error types from bootloader_types
  - CRITICAL: MOSFET GPIO15 must be LOW, pEMF pulse must be INACTIVE for safety

Task 3: CREATE src/bootloader/task_shutdown.rs
  - IMPLEMENT: TaskShutdownSequence, TaskShutdownStatus enums  
  - FOLLOW pattern: /home/dustin/projects/ass-easy-loop/src/bootloader.rs:125-269 (shutdown coordination)
  - NAMING: TaskShutdownSequence methods use imperative verbs (start_shutdown, update_progress)
  - DEPENDENCIES: Import TaskPriority from bootloader_types
  - CRITICAL: Reverse priority shutdown (High -> Medium -> Low), 5000ms total timeout

Task 4: CREATE src/bootloader/entry_manager.rs
  - IMPLEMENT: BootloaderEntryManager struct with state machine
  - FOLLOW pattern: /home/dustin/projects/ass-easy-loop/src/bootloader.rs:353-655 (entry coordination)
  - NAMING: BootloaderEntryManager methods return Result types for error handling
  - DEPENDENCIES: Import hardware_safety, task_shutdown modules
  - CRITICAL: State machine validation, timeout enforcement, error recovery

Task 5: CREATE src/bootloader/reset_handler.rs
  - IMPLEMENT: Magic value writing and system reset functionality
  - FOLLOW pattern: /home/dustin/projects/ass-easy-loop/src/bootloader.rs:558-598 (reset implementation)
  - NAMING: enter_bootloader_mode function, magic constants in SCREAMING_SNAKE_CASE
  - CRITICAL: Memory barriers (dsb/isb), SCB AIRCR register write 0x05FA0004

Task 6: CREATE src/types/bootloader_types.rs
  - IMPLEMENT: BootloaderError, BootloaderEntryState, TaskPriority enums
  - FOLLOW pattern: /home/dustin/projects/ass-easy-loop/src/bootloader.rs:31-67 (error types)
  - NAMING: Error types end with 'Error', State types end with 'State'
  - PLACEMENT: Type definitions in types module

Task 7: CREATE src/security/command_validator.rs
  - IMPLEMENT: AuthenticationValidator, rate limiting, security validation
  - FOLLOW pattern: /home/dustin/projects/ass-easy-loop/src/command/parsing.rs:AuthenticationValidator
  - NAMING: validate_* methods return bool or Result types
  - CRITICAL: XOR checksum validation, rate limiting (max 1 command per 10 seconds)

Task 8: CREATE src/drivers/usb_command_handler.rs
  - IMPLEMENT: UsbCommandHandler with 64-byte report processing
  - FOLLOW pattern: /home/dustin/projects/ass-easy-loop/src/command/handler.rs:19-190 (command processing)
  - NAMING: process_* methods, create_* response methods
  - DEPENDENCIES: Import CommandReport, AuthenticationValidator
  - CRITICAL: Exactly 64-byte reports, FIFO command queuing

Task 9: MODIFY src/types/usb_commands.rs
  - ENHANCE: Add bootloader command types, response formats
  - FOLLOW pattern: /home/dustin/projects/ass-easy-loop/src/command/parsing.rs:TestCommand enum
  - ADD: EnterBootloader = 0x80, SystemStateQuery = 0x81 command types
  - PRESERVE: Existing UsbCommand enum structure

Task 10: CREATE src/tasks/bootloader_task.rs
  - IMPLEMENT: High-priority RTIC task for bootloader entry coordination
  - FOLLOW pattern: /home/dustin/projects/ass-easy-loop/src/main.rs:1754 (bootloader_entry_task)
  - NAMING: bootloader_entry_task with priority = 3 (highest)
  - DEPENDENCIES: Import BootloaderEntryManager, shared resources
  - CRITICAL: Cannot be preempted, coordinates shutdown sequence

Task 11: CREATE src/tasks/usb_command_task.rs
  - IMPLEMENT: Medium-priority RTIC task for USB command processing  
  - FOLLOW pattern: /home/dustin/projects/ass-easy-loop/src/main.rs:usb_hid_task (priority = 2)
  - NAMING: usb_command_task with appropriate priority level
  - DEPENDENCIES: Import UsbCommandHandler, command processing logic

Task 12: MODIFY src/main.rs
  - ENHANCE: Complete RTIC app with bootloader tasks and resources
  - FOLLOW pattern: /home/dustin/projects/ass-easy-loop/src/main.rs:227-466 (RTIC app structure)
  - ADD: Bootloader-related shared resources, task spawning, USB initialization
  - PRESERVE: Existing project structure and naming conventions
  - CRITICAL: Static USB bus allocation, proper resource sharing

Task 13: CREATE host_tools/bootloader_framework/device_manager.py
  - IMPLEMENT: UsbHidDeviceManager class with multi-device support
  - FOLLOW pattern: /home/dustin/projects/ass-easy-loop/test_framework/device_manager.py:37-100
  - NAMING: CamelCase class names, snake_case method names
  - CRITICAL: VID=0x2E8A, PID=0x000A (normal), PID=0x0003 (bootloader)

Task 14: CREATE host_tools/bootloader_framework/command_handler.py
  - IMPLEMENT: Command transmission, authentication, response handling
  - FOLLOW pattern: /home/dustin/projects/ass-easy-loop/test_framework/command_handler.py
  - NAMING: CommandHandler class with send_* and receive_* methods
  - CRITICAL: 64-byte reports, XOR checksum calculation, timeout handling

Task 15: CREATE host_tools/bootloader_framework/bootloader_manager.py
  - IMPLEMENT: High-level bootloader entry coordination
  - NAMING: BootloaderManager class with enter_bootloader_mode method
  - DEPENDENCIES: Import device_manager, command_handler modules

Task 16: CREATE tests/hardware/test_bootloader_entry.rs
  - IMPLEMENT: Hardware-in-loop tests for bootloader functionality
  - FOLLOW pattern: /home/dustin/projects/ass-easy-loop/tests/hardware_validation_tests.rs
  - NAMING: test_* functions with descriptive scenario names
  - COVERAGE: Safety validation, task shutdown, state machine transitions
```

### Integration Points

```yaml
RTIC_APP:
  - shared_resources: "bootloader_manager: BootloaderEntryManager"
  - task_priorities: "bootloader_entry_task priority = 3 (highest)"
  - spawning: "bootloader_entry_task::spawn(command_id, timeout_ms).ok()"

USB_CONFIGURATION:
  - vendor_id: "0x2E8A  # Raspberry Pi Foundation"
  - product_id: "0x000A  # Custom HID device"
  - report_size: "64 bytes exactly"
  - hid_class: "HIDClass::new(usb_bus_ref, LogReport::descriptor(), 60)"

HARDWARE_SAFETY:
  - gpio_validation: "GPIO15 (MOSFET) must be LOW before bootloader entry"
  - pemf_validation: "pEMF pulse must be INACTIVE before entry"
  - timeout_enforcement: "Hardware validation timeout: 500ms"

AUTHENTICATION:
  - checksum_algorithm: "XOR of command_type ^ command_id ^ payload_length ^ payload_bytes"
  - rate_limiting: "Maximum 1 bootloader command per 10 seconds"
  - validation: "Command format, length, and checksum validation"

HOST_FRAMEWORK:
  - device_discovery: "USB HID enumeration with VID/PID filtering"
  - parallel_testing: "Multi-device support with independent status tracking"
  - ci_integration: "JUnit XML and JSON result formats"
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation - fix before proceeding
cargo check                           # Basic syntax validation
cargo clippy -- -D warnings          # Linting with strict warnings
cargo fmt                            # Code formatting

# Python framework validation  
cd host_tools && python -m py_compile bootloader_framework/*.py
cd host_tools && python -m flake8 bootloader_framework/

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test each Rust component as created
cargo test bootloader::hardware_safety::tests --lib
cargo test bootloader::task_shutdown::tests --lib
cargo test bootloader::entry_manager::tests --lib

# Python framework unit tests
cd host_tools && python -m pytest bootloader_framework/tests/ -v

# Integration tests for USB command processing
cargo test usb_command_integration --test integration_tests

# Expected: All tests pass. If failing, debug root cause and fix implementation.
```

### Level 3: Integration Testing (System Validation)

```bash
# Hardware-in-loop testing with actual RP2040 device
cargo test --test hardware_tests test_bootloader_entry_timing
cargo test --test hardware_tests test_hardware_safety_validation
cargo test --test hardware_tests test_task_shutdown_coordination

# Host framework integration testing
cd host_tools && python examples/basic_bootloader_entry.py --test-mode
cd host_tools && python examples/multi_device_testing.py --device-count 2

# Bootloader functionality validation
cd host_tools && python -c "
from bootloader_framework import BootloaderManager
manager = BootloaderManager()
result = manager.test_bootloader_sequence()
assert result.success, f'Bootloader test failed: {result.error}'
print('Bootloader integration test passed')
"

# Expected: All hardware tests pass, actual bootloader entry works, device recovery successful
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Performance impact validation
cargo test --release test_bootloader_performance_impact
# Expected: <1% impact on normal operation timing

# Multi-device parallel testing
cd host_tools && python examples/stress_test_parallel_devices.py --device-count 4 --duration 300
# Expected: All devices handle bootloader commands reliably

# CI/CD pipeline integration testing
cd ci && python test_github_actions_integration.py
cd ci && python test_jenkins_pipeline_integration.py
# Expected: CI integration works with standard result formats

# Failure recovery validation  
cargo test test_bootloader_failure_recovery --test integration_tests
# Expected: Graceful recovery from hardware failures, communication timeouts

# Security validation
cargo test test_authentication_security --test security_tests
cd host_tools && python security_tests/test_rate_limiting.py
# Expected: Unauthorized commands rejected, rate limiting active

# Long-term stability testing
cargo test --release test_extended_bootloader_cycles --test stability_tests -- --ignored
# Expected: 1000+ bootloader cycles without degradation
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All Rust tests pass: `cargo test --all`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Code formatted: `cargo fmt --check`
- [ ] Python tests pass: `python -m pytest host_tools/ -v`

### Feature Validation

- [ ] Bootloader entry completes within 500ms from USB command
- [ ] Hardware safety validation prevents unsafe entry conditions
- [ ] Task shutdown coordination works in reverse priority order
- [ ] Authentication prevents unauthorized bootloader commands
- [ ] Multi-device testing framework operates in parallel
- [ ] CI/CD integration produces standard test result formats
- [ ] Error recovery mechanisms handle all failure scenarios

### Code Quality Validation

- [ ] Follows RTIC task priority patterns from working example
- [ ] USB HID implementation matches 64-byte report format
- [ ] File placement matches desired codebase tree structure
- [ ] Bootloader safety patterns match hardware requirements
- [ ] Authentication follows security best practices
- [ ] Python framework follows existing project conventions

### Documentation & Deployment

- [ ] Code is self-documenting with clear type definitions
- [ ] Error messages provide actionable debugging information
- [ ] Host framework provides comprehensive usage examples
- [ ] CI/CD workflows integrate with existing infrastructure

---

## Anti-Patterns to Avoid

- ❌ Don't skip hardware safety validation - always check MOSFET and pEMF state
- ❌ Don't ignore task shutdown sequence - follow reverse priority order strictly
- ❌ Don't use synchronous operations in RTIC tasks - use async/await patterns
- ❌ Don't bypass authentication - validate every bootloader command
- ❌ Don't hardcode timeouts - use configurable constants from working example
- ❌ Don't ignore USB report size - must be exactly 64 bytes
- ❌ Don't skip memory barriers in reset sequence - use dsb/isb instructions
- ❌ Don't forget static USB bus allocation - required for embedded USB