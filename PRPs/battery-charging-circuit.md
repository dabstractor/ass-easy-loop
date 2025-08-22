name: "Battery Charging Circuit Implementation PRP"
description: |
  Complete implementation of TP4056-based battery charging circuit with RP2040 integration,
  safety monitoring, and seamless operation during charging cycles.

---

## Goal

**Feature Goal**: Implement a complete battery charging circuit that safely charges single-cell LiPo batteries (3.7V nominal) using TP4056 IC, with real-time voltage monitoring via RP2040 ADC, automatic charge detection, and safety protections, while maintaining ±1% pEMF timing accuracy during all charging operations.

**Deliverable**: 
- Enhanced ADC monitoring system with 10Hz battery voltage sampling
- Battery state machine (Low/Normal/Charging/Full) with safety transitions
- TP4056 charging circuit integration with DW01A protection
- USB HID logging of battery status and charging events
- Safety fault detection and emergency shutdown capabilities
- Validation framework ensuring charging safety and performance

**Success Definition**: 
1. Battery voltage monitoring with ±50mV accuracy via GPIO 26 ADC
2. Automatic charging state detection when voltage > 3.6V (ADC ≥ 1675)
3. Complete safety protection: over-voltage (>4.2V), under-voltage (<3.0V), over-current (>1A)
4. Zero degradation of existing pEMF timing accuracy (maintains ±1% tolerance)
5. Seamless USB/battery power switching without operation interruption
6. All safety tests pass before any hardware implementation begins

## User Persona

**Target User**: Embedded systems engineer implementing medical device battery management

**Use Case**: Development and testing of therapeutic pEMF device with integrated LiPo battery charging

**User Journey**: 
1. Flash firmware with battery monitoring enabled
2. Connect device to USB power to initiate charging
3. Monitor real-time battery status via Python host tools
4. Verify charging safety protections through validation tests
5. Operate device seamlessly during charging with maintained pEMF accuracy

**Pain Points Addressed**: 
- Manual battery management and charging monitoring
- Safety concerns with LiPo charging in medical devices
- Performance degradation during charging operations
- Lack of automated charging state detection and logging

## Why

- **Safety Critical**: LiPo battery charging requires comprehensive safety monitoring to prevent fire/explosion hazards
- **Medical Device Compliance**: Therapeutic pEMF device requires uninterrupted operation during charging for continuous therapy
- **User Experience**: Automated charging with clear status indication eliminates manual battery management
- **System Integration**: Seamless power management enables portable operation with USB charging capability
- **Regulatory Requirements**: Battery charging safety standards compliance for medical device certification

## What

The system implements a complete battery charging circuit with the following user-visible behavior:

### Hardware Integration
- TP4056 linear charging IC (1A max) with USB-C power input
- DW01A + FS8205A protection circuit for over/under-voltage and over-current protection  
- Voltage divider (10kΩ:5.1kΩ) for safe ADC monitoring via GPIO 26
- Status LED indication: solid ON during charging, OFF when complete/not charging

### Software Behavior
- Real-time battery voltage monitoring at 10Hz sampling rate
- Automatic state detection: Low (<3.1V), Normal (3.1-3.6V), Charging (>3.6V), Full (4.2V)
- Enhanced USB HID logging with battery status, charging events, and safety alerts
- Safety fault detection with immediate response and logging
- Maintained pEMF timing accuracy (±1% tolerance) during all charging operations

### User Interface
- Python host tools display real-time battery status and charging progression
- CSV logging for validation and long-term monitoring
- Safety fault alerts with specific error codes and recovery procedures
- Automated validation testing with comprehensive safety limit verification

### Success Criteria

- [ ] ADC monitoring accuracy: ±50mV battery voltage measurement
- [ ] State detection reliability: 100% accurate charging state transitions
- [ ] Safety protection response: <100ms fault detection and shutdown
- [ ] Timing preservation: ±1% pEMF accuracy maintained during charging
- [ ] Power management: Seamless USB/battery switching without interruption
- [ ] Logging integration: Complete battery events captured in USB HID logs
- [ ] Validation framework: All safety tests pass before hardware implementation
- [ ] Long-term reliability: >100 charge/discharge cycles without degradation

## All Needed Context

### Context Completeness Check

_This PRP provides complete implementation context for an engineer unfamiliar with the codebase, including exact file patterns, RTIC task priorities, ADC configurations, safety thresholds, and validation procedures._

### Documentation & References

```yaml
# MUST READ - Critical for safety-first implementation
- url: https://rtic.rs/2/book/en/by-example.html#init
  why: RTIC 1.x initialization patterns for ADC and timer setup
  critical: Proper shared/local resource management prevents race conditions

- url: https://docs.rs/rp2040-hal/latest/rp2040_hal/adc/struct.Adc.html
  why: RP2040 ADC configuration and reading patterns
  critical: GPIO 26 ADC setup with proper voltage divider integration

- file: src/main.rs
  why: Existing RTIC 1.x app structure, task priorities, and USB integration
  pattern: Follow exact RTIC macro syntax, shared/local resource patterns
  gotcha: Uses RTIC 1.x (not 2.x) - different resource syntax and timer patterns

- file: types/logging.rs
  why: LogMessage structure and LogCategory::Battery enum usage
  pattern: 64-byte HID report format with timestamp, level, category, content
  gotcha: Content limited to 52 bytes, requires length field and null-termination

- file: drivers/usb_command_handler.rs  
  why: USB HID communication patterns and report parsing
  pattern: 64-byte report structure with command ID in byte 0
  gotcha: All multi-byte values use little-endian encoding

- file: host_tools/battery_monitor.py
  why: Existing comprehensive battery monitoring and validation framework
  pattern: Safety limit validation, state transition checking, real-time monitoring
  gotcha: Expects specific data format with timestamp, ADC value, voltage, state fields

- docfile: PRPs/ai_docs/battery_adc_mapping.md
  why: Exact voltage thresholds and ADC conversion formulas for state detection
  section: Threshold Values and Conversion Formula sections
  critical: Uses scale factor 0.337 for 10kΩ:5.1kΩ voltage divider

- docfile: PRPs/ai_docs/pemf_timing_specs.md
  why: Critical timing requirements that must be preserved during battery monitoring
  section: Timing Tolerance and Real-Time Requirements
  critical: ±1% accuracy (±5ms) must be maintained, highest task priority required

- docfile: PRPs/ai_docs/rtic1x_battery_patterns.md
  why: Production-ready RTIC 1.x patterns for safety-critical battery monitoring
  section: All sections - task priorities, shared resources, safety patterns
  critical: Priority 4 for emergency shutdown, priority 3 for safety checks

- docfile: docs/tp4056_battery_integration.md
  why: Complete TP4056 circuit design and safety implementation patterns
  section: Safety Implementation and Testing Procedures
  critical: Exact component values, protection thresholds, and fault responses
```

### Current Codebase Tree

```bash
src/
├── main.rs                 # RTIC 1.x app with USB HID, RTIC tasks, shared resources
├── lib.rs                  # Module organization and exports
├── config/
│   └── usb.rs             # USB device configuration (VID: 0xfade, PID: 0x1212)
├── drivers/
│   ├── adc_battery.rs     # Battery monitoring (PLACEHOLDER - needs implementation)
│   ├── logging.rs         # USB HID logging with feature flags
│   └── usb_command_handler.rs  # Command parsing and response handling
└── types/
    ├── errors.rs          # System error types
    ├── logging.rs         # LogMessage, LogLevel, LogCategory definitions
    └── usb_commands.rs    # USB HID report structures

host_tools/
├── battery_monitor.py     # Comprehensive validation framework (542 lines)
├── log_monitor.py         # Real-time USB HID monitoring (261 lines)
└── requirements.txt       # hidapi>=0.14.0

Cargo.toml                 # RTIC 1.x dependencies, feature flags (battery-logs, development)
```

### Desired Codebase Tree with New Files

```bash
src/
├── main.rs                 # ENHANCED: Battery monitoring tasks, ADC initialization
├── drivers/
│   ├── adc_battery.rs     # IMPLEMENT: Complete battery monitoring with state machine
│   ├── battery_safety.rs  # CREATE: Safety fault detection and emergency shutdown
│   └── logging.rs         # ENHANCE: Battery-specific logging functions
├── types/
│   ├── battery.rs         # CREATE: BatteryState, SafetyFlags, BatteryReading types
│   └── errors.rs          # ENHANCE: Battery-specific error types
└── tests/
    ├── battery_safety_tests.rs      # CREATE: Comprehensive safety validation
    ├── battery_hardware_tests.rs    # CREATE: Hardware-in-the-loop testing
    └── battery_performance_tests.rs # CREATE: Timing accuracy validation

host_tools/
└── battery_monitor.py     # ENHANCE: Extended validation for charging circuit
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: RTIC 1.x uses different syntax than 2.x documentation
#[rtic::app(device = rp2040_hal::pac)]  // NOT #[rtic::app(v2)]

// ADC reading requires proper error handling
let adc_result: nb::Result<u16, _> = adc.read(&mut pin);
// nb crate requires match on WouldBlock vs Other errors

// USB HID reports MUST be exactly 64 bytes
pub struct CommandReport {
    pub data: [u8; 64],  // Fixed size required by HID descriptor
}

// Feature flags control logging compilation
#[cfg(feature = "battery-logs")]
fn log_battery_event() { /* Only compiled with feature enabled */ }

// Atomic types required for RTIC shared resources
use core::sync::atomic::{AtomicU16, Ordering};
// SeqCst ordering ensures consistency across task priorities

// Timer scheduling in RTIC 1.x uses specific Duration type
spawn_after(Duration::<u64, 1, 1000>::millis(100)).unwrap();

// RP2040 ADC requires specific GPIO pins (26-29 only)
let battery_pin = pins.gpio26.into_floating_input().into();
// GPIO 26 = ADC channel 0, must use AdcPin<Gpio26> type
```

## Implementation Blueprint

### Data Models and Structure

Create type-safe data models ensuring safety and consistency:

```rust
// Core battery state enumeration
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatteryState {
    Low = 0,      // < 3.1V - ADC ≤ 1425, requires immediate attention
    Normal = 1,   // 3.1V - 3.6V - ADC 1426-1674, normal operation
    Charging = 2, // > 3.6V - ADC ≥ 1675, charging detected
    Full = 3,     // 4.2V - ADC ~1800, fully charged
    Fault = 4,    // Error condition, immediate safety response required
}

// Safety flags for critical monitoring
#[derive(Debug)]
pub struct SafetyFlags {
    pub over_voltage: AtomicBool,     // >4.2V battery voltage
    pub under_voltage: AtomicBool,    // <3.0V battery voltage  
    pub over_current: AtomicBool,     // >1A charge current
    pub over_temperature: AtomicBool, // >50°C battery temperature
    pub emergency_stop: AtomicBool,   // Immediate shutdown required
}

// Complete battery reading structure
#[derive(Clone, Copy, Debug)]
pub struct BatteryReading {
    pub timestamp_ms: u32,
    pub adc_value: u16,        // Raw ADC reading (0-4095)
    pub voltage_mv: u16,       // Calculated battery voltage in mV
    pub state: BatteryState,   // Current battery state
    pub is_charging: bool,     // Charging circuit active
    pub safety_flags: u8,      // Packed safety flag bits
}

// Voltage threshold constants (from PRPs/ai_docs/battery_adc_mapping.md)
pub const LOW_BATTERY_ADC_THRESHOLD: u16 = 1425;    // 3.1V battery
pub const CHARGING_ADC_THRESHOLD: u16 = 1675;       // 3.6V battery  
pub const OVERVOLTAGE_ADC_THRESHOLD: u16 = 1800;    // 4.2V battery
pub const UNDERVOLTAGE_ADC_THRESHOLD: u16 = 1200;   // 3.0V battery
pub const VOLTAGE_SCALE_FACTOR: f32 = 0.337;        // 10kΩ:5.1kΩ divider
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/types/battery.rs
  - IMPLEMENT: BatteryState, SafetyFlags, BatteryReading types with exact threshold constants
  - FOLLOW pattern: types/errors.rs (enum structure with repr(u8), derive traits)
  - NAMING: BatteryState enum variants, SCREAMING_CASE for constants
  - PLACEMENT: Type definitions in src/types/ following existing pattern
  - CRITICAL: Use exact ADC threshold values from PRPs/ai_docs/battery_adc_mapping.md

Task 2: ENHANCE src/types/errors.rs
  - IMPLEMENT: BatteryError variants (AdcFailed, OverVoltage, UnderVoltage, SafetyTimeout)
  - FOLLOW pattern: existing SystemError enum structure and derive traits
  - NAMING: Consistent with existing error naming conventions
  - DEPENDENCIES: Import BatteryState from Task 1 for error context
  - PLACEMENT: Add to existing errors.rs file, maintain compatibility

Task 3: IMPLEMENT src/drivers/adc_battery.rs (CRITICAL TASK)
  - IMPLEMENT: Complete BatteryMonitor struct with ADC reading, state detection, safety checking
  - FOLLOW pattern: PRPs/ai_docs/rtic1x_battery_patterns.md (shared resource patterns, safety priorities)
  - NAMING: BatteryMonitor struct, read_voltage(), get_state(), check_safety() methods
  - DEPENDENCIES: Use types from Tasks 1-2, rp2040-hal ADC patterns from existing codebase
  - PLACEMENT: Replace placeholder in src/drivers/adc_battery.rs
  - CRITICAL: Implement 10Hz sampling with exact ADC conversion using VOLTAGE_SCALE_FACTOR

Task 4: CREATE src/drivers/battery_safety.rs (SAFETY-CRITICAL TASK)
  - IMPLEMENT: SafetyMonitor struct with immediate fault detection and emergency shutdown
  - FOLLOW pattern: PRPs/ai_docs/rtic1x_battery_patterns.md (priority 4 emergency tasks)
  - NAMING: SafetyMonitor struct, check_safety_limits(), trigger_emergency_shutdown() methods
  - DEPENDENCIES: Import SafetyFlags, BatteryError from previous tasks
  - PLACEMENT: New safety-specific driver module
  - CRITICAL: <100ms response time for safety violations, highest RTIC task priority

Task 5: ENHANCE src/main.rs RTIC App (INTEGRATION TASK)
  - IMPLEMENT: Battery monitoring tasks at correct priorities, ADC initialization, shared resources
  - FOLLOW pattern: Existing RTIC 1.x app structure, task definitions, resource management
  - NAMING: sample_battery_adc, update_battery_state, battery_safety_check task functions  
  - DEPENDENCIES: Import battery drivers from Tasks 3-4, integrate with existing USB/logging
  - PLACEMENT: Add to existing #[rtic::app] module
  - CRITICAL: Maintain existing pEMF task priorities, add battery tasks without conflicts

Task 6: ENHANCE drivers/logging.rs
  - IMPLEMENT: Battery-specific logging functions with feature flag guards
  - FOLLOW pattern: Existing feature-gated logging, LogCategory::Battery usage
  - NAMING: log_battery_state_change(), log_battery_safety_event() functions
  - DEPENDENCIES: Use LogMessage structure, BatteryReading format from previous tasks
  - PLACEMENT: Add to existing drivers/logging.rs file
  - CRITICAL: Non-blocking logging that won't interfere with safety-critical tasks

Task 7: CREATE comprehensive safety validation tests
  - IMPLEMENT: Battery safety test suite covering all fault conditions and recovery
  - FOLLOW pattern: Feature-validation-architect recommendations for safety-first testing
  - NAMING: test_overvoltage_protection(), test_undervoltage_shutdown(), test_state_transitions()
  - DEPENDENCIES: All previous tasks completed, battery drivers functional
  - PLACEMENT: tests/ directory with battery_safety_tests.rs
  - CRITICAL: All safety tests must pass before any hardware implementation
```

### Implementation Patterns & Key Details

```rust
// RTIC 1.x Task Implementation Pattern (Priority-based safety)
#[rtic::app(device = rp2040_hal::pac, dispatchers = [TIMER_IRQ_1, TIMER_IRQ_2, TIMER_IRQ_3])]
mod app {
    #[shared]
    struct Shared {
        battery_state: BatteryState,
        battery_voltage: AtomicU16,     // Thread-safe ADC value sharing
        safety_flags: SafetyFlags,      // Critical safety monitoring
        usb_dev: UsbDevice<'static, UsbBus>,
        hid_class: HIDClass<'static, UsbBus>,
    }

    #[local] 
    struct Local {
        adc: Adc,
        battery_pin: AdcPin<Gpio26>,    // GPIO 26 for battery monitoring
        alarm: Alarm0,                  // Timer for 10Hz sampling
        safety_monitor: SafetyMonitor,  // Safety fault detection
    }

    // CRITICAL: Highest priority for immediate safety response (Priority 4)
    #[task(shared = [safety_flags], local = [safety_monitor], priority = 4)]
    fn emergency_battery_shutdown(mut ctx: emergency_battery_shutdown::Context, fault: BatteryError) {
        // Immediate hardware safety actions
        ctx.local.safety_monitor.disable_charging_circuit();
        
        // Set safety flags atomically
        ctx.shared.safety_flags.lock(|flags| {
            match fault {
                BatteryError::OverVoltage => flags.over_voltage.store(true, Ordering::SeqCst),
                BatteryError::UnderVoltage => flags.under_voltage.store(true, Ordering::SeqCst),
                // Handle all fault types
            }
            flags.emergency_stop.store(true, Ordering::SeqCst);
        });
    }

    // High priority for safety limit checking (Priority 3)
    #[task(shared = [battery_voltage, safety_flags], priority = 3)]
    fn check_battery_safety(mut ctx: check_battery_safety::Context) {
        let adc_value = ctx.shared.battery_voltage.lock(|v| v.load(Ordering::SeqCst));
        
        // CRITICAL: Check against exact thresholds from battery_adc_mapping.md
        if adc_value > OVERVOLTAGE_ADC_THRESHOLD {
            emergency_battery_shutdown::spawn(BatteryError::OverVoltage).unwrap();
        } else if adc_value < UNDERVOLTAGE_ADC_THRESHOLD {
            emergency_battery_shutdown::spawn(BatteryError::UnderVoltage).unwrap();
        }
    }

    // Normal priority for ADC sampling (Priority 2) 
    #[task(binds = TIMER_IRQ_0, local = [adc, battery_pin, alarm], 
           shared = [battery_voltage], priority = 2)]
    fn sample_battery_adc(mut ctx: sample_battery_adc::Context) {
        ctx.local.alarm.clear_interrupt();
        
        // Read ADC with proper error handling
        match ctx.local.adc.read(ctx.local.battery_pin) {
            Ok(adc_value) => {
                // Store atomically for other tasks
                ctx.shared.battery_voltage.lock(|voltage| {
                    voltage.store(adc_value, Ordering::SeqCst);
                });
                
                // Trigger immediate safety check
                check_battery_safety::spawn().unwrap();
                
                // Schedule next sample (100ms = 10Hz)
                ctx.local.alarm.schedule(Duration::<u64, 1, 1000>::millis(100)).unwrap();
            },
            Err(nb::Error::WouldBlock) => {
                // Retry soon if ADC not ready
                ctx.local.alarm.schedule(Duration::<u64, 1, 1000>::millis(1)).unwrap();
            },
            Err(nb::Error::Other(_)) => {
                // ADC hardware failure - critical safety response
                emergency_battery_shutdown::spawn(BatteryError::AdcFailed).unwrap();
            }
        }
    }
}

// Battery State Detection with Exact Thresholds
impl BatteryState {
    fn from_adc_reading(adc_value: u16) -> Self {
        match adc_value {
            0..=LOW_BATTERY_ADC_THRESHOLD => BatteryState::Low,           // ≤ 1425 (3.1V)
            LOW_BATTERY_ADC_THRESHOLD+1..=CHARGING_ADC_THRESHOLD-1 => BatteryState::Normal, // 1426-1674 (3.1V-3.6V)  
            CHARGING_ADC_THRESHOLD.. => BatteryState::Charging,          // ≥ 1675 (>3.6V)
        }
    }
}

// Feature-Gated Battery Logging Pattern
#[cfg(feature = "battery-logs")]
pub fn log_battery_reading(reading: &BatteryReading) {
    let mut content = [0u8; 52];
    
    // Format: "BAT: State={} ADC={} V={}mV Flags={:02x}"
    let formatted = format_args!("BAT: State={:?} ADC={} V={}mV Flags={:02x}", 
                                reading.state, reading.adc_value, 
                                reading.voltage_mv, reading.safety_flags);
    
    let msg = LogMessage {
        timestamp_ms: reading.timestamp_ms,
        level: LogLevel::Info,
        category: LogCategory::Battery,  // Uses existing category
        content,
        content_len: formatted.len().min(51) as u8,
    };
    
    // Non-blocking enqueue - critical for safety task performance
    unsafe {
        let _ = LOG_QUEUE.try_enqueue(msg);  // Won't block if queue full
    }
}

// Voltage Conversion with Exact Scale Factor
fn convert_adc_to_voltage_mv(adc_value: u16) -> u16 {
    // From PRPs/ai_docs/battery_adc_mapping.md: scale factor = 0.337
    let adc_voltage = (adc_value as f32 * 3300.0) / 4095.0;  // 3.3V reference
    let battery_voltage = adc_voltage / VOLTAGE_SCALE_FACTOR; // Undo voltage divider
    battery_voltage as u16
}
```

### Integration Points

```yaml
RTIC TASKS:
  - priority: 4 (highest) - Emergency safety shutdown
  - priority: 3 - Safety limit monitoring  
  - priority: 2 - ADC sampling and state processing
  - priority: 1 - USB communication (existing)
  - priority: 0 - Status updates and non-critical logging

SHARED RESOURCES:
  - add: battery_state (BatteryState enum)
  - add: battery_voltage (AtomicU16 for thread-safe sharing)
  - add: safety_flags (SafetyFlags struct with atomic booleans)
  - preserve: existing USB and logging shared resources

ADC HARDWARE:
  - gpio: GPIO 26 (ADC channel 0) for battery voltage monitoring
  - setup: 10kΩ:5.1kΩ voltage divider, scale factor 0.337
  - sampling: 10Hz continuous sampling via timer interrupt
  - safety: Immediate fault detection on ADC read errors

FEATURE FLAGS:
  - use: battery-logs (existing) for battery-specific logging
  - use: development (existing) includes battery logging
  - use: testing (existing) for comprehensive safety validation
```

## Validation Loop

### Level 1: Safety Validation (CRITICAL - Must Pass First)

```bash
# MANDATORY: Run safety validation before any hardware work
cargo test --features testing battery_safety_tests --lib
cargo test --features testing test_battery_thresholds
cargo test --features testing test_emergency_shutdown_response
cargo test --features testing test_fault_detection_timing

# Expected: 100% pass rate for all safety tests
# If ANY safety test fails, DO NOT proceed to hardware implementation
# Fix safety issues and re-run until all tests pass
```

### Level 2: Hardware-in-Loop Testing (Progressive Hardware Integration) 

```bash
# Phase 1: ADC monitoring only (NO CHARGING CIRCUIT)
cargo run --features development,battery-logs --target thumbv6m-none-eabi
python3 host_tools/battery_monitor.py --duration 60 --log adc_validation.csv

# Phase 2: Voltage divider validation with known reference voltages  
# Apply 3.1V, 3.6V, 4.2V via adjustable power supply to voltage divider input
# Verify ADC readings match expected values within ±50mV tolerance

# Phase 3: Complete integration with TP4056 circuit (EXTREME CAUTION)
# Use battery protection circuit, monitor for any safety violations
python3 host_tools/battery_monitor.py --duration 300 --log charging_validation.csv

# Expected: All voltage readings within specification, no safety violations
```

### Level 3: Performance Integration (Timing Accuracy Validation)

```bash
# Verify pEMF timing maintained during battery monitoring
cargo run --features development --target thumbv6m-none-eabi
python3 host_tools/battery_monitor.py --config timing_validation.json

# Monitor for timing degradation during battery sampling
# Measure pEMF pulse accuracy: must maintain ±1% (±5ms) tolerance
# Test under various battery states: Low, Normal, Charging

# Long-term stability test (minimum 30 minutes continuous operation)
python3 host_tools/battery_monitor.py --duration 1800 --log stability_test.csv

# Expected: Zero timing violations, consistent pEMF accuracy throughout test
```

### Level 4: Complete System Validation (Production Readiness)

```bash
# Full charge/discharge cycle testing
cargo run --features production --target thumbv6m-none-eabi
python3 host_tools/battery_monitor.py --full-cycle-test --log production_test.csv

# Safety fault injection testing
python3 host_tools/battery_monitor.py --fault-injection --log safety_validation.csv

# Regulatory compliance validation
python3 host_tools/battery_monitor.py --compliance-test --duration 3600

# Long-term reliability testing (minimum 100 charge cycles)
python3 host_tools/battery_monitor.py --endurance-test --cycles 100

# Expected: All tests pass, complete regulatory compliance, long-term stability
```

## Final Validation Checklist

### Technical Validation

- [ ] **Safety Tests**: All battery safety tests pass with 100% success rate
- [ ] **ADC Accuracy**: Battery voltage readings within ±50mV of actual voltage
- [ ] **State Detection**: 100% accurate state transitions (Low/Normal/Charging/Full)
- [ ] **Timing Preservation**: pEMF timing maintained at ±1% accuracy during charging
- [ ] **Fault Response**: Safety violations trigger shutdown within <100ms
- [ ] **No Linting Errors**: `cargo clippy --features development --target thumbv6m-none-eabi`
- [ ] **No Type Errors**: `cargo check --features development --target thumbv6m-none-eabi`
- [ ] **Build Success**: `cargo build --release --target thumbv6m-none-eabi --features production`

### Feature Validation

- [ ] **Charging Detection**: Automatic detection when USB power applied and battery voltage >3.6V
- [ ] **Safety Protection**: Over-voltage (>4.2V), under-voltage (<3.0V), over-current (>1A) protection active
- [ ] **State Transitions**: Proper state machine operation with safety interlocks
- [ ] **USB Logging**: Complete battery events captured in USB HID logs with timestamps
- [ ] **Host Tool Integration**: battery_monitor.py displays real-time status and validates operation
- [ ] **Power Management**: Seamless switching between USB and battery power without interruption
- [ ] **LED Status**: Clear visual indication of charging state (ON during charging)

### Safety Validation

- [ ] **Emergency Shutdown**: Immediate response to safety violations with hardware disconnect
- [ ] **Fault Recovery**: System recovery after fault clearance with proper state restoration  
- [ ] **Temperature Protection**: Over-temperature detection and charging termination
- [ ] **Current Limiting**: Charge current limited to 1A maximum with protection circuit
- [ ] **Voltage Monitoring**: Continuous monitoring with <1% ADC accuracy
- [ ] **Regulatory Compliance**: TP4056 circuit meets LiPo charging safety standards
- [ ] **Long-term Reliability**: >100 charge/discharge cycles without performance degradation

### Code Quality Validation

- [ ] **RTIC 1.x Compliance**: Proper shared/local resource management and task priorities
- [ ] **Feature Flag Integration**: Proper use of battery-logs, development, testing flags  
- [ ] **Error Handling**: Comprehensive error handling for ADC failures and safety violations
- [ ] **Documentation**: Code is self-documenting with clear variable/function names
- [ ] **Safety-First Design**: Highest priority tasks handle safety-critical functions
- [ ] **Non-blocking Operations**: Logging and USB communication don't block safety tasks
- [ ] **Thread-Safe Sharing**: Proper atomic operations for cross-task data sharing

---

## Anti-Patterns to Avoid

- ❌ **Don't implement hardware before safety software tests pass** - Safety validation is mandatory first step
- ❌ **Don't block safety-critical tasks with logging operations** - Use non-blocking queues and lower priorities  
- ❌ **Don't ignore ADC read errors** - Hardware failures require immediate safety response
- ❌ **Don't use RTIC 2.x patterns** - Codebase uses RTIC 1.x with different syntax and resource management
- ❌ **Don't hardcode voltage thresholds** - Use exact constants from PRPs/ai_docs/battery_adc_mapping.md
- ❌ **Don't compromise pEMF timing accuracy** - Battery monitoring must not degrade existing ±1% requirement
- ❌ **Don't skip validation steps** - Each level must pass before proceeding to next
- ❌ **Don't disable safety features for debugging** - Safety protections must remain active at all times
