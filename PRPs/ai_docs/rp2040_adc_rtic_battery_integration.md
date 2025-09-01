# RP2040 ADC Battery Integration with RTIC

## Critical ADC Configuration Patterns for RP2040

### ADC Peripheral Initialization
```rust
use rp2040_hal::{
    adc::{Adc, AdcPin},
    gpio::{Pin, bank0::Gpio26, FunctionSioInput, PullNone},
    pac,
    Sio
};
use embedded_hal::adc::OneShot;

// Initialize ADC in RTIC init() function
let mut adc = Adc::new(ctx.device.ADC, &mut ctx.device.RESETS);

// Configure GPIO26 as ADC input (standard battery monitoring pin)
let sio = Sio::new(ctx.device.SIO);
let pins = rp2040_hal::gpio::Pins::new(
    ctx.device.IO_BANK0,
    ctx.device.PADS_BANK0,
    sio.gpio_bank0,
    &mut ctx.device.RESETS,
);
let battery_pin: AdcPin<Pin<Gpio26, FunctionSioInput, PullNone>> = 
    pins.gpio26.into_floating_input().into();
```

### RTIC Resource Management for ADC
```rust
#[local]
struct Local {
    adc: Adc,
    battery_pin: AdcPin<Pin<Gpio26, FunctionSioInput, PullNone>>,
}

#[shared]  
struct Shared {
    battery_monitor: BatteryMonitor,
    safety_flags: SafetyFlags,
    battery_state: BatteryState,
}
```

### Critical RP2040 ADC Gotchas

#### 1. First Reading Inaccuracy
```rust
// ALWAYS discard the first ADC reading - it's often inaccurate on RP2040
let _discard = adc.read(&mut battery_pin).unwrap();
let actual_reading: u16 = adc.read(&mut battery_pin).unwrap();
```

#### 2. Effective Resolution Limitations
- RP2040 ADC has ~8.6 bits effective resolution, not full 12-bit
- Use software averaging for better accuracy:
```rust
fn read_adc_averaged(adc: &mut Adc, pin: &mut AdcPin<...>, samples: usize) -> u16 {
    let mut sum: u32 = 0;
    for _ in 0..samples {
        sum += adc.read(pin).unwrap() as u32;
    }
    (sum / samples as u32) as u16
}
```

#### 3. Known Problematic ADC Values
Avoid these ADC readings as they may indicate stuck values:
- 512, 1536, 2560, 3584 (quarter-scale boundaries)

### Battery Voltage Calculation

#### Voltage Divider for 3.7V LiPo Battery
```rust
// Hardware: 10kΩ (R1) and 5.1kΩ (R2) voltage divider
// Scale factor: R2/(R1+R2) = 5.1kΩ/15.1kΩ = 0.337
const VOLTAGE_SCALE_FACTOR: f32 = 0.337;
const ADC_REFERENCE_VOLTAGE_MV: f32 = 3300.0;
const ADC_RESOLUTION: f32 = 4095.0;

fn convert_adc_to_battery_voltage_mv(adc_value: u16) -> u16 {
    let adc_voltage_mv = (adc_value as f32 * ADC_REFERENCE_VOLTAGE_MV) / ADC_RESOLUTION;
    let battery_voltage_mv = adc_voltage_mv / VOLTAGE_SCALE_FACTOR;
    battery_voltage_mv as u16
}
```

### RTIC Task Integration Pattern
```rust
#[task(
    local = [adc, battery_pin],
    shared = [battery_monitor, safety_flags, log_queue],
    priority = 4
)]
fn battery_monitor_task(mut ctx: battery_monitor_task::Context) {
    // Read ADC with error handling
    let adc_result = ctx.local.adc.read(ctx.local.battery_pin);
    
    let adc_value = match adc_result {
        Ok(value) => value,
        Err(_) => {
            // Handle ADC read failure - use last known good value
            ctx.shared.battery_monitor.lock(|monitor| {
                monitor.get_last_adc_reading()
            })
        }
    };
    
    // Process battery sample using existing driver
    let result = ctx.shared.battery_monitor.lock(|monitor| {
        ctx.shared.safety_flags.lock(|flags| {
            monitor.process_sample(ctx.local.battery_pin, flags)
        })
    });
    
    // Handle results and reschedule
    match result {
        Ok(reading) => {
            // Log successful reading if enabled
        },
        Err(battery_error) => {
            if battery_error.requires_emergency_shutdown() {
                // Trigger emergency safety response
                ctx.shared.safety_flags.lock(|flags| {
                    flags.set_emergency_stop(true);
                });
            }
        }
    }
    
    // Reschedule for next reading (100ms = 10Hz)
    battery_monitor_task::spawn_after(Duration::<u64, 1, 1000>::millis(100)).unwrap();
}
```

### Safety Considerations for Battery Monitoring

#### Critical Voltage Thresholds (for 3.7V LiPo)
```rust
// These constants are safety-critical and validated against hardware
pub const LOW_BATTERY_ADC_THRESHOLD: u16 = 1425;      // 3.1V
pub const CHARGING_ADC_THRESHOLD: u16 = 1675;         // 3.6V  
pub const OVERVOLTAGE_ADC_THRESHOLD: u16 = 1800;      // 4.2V (DANGER)
pub const UNDERVOLTAGE_ADC_THRESHOLD: u16 = 1200;     // 3.0V (DANGER)
```

#### Emergency Response Pattern
```rust
// Battery errors that require immediate response
match battery_error {
    BatteryError::OverVoltage { .. } => {
        // CRITICAL: Disable charging immediately
        safety_monitor.emergency_disable_charging()?;
    },
    BatteryError::UnderVoltage { .. } => {
        // CRITICAL: Reduce load, warn user
        safety_flags.set_emergency_stop(true);
    },
    _ => {
        // Non-critical errors - log and continue
    }
}
```

### Hardware Integration Requirements

#### GPIO26 Pin Configuration
- GPIO26 is ADC0 on RP2040
- Must be configured as floating input (no pull resistors)
- Connect to voltage divider output (between R1 and R2)

#### Voltage Divider Circuit
```
Battery+ ─── 10kΩ (R1) ─── GPIO26 ─── 5.1kΩ (R2) ─── GND
                            │
                        100nF (C1) (optional filtering)
                            │
                           GND
```

### Common Integration Pitfalls

1. **USB Enumeration Interference**: Never use Priority 1 for battery tasks
2. **Resource Contention**: Use proper RTIC resource locking patterns  
3. **ADC Timing**: Don't sample ADC faster than 10Hz for battery monitoring
4. **Error Propagation**: Battery errors must not break USB functionality
5. **Safety Response**: Critical errors require immediate hardware response

### Validation Commands
```bash
# Verify USB enumeration is preserved
lsusb | grep fade

# Build with battery monitoring enabled  
cargo build --target thumbv6m-none-eabi --features battery-logs

# Flash and test
cargo run
```

This documentation provides the essential patterns for successful RP2040 ADC integration with RTIC for battery monitoring applications, incorporating hardware-specific considerations and safety-critical requirements.