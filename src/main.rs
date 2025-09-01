#![no_std]
#![no_main]

// Import required crates - exact same imports as working reference
use panic_probe as _;
use rp2040_hal::{
    adc::{Adc, AdcPin},
    clocks::{init_clocks_and_plls, Clock},
    gpio::{Pin, bank0::Gpio26, FunctionSioInput, PullNone},
    Sio,
    usb::UsbBus,
    watchdog::Watchdog,
};
use systick_monotonic::{fugit::Duration, Systick};
use usb_device::{
    bus::UsbBusAllocator, descriptor::lang_id::LangID, device::StringDescriptors, prelude::*,
};
use usbd_hid::hid_class::HIDClass;

// Import our modules
mod config;
mod drivers;
mod tasks;
mod types;
mod utils;

use crate::config::usb;
use crate::types::{
    bootloader_types::{BootloaderConfig, BootloaderState},
    logging::LogMessage,
    usb_commands::CommandReport,
    waveform::WaveformConfig,
};

#[cfg(feature = "usb-logs")]
use crate::types::logging::LoggingConfig;
#[cfg(feature = "usb-logs")]
use heapless::spsc::Queue;

// Bootloader - exact same as working reference
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

// CRITICAL: Static USB bus allocation required for USB device - exact same as working reference
static mut USB_BUS: Option<UsbBusAllocator<UsbBus>> = None;

// RTIC app structure - exact same pattern as working reference
#[rtic::app(device = rp2040_hal::pac, peripherals = true, dispatchers = [TIMER_IRQ_0, TIMER_IRQ_1, TIMER_IRQ_2, TIMER_IRQ_3])]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        usb_dev: UsbDevice<'static, UsbBus>,
        hid_class: HIDClass<'static, UsbBus>,
        bootloader_state: BootloaderState,
        log_queue: Queue<LogMessage, 32>,
        logging_config: LoggingConfig,
        waveform_config: WaveformConfig,
        battery_monitor: crate::drivers::adc_battery::BatteryMonitor,
        safety_flags: crate::types::battery::SafetyFlags,
        battery_state: crate::types::battery::BatteryState,
    }

    #[local]
    struct Local {
        battery_pin: AdcPin<Pin<Gpio26, FunctionSioInput, PullNone>>,
    }

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = Systick<1000>;

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        // Set up clocks and PLL with 12MHz external crystal - exact same as working reference
        let mut watchdog = Watchdog::new(ctx.device.WATCHDOG);
        let clocks = init_clocks_and_plls(
            12_000_000u32, // Crystal frequency - must be exact
            ctx.device.XOSC,
            ctx.device.CLOCKS,
            ctx.device.PLL_SYS,
            ctx.device.PLL_USB, // REQUIRED for 48MHz USB clock
            &mut ctx.device.RESETS,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        // Initialize monotonic timer - required for RTIC 1.x
        let mono = Systick::new(ctx.core.SYST, clocks.system_clock.freq().to_Hz());

        // Initialize ADC and GPIO26 for battery monitoring
        let adc = Adc::new(ctx.device.ADC, &mut ctx.device.RESETS);
        let sio = Sio::new(ctx.device.SIO);
        let pins = rp2040_hal::gpio::Pins::new(
            ctx.device.IO_BANK0,
            ctx.device.PADS_BANK0,
            sio.gpio_bank0,
            &mut ctx.device.RESETS,
        );
        let battery_pin = AdcPin::new(pins.gpio26.into_floating_input()).unwrap();

        // Create battery monitor with ADC instance
        let battery_monitor = crate::drivers::adc_battery::BatteryMonitor::new(adc);

        // Set up USB bus allocator and HID class device - exact same as working reference
        let usb_bus = UsbBus::new(
            ctx.device.USBCTRL_REGS,
            ctx.device.USBCTRL_DPRAM,
            clocks.usb_clock, // 48MHz from PLL_USB
            true,             // Force VBUS detect
            &mut ctx.device.RESETS,
        );

        unsafe {
            USB_BUS = Some(UsbBusAllocator::new(usb_bus));
        }

        let usb_bus_ref = unsafe { USB_BUS.as_ref().unwrap() };

        // Create HID class device with custom report descriptor - exact same as working reference
        let hid_class = HIDClass::new(usb_bus_ref, CommandReport::descriptor(), 60);

        // Configure USB device descriptors with custom VID/PID and device strings
        let usb_dev = UsbDeviceBuilder::new(
            usb_bus_ref,
            UsbVidPid(usb::usb::VENDOR_ID, usb::usb::PRODUCT_ID),
        )
        .device_release(usb::usb::DEVICE_RELEASE)
        .device_class(0x00) // Use interface class instead of device class
        .strings(&[StringDescriptors::new(LangID::EN_US)
            .manufacturer(usb::usb::MANUFACTURER)
            .product(usb::usb::PRODUCT)
            .serial_number(usb::usb::SERIAL_NUMBER)])
        .expect("Failed to set USB strings")
        .build();

        // Spawn the USB polling task immediately - exact same as working reference
        usb_poll_task::spawn_after(Duration::<u64, 1, 1000>::millis(10)).unwrap();

        // Spawn the USB command handler task
        usb_command_handler_task::spawn_after(Duration::<u64, 1, 1000>::millis(20)).unwrap();

        // Spawn the logging transmission task if logging is enabled
        #[cfg(feature = "usb-logs")]
        {
            logging_transmit_task::spawn_after(Duration::<u64, 1, 1000>::millis(30)).unwrap();
        }

        // Spawn the battery monitor task
        battery_monitor_task::spawn_after(Duration::<u64, 1, 1000>::millis(100)).unwrap();

        // Initialize logging system only
        #[cfg(feature = "system-logs")]
        {
            use crate::drivers::logging;
            logging::init();
        }

        (
            Shared {
                usb_dev,
                hid_class,
                bootloader_state: BootloaderState::Normal,
                log_queue: Queue::new(),
                logging_config: LoggingConfig {
                    enabled_categories: 0xF, // All categories enabled by default
                    verbosity_level: crate::types::logging::LogLevel::Debug,
                    enabled: true,
                },
                waveform_config: WaveformConfig::default(), // 10Hz sawtooth with 33% duty cycle
                battery_monitor,
                safety_flags: crate::types::battery::SafetyFlags::new(),
                battery_state: crate::types::battery::BatteryState::Normal,
            },
            Local {
                battery_pin,
            },
            init::Monotonics(mono),
        )
    }

    /// USB polling task - CRITICAL for USB enumeration
    ///
    /// This task runs at priority 1 and polls the USB device every 10ms.
    /// Without frequent polling, the device will disappear from USB enumeration.
    /// This is the core functionality that makes the device visible to lsusb.
    #[task(
        shared = [usb_dev, hid_class],
        priority = 1
    )]
    fn usb_poll_task(mut ctx: usb_poll_task::Context) {
        // Lock shared resources for USB operations - exact same as working reference
        ctx.shared.usb_dev.lock(|usb_dev| {
            ctx.shared.hid_class.lock(|hid_class| {
                // CRITICAL: This poll() call maintains USB enumeration
                // Without this, device disappears from lsusb output
                usb_dev.poll(&mut [hid_class])
            })
        });

        // Update timestamp for logging - increment by 10ms each poll
        #[cfg(feature = "usb-logs")]
        {
            use crate::drivers::logging::set_timestamp_ms;
            static mut TIMESTAMP_COUNTER: u32 = 0;
            static mut INIT_LOG_SENT: bool = false;
            unsafe {
                TIMESTAMP_COUNTER += 10; // Increment by 10ms
                set_timestamp_ms(TIMESTAMP_COUNTER);

                // Send init log once after USB is ready
                if !INIT_LOG_SENT && TIMESTAMP_COUNTER > 100 {
                    #[cfg(feature = "system-logs")]
                    {
                        use crate::drivers::logging;
                        logging::log_system_event("System booting"); // todo/fixme:this part doesn't work
                    }
                    INIT_LOG_SENT = true;
                }
            }
        }

        // Schedule next poll in 10ms - critical for USB enumeration - exact same as working reference
        usb_poll_task::spawn_after(Duration::<u64, 1, 1000>::millis(10)).unwrap();
    }

    /// USB command handler task - processes HID reports for bootloader commands
    #[task(
        shared = [hid_class, bootloader_state],
        priority = 1
    )]
    fn usb_command_handler_task(mut ctx: usb_command_handler_task::Context) {
        use crate::drivers::usb_command_handler::parse_hid_report;
        use crate::types::{bootloader_types::BootloaderConfig, usb_commands::UsbCommand};

        let mut buffer = [0u8; 64];

        let command = ctx.shared.hid_class.lock(|hid_class| {
            if let Ok(size) = hid_class.pull_raw_output(&mut buffer) {
                if size == 64 {
                    parse_hid_report(&buffer)
                } else {
                    None
                }
            } else {
                None
            }
        });

        if let Some(cmd) = command {
            match cmd {
                UsbCommand::EnterBootloader => {
                    let can_enter = ctx
                        .shared
                        .bootloader_state
                        .lock(|state| matches!(*state, BootloaderState::Normal));

                    if can_enter {
                        let config = BootloaderConfig::default();
                        bootloader_entry_task::spawn(config).ok();
                    }
                }
                _ => {
                    // Handle other commands in future implementations
                }
            }
        }

        usb_command_handler_task::spawn_after(Duration::<u64, 1, 1000>::millis(10)).unwrap();
    }

    /// Bootloader entry task - handles safe transition to ROM bootloader
    #[task(shared = [bootloader_state], priority = 2)]
    fn bootloader_entry_task(mut ctx: bootloader_entry_task::Context, config: BootloaderConfig) {
        use crate::types::bootloader_types::BootloaderState;

        ctx.shared.bootloader_state.lock(|state| {
            *state = BootloaderState::PrepareEntry;
        });

        // Allow cleanup time before reset
        for _ in 0..(config.prep_delay_ms * 1000) {
            cortex_m::asm::nop();
        }

        ctx.shared.bootloader_state.lock(|state| {
            *state = BootloaderState::EnteringBootloader;
        });

        // Disable interrupts before ROM call
        cortex_m::interrupt::disable();

        // Enter bootloader mode using RP2040 ROM function
        unsafe {
            rp2040_hal::rom_data::reset_to_usb_boot(
                config.activity_pin_mask,
                config.disable_interface_mask,
            );
        }
        // Note: This function does not return - device resets
    }

    /// Logging transmission task - sends log messages via USB HID
    #[cfg(feature = "usb-logs")]
    #[task(
        shared = [hid_class, log_queue, logging_config],
        priority = 3
    )]
    fn logging_transmit_task(mut ctx: logging_transmit_task::Context) {
        use crate::drivers::logging::{dequeue_message, format_log_message};
        use systick_monotonic::fugit::Duration;

        // Check if logging is enabled before processing
        let is_enabled = ctx.shared.logging_config.lock(|config| config.enabled);
        if !is_enabled {
            logging_transmit_task::spawn_after(Duration::<u64, 1, 1000>::millis(100)).unwrap();
            return;
        }

        // Non-blocking queue operations to prevent task blocking
        if let Some(message) = dequeue_message() {
            // Add debug log to see if we're processing messages
            #[cfg(feature = "system-logs")]
            {
                use crate::drivers::logging;
                // Only log if it's not a system log to avoid infinite loop
                if message.category != crate::types::logging::LogCategory::System {
                    logging::log_system_event("Processing queued message");
                }
            }

            let report = format_log_message(&message);

            // Error handling for USB transmission with retry logic
            let mut retry_count = 0;
            loop {
                match ctx
                    .shared
                    .hid_class
                    .lock(|hid_class| hid_class.push_raw_input(&report.data))
                {
                    Ok(_) => {
                        // Add debug log to see if transmission succeeded
                        #[cfg(feature = "system-logs")]
                        {
                            use crate::drivers::logging;
                            if message.category != crate::types::logging::LogCategory::System {
                                logging::log_system_event("Log transmitted");
                            }
                        }
                        break; // Success
                    }
                    Err(_) if retry_count < 3 => {
                        retry_count += 1;
                        // Exponential backoff: 10ms, 20ms, 40ms
                        cortex_m::asm::delay(10000 * (1 << retry_count));
                    }
                    Err(_) => {
                        // Log transmission failed after 3 retries
                        // Continue with next message to prevent blocking
                        #[cfg(feature = "system-logs")]
                        {
                            use crate::drivers::logging;
                            if message.category != crate::types::logging::LogCategory::System {
                                logging::log_system_event("Log transmission failed");
                            }
                        }
                        break;
                    }
                }
            }
        }

        // Reschedule task for periodic checking
        logging_transmit_task::spawn_after(
            systick_monotonic::fugit::Duration::<u64, 1, 1000>::millis(50),
        )
        .unwrap();
    }

    /// Battery monitoring RTIC task - runs every 100ms (10Hz) at Priority 4
    /// 
    /// This task safely reads battery voltage via ADC, processes the reading through
    /// the BatteryMonitor driver, handles safety violations, and logs battery state
    /// changes when the usb-logs feature is enabled.
    /// 
    /// CRITICAL: Uses Priority 4 to avoid conflicts with USB tasks (Priority 1)
    /// and logging tasks (Priority 3). Never change this priority.
    #[task(
        local = [battery_pin],
        shared = [battery_monitor, safety_flags, log_queue, logging_config, battery_state],
        priority = 4
    )]
    fn battery_monitor_task(mut ctx: battery_monitor_task::Context) {
        use crate::types::errors::BatteryError;
        
        // Process battery sample and handle all responses within proper locking
        let (battery_reading, requires_emergency_action) = (ctx.shared.battery_monitor, ctx.shared.safety_flags, ctx.shared.battery_state).lock(|monitor, flags, state| {
            // Process the battery sample
            match monitor.process_sample(ctx.local.battery_pin, flags) {
                Ok(reading) => {
                    // Update battery state and check for changes
                    let state_changed = if *state != reading.state {
                        let old_state = *state;
                        *state = reading.state;
                        Some((old_state, reading.state))
                    } else {
                        None
                    };
                    
                    (Ok((reading, state_changed)), false)
                },
                Err(error) => {
                    // Determine if emergency response is needed
                    let needs_emergency = match &error {
                        BatteryError::OverVoltage { .. } => true,
                        BatteryError::UnderVoltage { .. } => true,
                        BatteryError::OverCurrent { .. } => true,
                        BatteryError::OverTemperature { .. } => true,
                        BatteryError::SafetyTimeout { .. } => true,
                        BatteryError::AdcFailed => false,
                        _ => false,
                    };
                    
                    if needs_emergency {
                        // Set emergency stop immediately while we have the lock
                        flags.set_emergency_stop(true);
                        *state = crate::types::battery::BatteryState::Fault;
                    }
                    
                    (Err(error), needs_emergency)
                }
            }
        });
        
        // Handle results and logging outside of the main lock
        match battery_reading {
            Ok((reading, state_changed)) => {
                // Log battery information if usb-logs feature is enabled
                #[cfg(feature = "usb-logs")]
                {
                    use crate::types::logging::{LogMessage, LogCategory, LogLevel};
                    use heapless::spsc::Queue;
                    
                    let should_log = ctx.shared.logging_config.lock(|config| {
                        config.enabled && config.enabled_categories & (1 << LogCategory::Battery as u8) != 0
                    });
                    
                    if should_log {
                        let log_message = if state_changed.is_some() {
                            // Log state transition
                            let mut content = [0u8; 52];
                            let msg_bytes = b"Battery state changed";
                            let len = core::cmp::min(msg_bytes.len(), 52);
                            content[..len].copy_from_slice(&msg_bytes[..len]);
                            
                            LogMessage {
                                timestamp_ms: reading.timestamp_ms,
                                category: LogCategory::Battery,
                                level: LogLevel::Info,
                                content,
                                content_len: len as u8,
                            }
                        } else {
                            // Log periodic reading (at lower verbosity)
                            let mut content = [0u8; 52];
                            let msg_bytes = b"Battery reading";
                            let len = core::cmp::min(msg_bytes.len(), 52);
                            content[..len].copy_from_slice(&msg_bytes[..len]);
                            
                            LogMessage {
                                timestamp_ms: reading.timestamp_ms,
                                category: LogCategory::Battery,
                                level: LogLevel::Debug,
                                content,
                                content_len: len as u8,
                            }
                        };
                        
                        ctx.shared.log_queue.lock(|queue: &mut Queue<LogMessage, 32>| {
                            let _ = queue.enqueue(log_message);
                        });
                    }
                }
            },
            Err(_error) => {
                // Log the error if logging is available
                #[cfg(feature = "usb-logs")]
                {
                    use crate::types::logging::{LogMessage, LogCategory, LogLevel};
                    use heapless::spsc::Queue;
                    
                    let should_log = ctx.shared.logging_config.lock(|config| {
                        config.enabled
                    });
                    
                    if should_log {
                        let mut content = [0u8; 52];
                        let msg_bytes = b"Battery error detected";
                        let len = core::cmp::min(msg_bytes.len(), 52);
                        content[..len].copy_from_slice(&msg_bytes[..len]);
                        
                        let log_message = LogMessage {
                            timestamp_ms: 0, // Would need actual timestamp from monotonic timer
                            category: LogCategory::Battery,
                            level: if requires_emergency_action { LogLevel::Error } else { LogLevel::Warn },
                            content,
                            content_len: len as u8,
                        };
                        
                        ctx.shared.log_queue.lock(|queue: &mut Queue<LogMessage, 32>| {
                            let _ = queue.enqueue(log_message);
                        });
                    }
                }
            }
        }
        
        // Reschedule task for next reading in 100ms (10Hz operation)
        // This maintains the required 10Hz battery monitoring frequency
        battery_monitor_task::spawn_after(Duration::<u64, 1, 1000>::millis(100)).unwrap();
    }
}
