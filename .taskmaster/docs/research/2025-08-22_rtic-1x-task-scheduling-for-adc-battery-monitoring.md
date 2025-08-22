---
title: Research Session
query: "RTIC 1.x task scheduling for ADC battery monitoring RP2040 periodic sampling timer-based task spawning shared resources priority assignment best practices"
date: 8/22/2025
time: 8:49:09 AM
timestamp: 2025-08-22T12:49:09.326Z
exchanges: 1
---

# Research Session

## Initial Query

**Question:** RTIC 1.x task scheduling for ADC battery monitoring RP2040 periodic sampling timer-based task spawning shared resources priority assignment best practices

**Response:**

# RTIC 1.x Task Scheduling for ADC Battery Monitoring on RP2040

## Introduction

Battery monitoring is a critical component in embedded systems, particularly for portable or battery-powered devices. The RP2040 microcontroller, with its dual-core Arm Cortex-M0+ processor and rich peripheral set, provides an excellent platform for implementing battery monitoring solutions. Real-Time Interrupt-driven Concurrency (RTIC) framework version 1.x offers a robust approach to handling concurrent tasks in embedded Rust applications. This research explores how to effectively implement ADC-based battery monitoring on the RP2040 using RTIC 1.x, focusing on periodic sampling, timer-based task spawning, shared resource management, and priority assignment best practices.

## RP2040 ADC Capabilities Overview

The RP2040 includes a 4-channel, 12-bit successive approximation register (SAR) ADC with the following specifications:

- 4 analog input channels (GPIO26-GPIO29)
- 12-bit resolution (0-4095 range)
- Sampling rate up to 500 ksps (kilosamples per second)
- Built-in temperature sensor as a fifth input
- 3.3V reference voltage
- DMA support for autonomous sampling

For battery monitoring applications, these capabilities allow for precise voltage measurements with minimal CPU intervention. The ADC can be configured to sample at regular intervals, with results either processed immediately or stored for later analysis.

## RTIC 1.x Architecture for Battery Monitoring

### Basic RTIC Structure

RTIC 1.x provides a framework for real-time concurrent applications with the following key components:

```rust
#[rtic::app(device = rp2040_hal::pac, dispatchers = [PIO0_IRQ_0])]
mod app {
    use rp2040_hal::{adc::Adc, clocks::init_clocks_and_plls, watchdog::Watchdog};
    // Additional imports...

    #[shared]
    struct Shared {
        battery_voltage: u16,
        // Other shared resources...
    }

    #[local]
    struct Local {
        adc: Adc,
        // Other local resources...
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Initialize clocks, ADC, etc.
        // Setup timer/alarm for periodic sampling
        
        // Schedule the first ADC reading
        sample_battery::spawn().unwrap();
        
        (
            Shared {
                battery_voltage: 0,
                // Initialize other shared resources...
            },
            Local {
                adc,
                // Initialize other local resources...
            },
            init::Monotonics()
        )
    }

    // Tasks definitions follow...
}
```

## Periodic ADC Sampling Implementation

### Timer-Based Task Spawning

For battery monitoring, a consistent sampling interval is crucial. RTIC 1.x offers several approaches to implement periodic sampling:

#### 1. Self-Scheduling Tasks

```rust
#[task(local = [adc], shared = [battery_voltage])]
fn sample_battery(mut cx: sample_battery::Context) {
    // Perform ADC reading
    let adc = cx.local.adc;
    let pin = // Get the appropriate ADC pin
    let reading = adc.read(pin).unwrap();
    
    // Convert ADC reading to voltage
    let voltage = convert_to_voltage(reading);
    
    // Update shared resource
    cx.shared.battery_voltage.lock(|v| {
        *v = voltage;
    });
    
    // Schedule next execution in 1 second
    sample_battery::spawn_after(1.secs()).unwrap();
}
```

#### 2. Timer Interrupt Approach

```rust
#[task(binds = TIMER_IRQ_0, priority = 2, local = [alarm])]
fn timer_irq(cx: timer_irq::Context) {
    let alarm = cx.local.alarm;
    
    // Clear the interrupt
    alarm.clear_interrupt();
    
    // Set the next alarm (e.g., 1 second later)
    alarm.schedule(1.secs()).unwrap();
    
    // Spawn the ADC sampling task
    sample_battery::spawn().unwrap();
}

#[task(local = [adc], shared = [battery_voltage], priority = 1)]
fn sample_battery(mut cx: sample_battery::Context) {
    // Perform ADC reading and update shared resource
    // (similar to previous example)
}
```

### Optimizing ADC Configuration for Battery Monitoring

For battery voltage monitoring, proper ADC configuration is essential:

```rust
fn configure_adc(adc: &mut Adc) {
    // Set ADC clock divider for appropriate sampling rate
    // For battery monitoring, a slower rate is typically sufficient
    adc.set_clkdiv(48000.0); // Example: 1kHz sampling rate with 48MHz system clock
    
    // Set up ADC for battery monitoring
    // - Enable temperature sensor if monitoring battery temperature
    // - Configure input scaling if using voltage dividers
    // - Set up round-robin sampling if monitoring multiple channels
}
```

## Shared Resource Management

### Lock-Free Approaches

For simple battery monitoring, consider atomic operations to avoid locking overhead:

```rust
#[shared]
struct Shared {
    battery_voltage: AtomicU16,
    // Other shared resources...
}

#[task(local = [adc], shared = [battery_voltage])]
fn sample_battery(mut cx: sample_battery::Context) {
    // Perform ADC reading
    let reading = cx.local.adc.read(/* pin */).unwrap();
    let voltage = convert_to_voltage(reading);
    
    // Update shared resource without locking
    cx.shared.battery_voltage.store(voltage, Ordering::Relaxed);
    
    // Schedule next execution
    sample_battery::spawn_after(1.secs()).unwrap();
}
```

### Resource Contention Management

When multiple tasks need access to the ADC or battery voltage data:

```rust
#[task(local = [adc], shared = [battery_voltage], priority = 2)]
fn sample_battery(mut cx: sample_battery::Context) {
    // High-priority task that samples the battery
    let reading = cx.local.adc.read(/* pin */).unwrap();
    let voltage = convert_to_voltage(reading);
    
    cx.shared.battery_voltage.lock(|v| {
        *v = voltage;
    });
    
    // Schedule next execution
    sample_battery::spawn_after(1.secs()).unwrap();
}

#[task(shared = [battery_voltage], priority = 1)]
fn process_battery_data(mut cx: process_battery_data::Context) {
    // Lower-priority task that processes battery data
    let voltage = cx.shared.battery_voltage.lock(|v| *v);
    
    // Process the voltage data (e.g., check if battery is low)
    if voltage < LOW_BATTERY_THRESHOLD {
        // Handle low battery condition
    }
}
```

## Priority Assignment Best Practices

### Priority Hierarchy for Battery Monitoring

For a battery monitoring system, consider the following priority structure:

1. **Highest Priority (3-4)**: Critical system tasks (e.g., safety shutdowns)
2. **Medium-High Priority (2)**: ADC sampling and immediate processing
3. **Medium Priority (1)**: Data analysis and decision making
4. **Low Priority (0)**: Reporting, logging, and non-critical tasks

```rust
// Critical battery-related safety task
#[task(shared = [battery_voltage], priority = 3)]
fn battery_safety_monitor(mut cx: battery_safety_monitor::Context) {
    let voltage = cx.shared.battery_voltage.lock(|v| *v);
    
    if voltage < CRITICAL_THRESHOLD {
        // Perform emergency shutdown or enter low-power mode
        emergency_shutdown::spawn().unwrap();
    }
}

// ADC sampling task
#[task(local = [adc], shared = [battery_voltage], priority = 2)]
fn sample_battery(mut cx: sample_battery::Context) {
    // ADC sampling code...
}

// Battery data analysis
#[task(shared = [battery_voltage, battery_status], priority = 1)]
fn analyze_battery(mut cx: analyze_battery::Context) {
    // Analysis code...
}

// Reporting task
#[task(shared = [battery_status], priority = 0)]
fn report_battery_status(mut cx: report_battery_status::Context) {
    // Reporting code...
}
```

### Avoiding Priority Inversion

Priority inversion occurs when a high-priority task is blocked waiting for a resource held by a lower-priority task. RTIC's priority ceiling protocol helps prevent this, but careful design is still necessary:

1. Keep critical sections short
2. Avoid complex operations while holding locks
3. Consider using separate resources for different concerns

## Advanced Implementation Techniques

### DMA-Based Sampling

For more efficient battery monitoring, leverage the RP2040's DMA capabilities:

```rust
#[init]
fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
    // Standard initialization...
    
    // Configure DMA for ADC sampling
    let dma = cx.device.DMA.split(&mut resets);
    let dma_channel = dma.ch0;
    
    // Configure ADC for DMA transfers
    adc.configure_dma_transfer(dma_channel, buffer_address, buffer_size);
    
    // Start periodic DMA transfers
    adc.start_dma_transfers();
    
    // Rest of initialization...
}

#[task(binds = DMA_IRQ_0, priority = 2, local = [dma_channel, buffer])]
fn dma_complete(cx: dma_complete::Context) {
    // DMA transfer complete - process the buffer
    let buffer = cx.local.buffer;
    
    // Process the ADC readings
    process_battery_readings::spawn(buffer[0]).unwrap();
    
    // Restart DMA transfer
    cx.local.dma_channel.restart();
}
```

### Multi-Channel Battery Monitoring

For systems with multiple batteries or monitoring points:

```rust
#[shared]
struct Shared {
    main_battery: u16,
    backup_battery: u16,
    system_voltage: u16,
}

#[task(local = [adc, current_channel: usize = 0], shared = [main_battery, backup_battery, system_voltage])]
fn sample_batteries(mut cx: sample_batteries::Context) {
    let adc = cx.local.adc;
    let channel = *cx.local.current_channel;
    
    // Rotate through channels
    match channel {
        0 => {
            let reading = adc.read(Channel::Channel0).unwrap();
            cx.shared.main_battery.lock(|v| *v = reading);
        },
        1 => {
            let reading = adc.read(Channel::Channel1).unwrap();
            cx.shared.backup_battery.lock(|v| *v = reading);
        },
        2 => {
            let reading = adc.read(Channel::Channel2).unwrap();
            cx.shared.system_voltage.lock(|v| *v = reading);
        },
        _ => unreachable!(),
    }
    
    // Update channel for next iteration
    *cx.local.current_channel = (channel + 1) % 3;
    
    // Schedule next sample
    sample_batteries::spawn_after(333.millis()).unwrap();
}
```

## Power Optimization Considerations

Battery monitoring systems should themselves be power-efficient:

1. **Adjust Sampling Frequency**: Reduce sampling rate when battery status is stable
2. **Use Sleep Modes**: Put the processor to sleep between samples
3. **Optimize ADC Configuration**: Use the minimum required sampling rate and precision

```rust
#[task(local = [adc, sample_interval: u32 = 1000], shared = [battery_voltage, battery_status])]
fn sample_battery(mut cx: sample_battery::Context) {
    // Perform ADC reading
    let reading = cx.local.adc.read(/* pin */).unwrap();
    
    // Update shared resources
    let voltage = convert_to_voltage(reading);
    let mut status_changed = false;
    
    cx.shared.battery_voltage.lock(|v| {
        *v = voltage;
    });
    
    cx.shared.battery_status.lock(|status| {
        let new_status = determine_battery_status(voltage);
        status_changed = *status != new_status;
        *status = new_status;
    });
    
    // Adjust sampling interval based on status changes
    if status_changed {
        // Sample more frequently when status is changing
        *cx.local.sample_interval = 500; // ms
    } else {
        // Gradually increase interval when stable
        *cx.local.sample_interval = (*cx.local.sample_interval * 11 / 10).min(10000);
    }
    
    // Schedule next execution with dynamic interval
    let interval = *cx.local.sample_interval;
    sample_battery::spawn_after(interval.millis()).unwrap();
}
```

## Complete Implementation Example

Here's a comprehensive example integrating the concepts discussed:

```rust
#[rtic::app(device = rp2040_hal::pac, dispatchers = [PIO0_IRQ_0, PIO0_IRQ_1, PIO1_IRQ_0])]
mod app {
    use core::sync::atomic::{AtomicU16, Ordering};
    use rp2040_hal::{
        adc::{Adc, AdcPin, Channel},
        clocks::{init_clocks_and_plls, Clock},
        gpio::{bank0::Gpio26, Pins},
        pac,
        sio::Sio,
        timer::{Alarm, Alarm0},
        watchdog::Watchdog,
    };
    use fugit::{ExtU32, RateExtU32};
    
    // Constants for battery monitoring
    const VREF: f32 = 3.3; // Reference voltage
    const ADC_MAX: f32 = 4095.0; // 12-bit ADC
    const CRITICAL_THRESHOLD: u16 = 3000; // 3.0V (scaled to millivolts)
    const LOW_THRESHOLD: u16 = 3300; // 3.3V (scaled to millivolts)
    
    #[shared]
    struct Shared {
        battery_voltage: AtomicU16, // Store in millivolts
        battery_status: BatteryStatus,
    }
    
    #[local]
    struct Local {
        adc: Adc,
        battery_pin: AdcPin<Gpio26>,
        alarm: Alarm0,
    }
    
    #[derive(Clone, Copy, PartialEq)]
    enum BatteryStatus {
        Critical,
        Low,
        Normal,
    }
    
    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Initialize device
        let mut resets = cx.device.RESETS;
        let mut watchdog = Watchdog::new(cx.device.WATCHDOG);
        let clocks = init_clocks_and_plls(
            12_000_000u32,
            cx.device.XOSC,
            cx.device.CLOCKS,
            cx.device.PLL_SYS,
            cx.device.PLL_USB,
            &mut resets,
            &mut watchdog,
        )
        .ok()
        .unwrap();
        
        // Initialize GPIO
        let sio = Sio::new(cx.device.SIO);
        let pins = Pins::new(
            cx.device.IO_BANK0,
            cx.device.PADS_BANK0,
            sio.gpio_bank0,
            &mut resets,
        );
        
        // Initialize ADC
        let mut adc = Adc::new(cx.device.ADC, &mut resets);
        let battery_pin = pins.gpio26.into_analog();
        
        // Configure ADC for battery monitoring
        adc.set_clkdiv(48000.0); // 1kHz sampling with 48MHz clock
        
        // Initialize timer for periodic sampling
        let mut timer = rp2040_hal::Timer::new(cx.device.TIMER, &mut resets);
        let mut alarm = timer.alarm_0().unwrap();
        
        // Set up alarm for 1-second intervals
        alarm.schedule(1.secs()).unwrap();
        alarm.enable_interrupt();
        
        (
            Shared {
                battery_voltage: AtomicU16::new(0),
                battery_status: BatteryStatus::Normal,
            },
            Local {
                adc,
                battery_pin,
                alarm,
            },
            init::Monotonics()
        )
    }
    
    #[task(binds = TIMER_IRQ_0, priority = 2, local = [alarm])]
    fn timer_irq(cx: timer_irq::Context) {
        // Clear the interrupt
        cx.local.alarm.clear_interrupt();
        
        // Schedule the next alarm
        cx.local.alarm.schedule(1.secs()).unwrap();
        
        // Spawn the battery sampling task
        sample_battery::spawn().unwrap();
    }
    
    #[task(local = [adc, battery_pin], shared = [battery_voltage], priority = 2)]
    fn sample_battery(mut cx: sample_battery::Context) {
        // Perform ADC reading
        let adc = cx.local.adc;
        let pin = cx.local.battery_pin;
        
        let reading = adc.read(pin).unwrap();
        
        // Convert ADC reading to voltage (in millivolts)
        let voltage_mv = ((reading as f32 / ADC_MAX) * VREF * 1000.0) as u16;
        
        // Update shared resource without locking (using atomic)
        cx.shared.battery_voltage.store(voltage_mv, Ordering::Relaxed);
        
        // Spawn the analysis task
        analyze_battery::spawn().unwrap();
    }
    
    #[task(shared = [battery_voltage, battery_status], priority = 1)]
    fn analyze_battery(mut cx: analyze_battery::Context) {
        // Get the current voltage
        let voltage = cx.shared.battery_voltage.load(Ordering::Relaxed);
        
        // Determine battery status
        let new_status = if voltage < CRITICAL_THRESHOLD {
            BatteryStatus::Critical
        } else if voltage < LOW_THRESHOLD {
            BatteryStatus::Low
        } else {
            BatteryStatus::Normal
        };
        
        // Update status if changed
        cx.shared.battery_status.lock(|status| {
            if *status != new_status {
                *status = new_status;
                
                // Spawn appropriate response task based on status
                match new_status {
                    BatteryStatus::Critical => emergency_response::spawn().unwrap(),
                    BatteryStatus::Low => low_battery_warning::spawn().unwrap(),
                    BatteryStatus::Normal => {}, // No action needed
                }
            }
        });
    }
    
    #[task(priority = 3)]
    fn emergency_response(_: emergency_response::Context) {
        // Handle critical battery level
        // e.g., save critical data, prepare for shutdown
    }
    
    #[task(priority = 0)]
    fn low_battery_warning(_: low_battery_warning::Context) {
        // Handle low battery warning
        // e.g., notify user, reduce power consumption
    }
}
```

## Conclusion

Implementing ADC-based battery monitoring on the RP2040 using RTIC 1.x requires careful consideration of task scheduling, resource sharing, and priority assignment. By following the best practices outlined in this research, you can create a robust, efficient battery monitoring system that provides reliable data while minimizing power consumption and system overhead.

The key takeaways include:
1. Use timer-based or self-scheduling tasks for consistent sampling intervals
2. Leverage atomic operations for simple shared resources to avoid locking overhead
3. Implement a clear priority hierarchy with critical battery-related tasks at higher priorities
4. Consider power optimization by dynamically adjusting sampling rates
5. Use DMA for more efficient sampling when appropriate
6. Implement proper error handling and safety mechanisms for battery monitoring

By applying these principles, you can create a battery monitoring system that is both reliable and efficient, ensuring your RP2040-based device operates safely within its power constraints.


---

*Generated by Task Master Research Command*  
*Timestamp: 2025-08-22T12:49:09.326Z*
