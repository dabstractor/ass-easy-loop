/// USB Device Configuration
///
/// Configuration constants for USB HID device enumeration and communication.
/// Based on working reference implementation with updated VID/PID for AsEasyLoop device.
pub mod usb {
    /// USB Vendor ID - Using a custom VID that's not in the database
    pub const VENDOR_ID: u16 = 0xfade;

    /// USB Product ID - Custom PID for AsEasyLoop waveform generator
    pub const PRODUCT_ID: u16 = 0x1212;

    /// USB device manufacturer string descriptor
    pub const MANUFACTURER: &str = "dabstractor";

    /// USB device product string descriptor  
    pub const PRODUCT: &str = "Ass-Easy Loop";

    /// USB device serial number string descriptor
    pub const SERIAL_NUMBER: &str = "001";

    /// USB device release number in BCD format (version 1.0)
    pub const DEVICE_RELEASE: u16 = 0x0100;
}
