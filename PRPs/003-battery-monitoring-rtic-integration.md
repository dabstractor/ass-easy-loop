name: "Battery Monitoring RTIC Integration - Complete Integration Implementation"
description: |
  Complete the integration of the existing battery monitoring system into the RTIC application.
  The battery drivers, types, and safety systems are fully implemented - only the RTIC task 
  integration is missing from the main application.

---

## Goal

**Feature Goal**: Complete integration of the existing battery monitoring system into the main RTIC application, enabling real-time battery voltage monitoring, safety protection, and state tracking for the 3.7V LiPo battery system

**Deliverable**: Fully functional battery monitoring RTIC task that integrates seamlessly with the existing USB HID and logging systems without disrupting USB enumeration

**Success Definition**: 
- Battery monitoring task runs at 10Hz (100ms intervals) with Priority 4
- ADC readings from GPIO26 accurately detect battery states (Low, Normal, Charging, Full, Fault)
- Safety systems respond to over/under voltage conditions within 100ms
- USB enumeration remains stable (`lsusb | grep fade` continues to work)
- Battery state changes are logged via existing USB logging system
- Manual testing shows accurate voltage readings and proper state transitions

## User Persona

**Target User**: Firmware developer using the pEMF device for research applications

**Use Case**: Monitor battery voltage and charging state during extended research sessions, receive warnings for low battery conditions, ensure safe charging/discharging cycles

**User Journey**: 
1. Connect device to USB (battery monitoring starts automatically)
2. View battery status via host tools (`python host_tools/log_monitor.py`)
3. Receive automatic low battery warnings
4. Observe charging detection when USB power is connected
5. Safety systems prevent over/under voltage damage

**Pain Points Addressed**: 
- Currently no battery monitoring despite complete hardware setup and driver implementation
- No visibility into battery state or charging status
- Risk of battery damage from over/under voltage conditions
- README describes functionality that doesn't work in current firmware

## Why

- **Critical Safety Feature**: Battery monitoring prevents over/under voltage damage to expensive LiPo batteries
- **Hardware Investment Protection**: TP4056 charging circuit and voltage divider are installed but unused
- **User Experience**: Provides essential battery status feedback for portable device operation
- **System Reliability**: Enables predictable operation and prevents unexpected shutdowns
- **Documentation Alignment**: Makes the firmware match the comprehensive hardware documentation in README.md

## What

Complete integration of battery monitoring into the RTIC application by implementing the missing RTIC task layer while preserving existing USB functionality.

### Success Criteria

- [ ] Battery monitoring task executes every 100ms with proper ADC readings from GPIO26
- [ ] Battery state detection works correctly (Low: <3.1V, Normal: 3.1-3.6V, Charging: >3.6V, Full: ~4.2V)  
- [ ] Safety systems trigger emergency responses for over-voltage (>4.2V) and under-voltage (<3.0V) conditions
- [ ] USB enumeration continues to work reliably (`lsusb | grep fade` shows device)
- [ ] Battery state transitions are logged via existing USB logging system when `usb-logs` feature is enabled
- [ ] No impact on existing USB polling, command handling, or bootloader functionality
- [ ] Manual voltage validation shows readings within ±50mV of actual battery voltage

## All Needed Context

### Context Completeness Check

_This PRP provides complete implementation context for someone unfamiliar with the codebase. All necessary files, patterns, hardware details, and safety requirements are included with specific references._

### Documentation & References

```yaml
# MUST READ - Include these in your context window
- file: src/main.rs
  why: Contains existing RTIC application structure and all patterns to follow exactly
  pattern: Task definition, shared resources, scheduling, USB preservation patterns
  gotcha: NEVER modify USB task priorities (Priority 1) or polling intervals (10ms)

- file: src/drivers/adc_battery.rs  
  why: Complete BatteryMonitor driver implementation ready for RTIC integration
  pattern: process_sample() method designed for RTIC task usage, error handling patterns
  gotcha: Driver expects 10Hz calling frequency and proper safety flags

- file: src/drivers/battery_safety.rs
  why: Complete SafetyMonitor with RTIC-specific helper methods
  pattern: continuous_safety_check() method for Priority 3/4 RTIC tasks
  gotcha: Safety system starts armed and ready - requires proper error propagation

- file: src/types/battery.rs
  why: Complete type definitions, constants, and conversion functions
  pattern: BatteryState enum with exact ADC thresholds, SafetyFlags with atomic operations
  gotcha: Safety thresholds are validated against hardware - do not modify values

- file: IMPLEMENTATION_CHECKLIST.md
  why: Original planned approach with exact code patterns for RTIC integration
  pattern: Shared resources structure, task priority (Priority 2), 100ms scheduling
  gotcha: Checklist shows Priority 2, but Priority 4 is safer to avoid USB conflicts

- file: README.md
  why: Complete hardware documentation including GPIO pin assignments and voltage divider
  pattern: GPIO26 for ADC, Pin 39 (VSYS) for battery connection, voltage calculations
  gotcha: Pin 39 is VSYS on RP2040, not a GPIO pin - connects directly to battery positive

- docfile: PRPs/ai_docs/rp2040_adc_rtic_battery_integration.md
  why: Critical RP2040 ADC integration patterns and hardware-specific gotchas
  section: ADC Configuration, RTIC Resource Management, Safety Considerations
  
- file: .claude/USB_ENUMERATION_CHEAT_SHEET.md
  why: Critical rules for preserving USB functionality during RTIC modifications
  pattern: Priority preservation, polling timing, validation commands
  gotcha: Any changes that break USB enumeration make device unusable
```

### Current Codebase Tree (relevant files)

```bash
src/
├── main.rs                        # RTIC app - needs battery task integration
├── drivers/
│   ├── adc_battery.rs            # Complete battery monitor (ready to use)
│   ├── battery_safety.rs         # Complete safety monitor (ready to use)
│   ├── logging.rs                # Logging system for battery events
│   └── mod.rs                    # Driver module definitions
├── types/
│   ├── battery.rs                # Complete battery types and constants
│   ├── errors.rs                 # Complete battery error handling  
│   └── mod.rs                    # Type module definitions
├── tasks/
│   ├── battery_monitor.rs        # PLACEHOLDER - needs complete implementation
│   └── mod.rs                    # Task module definitions
└── config/
    └── flash_storage.rs          # Configuration storage

PRPs/ai_docs/
└── rp2040_adc_rtic_battery_integration.md  # Critical ADC integration patterns

README.md                          # Hardware documentation and pin assignments  
IMPLEMENTATION_CHECKLIST.md       # Original integration plan with code examples
CLAUDE.md                          # Project rules (always use `cargo run`)
```

### Desired Codebase Tree (files to be modified)

```bash
src/
├── main.rs                        # MODIFIED: Add battery RTIC integration
└── tasks/
    └── battery_monitor.rs         # REWRITTEN: Complete RTIC task implementation
```

### Known Gotchas of Codebase & Library Quirks

```rust
// CRITICAL: RP2040 ADC requires embedded_hal_0_2::adc::OneShot trait  
// The first ADC reading is often inaccurate - discard it
let _discard = adc.read(&mut battery_pin).unwrap();
let actual_reading = adc.read(&mut battery_pin).unwrap();

// CRITICAL: RTIC task priorities are sacred for USB functionality
// Priority 1: USB tasks (NEVER change these)
// Priority 2: System operations (bootloader) 
// Priority 3: Logging
// Priority 4+: Available (use Priority 4 for battery monitoring)

// CRITICAL: USB enumeration preservation rules
// NEVER modify dispatchers: [TIMER_IRQ_1, TIMER_IRQ_2, TIMER_IRQ_3]
// NEVER change USB polling timing from 10ms intervals
// ALWAYS test with: lsusb | grep fade

// CRITICAL: This project uses `cargo run` ONLY - never use external flashing tools
// Build command: cargo run (builds, converts, flashes automatically)

// CRITICAL: Battery safety thresholds are hardware-validated constants
// LOW_BATTERY_ADC_THRESHOLD: 1425   // 3.1V - do NOT modify
// CHARGING_ADC_THRESHOLD: 1675      // 3.6V - do NOT modify  
// OVERVOLTAGE_ADC_THRESHOLD: 1800   // 4.2V - do NOT modify
```

## Implementation Blueprint

### Data Models and Structure

The battery monitoring system uses existing complete data models - no new models needed:

```rust
// Existing complete types from src/types/battery.rs
pub enum BatteryState {
    Low,        // < 3.1V - requires attention
    Normal,     // 3.1V - 3.6V - normal operation
    Charging,   // > 3.6V - charging detected  
    Full,       // ~4.2V - fully charged
    Fault,      // Safety violation
}

pub struct SafetyFlags {
    pub over_voltage: AtomicBool,     // >4.2V emergency
    pub under_voltage: AtomicBool,    // <3.0V emergency
    // ... other safety flags
}

pub struct BatteryReading {
    pub timestamp_ms: u32,
    pub adc_value: u16,
    pub voltage_mv: u16, 
    pub state: BatteryState,
    // ... complete reading data
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: MODIFY src/main.rs - Add Battery Resources to RTIC Shared Struct
  - ADD: battery_monitor: BatteryMonitor to #[shared] struct
  - ADD: safety_flags: SafetyFlags to #[shared] struct  
  - ADD: battery_state: BatteryState to #[shared] struct
  - FOLLOW pattern: existing shared resources (usb_dev, hid_class, bootloader_state)
  - NAMING: exact names as shown - battery_monitor, safety_flags, battery_state
  - PLACEMENT: within existing Shared struct definition
  - PRESERVE: all existing shared resources unchanged

Task 2: MODIFY src/main.rs - Add ADC Local Resources
  - ADD: adc: Adc to #[local] struct
  - ADD: battery_pin: AdcPin<Pin<Gpio26, FunctionSioInput, PullNone>> to #[local] struct
  - FOLLOW pattern: simple Local struct extension
  - NAMING: adc, battery_pin (exact names for task context)
  - PLACEMENT: within Local struct (currently empty)
  - IMPORT: required types from rp2040_hal::adc and gpio modules

Task 3: MODIFY src/main.rs - Initialize ADC in init() Function  
  - IMPLEMENT: ADC peripheral initialization after clocks setup
  - IMPLEMENT: GPIO26 pin configuration as floating input ADC pin
  - IMPLEMENT: BatteryMonitor creation with ADC instance
  - FOLLOW pattern: existing peripheral initialization (USB setup pattern)
  - NAMING: mut adc, battery_pin, battery_monitor variables
  - DEPENDENCIES: import rp2040_hal modules, BatteryMonitor from drivers
  - PLACEMENT: after clock initialization, before USB setup
  - PRESERVE: all existing initialization code unchanged

Task 4: MODIFY src/main.rs - Add Battery Resources to init() Return
  - ADD: battery_monitor: BatteryMonitor to Shared struct initialization
  - ADD: safety_flags: SafetyFlags::new() to Shared struct
  - ADD: battery_state: BatteryState::Normal to Shared struct  
  - ADD: adc: adc to Local struct initialization
  - ADD: battery_pin: battery_pin to Local struct  
  - FOLLOW pattern: existing resource initialization in return statement
  - PRESERVE: all existing resource assignments

Task 5: REWRITE src/tasks/battery_monitor.rs - Complete RTIC Task Implementation
  - DELETE: existing placeholder content (3 lines of comments)
  - IMPLEMENT: complete RTIC task with proper signature and priority
  - IMPLEMENT: ADC reading with error handling and first-read discard
  - IMPLEMENT: BatteryMonitor.process_sample() integration
  - IMPLEMENT: safety error handling with emergency response
  - IMPLEMENT: logging integration with existing log_queue
  - IMPLEMENT: 100ms rescheduling pattern
  - FOLLOW pattern: existing task structure from main.rs (usb_poll_task, logging_transmit_task)
  - NAMING: battery_monitor_task function, Priority 4, exact resource names
  - DEPENDENCIES: import Duration, BatteryError, logging functions
  - PLACEMENT: complete file rewrite with proper RTIC task structure

Task 6: MODIFY src/main.rs - Spawn Battery Monitor Task in init()
  - ADD: battery_monitor_task::spawn_after(Duration::<u64, 1, 1000>::millis(100)).unwrap()
  - FOLLOW pattern: existing task spawning (usb_poll_task, usb_command_handler_task)
  - TIMING: 100ms initial delay, then 100ms intervals (10Hz operation)
  - PLACEMENT: after existing task spawns, before init() return
  - PRESERVE: all existing task spawning unchanged
```

### Implementation Patterns & Key Details

```rust
// RTIC Task Structure Pattern (follow existing tasks exactly)
#[task(
    local = [adc, battery_pin], 
    shared = [battery_monitor, safety_flags, log_queue, logging_config],
    priority = 4  // Higher than logging (3), safe from USB conflicts
)]
fn battery_monitor_task(mut ctx: battery_monitor_task::Context) {
    // PATTERN: ADC reading with RP2040 first-read discard
    let _discard = ctx.local.adc.read(ctx.local.battery_pin).unwrap_or(0);
    let adc_value = ctx.local.adc.read(ctx.local.battery_pin).unwrap_or_else(|_| {
        // Error fallback: get last known reading
        ctx.shared.battery_monitor.lock(|monitor| monitor.get_last_adc_reading())
    });
    
    // PATTERN: Process sample using existing driver (thread-safe locking)
    let result = ctx.shared.battery_monitor.lock(|monitor| {
        ctx.shared.safety_flags.lock(|flags| {
            monitor.process_sample(ctx.local.battery_pin, flags)
        })
    });
    
    // PATTERN: Error handling with safety response
    match result {
        Ok(reading) => {
            // Success - optionally log reading
            #[cfg(feature = "usb-logs")]
            {
                // Log successful reading if enabled (follow logging_transmit_task pattern)
            }
        },
        Err(error) => {
            if error.requires_emergency_shutdown() {
                // CRITICAL: Emergency response - set safety flags
                ctx.shared.safety_flags.lock(|flags| {
                    flags.set_emergency_stop(true);
                });
            }
        }
    }
    
    // PATTERN: Reschedule task (follow existing task patterns exactly)
    battery_monitor_task::spawn_after(Duration::<u64, 1, 1000>::millis(100)).unwrap();
}

// ADC Initialization Pattern (in init() function)
let mut adc = Adc::new(ctx.device.ADC, &mut ctx.device.RESETS);
let sio = Sio::new(ctx.device.SIO);
let pins = rp2040_hal::gpio::Pins::new(
    ctx.device.IO_BANK0,
    ctx.device.PADS_BANK0, 
    sio.gpio_bank0,
    &mut ctx.device.RESETS,
);
let battery_pin = pins.gpio26.into_floating_input().into();

// GOTCHA: Battery drivers expect these exact variable names for type safety
// CRITICAL: Use embedded_hal::adc::OneShot trait - import required
```

### Integration Points

```yaml
RTIC_APPLICATION:
  - shared_resources: "Add battery_monitor, safety_flags, battery_state to Shared struct"
  - local_resources: "Add adc, battery_pin to Local struct"
  - task_priority: "Use Priority 4 to avoid USB conflicts (Priority 1 reserved)"

ADC_HARDWARE:
  - gpio_pin: "GPIO26 configured as floating input ADC pin" 
  - peripheral: "ADC initialized with proper reset sequence"
  - timing: "10Hz sampling rate (100ms intervals)"

SAFETY_INTEGRATION:
  - emergency_response: "Safety flags integration with existing logging system"
  - error_propagation: "Battery errors logged via USB HID logging"
  - threshold_monitoring: "Continuous voltage limit checking"

LOGGING_INTEGRATION:
  - feature_flag: "usb-logs feature enables battery state logging"
  - log_queue: "Shared resource with existing logging system"
  - message_format: "Follow existing LogMessage patterns"
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file modification - fix before proceeding
cargo check --target thumbv6m-none-eabi
ruff check src/main.rs src/tasks/battery_monitor.rs  # If available
cargo clippy --target thumbv6m-none-eabi

# Project-wide compilation check
cargo build --target thumbv6m-none-eabi

# Expected: Zero compilation errors. If errors exist, fix before proceeding.
```

### Level 2: Feature Integration Testing (Component Validation)

```bash
# Test with battery logging enabled
cargo check --target thumbv6m-none-eabi --features battery-logs,usb-logs

# Test development build
cargo build --target thumbv6m-none-eabi --features development

# Expected: Clean builds with battery features enabled
```

### Level 3: Hardware Integration Testing (System Validation)

```bash
# CRITICAL: Flash firmware and test USB enumeration
cargo run

# Verify USB device enumeration still works (CRITICAL TEST)
lsusb | grep fade
# Expected: Device should appear in lsusb output

# Test bootloader entry functionality
python host_tools/bootloader_entry.py
# Expected: Device should enter bootloader mode successfully

# Monitor battery monitoring logs (if usb-logs enabled)
python host_tools/log_monitor.py
# Expected: Should see battery monitoring messages with ADC readings and states

# Test battery voltage accuracy with multimeter
# Set power supply to 3.6V, verify ADC reading ~1675
# Set power supply to 4.0V, verify ADC reading ~1741
# Expected: ADC readings within ±50 of calculated values
```

### Level 4: Safety & Performance Validation

```bash
# Long-term stability test (run for 10+ minutes)
python host_tools/log_monitor.py --duration 600
# Expected: Continuous battery monitoring without errors or crashes

# Safety limit testing (CAREFULLY with current-limited power supply)
# Slowly increase voltage to 4.1V - should detect charging state
# Expected: Battery state transitions logged correctly

# Performance validation - verify 10Hz operation
# Expected: ~10 battery readings per second in logs

# USB functionality preservation test
lsusb | grep fade && echo "USB enumeration OK"
python host_tools/bootloader_entry.py && echo "Bootloader entry OK"
# Expected: Both tests pass - USB functionality unchanged
```

## Final Validation Checklist

### Technical Validation

- [ ] Clean compilation: `cargo build --target thumbv6m-none-eabi`
- [ ] Feature compilation: `cargo build --target thumbv6m-none-eabi --features battery-logs`
- [ ] Firmware flashing: `cargo run` completes successfully
- [ ] USB enumeration: `lsusb | grep fade` shows device
- [ ] Bootloader entry: `python host_tools/bootloader_entry.py` works

### Battery Monitoring Validation

- [ ] ADC readings: Battery voltage displayed in logs (with usb-logs feature)
- [ ] State detection: Battery state transitions work (Normal, Charging, etc.)
- [ ] Safety limits: Over-voltage detection works (test carefully at 4.2V)
- [ ] Timing accuracy: ~10 readings per second (100ms intervals)
- [ ] Error handling: ADC read failures don't crash system

### System Integration Validation

- [ ] USB functionality: All existing USB features continue to work
- [ ] Logging system: Battery events integrate with existing logs
- [ ] Task scheduling: No timing conflicts with USB polling
- [ ] Resource sharing: No deadlocks or resource contention
- [ ] Performance: No impact on pEMF timing or USB responsiveness

### Hardware Validation

- [ ] GPIO26 ADC: Voltage readings match multimeter ±50mV
- [ ] Voltage divider: 10kΩ:5.1kΩ ratio provides correct scaling
- [ ] Pin 39 (VSYS): Battery power connection working correctly
- [ ] Safety circuits: Over/under voltage protection functional
- [ ] Charging detection: >3.6V properly detected as charging state

---

## Anti-Patterns to Avoid

- ❌ Don't modify USB task priorities (Priority 1) or timing (10ms)
- ❌ Don't change RTIC dispatchers [TIMER_IRQ_1, TIMER_IRQ_2, TIMER_IRQ_3] 
- ❌ Don't use sync functions in async context (N/A for embedded)
- ❌ Don't modify battery safety thresholds without hardware validation
- ❌ Don't skip the first ADC reading discard (RP2040 requirement)
- ❌ Don't use Priority 1-3 for battery monitoring (USB conflict risk)
- ❌ Don't forget to test USB enumeration after every change