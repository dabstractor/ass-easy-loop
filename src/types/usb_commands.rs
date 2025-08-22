use usbd_hid::descriptor::SerializedDescriptor;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UsbCommand {
    SetFrequency(u32),
    SetDutyCycle(u8),
    EnterBootloader,
}

/// Minimal HID Report structure for USB enumeration
///
/// 64-byte report format for basic device visibility
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CommandReport {
    pub data: [u8; 64],
}

impl CommandReport {
    pub fn new() -> Self {
        Self { data: [0u8; 64] }
    }
}

/// HID Report Descriptor for basic enumeration
impl SerializedDescriptor for CommandReport {
    fn desc() -> &'static [u8] {
        &[
            // This is a minimal HID descriptor that will allow the device to enumerate
            0x06, 0x00, 0xFF, // Usage Page (Vendor Defined 0xFF00)
            0x09, 0x01, // Usage (Vendor Usage 0x01)
            0xA1, 0x01, // Collection (Application)
            0x15, 0x00, // Logical Minimum (0)
            0x26, 0xFF, 0x00, // Logical Maximum (255)
            0x75, 0x08, // Report Size (8 bits)
            0x95, 0x40, // Report Count (64 bytes)
            0x09, 0x01, // Usage (Vendor Usage 0x01)
            0x81, 0x02, // Input (Data, Variable, Absolute)
            0x09, 0x01, // Usage (Vendor Usage 0x01)
            0x91, 0x02, // Output (Data, Variable, Absolute)
            0xC0, // End Collection
        ]
    }
}

impl CommandReport {
    /// Get the HID report descriptor
    pub fn descriptor() -> &'static [u8] {
        Self::desc()
    }
}
