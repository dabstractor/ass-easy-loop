# RTIC 2.0 Implementation Patterns for Multi-Subsystem Embedded Projects

## Overview

This document provides comprehensive patterns for implementing complex multi-subsystem embedded projects using RTIC 2.0, specifically targeting the RP2040-based pEMF device architecture. These patterns ensure real-time constraints, proper resource sharing, and maintainable code organization.

## Core RTIC 2.0 Architecture Patterns

### App Structure Pattern

```rust
#[rtic::app(device = rp2040_hal::pac, peripherals = true)]
mod app {
    use super::*;
    
    // PATTERN: Separate shared resources by access pattern and contention
    #[shared]
    struct Shared {
        // High-contention resources (accessed by multiple priorities)
        system_config: SystemConfig,
        battery_state: BatteryState,
        
        // Communication resources
        usb_command_queue: heapless::spsc::Queue<UsbCommand, 8>,
        log_message_queue: heapless::spsc::Queue<LogMessage, 32>,
    }

    // PATTERN: Group local resources by subsystem
    #[local]
    struct Local {
        // Waveform generation subsystem (priority 1)
        pwm_channel: pwm::Channel<rp2040_hal::pwm::Pwm0, pwm::A>,
        waveform_phase_accumulator: u32,
        
        // Battery monitoring subsystem (priority 2)
        adc: adc::Adc,
        adc_pin: gpio::Pin<gpio::bank0::Gpio26, gpio::Input<gpio::Floating>>,
        battery_sample_buffer: heapless::Vec<u16, 10>,
        
        // Communication subsystem (priority 3)
        usb_device: usb_device::UsbDevice<'static, rp2040_hal::usb::UsbBus>,
        hid_class: usbd_hid::HidClass<'static, rp2040_hal::usb::UsbBus>,
        led_pin: gpio::Pin<gpio::bank0::Gpio25, gpio::Output<gpio::PushPull>>,
    }

    // PATTERN: Use monotonic timer for precise scheduling
    #[monotonic(binds = SysTick, default = true)]
    type MonoTimer = systick_monotonic::Systick<1000>; // 1kHz timer
}
```

## Priority Hierarchy Design Pattern

### Three-Tier Priority System

```rust
// PRIORITY 1: Hard real-time, deterministic timing (waveform generation)
#[task(
    priority = 1,
    shared = [system_config],
    local = [pwm_channel, waveform_phase_accumulator]
)]
fn waveform_update(mut ctx: waveform_update::Context) {
    // PATTERN: Minimize lock duration for shared resources
    let config = ctx.shared.system_config.lock(|cfg| cfg.waveform.clone());
    
    // PATTERN: All calculations use local resources (no contention)
    let phase = update_phase_accumulator(
        ctx.local.waveform_phase_accumulator, 
        config.frequency_hz
    );
    
    let duty_cycle = calculate_waveform_duty_cycle(&config, phase);
    ctx.local.pwm_channel.set_duty(duty_cycle);
    
    // PATTERN: Schedule next execution at precise intervals
    waveform_update::spawn_after(WAVEFORM_UPDATE_INTERVAL).ok();
}

// PRIORITY 2: Soft real-time, periodic monitoring (battery, sensors)
#[task(
    priority = 2,
    shared = [battery_state, log_message_queue],
    local = [adc, adc_pin, battery_sample_buffer]
)]
fn battery_monitor(mut ctx: battery_monitor::Context) {
    // PATTERN: Use local buffering to smooth readings
    let raw_reading = ctx.local.adc.read(ctx.local.adc_pin).unwrap_or(0);
    ctx.local.battery_sample_buffer.push(raw_reading).ok();
    
    if ctx.local.battery_sample_buffer.is_full() {
        let averaged_reading = calculate_average(&ctx.local.battery_sample_buffer);
        ctx.local.battery_sample_buffer.clear();
        
        let new_state = BatteryState::from_adc_reading(averaged_reading);
        
        // PATTERN: Update shared state and queue notifications
        let state_changed = ctx.shared.battery_state.lock(|state| {
            if *state != new_state {
                *state = new_state;
                true
            } else {
                false
            }
        });
        
        if state_changed {
            let log_msg = LogMessage::BatteryStateChange(new_state, averaged_reading);
            ctx.shared.log_message_queue.lock(|queue| queue.enqueue(log_msg).ok());
        }
    }
    
    battery_monitor::spawn_after(BATTERY_MONITOR_INTERVAL).ok();
}

// PRIORITY 3: Background tasks, user interface, communications
#[task(
    priority = 3,
    shared = [log_message_queue, usb_command_queue, battery_state],
    local = [usb_device, hid_class, led_pin]
)]
fn usb_handler(mut ctx: usb_handler::Context) {
    // PATTERN: Non-blocking USB device polling
    if ctx.local.usb_device.poll(&mut [ctx.local.hid_class]) {
        // Handle USB events and commands
        process_usb_commands(&mut ctx);
        send_pending_logs(&mut ctx);
    }
    
    usb_handler::spawn_after(USB_POLL_INTERVAL).ok();
}
```

## Resource Sharing Patterns

### Lock Minimization Pattern

```rust
// ANTI-PATTERN: Long lock duration blocks other tasks
fn bad_shared_access(mut ctx: some_task::Context) {
    ctx.shared.config.lock(|cfg| {
        // DON'T: Complex calculations while holding lock
        let result = expensive_calculation(&cfg.parameters);
        cfg.cached_result = Some(result);
    });
}

// PATTERN: Minimize lock duration
fn good_shared_access(mut ctx: some_task::Context) {
    // Read shared data quickly
    let parameters = ctx.shared.config.lock(|cfg| cfg.parameters.clone());
    
    // Do expensive work with local copy
    let result = expensive_calculation(&parameters);
    
    // Update shared state quickly
    ctx.shared.config.lock(|cfg| cfg.cached_result = Some(result));
}
```

### Resource Access Pattern by Priority

```rust
// PATTERN: Higher priority tasks access shared resources briefly
#[task(priority = 1, shared = [waveform_config])]
fn high_priority_task(mut ctx: high_priority_task::Context) {
    // Quick read of configuration
    let freq = ctx.shared.waveform_config.lock(|cfg| cfg.frequency_hz);
    // Use freq immediately in time-critical calculation
}

// PATTERN: Lower priority tasks can hold locks longer but should yield
#[task(priority = 3, shared = [waveform_config])]
fn low_priority_task(mut ctx: low_priority_task::Context) {
    ctx.shared.waveform_config.lock(|cfg| {
        // Longer operations acceptable at low priority
        validate_configuration(cfg);
        apply_safety_limits(cfg);
    });
}
```

## Task Communication Patterns

### Message Queue Pattern

```rust
// PATTERN: Use typed message queues for inter-task communication
#[derive(Clone, Debug)]
enum SystemMessage {
    BatteryStateChange(BatteryState),
    ConfigurationUpdate(WaveformConfig),
    UsbCommand(UsbCommand),
    EmergencyShutdown,
}

// Producer pattern
fn producer_task(mut ctx: producer_task::Context) {
    let message = SystemMessage::BatteryStateChange(BatteryState::Low);
    ctx.shared.message_queue.lock(|queue| {
        // PATTERN: Handle queue full condition gracefully
        if queue.enqueue(message.clone()).is_err() {
            // Queue full - log error and potentially discard oldest
            queue.dequeue(); // Remove oldest
            queue.enqueue(message).ok(); // Try again
        }
    });
}

// Consumer pattern
fn consumer_task(mut ctx: consumer_task::Context) {
    while let Some(message) = ctx.shared.message_queue.lock(|queue| queue.dequeue()) {
        match message {
            SystemMessage::BatteryStateChange(state) => handle_battery_state(state),
            SystemMessage::ConfigurationUpdate(config) => handle_config_update(config),
            SystemMessage::UsbCommand(cmd) => handle_usb_command(cmd),
            SystemMessage::EmergencyShutdown => initiate_shutdown(),
        }
    }
}
```

### Event-Driven State Machine Pattern

```rust
// PATTERN: State machines for complex subsystem behavior
#[derive(Clone, Copy, Debug)]
enum SystemState {
    Initializing,
    Normal,
    LowBattery,
    Charging,
    Error(SystemError),
}

impl SystemState {
    // PATTERN: State transition logic encapsulated in methods
    fn handle_event(self, event: SystemEvent) -> Self {
        match (self, event) {
            (SystemState::Initializing, SystemEvent::InitComplete) => SystemState::Normal,
            (SystemState::Normal, SystemEvent::BatteryLow) => SystemState::LowBattery,
            (SystemState::LowBattery, SystemEvent::ChargingStarted) => SystemState::Charging,
            (SystemState::Charging, SystemEvent::ChargingComplete) => SystemState::Normal,
            (_, SystemEvent::CriticalError(err)) => SystemState::Error(err),
            _ => self, // No transition for this event in current state
        }
    }
    
    // PATTERN: State-specific behavior methods
    fn get_led_pattern(self) -> LedPattern {
        match self {
            SystemState::Normal => LedPattern::Off,
            SystemState::LowBattery => LedPattern::FastBlink,
            SystemState::Charging => LedPattern::SolidOn,
            SystemState::Error(_) => LedPattern::SlowBlink,
            _ => LedPattern::Off,
        }
    }
}
```

## Timing and Scheduling Patterns

### Periodic Task Scheduling Pattern

```rust
// PATTERN: Use spawn_after for precise periodic execution
const WAVEFORM_UPDATE_PERIOD: fugit::Duration<u32, 1, 1000> = 
    fugit::Duration::<u32, 1, 1000>::from_ticks(1); // 1ms = 1000Hz

#[task(priority = 1)]
fn waveform_update(ctx: waveform_update::Context) {
    // Perform waveform update
    update_pwm_output();
    
    // PATTERN: Schedule next execution precisely
    waveform_update::spawn_after(WAVEFORM_UPDATE_PERIOD).unwrap_or_else(|_| {
        // PATTERN: Handle scheduling errors gracefully
        log_error("Failed to schedule waveform update");
    });
}
```

### Deadline Monitoring Pattern

```rust
// PATTERN: Monitor task execution times for deadline violations
#[task(priority = 2)]
fn monitored_task(ctx: monitored_task::Context) {
    let start_time = monotonic::now();
    
    // Perform task work
    do_periodic_work();
    
    let execution_time = monotonic::now() - start_time;
    const MAX_EXECUTION_TIME: fugit::Duration<u32, 1, 1000> = 
        fugit::Duration::<u32, 1, 1000>::from_ticks(5); // 5ms max
    
    if execution_time > MAX_EXECUTION_TIME {
        // PATTERN: Log timing violations for debugging
        log_timing_violation("monitored_task", execution_time);
    }
    
    monitored_task::spawn_after(MONITOR_PERIOD).ok();
}
```

## Error Handling Patterns

### Graceful Degradation Pattern

```rust
// PATTERN: Continue operation even when subsystems fail
#[task(priority = 2, shared = [system_state], local = [adc])]
fn battery_monitor(mut ctx: battery_monitor::Context) {
    match ctx.local.adc.read(&mut adc_pin) {
        Ok(reading) => {
            // Normal operation
            let battery_state = BatteryState::from_adc_reading(reading);
            ctx.shared.system_state.lock(|state| state.battery = battery_state);
        }
        Err(adc_error) => {
            // PATTERN: Graceful degradation - continue with last known state
            log_error("ADC read failed, using last known battery state");
            
            // PATTERN: Attempt recovery on next cycle
            if let Err(_) = attempt_adc_recovery(ctx.local.adc) {
                // PATTERN: Escalate to higher-level error handling
                ctx.shared.system_state.lock(|state| {
                    state.errors.insert(SystemError::AdcFailure);
                });
            }
        }
    }
    
    battery_monitor::spawn_after(BATTERY_MONITOR_INTERVAL).ok();
}
```

### Error Propagation and Recovery Pattern

```rust
// PATTERN: Structured error types for different recovery strategies
#[derive(Debug, Clone, Copy)]
enum SystemError {
    Recoverable(RecoverableError),
    Critical(CriticalError),
}

#[derive(Debug, Clone, Copy)]
enum RecoverableError {
    AdcReadFailure,
    UsbCommunicationTimeout,
    ConfigurationInvalid,
}

#[derive(Debug, Clone, Copy)]  
enum CriticalError {
    ClockFailure,
    MemoryCorruption,
    HardwareFailure,
}

impl SystemError {
    // PATTERN: Error-specific recovery strategies
    fn recovery_strategy(self) -> RecoveryAction {
        match self {
            SystemError::Recoverable(RecoverableError::AdcReadFailure) => 
                RecoveryAction::RetryWithBackoff,
            SystemError::Recoverable(RecoverableError::UsbCommunicationTimeout) => 
                RecoveryAction::ResetPeripheral,
            SystemError::Critical(_) => 
                RecoveryAction::SafeShutdown,
        }
    }
}

// PATTERN: Centralized error handling task
#[task(priority = 2, shared = [system_state], local = [error_queue])]
fn error_handler(mut ctx: error_handler::Context) {
    while let Some(error) = ctx.local.error_queue.dequeue() {
        match error.recovery_strategy() {
            RecoveryAction::RetryWithBackoff => {
                // Implement exponential backoff retry
                schedule_retry(error, calculate_backoff_delay(error));
            }
            RecoveryAction::ResetPeripheral => {
                // Attempt peripheral reset
                if let Err(_) = reset_peripheral(error) {
                    // Escalate to critical error
                    promote_to_critical_error(error);
                }
            }
            RecoveryAction::SafeShutdown => {
                // Initiate safe system shutdown
                initiate_safe_shutdown(error);
            }
        }
    }
}
```

## Performance Optimization Patterns

### Cache-Friendly Data Layout Pattern

```rust
// PATTERN: Group frequently accessed data together
#[repr(C)]
struct WaveformState {
    // Hot data - accessed every cycle
    phase_accumulator: u32,
    current_duty_cycle: u16,
    frequency_increment: u32,
    
    // Warm data - accessed periodically  
    amplitude_scale: f32,
    waveform_type: WaveformType,
    
    // Cold data - accessed rarely
    configuration_checksum: u32,
    last_update_timestamp: u64,
}

// PATTERN: Use const generics for compile-time optimization
struct SampleBuffer<const N: usize> {
    buffer: [f32; N],
    index: usize,
    filled: bool,
}

impl<const N: usize> SampleBuffer<N> {
    // PATTERN: Compile-time optimized circular buffer operations
    fn push(&mut self, sample: f32) {
        self.buffer[self.index] = sample;
        self.index = (self.index + 1) % N;
        if self.index == 0 {
            self.filled = true;
        }
    }
}
```

### Memory Pool Pattern

```rust
// PATTERN: Pre-allocated message pools to avoid allocation
use heapless::pool::{Pool, Node};

// Global message pool
static mut MEMORY: [Node<LogMessage>; 16] = [Node::new(); 16];
static MESSAGE_POOL: Pool<LogMessage> = Pool::new();

// Initialize pool once during system startup
fn initialize_message_pool() {
    unsafe {
        MESSAGE_POOL.grow(&mut MEMORY);
    }
}

// PATTERN: Use pool for message allocation
fn send_log_message(level: LogLevel, message: &str) {
    if let Some(mut node) = MESSAGE_POOL.alloc() {
        *node = LogMessage::new(level, message);
        
        // Send message via queue
        if let Err(returned_node) = try_send_message(node) {
            // PATTERN: Return node to pool if send fails
            MESSAGE_POOL.free(returned_node);
        }
    } else {
        // Pool exhausted - drop message or implement overflow handling
        increment_dropped_message_counter();
    }
}
```

## Subsystem Integration Patterns

### Layered Architecture Pattern

```rust
// PATTERN: Clear separation between hardware abstraction, drivers, and application logic

// Hardware Abstraction Layer (HAL)
mod hal {
    pub trait AdcHal {
        fn read_channel(&mut self, channel: u8) -> Result<u16, AdcError>;
        fn configure_channel(&mut self, channel: u8, config: AdcConfig);
    }
    
    pub trait PwmHal {
        fn set_duty_cycle(&mut self, channel: u8, duty: u16);
        fn set_frequency(&mut self, channel: u8, freq: u32);
    }
}

// Driver Layer
mod drivers {
    use super::hal::*;
    
    pub struct BatteryMonitor<A: AdcHal> {
        adc: A,
        channel: u8,
        calibration: CalibrationData,
    }
    
    impl<A: AdcHal> BatteryMonitor<A> {
        pub fn read_battery_voltage(&mut self) -> Result<f32, BatteryError> {
            let raw_reading = self.adc.read_channel(self.channel)?;
            Ok(self.calibration.raw_to_voltage(raw_reading))
        }
    }
}

// Application Layer
mod application {
    use super::drivers::*;
    
    #[task(priority = 2, local = [battery_monitor])]
    fn battery_task(ctx: battery_task::Context) {
        match ctx.local.battery_monitor.read_battery_voltage() {
            Ok(voltage) => handle_battery_reading(voltage),
            Err(error) => handle_battery_error(error),
        }
    }
}
```

These patterns provide a solid foundation for implementing complex, multi-subsystem embedded projects with RTIC 2.0 while maintaining real-time guarantees, proper resource management, and code maintainability.