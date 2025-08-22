# RTIC 1.x Battery Monitoring Implementation Patterns

## Overview

This document provides specific RTIC 1.x implementation patterns for battery monitoring based on production-ready examples from the embedded Rust community. These patterns are designed for safety-critical battery charging applications.

## RTIC 1.x Task Architecture

### App Structure for Battery Monitoring

```rust
#[rtic::app(
    device = rp2040_hal::pac, 
    peripherals = true, 
    dispatchers = [TIMER_IRQ_1, TIMER_IRQ_2, TIMER_IRQ_3]
)]
mod app {
    #[shared]
    struct Shared {
        battery_state: BatteryState,
        battery_voltage: AtomicU16,
        charging_enabled: AtomicBool,
        safety_flags: SafetyFlags,
    }

    #[local]
    struct Local {
        adc: Adc,
        battery_pin: AdcPin<Gpio26>,
        alarm: Alarm0,
        led: Pin<Gpio25, Output<PushPull>>,
    }

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = Systick<1000>;  // 1kHz monotonic timer
}
```

## Priority Assignment for Safety-Critical Systems

### Battery Monitoring Priority Levels

- **Priority 4**: Emergency safety response (immediate fault shutdown)
- **Priority 3**: Critical monitoring (over-voltage, under-voltage detection) 
- **Priority 2**: Normal ADC sampling and state processing
- **Priority 1**: USB communication and logging
- **Priority 0**: Non-critical status updates

### Safety-First Task Implementation

```rust
// CRITICAL: Highest priority for immediate safety response
#[task(shared = [safety_flags, charging_enabled], priority = 4)]
fn emergency_shutdown(mut ctx: emergency_shutdown::Context, fault_type: FaultType) {
    ctx.shared.charging_enabled.lock(|enabled| {
        enabled.store(false, Ordering::SeqCst);
    });
    
    ctx.shared.safety_flags.lock(|flags| {
        flags.set_emergency_stop(true);
        flags.set_fault_type(fault_type);
    });
    
    // Immediate hardware actions
    // Disable charging circuit via GPIO
    // Log critical fault (high priority)
}

// High priority for voltage threshold monitoring  
#[task(shared = [battery_voltage, safety_flags], priority = 3)]
fn voltage_safety_check(mut ctx: voltage_safety_check::Context) {
    let voltage = ctx.shared.battery_voltage.lock(|v| v.load(Ordering::SeqCst));
    
    // Check critical thresholds
    if voltage > OVERVOLTAGE_ADC_THRESHOLD {
        emergency_shutdown::spawn(FaultType::Overvoltage).unwrap();
    } else if voltage < UNDERVOLTAGE_ADC_THRESHOLD {
        emergency_shutdown::spawn(FaultType::Undervoltage).unwrap();
    }
}
```

## Periodic ADC Sampling Pattern

### Timer-Based 10Hz Sampling

```rust
#[task(binds = TIMER_IRQ_0, local = [adc, battery_pin, alarm], 
       shared = [battery_voltage], priority = 2)]
fn sample_battery_adc(mut ctx: sample_battery_adc::Context) {
    // Clear timer interrupt
    ctx.local.alarm.clear_interrupt();
    
    // Read ADC value
    let adc_result: nb::Result<u16, _> = ctx.local.adc.read(ctx.local.battery_pin);
    
    match adc_result {
        Ok(adc_value) => {
            // Store atomic value for other tasks
            ctx.shared.battery_voltage.lock(|voltage| {
                voltage.store(adc_value, Ordering::SeqCst);
            });
            
            // Trigger safety check immediately
            voltage_safety_check::spawn().unwrap();
            
            // Schedule next sample (100ms = 10Hz)
            ctx.local.alarm.schedule(100.millis()).unwrap();
        },
        Err(nb::Error::WouldBlock) => {
            // ADC not ready, try again soon
            ctx.local.alarm.schedule(1.millis()).unwrap();
        },
        Err(nb::Error::Other(_)) => {
            // ADC error - trigger fault condition
            emergency_shutdown::spawn(FaultType::AdcError).unwrap();
        }
    }
}
```

## State Machine Implementation 

### Battery State Management

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatteryState {
    Low,      // < 3.1V - ADC ≤ 1425  
    Normal,   // 3.1V - 3.6V - ADC 1426-1674
    Charging, // > 3.6V - ADC ≥ 1675
    Fault,    // Error condition
}

#[task(shared = [battery_state, battery_voltage], priority = 2)]
fn update_battery_state(mut ctx: update_battery_state::Context) {
    let adc_value = ctx.shared.battery_voltage.lock(|v| v.load(Ordering::SeqCst));
    
    let new_state = match adc_value {
        0..=1425 => BatteryState::Low,
        1426..=1674 => BatteryState::Normal,
        1675.. => BatteryState::Charging,
    };
    
    let state_changed = ctx.shared.battery_state.lock(|state| {
        let changed = *state != new_state;
        *state = new_state;
        changed
    });
    
    if state_changed {
        // Log state transition with high priority
        log_battery_state_change::spawn(new_state, adc_value).unwrap();
        
        // Update LED status
        update_status_led::spawn(new_state).unwrap();
    }
}
```

## Shared Resource Management

### Thread-Safe Battery Data Sharing

```rust
use core::sync::atomic::{AtomicU16, AtomicBool, Ordering};

struct SafetyFlags {
    emergency_stop: AtomicBool,
    over_voltage: AtomicBool,
    under_voltage: AtomicBool,
    over_temperature: AtomicBool,
}

impl SafetyFlags {
    fn set_emergency_stop(&self, value: bool) {
        self.emergency_stop.store(value, Ordering::SeqCst);
    }
    
    fn is_safe(&self) -> bool {
        !self.emergency_stop.load(Ordering::SeqCst) &&
        !self.over_voltage.load(Ordering::SeqCst) &&
        !self.under_voltage.load(Ordering::SeqCst) &&
        !self.over_temperature.load(Ordering::SeqCst)
    }
}
```

## Integration with USB Logging

### Non-Blocking Battery Logging

```rust
#[task(priority = 1)]
fn log_battery_state_change(ctx: log_battery_state_change::Context, 
                           state: BatteryState, adc_value: u16) {
    #[cfg(feature = "battery-logs")]
    {
        let voltage_mv = ((adc_value as f32 * 3300.0 / 4095.0) / 0.337) as u16;
        
        let msg = LogMessage {
            timestamp_ms: get_timestamp_ms(),
            level: LogLevel::Info,
            category: LogCategory::Battery,
            content: format_battery_message(state, adc_value, voltage_mv),
            content_len: calculate_content_length(),
        };
        
        // Non-blocking enqueue - won't block critical tasks
        if let Err(_) = LOG_QUEUE.try_enqueue(msg) {
            // Queue full - oldest message will be discarded
            // This prevents blocking critical battery monitoring
        }
    }
}
```

## Hardware Initialization Patterns

### RP2040 ADC and GPIO Setup

```rust
#[init]
fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
    let mut resets = ctx.device.RESETS;
    let mut watchdog = Watchdog::new(ctx.device.WATCHDOG);
    
    let clocks = init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        ctx.device.XOSC,
        ctx.device.CLOCKS,
        ctx.device.PLL_SYS,
        ctx.device.PLL_USB,
        &mut resets,
        &mut watchdog,
    ).ok().unwrap();

    // Initialize ADC for battery monitoring
    let mut adc = Adc::new(ctx.device.ADC, &mut resets);
    let pins = rp_pico::Pins::new(
        ctx.device.IO_BANK0,
        ctx.device.PADS_BANK0,
        sio.gpio_bank0,
        &mut resets,
    );
    
    // GPIO26 for battery voltage monitoring (ADC0)
    let battery_pin = pins.gpio26.into_floating_input().into();
    
    // Configure timer for 10Hz ADC sampling
    let mut timer = Timer::new(ctx.device.TIMER, &mut resets);
    let alarm0 = timer.alarm_0().unwrap();
    
    // Start first ADC sample immediately
    sample_battery_adc::spawn().unwrap();

    let mono = Systick::new(ctx.core.SYST, clocks.system_clock.freq().to_Hz());

    (
        Shared {
            battery_state: BatteryState::Normal,
            battery_voltage: AtomicU16::new(2000), // Safe default
            charging_enabled: AtomicBool::new(false),
            safety_flags: SafetyFlags::new(),
        },
        Local {
            adc,
            battery_pin,
            alarm: alarm0,
            led: pins.led.into_push_pull_output(),
        },
        init::Monotonics(mono),
    )
}
```

## Validation and Testing Patterns

### Built-in Self-Test

```rust
#[task(priority = 1)]
fn battery_self_test(ctx: battery_self_test::Context) {
    // Test ADC functionality
    // Verify voltage divider readings
    // Check safety threshold responses
    // Validate state transitions
    
    let test_results = BatteryTestResults {
        adc_functional: test_adc_reading(),
        voltage_divider_calibrated: test_voltage_accuracy(),
        safety_thresholds_active: test_safety_limits(),
        state_machine_responsive: test_state_transitions(),
    };
    
    if !test_results.all_pass() {
        emergency_shutdown::spawn(FaultType::SelfTestFailed).unwrap();
    }
}
```

## Key Implementation Notes

### RTIC 1.x Specific Patterns
- Use `#[rtic::app]` macro (not `#[rtic::app(v2)]`)
- Resource management with `shared` and `local` sections
- Timer-based task spawning with `spawn_after(Duration)`
- Atomic types for cross-task data sharing
- Priority-based preemptive multitasking

### Safety-Critical Considerations
- Highest priority tasks handle immediate safety responses
- Non-blocking operations for critical paths
- Hardware fault detection with software backup
- Graceful degradation during fault conditions
- Comprehensive self-testing and validation

### Performance Requirements
- 10Hz ADC sampling maintained under all conditions
- ±1% pEMF timing accuracy preserved during battery monitoring
- Non-blocking USB communication
- Minimal interrupt latency for safety-critical tasks