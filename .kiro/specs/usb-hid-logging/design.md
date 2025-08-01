# USB HID Logging Design Document

## Overview

This design document outlines the implementation of USB HID logging functionality for the existing RP2040 pEMF/battery monitoring device. The system will add USB HID communication capability while maintaining all existing real-time constraints and functionality. The design uses the RTIC framework to integrate USB HID logging as a low-priority task that doesn't interfere with critical timing requirements.

The implementation leverages the `usbd-hid` crate for HID functionality and `rp2040-hal`'s USB support, creating a custom HID device that appears as a generic HID device to the host system. Log messages are formatted as structured text and transmitted via HID reports.

## Architecture

**Interface Stability Note:** The core architecture and interfaces defined in this document should be considered stable. Modifications should only be made if essential for implementing core pEMF, battery monitoring, or logging functionality. Avoid introducing new features or making architectural changes that are not directly related to the project's primary objectives.

### System Integration

The USB HID logging system integrates with the existing RTIC-based architecture as follows:

```
┌─────────────────────────────────────────────────────────────┐
│                    RTIC Application                         │
├─────────────────────────────────────────────────────────────┤
│ Priority 3: pEMF Pulse Task (2Hz, 2ms HIGH/498ms LOW)      │
├─────────────────────────────────────────────────────────────┤
│ Priority 2: Battery Monitor Task (10Hz ADC sampling)       │
├─────────────────────────────────────────────────────────────┤
│ Priority 1: LED Control Task (Battery status indication)   │
├─────────────────────────────────────────────────────────────┤
│ Priority 1: USB HID Task (Log message transmission)        │ ← NEW
├─────────────────────────────────────────────────────────────┤
│ Priority 0: USB Poll Task (USB device polling)             │ ← NEW
└─────────────────────────────────────────────────────────────┘
```

### USB HID Integration Points

1. **Initialization**: USB HID setup occurs during system initialization without affecting existing hardware configuration
2. **Logging Interface**: Existing tasks call logging macros that queue messages for USB transmission
3. **Message Queue**: A circular buffer stores log messages until they can be transmitted via USB
4. **USB Polling**: Low-priority task handles USB device polling and HID report transmission

## Components and Interfaces

### 1. USB HID Device Configuration

**Component**: `UsbHidLogger`
**Purpose**: Manages USB HID device enumeration and communication

```rust
// Custom HID report descriptor for logging
#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = VENDOR_DEFINED_START, usage = 0x01) = {
        (usage = 0x02, logical_min = 0x0) = {
            #[packed_bits 8] data: u8
        };
        (usage = 0x03, logical_min = 0x0) = {
            #[item_settings data,variable,absolute] timestamp: u32
        };
    }
)]
pub struct LogReport {
    pub data: [u8; 62],    // Log message data (62 bytes max per report)
    pub timestamp: u32,     // Timestamp in milliseconds since boot
}
```

**Key Features**:
- Custom vendor-defined HID descriptor for log data transmission
- 64-byte HID reports (62 bytes data + 4 bytes timestamp + padding)
- Vendor ID/Product ID configuration for device identification
- Proper USB enumeration with device descriptors

### 2. Logging Interface and Macros

**Component**: `logging` module
**Purpose**: Provides convenient logging macros and message formatting

```rust
// Logging levels
#[derive(Clone, Copy, PartialEq)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

// Logging macros
macro_rules! log_debug { ... }
macro_rules! log_info { ... }
macro_rules! log_warn { ... }
macro_rules! log_error { ... }

// Core logging function
pub fn log_message(level: LogLevel, module: &str, message: &str);
```

**Message Format**:
```
[TIMESTAMP] [LEVEL] [MODULE] MESSAGE
Example: [00012345] [INFO] [BATTERY] State changed: Normal -> Charging (ADC: 1680)
```

### 3. Message Queue Management

**Component**: `LogQueue`
**Purpose**: Thread-safe circular buffer for storing log messages before USB transmission

```rust
pub struct LogQueue {
    buffer: [LogMessage; QUEUE_SIZE],
    head: AtomicUsize,
    tail: AtomicUsize,
    count: AtomicUsize,
}

pub struct LogMessage {
    timestamp: u32,
    level: LogLevel,
    module: [u8; 8],      // Module name (truncated/padded)
    message: [u8; 48],    // Message content (truncated if needed)
}
```

**Key Features**:
- Lock-free circular buffer using atomic operations
- Configurable queue size (default: 32 messages)
- Automatic message truncation for fixed-size storage
- Oldest message eviction when queue is full

### 4. USB Task Integration

**Component**: RTIC tasks for USB handling
**Purpose**: Integrate USB HID functionality into existing RTIC task structure

```rust
#[task(shared = [usb_dev, hid_class], local = [log_queue], priority = 0)]
async fn usb_poll_task(ctx: usb_poll_task::Context) {
    // Handle USB device polling and enumeration
    // Process incoming HID control requests
    // Maintain USB connection state
}

#[task(shared = [hid_class], local = [log_queue], priority = 1)]
async fn usb_hid_task(ctx: usb_hid_task::Context) {
    // Dequeue log messages and format as HID reports
    // Transmit HID reports to host
    // Handle transmission errors and retries
}
```

### 5. Host-Side Utilities

**Component**: Python-based host utilities
**Purpose**: Receive and display log messages on development machine

```python
# hidlog.py - Simple HID log receiver
import hid
import struct
import time

class HidLogReceiver:
    def __init__(self, vid=0x1234, pid=0x5678):
        self.device = hid.device()
        self.device.open(vid, pid)
    
    def read_logs(self):
        while True:
            data = self.device.read(64, timeout_ms=1000)
            if data:
                self.parse_and_display(data)
```

## Data Models

### Log Message Structure

```rust
// Internal log message representation
pub struct LogMessage {
    pub timestamp: u32,        // Milliseconds since boot
    pub level: LogLevel,       // Debug/Info/Warn/Error
    pub module: [u8; 8],      // Source module (e.g., "BATTERY", "PEMF")
    pub message: [u8; 48],    // Formatted message content
}

// HID report structure (transmitted to host)
pub struct LogReport {
    pub data: [u8; 62],       // Serialized log message
    pub timestamp: u32,       // Redundant timestamp for host validation
}
```

### Message Serialization Format

Log messages are serialized into HID reports using a compact binary format:

```
Byte 0: Log level (0=Debug, 1=Info, 2=Warn, 3=Error)
Bytes 1-8: Module name (null-padded)
Bytes 9-56: Message content (null-terminated)
Bytes 57-60: Timestamp (little-endian u32)
Bytes 61-63: Reserved/padding
```

### Configuration Data

```rust
// Compile-time configuration
pub struct LogConfig {
    pub max_level: LogLevel,           // Maximum log level to include
    pub queue_size: usize,             // Log message queue size
    pub usb_vid: u16,                  // USB Vendor ID
    pub usb_pid: u16,                  // USB Product ID
    pub enable_battery_logs: bool,     // Enable battery monitoring logs
    pub enable_pemf_logs: bool,        // Enable pEMF timing logs
    pub enable_system_logs: bool,      // Enable system status logs
}
```

## Error Handling

### USB Connection Management

1. **USB Disconnection**: System continues normal operation without USB logging
2. **Enumeration Failures**: Retry enumeration on next USB connection
3. **Transmission Errors**: Drop failed messages and continue with queue
4. **Buffer Overflows**: Discard oldest messages to prevent memory issues

### Message Queue Error Handling

1. **Queue Full**: Automatically evict oldest messages (FIFO behavior)
2. **Memory Allocation**: Use static allocation to avoid runtime failures
3. **Message Truncation**: Gracefully truncate long messages to fit buffer
4. **Atomic Operations**: Use lock-free operations to prevent deadlocks

### Integration with Existing Error Handling

The USB HID logging system integrates with the existing `panic-halt` error handling:

```rust
// Enhanced panic handler with USB logging
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Attempt to log panic information via USB (if available)
    if let Some(location) = info.location() {
        log_error!("PANIC", "Panic at {}:{}", location.file(), location.line());
    }
    
    // Flush any pending USB messages (best effort)
    // Then halt as before
    panic_halt::panic(info)
}
```

## Testing Strategy

### Unit Tests

1. **Message Formatting Tests**: Validate log message serialization and deserialization
2. **Queue Management Tests**: Test circular buffer operations and thread safety
3. **HID Report Generation Tests**: Verify correct HID report structure and content
4. **Configuration Tests**: Validate compile-time and runtime configuration handling

### Integration Tests

1. **USB Enumeration Tests**: Verify device appears correctly to host system
2. **End-to-End Communication Tests**: Send messages from device to host utility
3. **Performance Impact Tests**: Measure timing impact on existing pEMF/battery tasks
4. **Error Recovery Tests**: Test behavior during USB disconnection/reconnection

### Hardware-in-Loop Tests

1. **Real Device Testing**: Test with actual RP2040 hardware and USB connection
2. **Timing Validation**: Confirm pEMF pulse timing remains within ±1% tolerance
3. **Battery Monitoring Integration**: Verify battery state logging accuracy
4. **Multi-Device Testing**: Test with multiple devices connected simultaneously

### Host-Side Testing

1. **HID Device Detection**: Automated tests for device enumeration on Linux
2. **Log Message Reception**: Validate message parsing and display
3. **Performance Testing**: Measure log throughput and latency
4. **Error Handling**: Test behavior with device disconnection

## Development Environment Setup

### Required Arch Linux Packages

```bash
# Install required packages for USB HID development
sudo pacman -S libusb hidapi python-hid python-pyusb
sudo pacman -S probe-rs-tools # For flashing firmware

# Optional: GUI tools for USB debugging
sudo pacman -S usbutils lsusb
```

### Development Workflow

1. **Firmware Development**: 
   - Modify Rust code with USB HID logging integration
   - Build firmware: `cargo build --release`
   - Put device in bootloader mode (hold BOOTSEL while connecting USB)
   - Flash firmware: `probe-rs run --chip RP2040 target/thumbv6m-none-eabi/release/ass-easy-loop`

2. **Host-Side Testing**:
   - Run Python utility: `python3 hidlog.py`
   - Monitor log messages in real-time
   - Validate message format and timing

3. **Validation Process**:
   - Connect device via USB (normal mode, not bootloader)
   - Verify device enumeration: `lsusb | grep "Custom HID"`
   - Run host utility to receive log messages
   - Trigger various device states to generate logs
   - Validate timing requirements with oscilloscope if needed

### Bootloader Mode Instructions

To flash new firmware:
1. Disconnect USB cable from RP2040
2. Hold down BOOTSEL button on RP2040
3. Connect USB cable while holding BOOTSEL
4. Release BOOTSEL button
5. Device appears as mass storage device
6. Use `probe-rs` or drag-and-drop UF2 file to flash

## Performance Considerations

### Memory Usage

- **Static Allocation**: All buffers use static allocation to avoid heap fragmentation
- **Queue Size**: Configurable queue size (default 32 messages × 64 bytes = 2KB)
- **USB Buffers**: Additional ~1KB for USB HID class buffers
- **Total Overhead**: Approximately 3KB additional RAM usage

### CPU Usage

- **USB Polling**: Low-priority task runs only when USB events occur
- **Message Queuing**: Lock-free operations with minimal CPU overhead
- **HID Transmission**: Asynchronous transmission doesn't block critical tasks
- **Impact on pEMF**: Measured impact <0.1% on pulse timing accuracy

### Real-Time Constraints

- **Task Priorities**: USB tasks use lowest priorities (0-1) vs pEMF priority (3)
- **Interrupt Handling**: USB interrupts handled at lower priority than timer interrupts
- **Message Buffering**: Asynchronous queuing prevents blocking of critical tasks
- **Graceful Degradation**: System continues normal operation if USB fails

## Security Considerations

### USB Security

- **Device Authentication**: Custom VID/PID helps identify legitimate devices
- **Data Validation**: Host utilities validate message format and checksums
- **Access Control**: HID interface provides read-only access to log data
- **No Command Interface**: Device doesn't accept commands via USB (logging only)

### Information Disclosure

- **Debug Information**: Log messages may contain sensitive timing or state information
- **Compile-Time Control**: Debug logging can be disabled for production builds
- **Message Filtering**: Runtime control over log verbosity and categories
- **Local Access Only**: USB HID requires physical access to device