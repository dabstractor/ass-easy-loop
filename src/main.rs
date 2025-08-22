#![no_std]
#![no_main]

// Import required crates - exact same imports as working reference
use panic_probe as _;
use rp2040_hal::{
    clocks::{init_clocks_and_plls, Clock},
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
#[rtic::app(device = rp2040_hal::pac, peripherals = true, dispatchers = [TIMER_IRQ_1, TIMER_IRQ_2, TIMER_IRQ_3])]
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
    }

    #[local]
    struct Local {}

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
            },
            Local {},
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
}
