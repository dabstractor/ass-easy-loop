# Design Document

## Overview

The ass-easy-loop application is designed as a real-time embedded system using the RTIC 2.0 framework on the Raspberry Pi Pico (RP2040). The system implements two primary subsystems: a high-precision pEMF driver and a battery monitoring loop with visual feedback. The design prioritizes timing accuracy for the pEMF driver while maintaining responsive battery monitoring and LED control.

## Architecture

### System Architecture

The application follows a task-based architecture using RTIC's interrupt-driven concurrency model:

```
┌─────────────────────────────────────────────────────────────┐
│                    RTIC Application                         │
├─────────────────────────────────────────────────────────────┤
│  Hardware Timer    │  Periodic Timer   │  LED Control      │
│  (Highest Priority)│  (Medium Priority)│  (Low Priority)   │
│                    │                   │                   │
│  pemf_pulse_task   │  battery_monitor  │  led_control_task │
│  - 2Hz square wave │  - ADC sampling   │  - Status display │
│  - 2ms/498ms cycle │  - State machine  │  - Flash patterns │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                   Shared Resources                          │
│  - LED Pin (GPIO 25)                                       │
│  - ADC Reading (u16)                                       │
│  - Battery State (enum)                                    │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                   Local Resources                           │
│  - MOSFET Pin (GPIO 15)                                    │
│  - ADC Peripheral                                          │
│  - ADC Pin (GPIO 26)                                       │
│  - Timer Alarm                                             │
│  - Pulse State (bool)                                      │
└─────────────────────────────────────────────────────────────┘
```

### Task Priority Hierarchy

1. **Highest Priority**: `pemf_pulse_task` - Hardware timer driven, cannot be preempted
2. **Medium Priority**: `battery_monitor_task` - Periodic ADC sampling and state updates
3. **Low Priority**: `led_control_task` - Visual feedback based on battery state

## Components and Interfaces

### Hardware Abstraction Layer

**Clock Configuration**:
- External 12MHz crystal with PLL configuration
- System clock derived using `init_clocks_and_plls()`
- Timer peripheral enabled for precise timing

**GPIO Configuration**:
```rust
// GPIO 15: MOSFET control output
let mosfet_pin = pins.gpio15.into_push_pull_output();

// GPIO 25: Built-in LED output  
let led_pin = pins.gpio25.into_push_pull_output();

// GPIO 26: ADC input with voltage divider
let adc_pin = AdcPin::new(pins.gpio26.into_floating_input());
```

**ADC Configuration**:
- 12-bit resolution (0-4095 range)
- 3.3V reference voltage
- Single-ended input on GPIO 26
- Voltage divider: R1=10kΩ, R2=5.1kΩ for 3.7V battery scaling

### RTIC Resource Management

**Shared Resources** (protected by RTIC's resource sharing):
```rust
#[shared]
struct Shared {
    led: Pin<Gpio25, Output<PushPull>>,
    adc_reading: u16,
    battery_state: BatteryState,
}
```

**Local Resources** (task-exclusive access):
```rust
#[local]
struct Local {
    mosfet_pin: Pin<Gpio15, Output<PushPull>>,
    adc: Adc,
    adc_pin: AdcPin<Pin<Gpio26, Input<Floating>>>,
    alarm: Alarm0,
    pulse_active: bool,
}
```

### State Machine Design

**Battery State Machine**:
```rust
#[derive(Clone, Copy, PartialEq)]
enum BatteryState {
    Low,      // ADC ≤ 1425 (< 3.1V)
    Normal,   // 1425 < ADC < 1675 (3.1V - 3.6V)  
    Charging, // ADC ≥ 1675 (> 3.6V)
}
```

State transitions are based on ADC thresholds with hysteresis to prevent oscillation.

## Data Models

### ADC Voltage Mapping

The voltage divider scales the 3.7V LiPo battery to the ADC's 3.3V range:

```
Voltage Divider Ratio = R2 / (R1 + R2) = 5.1kΩ / (10kΩ + 5.1kΩ) = 0.337

Battery Voltage → ADC Voltage → ADC Value
3.0V → 1.01V → 1260
3.1V → 1.04V → 1296  
3.6V → 1.21V → 1508
3.7V → 1.25V → 1556
4.2V → 1.42V → 1769
```

**Threshold Mapping**:
- Low Battery: ADC ≤ 1425 (≈ 3.1V battery)
- Normal: 1425 < ADC < 1675 (3.1V - 3.6V battery)
- Charging: ADC ≥ 1675 (> 3.6V battery)

### Timing Models

**pEMF Pulse Timing**:
- Period: 500ms (2 Hz frequency)
- Pulse High: 2ms
- Pulse Low: 498ms
- Precision: Hardware timer with ±1% accuracy

**System Timing**:
- Battery monitoring: 100ms intervals (10 Hz)
- LED flash rate: 250ms intervals (2 Hz for low battery)
- LED update latency: <500ms after state change

## Error Handling

### Error Categories

1. **Initialization Errors**: Hardware setup failures handled with `panic-halt`
2. **Runtime Errors**: ADC read failures logged but system continues
3. **Timing Errors**: Hardware timer failures cause system panic
4. **Resource Conflicts**: Prevented by RTIC's compile-time resource analysis

### Error Recovery Strategy

- **Critical Path (pEMF)**: No recovery - system panic on timer failure
- **Non-Critical (Battery/LED)**: Log error, use last known good value, continue operation
- **Hardware Faults**: Watchdog reset (if implemented) or manual reset required

## Testing Strategy

### Unit Testing

**Component Tests**:
- Battery state machine transitions
- ADC value to voltage conversion
- Timing calculation functions
- LED pattern generation logic

**Mock Testing**:
- Hardware abstraction layer mocking for CI/CD
- Timer simulation for timing validation
- ADC value injection for state machine testing

### Integration Testing

**Hardware-in-Loop Testing**:
- Oscilloscope verification of pEMF pulse timing
- Multimeter validation of ADC readings
- Visual verification of LED patterns
- Long-term stability testing (24+ hours)

**Timing Validation**:
- Real-time analysis of task execution times
- Interrupt latency measurement
- Priority inversion detection
- Jitter analysis for pEMF pulses

### Performance Testing

**Benchmarks**:
- Task execution time profiling
- Memory usage analysis
- Stack depth monitoring
- Interrupt response time measurement

**Stress Testing**:
- Maximum system load scenarios
- Rapid battery state transitions
- Extended operation testing
- Temperature variation testing

## Implementation Notes

### RTIC 2.0 Specific Considerations

- Use `spawn_after()` for self-scheduling tasks
- Leverage compile-time resource analysis for deadlock prevention
- Implement monotonic timer for precise scheduling
- Use shared resource locks only when necessary to minimize blocking

### RP2040 Specific Optimizations

- Utilize hardware timers for critical timing
- Configure ADC for optimal sampling rate
- Use DMA for non-blocking operations where applicable
- Optimize GPIO operations for minimal latency

### Memory Management

- Stack-allocated data structures only
- No dynamic memory allocation
- Compile-time resource sizing
- Minimal RAM footprint for embedded constraints