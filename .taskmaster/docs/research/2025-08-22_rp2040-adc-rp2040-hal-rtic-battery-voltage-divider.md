---
title: Research Session
query: "RP2040 ADC rp2040-hal RTIC battery voltage divider 10kΩ 5.1kΩ ADC calibration noise filtering DMA ADC sampling GitHub examples"
date: 8/22/2025
time: 8:50:21 AM
timestamp: 2025-08-22T12:50:21.658Z
exchanges: 1
---

# Research Session

## Initial Query

**Question:** RP2040 ADC rp2040-hal RTIC battery voltage divider 10kΩ 5.1kΩ ADC calibration noise filtering DMA ADC sampling GitHub examples

**Response:**

# RP2040 ADC Implementation with Battery Voltage Monitoring

## Overview of RP2040 ADC Capabilities

The RP2040 microcontroller, which powers the Raspberry Pi Pico and other compatible boards, features a 4-channel 12-bit SAR (Successive Approximation Register) ADC with a sample rate of up to 500 kSPS (kilo-samples per second). This ADC is suitable for many embedded applications, including battery voltage monitoring, sensor readings, and analog input processing. The ADC operates with a full-scale range of 0V to 3.3V, providing a resolution of approximately 0.8mV per bit (3.3V/4096).

The RP2040's ADC has several important characteristics to consider:
- 12-bit resolution (4096 discrete levels)
- 4 available input channels (GPIO26-GPIO29)
- A dedicated temperature sensor channel
- Input impedance of approximately 100kΩ
- Reference voltage derived from the internal 3.3V supply

When implementing battery voltage monitoring with the RP2040, it's essential to understand these characteristics to ensure accurate and reliable measurements, especially when using voltage dividers to measure voltages higher than the ADC's reference voltage.

## Implementing Battery Voltage Monitoring with a Voltage Divider

### Voltage Divider Circuit Design

To measure a battery voltage that exceeds the RP2040's 3.3V maximum input, a voltage divider is necessary. The specified 10kΩ and 5.1kΩ resistor combination creates a voltage divider with the following characteristics:

```
Vout = Vin × (5.1kΩ / (10kΩ + 5.1kΩ))
Vout = Vin × 0.338
```

This means that a battery voltage of 3.3V / 0.338 ≈ 9.76V would produce the maximum 3.3V at the ADC input. This is suitable for monitoring LiPo batteries (typically 3.7V-4.2V per cell) up to 2S configurations, or other battery types within this range.

The actual implementation requires connecting the battery positive terminal to the 10kΩ resistor, the junction between the resistors to the ADC input pin, and the 5.1kΩ resistor to ground. Additionally, a small capacitor (0.1μF to 1μF) in parallel with the 5.1kΩ resistor can help filter noise.

## ADC Implementation with rp2040-hal and RTIC

### Basic ADC Setup with rp2040-hal

The `rp2040-hal` crate provides a convenient abstraction for working with the RP2040's peripherals, including the ADC. Here's a basic implementation for initializing the ADC and reading a value:

```rust
use rp2040_hal::{adc::Adc, gpio::Pin, pac, Sio};

// Initialize ADC
let mut adc = Adc::new(pac.ADC, &mut pac.RESETS);

// Configure GPIO pin for ADC (e.g., GPIO26 as ADC0)
let mut adc_pin = pins.gpio26.into_floating_input();

// Read ADC value
let adc_value: u16 = adc.read(&mut adc_pin).unwrap();

// Convert to voltage (assuming 3.3V reference)
let voltage = (adc_value as f32) * 3.3 / 4096.0;

// Apply voltage divider conversion
let battery_voltage = voltage / 0.338;
```

### Integration with RTIC Framework

RTIC (Real-Time Interrupt-driven Concurrency) provides a framework for developing concurrent applications on embedded systems. Here's how to integrate ADC readings within an RTIC application:

```rust
#[app(device = rp2040_hal::pac, peripherals = true)]
mod app {
    use rp2040_hal::{adc::Adc, clocks::init_clocks_and_plls, gpio::Pins, pac, Sio, watchdog::Watchdog};
    
    #[shared]
    struct Shared {
        battery_voltage: f32,
    }
    
    #[local]
    struct Local {
        adc: Adc,
        adc_pin: Pin<Gpio26, FunctionNull>,
    }
    
    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Initialize clocks and peripherals
        let mut resets = ctx.device.RESETS;
        let mut watchdog = Watchdog::new(ctx.device.WATCHDOG);
        let clocks = init_clocks_and_plls(/* ... */);
        
        let sio = Sio::new(ctx.device.SIO);
        let pins = Pins::new(ctx.device.IO_BANK0, ctx.device.PADS_BANK0, sio.gpio_bank0, &mut resets);
        
        // Initialize ADC
        let adc = Adc::new(ctx.device.ADC, &mut resets);
        let adc_pin = pins.gpio26.into_floating_input();
        
        // Schedule the first ADC reading
        adc_task::spawn().unwrap();
        
        (
            Shared { battery_voltage: 0.0 },
            Local { adc, adc_pin },
            init::Monotonics(),
        )
    }
    
    #[task(local = [adc, adc_pin], shared = [battery_voltage])]
    fn adc_task(ctx: adc_task::Context) {
        // Read ADC value
        let adc_value: u16 = ctx.local.adc.read(ctx.local.adc_pin).unwrap();
        
        // Convert to voltage and apply voltage divider conversion
        let voltage = (adc_value as f32) * 3.3 / 4096.0;
        let battery_voltage = voltage / 0.338;
        
        // Update shared state
        ctx.shared.battery_voltage.lock(|v| *v = battery_voltage);
        
        // Schedule next reading (e.g., every 1 second)
        adc_task::spawn_after(1.secs()).unwrap();
    }
}
```

## ADC Calibration and Accuracy Improvement

### Calibration Techniques

The RP2040's ADC may have inherent inaccuracies due to manufacturing variations and reference voltage fluctuations. Implementing calibration can significantly improve measurement accuracy:

1. **Reference Voltage Calibration**: Measure the actual 3.3V supply with a precise multimeter and use this value in calculations.

```rust
// Measured reference voltage (example: 3.28V instead of nominal 3.3V)
const VREF_ACTUAL: f32 = 3.28;

// In ADC reading code
let voltage = (adc_value as f32) * VREF_ACTUAL / 4096.0;
```

2. **Two-Point Calibration**: Measure known voltages at two points and create a calibration curve.

```rust
// Calibration constants (determined experimentally)
const ADC_GAIN: f32 = 0.000825; // V/count
const ADC_OFFSET: f32 = 0.012;  // V

// In ADC reading code
let voltage = (adc_value as f32) * ADC_GAIN + ADC_OFFSET;
```

3. **Voltage Divider Calibration**: Measure the actual resistance values of your divider.

```rust
// Measured resistor values
const R1_ACTUAL: f32 = 9.94e3; // 10kΩ resistor measured as 9.94kΩ
const R2_ACTUAL: f32 = 5.08e3; // 5.1kΩ resistor measured as 5.08kΩ

// Calculate actual division ratio
const DIVIDER_RATIO: f32 = R2_ACTUAL / (R1_ACTUAL + R2_ACTUAL);

// In ADC reading code
let battery_voltage = voltage / DIVIDER_RATIO;
```

### Noise Filtering Techniques

Several techniques can be employed to reduce noise in ADC readings:

1. **Hardware Filtering**: Add a capacitor (0.1μF to 1μF) in parallel with the lower resistor of the voltage divider.

2. **Oversampling and Averaging**: Take multiple samples and average them.

```rust
fn read_adc_with_averaging(adc: &mut Adc, pin: &mut impl Channel<Adc>, samples: usize) -> u16 {
    let mut sum: u32 = 0;
    for _ in 0..samples {
        sum += adc.read(pin).unwrap() as u32;
    }
    (sum / samples as u32) as u16
}

// Usage
let adc_value = read_adc_with_averaging(&mut adc, &mut adc_pin, 16);
```

3. **Digital Filtering**: Implement a low-pass filter in software.

```rust
// Simple exponential filter
let alpha = 0.1; // Filter coefficient (0 < alpha < 1)
let filtered_value = alpha * new_reading + (1.0 - alpha) * previous_filtered_value;
```

4. **Median Filtering**: Take multiple samples and use the median value to reject outliers.

```rust
fn read_adc_with_median(adc: &mut Adc, pin: &mut impl Channel<Adc>, samples: usize) -> u16 {
    let mut readings = [0u16; 16]; // Adjust array size as needed
    for i in 0..samples.min(readings.len()) {
        readings[i] = adc.read(pin).unwrap();
    }
    
    // Sort the readings
    readings[0..samples].sort_unstable();
    
    // Return the median
    readings[samples / 2]
}
```

## DMA-Based ADC Sampling

For higher sampling rates or to offload the CPU, the RP2040's DMA controller can be used to automatically transfer ADC readings to memory.

### Basic DMA Setup for ADC

```rust
use rp2040_hal::{adc::Adc, dma::{Channel, ChannelIndex, DMAExt}, pac};

fn setup_adc_dma<I: ChannelIndex>(
    adc: &mut Adc,
    dma: &mut Channel<I>,
    buffer: &mut [u16],
) {
    // Configure ADC for free-running mode
    adc.set_round_robin(0);
    adc.enable_free_running();
    
    // Configure DMA channel
    dma.ch0().set_read_addr(adc.fifo_address() as u32, false);
    dma.ch0().set_write_addr(buffer.as_ptr() as u32, true);
    dma.ch0().set_transfer_count(buffer.len() as u32, true);
    
    // Start DMA transfer
    dma.ch0().enable();
}

// Usage
let mut dma = pac.DMA.split(&mut resets);
let mut buffer = [0u16; 1024];
setup_adc_dma(&mut adc, &mut dma.ch0, &mut buffer);
```

### RTIC Integration with DMA

```rust
#[app(device = rp2040_hal::pac, peripherals = true)]
mod app {
    use rp2040_hal::{adc::Adc, dma::{Channel, CH0, DMAExt}, pac};
    
    #[shared]
    struct Shared {
        battery_voltage: f32,
    }
    
    #[local]
    struct Local {
        adc: Adc,
        dma_ch0: Channel<CH0>,
        buffer: [u16; 1024],
        buffer_ready: bool,
    }
    
    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Initialize peripherals
        let mut resets = ctx.device.RESETS;
        let mut adc = Adc::new(ctx.device.ADC, &mut resets);
        
        // Configure ADC
        adc.set_round_robin(0);
        adc.enable_free_running();
        
        // Initialize DMA
        let dma = ctx.device.DMA.split(&mut resets);
        let mut dma_ch0 = dma.ch0;
        let mut buffer = [0u16; 1024];
        
        // Configure DMA channel
        dma_ch0.listen_transfer_complete();
        
        // Start first transfer
        start_dma_transfer(&mut adc, &mut dma_ch0, &mut buffer);
        
        (
            Shared { battery_voltage: 0.0 },
            Local { 
                adc, 
                dma_ch0, 
                buffer,
                buffer_ready: false,
            },
            init::Monotonics(),
        )
    }
    
    #[task(binds = DMA_IRQ_0, local = [adc, dma_ch0, buffer, buffer_ready], shared = [battery_voltage])]
    fn dma_handler(ctx: dma_handler::Context) {
        // Clear interrupt
        ctx.local.dma_ch0.clear_transfer_complete();
        
        // Mark buffer as ready for processing
        *ctx.local.buffer_ready = true;
        
        // Schedule processing task
        process_adc_data::spawn().unwrap();
        
        // Restart DMA transfer
        start_dma_transfer(ctx.local.adc, ctx.local.dma_ch0, ctx.local.buffer);
    }
    
    #[task(local = [buffer, buffer_ready], shared = [battery_voltage])]
    fn process_adc_data(ctx: process_adc_data::Context) {
        if !*ctx.local.buffer_ready {
            return;
        }
        
        // Process the buffer (e.g., average all samples)
        let sum: u32 = ctx.local.buffer.iter().map(|&x| x as u32).sum();
        let avg = sum / ctx.local.buffer.len() as u32;
        
        // Convert to voltage
        let voltage = (avg as f32) * 3.3 / 4096.0;
        let battery_voltage = voltage / 0.338;
        
        // Update shared state
        ctx.shared.battery_voltage.lock(|v| *v = battery_voltage);
        
        // Mark buffer as processed
        *ctx.local.buffer_ready = false;
    }
    
    fn start_dma_transfer(adc: &mut Adc, dma: &mut Channel<CH0>, buffer: &mut [u16]) {
        dma.set_read_addr(adc.fifo_address() as u32, false);
        dma.set_write_addr(buffer.as_ptr() as u32, true);
        dma.set_transfer_count(buffer.len() as u32, true);
        dma.enable();
    }
}
```

## GitHub Examples and Best Practices

### Notable GitHub Examples

1. **rp-hal/rp2040-hal Examples**: The official repository contains examples for ADC usage:
   - [adc.rs](https://github.com/rp-rs/rp-hal/blob/main/boards/rp-pico/examples/pico_adc.rs)

2. **Embassy RTIC Examples**: The Embassy project provides examples of RTIC with RP2040:
   - [Embassy RTIC Examples](https://github.com/embassy-rs/embassy/tree/main/examples/rp/src/bin)

3. **Raspberry Pi Pico Examples**: Official Raspberry Pi Pico examples repository:
   - [Pico Examples](https://github.com/raspberrypi/pico-examples/tree/master/adc)

### Best Practices for Battery Monitoring

1. **Low-Power Considerations**: Disable the ADC when not in use to save power.

```rust
// After reading
adc.disable();

// Before next reading
adc.enable();
```

2. **Error Handling**: Implement proper error handling for ADC readings.

```rust
match adc.read(&mut adc_pin) {
    Ok(value) => {
        // Process valid reading
    },
    Err(_) => {
        // Handle error (e.g., log, retry, or use last valid reading)
    }
}
```

3. **Battery Level Estimation**: Convert voltage to battery percentage based on discharge curve.

```rust
fn battery_percentage(voltage: f32) -> u8 {
    // Example for LiPo battery (adjust based on your battery type)
    if voltage >= 4.2 {
        return 100;
    } else if voltage <= 3.3 {
        return 0;
    } else {
        // Linear approximation (consider using a lookup table for better accuracy)
        return ((voltage - 3.3) / 0.9 * 100.0) as u8;
    }
}
```

4. **Hysteresis**: Implement hysteresis to prevent rapid fluctuations in reported battery level.

```rust
const HYSTERESIS_THRESHOLD: f32 = 0.05; // 50mV

fn update_battery_level(new_voltage: f32, prev_voltage: f32, prev_level: u8) -> u8 {
    let new_level = battery_percentage(new_voltage);
    
    // Apply hysteresis
    if (new_voltage - prev_voltage).abs() < HYSTERESIS_THRESHOLD {
        return prev_level;
    }
    
    new_level
}
```

## Conclusion

Implementing battery voltage monitoring on the RP2040 using a voltage divider requires careful consideration of ADC characteristics, calibration, and noise filtering. The rp2040-hal crate provides a solid foundation for ADC operations, while RTIC offers a framework for structured concurrent applications. DMA can be employed for more efficient sampling, especially at higher rates.

For the specific 10kΩ/5.1kΩ voltage divider mentioned, the system can measure battery voltages up to approximately 9.76V, making it suitable for various battery configurations. By implementing proper calibration and filtering techniques, you can achieve accurate and reliable battery voltage monitoring in your RP2040-based projects.

Remember to consider the power consumption implications of your implementation, especially for battery-powered devices, and implement appropriate error handling and battery level estimation algorithms for a robust solution.


---

*Generated by Task Master Research Command*  
*Timestamp: 2025-08-22T12:50:21.658Z*
