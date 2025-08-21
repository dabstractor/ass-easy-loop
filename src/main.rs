#![no_std]
#![no_main]

// Import required crates - exact same imports as working reference
use panic_probe as _;
use rp2040_hal::{
    clocks::{init_clocks_and_plls, Clock},
    usb::UsbBus,
    watchdog::Watchdog,
};
use usb_device::{bus::UsbBusAllocator, prelude::*, device::StringDescriptors, descriptor::lang_id::LangID};
use usbd_hid::hid_class::HIDClass;
use systick_monotonic::{fugit::Duration, Systick};

// Import our modules
mod config;
mod types;

use crate::config::usb;
use crate::types::usb_commands::CommandReport;

// Bootloader - exact same as working reference
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

// CRITICAL: Static USB bus allocation required for USB device - exact same as working reference
static mut USB_BUS: Option<UsbBusAllocator<UsbBus>> = None;

// RTIC app structure - exact same pattern as working reference
#[rtic::app(device = rp2040_hal::pac, peripherals = true, dispatchers = [TIMER_IRQ_1])]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        usb_dev: UsbDevice<'static, UsbBus>,
        hid_class: HIDClass<'static, UsbBus>,
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
            12_000_000u32,  // Crystal frequency - must be exact
            ctx.device.XOSC,
            ctx.device.CLOCKS,
            ctx.device.PLL_SYS,
            ctx.device.PLL_USB,  // REQUIRED for 48MHz USB clock
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
            clocks.usb_clock,    // 48MHz from PLL_USB
            true,                // Force VBUS detect
            &mut ctx.device.RESETS,
        );
        
        unsafe {
            USB_BUS = Some(UsbBusAllocator::new(usb_bus));
        }
        
        let usb_bus_ref = unsafe { USB_BUS.as_ref().unwrap() };

        // Create HID class device with custom report descriptor - exact same as working reference
        let hid_class = HIDClass::new(usb_bus_ref, CommandReport::descriptor(), 60);

        // Configure USB device descriptors with custom VID/PID and device strings
        let usb_dev = UsbDeviceBuilder::new(usb_bus_ref, UsbVidPid(usb::usb::VENDOR_ID, usb::usb::PRODUCT_ID))
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

        (Shared { usb_dev, hid_class }, Local {}, init::Monotonics(mono))
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

        // Schedule next poll in 10ms - critical for USB enumeration - exact same as working reference
        usb_poll_task::spawn_after(Duration::<u64, 1, 1000>::millis(10)).unwrap();
    }
}