name: "USB Device Enumeration Fix - Critical Infrastructure Implementation"
description: |

---

## Goal

**Feature Goal**: Fix catastrophic USB device enumeration failure where the device does not appear in `lsusb` after firmware flashing, despite showing correctly in bootloader mode as "Bus 001 Device 046: ID 2e8a:0003 Raspberry Pi RP2 Boot".

**Deliverable**: Complete USB device enumeration infrastructure that makes the device visible to the host system and ready for HID communication.

**Success Definition**: After flashing the firmware, the device appears in `lsusb` with a custom VID/PID and can be detected by host-side USB tools.

## User Persona

**Target User**: Embedded firmware developer working with RP2040 microcontroller

**Use Case**: Developer flashes firmware to RP2040 device and expects it to enumerate as a USB HID device for communication with host applications

**User Journey**: 
1. Put device into bootloader mode (manually)
2. Flash firmware via UF2 or other method
3. Device resets and runs firmware
4. Device should appear in `lsusb` output
5. Host applications can communicate with device

**Pain Points Addressed**: 
- Device completely invisible to host after firmware flash
- No USB enumeration occurring at all
- Project scaffolding based on incorrect assumptions about USB setup

## Why

- **Business Value**: Without USB enumeration, the device is completely non-functional for its intended purpose
- **Integration with Existing Features**: USB is the primary communication channel for all device functionality (HID logging, commands, waveform control)
- **Problems This Solves**: Complete communication failure between host and device, making the product unusable

## What

The current firmware has a critical architectural flaw: the main.rs contains only a no-op loop with no USB initialization whatsoever. This is why the device disappears from USB enumeration after bootloader exit.

### Success Criteria

- [ ] Device appears in `lsusb` with custom VID/PID after firmware flash
- [ ] USB HID interface is available for host communication
- [ ] Device maintains stable USB connection without dropouts
- [ ] Proper USB descriptor configuration with device identification
- [ ] USB polling task handles enumeration and maintains connection

## All Needed Context

### Context Completeness Check

_This PRP provides everything needed to implement USB enumeration from scratch, including working examples from the reference project, specific RP2040 requirements, and production-ready patterns._

### Documentation & References

```yaml
# MUST READ - Include these in your context window
- url: https://docs.rs/usb-device/latest/usb_device/
  why: Core USB device implementation patterns for embedded Rust
  critical: UsbDeviceBuilder configuration and polling requirements

- url: https://docs.rs/usbd-hid/latest/usbd_hid/
  why: USB HID class implementation for report-based communication
  critical: HID descriptor setup and report handling patterns

- url: https://docs.rs/rp2040-hal/latest/rp2040_hal/usb/index.html
  why: RP2040-specific USB peripheral configuration and clock requirements
  critical: UsbBus initialization with correct PLL configuration

- file: ~/projects/ass-easy-loop/src/main.rs
  why: Working reference implementation with proper USB initialization
  pattern: Lines 362-386 show complete USB bus setup and device configuration
  gotcha: Requires static USB_BUS allocation and specific clock setup

- file: ~/projects/ass-easy-loop/src/config.rs
  why: USB configuration constants (VID, PID, descriptors)
  pattern: Lines 12-37 show proper USB descriptor configuration
  gotcha: VID/PID must be unique and not conflict with existing devices

- docfile: PRPs/ai_docs/rtic2-patterns.md
  why: RTIC task patterns for USB polling and interrupt handling
  section: USB interrupt task configuration and shared resource management
```

### Current Codebase Tree

```bash
src/
├── main.rs                 # BROKEN: Only no-op loop, no USB init
├── drivers/
│   ├── mod.rs
│   └── usb_hid.rs         # EMPTY: Just placeholder comment
├── tasks/
│   ├── mod.rs
│   └── usb_handler.rs     # STUB: Empty function with no implementation
├── types/
│   ├── mod.rs
│   └── usb_commands.rs    # EXISTS: UsbCommand enum defined
└── config/
    ├── mod.rs
    └── defaults.rs        # NO USB CONFIG: Missing VID/PID constants
```

### Desired Codebase Tree with Files to be Added and Responsibility of File

```bash
src/
├── main.rs                 # RTIC app with USB initialization and tasks
├── drivers/
│   ├── mod.rs
│   └── usb_hid.rs         # Complete USB HID driver implementation
├── tasks/
│   ├── mod.rs
│   └── usb_handler.rs     # RTIC task for USB polling and enumeration
├── types/
│   ├── mod.rs
│   └── usb_commands.rs    # Extended with HID report structures
└── config/
    ├── mod.rs
    ├── defaults.rs
    └── usb.rs             # NEW: USB configuration constants (VID/PID/descriptors)
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: RP2040 requires exact 48MHz USB clock from PLL_USB
// Must use init_clocks_and_plls with correct crystal frequency

// CRITICAL: USB bus must be allocated statically due to lifetime requirements
static mut USB_BUS: Option<UsbBusAllocator<UsbBus>> = None;

// CRITICAL: RTIC requires specific priority levels for USB interrupt handling
// USB polling task should be priority 1, higher than application tasks

// CRITICAL: RP2040-E5 errata requires GPIO initialization before USB on some revisions
#[cfg(feature = "rp2040-e5")]
{
    // GPIO setup required for USB reset detection
}

// GOTCHA: usb-device crate requires frequent polling to maintain enumeration
// Missing or infrequent polls cause device to disappear from host

// GOTCHA: USB descriptors must be properly formed or enumeration fails
// VID/PID combination must not conflict with existing system devices
```

## Implementation Blueprint

### Data Models and Structure

Create the core USB configuration and HID report structures for type safety and consistency.

```rust
// USB configuration constants and descriptors
pub mod usb {
    pub const VENDOR_ID: u16 = 0x1209;      // OpenMoko open source VID
    pub const PRODUCT_ID: u16 = 0x0001;     // Custom PID for this device
    pub const DEVICE_RELEASE: u16 = 0x0100; // Version 1.0
    pub const MANUFACTURER: &str = "AsEasyLoop";
    pub const PRODUCT: &str = "Waveform Generator";
    pub const SERIAL_NUMBER: &str = "001";
    pub const HID_REPORT_SIZE: usize = 64;
}

// HID report structures for communication
#[derive(Clone, Copy, Debug)]
pub struct CommandReport {
    pub command_type: u8,
    pub command_id: u8,
    pub payload: [u8; 62],
}

#[derive(Clone, Copy, Debug)]
pub struct ResponseReport {
    pub response_type: u8,
    pub status: u8,
    pub data: [u8; 62],
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/config/usb.rs
  - IMPLEMENT: USB configuration constants (VID, PID, descriptors)
  - FOLLOW pattern: ~/projects/ass-easy-loop/src/config.rs lines 12-37
  - NAMING: usb module with UPPER_CASE constants
  - PLACEMENT: src/config/usb.rs with pub mod usb declaration
  - CRITICAL: Use unique VID/PID combination that won't conflict

Task 2: IMPLEMENT src/drivers/usb_hid.rs  
  - IMPLEMENT: Complete USB HID driver with proper initialization
  - FOLLOW pattern: ~/projects/ass-easy-loop USB setup in main.rs lines 362-386
  - DEPENDENCIES: Import usb configuration from Task 1
  - CRITICAL: Implement static USB bus allocation pattern
  - NAMING: UsbHid struct with new(), poll(), and send_report() methods

Task 3: MODIFY src/main.rs - Complete Rewrite Required
  - IMPLEMENT: RTIC application with USB initialization and tasks
  - FOLLOW pattern: ~/projects/ass-easy-loop main.rs structure and USB setup
  - REPLACE: Current no-op main function with proper RTIC app
  - DEPENDENCIES: Use USB driver from Task 2, config from Task 1
  - CRITICAL: Proper clock initialization with PLL_USB configuration
  - CRITICAL: Static allocation pattern for USB bus and device

Task 4: IMPLEMENT src/tasks/usb_handler.rs
  - IMPLEMENT: RTIC task for USB polling and enumeration handling  
  - FOLLOW pattern: ~/projects/ass-easy-loop usb_poll_task lines 1164-1200
  - DEPENDENCIES: Use USB driver from Task 2
  - NAMING: usb_poll_task with proper RTIC task attributes
  - PRIORITY: Set as priority 1 for consistent polling
  - CRITICAL: Must poll frequently enough to maintain enumeration

Task 5: EXTEND src/types/usb_commands.rs
  - IMPLEMENT: HID report structures for communication
  - FOLLOW pattern: CommandReport and ResponseReport from working example
  - DEPENDENCIES: Use report size from USB config (Task 1)
  - NAMING: CommandReport, ResponseReport with proper serialization
  - CRITICAL: Reports must fit within 64-byte HID report size limit

Task 6: UPDATE Cargo.toml dependencies if needed
  - VERIFY: Required USB crates are present (usb-device, usbd-hid)
  - FOLLOW pattern: Working example Cargo.toml for version compatibility
  - DEPENDENCIES: All previous tasks completed
  - CRITICAL: Ensure compatible crate versions for stable operation
```

### Implementation Patterns & Key Details

```rust
// CRITICAL: Static USB bus allocation pattern (from working example)
static mut USB_BUS: Option<UsbBusAllocator<UsbBus>> = None;

// USB initialization pattern in RTIC init function
#[init]
fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
    // Clock setup - CRITICAL for USB functionality
    let clocks = init_clocks_and_plls(
        12_000_000u32,  // Crystal frequency - must be exact
        ctx.device.XOSC,
        ctx.device.CLOCKS,
        ctx.device.PLL_SYS,
        ctx.device.PLL_USB,  // REQUIRED for 48MHz USB clock
        &mut ctx.device.RESETS,
        &mut watchdog,
    ).ok().unwrap();

    // USB bus setup pattern
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

    let usb_bus_ref = unsafe { 
        (*core::ptr::addr_of!(USB_BUS)).as_ref().unwrap() 
    };

    // USB device configuration
    let usb_dev = UsbDeviceBuilder::new(usb_bus_ref, UsbVidPid(usb_config::VENDOR_ID, usb_config::PRODUCT_ID))
        .device_release(usb_config::DEVICE_RELEASE)
        .manufacturer(usb_config::MANUFACTURER)
        .product(usb_config::PRODUCT)  
        .serial_number(usb_config::SERIAL_NUMBER)
        .device_class(0x00) // Use interface class instead
        .build();

    // HID class setup
    let hid_class = HIDClass::new(usb_bus_ref, CommandReport::descriptor(), 60);

    (Shared { usb_dev, hid_class }, Local {}, init::Monotonics())
}

// CRITICAL: USB polling task pattern
#[task(shared = [usb_dev, hid_class], priority = 1)]
async fn usb_poll_task(mut ctx: usb_poll_task::Context) {
    loop {
        ctx.shared.usb_dev.lock(|usb_dev| {
            ctx.shared.hid_class.lock(|hid_class| {
                // This poll() call is what makes enumeration work
                usb_dev.poll(&mut [hid_class]);
            });
        });
        
        // Poll every 10ms to maintain enumeration
        Timer::after(Duration::from_millis(10)).await;
    }
}
```

### Integration Points

```yaml
HARDWARE:
  - peripheral: "USBCTRL_REGS and USBCTRL_DPRAM must be properly initialized"
  - clocks: "PLL_USB must generate exactly 48MHz for USB clock"
  - reset: "USB peripheral must be released from reset state"

CONFIG:
  - add to: src/config/mod.rs
  - pattern: "pub mod usb;" to expose USB configuration module

MAIN_APP:
  - replace: src/main.rs entirely with RTIC application structure  
  - pattern: Follow ~/projects/ass-easy-loop main.rs RTIC app pattern
  - critical: Cannot be incremental - requires complete rewrite

DEPENDENCIES:
  - verify: Cargo.toml has correct usb-device and usbd-hid versions
  - ensure: rp2040-hal version compatible with USB requirements
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation - fix before proceeding
cargo check --target thumbv6m-none-eabi    # Basic compilation check
cargo clippy --target thumbv6m-none-eabi   # Linting and best practices

# Expected: Zero errors. USB-specific compilation issues often indicate:
# - Missing static allocations
# - Incorrect USB bus lifetime management  
# - Missing required dependencies
```

### Level 2: Build Validation (Component Validation)

```bash
# Full firmware build - must succeed before flashing
cargo build --release --target thumbv6m-none-eabi

# UF2 generation for flashing (if configured)
elf2uf2-rs target/thumbv6m-none-eabi/release/ass-easy-loop

# Expected: Clean build with USB symbols properly linked
# Check for USB-related symbols in output
nm target/thumbv6m-none-eabi/release/ass-easy-loop | grep -i usb
```

### Level 3: Hardware Validation (Device Testing)

```bash
# CRITICAL: Manual bootloader entry required for each test
# User must put device into bootloader mode before flashing

# Flash firmware to device
# Method 1: UF2 copy (if device in bootloader mode)
cp target/thumbv6m-none-eabi/release/ass-easy-loop.uf2 /path/to/rpi-rp2/

# Method 2: Using flash script
./flash.sh

# PRIMARY SUCCESS TEST: Check USB enumeration
lsusb | grep -E "(1209|ass-easy|waveform)"

# Expected output similar to:
# Bus 001 Device 047: ID 1209:0001 AsEasyLoop Waveform Generator

# Additional validation commands
dmesg | tail -10  # Check for USB device recognition messages
lsusb -v -d 1209:0001 | head -20  # Detailed device information

# Host-side HID validation (if device appears)
ls /dev/hidraw* # Should show new HID device
python3 -c "
import hid
devices = hid.enumerate(0x1209, 0x0001)  
print(f'Found {len(devices)} matching devices')
for d in devices: print(d)
"

# Expected: Device appears in all tests, no USB errors in dmesg
```

### Level 4: Functional Validation

```bash
# Test USB HID communication (if host tools available)
cd host_tools/
python3 device_control.py --enumerate

# Test device responsiveness
python3 device_control.py --ping

# Test basic HID report sending
python3 device_control.py --send-command SET_FREQUENCY 1000

# Advanced: USB protocol analyzer validation
# If available, use USB capture tools to verify enumeration sequence:
# - Device descriptor requests
# - Configuration descriptor requests  
# - HID descriptor parsing
# - Proper endpoint setup

# Expected: All host communication succeeds, device responds to commands
```

## Final Validation Checklist

### Technical Validation

- [ ] Clean build: `cargo build --release --target thumbv6m-none-eabi`
- [ ] No linting errors: `cargo clippy --target thumbv6m-none-eabi`
- [ ] Firmware flashes without errors
- [ ] Device appears in `lsusb` after flash and reset
- [ ] No USB errors in `dmesg` output
- [ ] Host can open HID device connection

### Feature Validation

- [ ] Device enumerates with correct VID/PID (1209:0001)
- [ ] Device descriptor shows correct manufacturer/product strings
- [ ] HID interface is available and accessible
- [ ] Device maintains stable USB connection (no disappearing/reappearing)
- [ ] Manual reset/reconnect preserves USB functionality

### Code Quality Validation  

- [ ] USB initialization follows working example patterns exactly
- [ ] RTIC task priorities set correctly for USB polling
- [ ] Static memory allocation used properly for USB bus
- [ ] Error handling implemented for USB operations  
- [ ] USB configuration constants properly defined and used

### Documentation & Deployment

- [ ] USB VID/PID documented for future reference
- [ ] Flashing procedure tested and documented
- [ ] Host-side detection confirmed on target development machine
- [ ] Device behavior documented (enumeration timing, etc.)

---

## Anti-Patterns to Avoid

- ❌ Don't skip USB polling - device will disappear from enumeration
- ❌ Don't use incorrect clock frequency - USB timing will fail
- ❌ Don't ignore static allocation requirements - lifetime errors will occur
- ❌ Don't use conflicting VID/PID - may interfere with existing devices  
- ❌ Don't skip RTIC task priorities - USB polling may be starved
- ❌ Don't hardcode values that should be in config module
- ❌ Don't panic on USB errors - implement graceful error handling