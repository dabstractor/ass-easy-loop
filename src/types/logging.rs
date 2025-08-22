use usbd_hid::descriptor::SerializedDescriptor;

/// Log severity levels
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

/// Log message categories
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LogCategory {
    Battery = 0,
    Pemf = 1,
    System = 2,
    Usb = 3,
}

/// Log message structure
#[derive(Clone, Copy, Debug)]
pub struct LogMessage {
    pub timestamp_ms: u32,
    pub level: LogLevel,
    pub category: LogCategory,
    pub content: [u8; 52], // 64 - 12 bytes for header
    pub content_len: u8,
}

/// Configuration structure for runtime control
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoggingConfig {
    pub enabled_categories: u8, // Bitmask for enabled categories
    pub verbosity_level: LogLevel,
    pub enabled: bool,
}

/// HID Report structure for log messages
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LogReport {
    pub data: [u8; 64], // 64-byte reports for HID compatibility
}

impl LogReport {
    pub fn new() -> Self {
        Self { data: [0u8; 64] }
    }
}

/// HID Report Descriptor for logging
impl SerializedDescriptor for LogReport {
    fn desc() -> &'static [u8] {
        &[
            // This is a minimal HID descriptor that will allow the device to enumerate
            0x06, 0x00, 0xFF, // Usage Page (Vendor Defined 0xFF00)
            0x09, 0x02, // Usage (Vendor Usage 0x02 - Logging)
            0xA1, 0x01, // Collection (Application)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xFF, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8 bits)
            0x95, 0x40, // Report Count (64 bytes)
            0x09, 0x02, // Usage (Vendor Usage 0x02 - Logging)
            0x81, 0x02, // Input (Data, Variable, Absolute)
            0x09, 0x02, // Usage (Vendor Usage 0x02 - Logging)
            0x91, 0x02, // Output (Data, Variable, Absolute)
            0xC0, // End Collection
        ]
    }
}

impl LogReport {
    /// Get the HID report descriptor
    pub fn descriptor() -> &'static [u8] {
        Self::desc()
    }
}

/// USB commands for logging control
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoggingUsbCommand {
    SetLogLevel(LogLevel),
    EnableCategory(LogCategory),
    DisableCategory(LogCategory),
    SetConfig(LoggingConfig),
    GetConfig,
}
