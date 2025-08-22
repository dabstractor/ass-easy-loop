name: "Automatic Bootloader Entry PRP - Implementation-Focused"
description: |

---

## Goal

**Feature Goal**: Implement automatic bootloader entry functionality that enables fully automated testing workflows by allowing host systems to remotely trigger the RP2040 device to enter bootloader mode without manual BOOTSEL button intervention.

**Deliverable**: USB HID command interface that triggers automatic RP2040 reset into bootloader mode, with host-side Python tools for automation.

**Success Definition**: Host system can send a USB command to trigger device bootloader mode, device automatically resets into BOOTSEL mode, and firmware flashing succeeds without manual intervention.

## User Persona

**Target User**: Firmware developers and test automation engineers

**Use Case**: Automated testing workflows requiring repeated firmware flashing cycles without physical device access

**User Journey**:
1. Developer runs automated test script
2. Script sends bootloader entry command via USB
3. Device automatically resets into bootloader mode
4. Script flashes new firmware via UF2 file upload
5. Testing continues with new firmware

**Pain Points Addressed**: Eliminates manual BOOTSEL button pressing, enables remote testing setups, and allows unattended test automation.

## Why

- **Business value**: Enables continuous integration testing for embedded firmware
- **Integration**: Leverages existing USB HID infrastructure and flash toolchain  
- **Problems solved**: Manual bootloader entry blocking automation workflows for firmware developers

## What

USB command interface that accepts bootloader entry requests and safely resets the RP2040 into ROM bootloader mode using the hardware watchdog and ROM reset function.

### Success Criteria

- [ ] USB HID command parsing recognizes EnterBootloader command
- [ ] Device safely preserves state before reset
- [ ] Device automatically enters bootloader mode (BOOTSEL) after reset
- [ ] Host tools can reliably trigger bootloader entry
- [ ] Integration with existing flash.sh script works seamlessly
- [ ] Device recovers properly after firmware flashing

## All Needed Context

### Context Completeness Check

_"If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"_ ✅ Yes - all patterns, infrastructure, and API details provided below.

### Documentation & References

```yaml
# MUST READ - Include these in your context window
- url: https://docs.rs/rp2040-hal/latest/rp2040_hal/rom_data/fn.reset_to_usb_boot.html
  why: Exact API for RP2040 ROM bootloader entry function
  critical: Takes gpio_activity_pin_mask and disable_interface_mask parameters

- url: https://docs.rs/rp2040-hal/latest/rp2040_hal/rom_data/index.html
  why: Complete ROM function reference and usage patterns
  critical: All ROM functions are unsafe and require careful usage

- file: /home/dustin/projects/asseasyloop/src/main.rs
  why: RTIC application structure with USB infrastructure
  pattern: USB device setup with UsbBusAllocator, shared resources, and polling tasks
  gotcha: USB_BUS static allocation required, usb_poll_task must run for enumeration

- file: /home/dustin/projects/asseasyloop/src/types/usb_commands.rs
  why: UsbCommand::EnterBootloader variant already defined
  pattern: Command enum with Copy, Debug, PartialEq traits
  gotcha: Command parsing from HID reports needs to be implemented

- file: /home/dustin/projects/asseasyloop/src/config/usb.rs
  why: USB device descriptor constants (VID/PID/strings)
  pattern: Module with public constants for USB device configuration
  gotcha: Keep consistent with existing USB enumeration

- file: /home/dustin/projects/asseasyloop/flash.sh
  why: Existing bootloader detection and UF2 flashing logic
  pattern: Mount point detection for BOOTSEL mode, UF2 file handling
  gotcha: Already handles bootloader mode detection - no changes needed
```

### Current Codebase Tree

```bash
/home/dustin/projects/asseasyloop/
├── src/
│   ├── config/             # USB config constants
│   │   ├── usb.rs         # VID/PID/device descriptors
│   │   └── defaults.rs    # Configuration defaults
│   ├── drivers/           # Hardware drivers
│   │   └── usb_hid.rs     # USB HID placeholder (needs implementation)
│   ├── tasks/             # RTIC tasks
│   │   ├── usb_handler.rs # USB handler placeholder (needs implementation)
│   │   └── mod.rs         # Task module organization
│   ├── types/             # Data structures
│   │   ├── usb_commands.rs # UsbCommand::EnterBootloader already defined
│   │   └── errors.rs      # System error types
│   └── main.rs            # RTIC app with USB infrastructure
├── host_tools/            # Python host communication tools
│   ├── device_control.py  # Device communication placeholder
│   └── requirements.txt   # Python dependencies
├── flash.sh               # Bootloader-aware flashing script
├── memory.x               # RP2040 memory layout
└── Cargo.toml             # Dependencies with rp2040-hal
```

### Desired Codebase Tree with Files to be Added

```bash
/home/dustin/projects/asseasyloop/
├── src/
│   ├── drivers/
│   │   └── usb_command_handler.rs  # Parse HID reports to UsbCommand enum
│   ├── tasks/
│   │   ├── usb_command_task.rs     # Process EnterBootloader commands
│   │   └── bootloader_task.rs      # Manage bootloader entry sequence
│   └── types/
│       └── bootloader_types.rs     # Bootloader-specific data structures
└── host_tools/
    └── bootloader_entry.py         # Host tool to trigger bootloader mode
```

### Known Gotchas of our Codebase & Library Quirks

```rust
// CRITICAL: rp2040_hal::rom_data functions are unsafe
unsafe { rp2040_hal::rom_data::reset_to_usb_boot(0, 0); }

// CRITICAL: USB polling task MUST run for enumeration - defined in main.rs
#[task(shared = [usb_dev, hid_class], priority = 1)]
fn usb_poll_task(mut ctx: usb_poll_task::Context) {
    // This poll() call maintains USB enumeration
    ctx.shared.usb_dev.lock(|usb_dev| {
        ctx.shared.hid_class.lock(|hid_class| {
            usb_dev.poll(&mut [hid_class])
        })
    });
}

// CRITICAL: RTIC shared resources require lock() for access
// Example: UsbDevice and HIDClass are shared between tasks

// CRITICAL: USB_BUS static allocation required in main.rs
static mut USB_BUS: Option<UsbBusAllocator<UsbBus>> = None;

// GOTCHA: RP2040 watchdog reset preserves some SRAM - may affect state
// GOTCHA: CommandReport has 64-byte HID report descriptor - follow existing pattern
```

## Implementation Blueprint

### Data models and structure

Create the core data models for bootloader entry with proper error handling and state management.

```rust
// Bootloader entry state tracking
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootloaderState {
    Normal,
    PrepareEntry,
    EnteringBootloader,
}

// Bootloader entry configuration
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BootloaderConfig {
    pub activity_pin_mask: u32,      // GPIO activity light (0 = none)
    pub disable_interface_mask: u32, // USB interface control (0 = both enabled)
    pub prep_delay_ms: u32,          // Delay before reset (for cleanup)
}

// Bootloader entry result for host feedback
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootloaderResult {
    Success,
    PrepareError,
    InvalidState,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/types/bootloader_types.rs
  - IMPLEMENT: BootloaderState, BootloaderConfig, BootloaderResult enums/structs
  - FOLLOW pattern: src/types/errors.rs (enum structure with Copy, Debug traits)
  - NAMING: CamelCase for enum variants, snake_case for struct fields
  - PLACEMENT: Domain-specific types in src/types/

Task 2: MODIFY src/types/errors.rs
  - ADD: BootloaderError variant to SystemError enum
  - FOLLOW pattern: Existing error variants (simple enum members)
  - NAMING: BootloaderError following existing naming
  - PRESERVE: All existing error variants unchanged

Task 3: CREATE src/drivers/usb_command_handler.rs
  - IMPLEMENT: parse_hid_report() function to extract UsbCommand from 64-byte reports
  - FOLLOW pattern: src/drivers/usb_hid.rs (driver module structure)
  - NAMING: snake_case functions, descriptive names like parse_hid_report
  - DEPENDENCIES: Import UsbCommand from src/types/usb_commands.rs
  - PLACEMENT: USB-specific driver in src/drivers/

Task 4: CREATE src/tasks/bootloader_task.rs
  - IMPLEMENT: bootloader_entry_task RTIC task with BootloaderState management
  - FOLLOW pattern: src/tasks/usb_handler.rs (RTIC task structure with context)
  - NAMING: bootloader_entry_task following RTIC task naming convention
  - DEPENDENCIES: Use BootloaderConfig, bootloader entry ROM function
  - PLACEMENT: Bootloader logic in src/tasks/

Task 5: MODIFY src/tasks/usb_command_task.rs (CREATE if not exists)
  - IMPLEMENT: Process UsbCommand::EnterBootloader via message passing to bootloader_task
  - FOLLOW pattern: Existing RTIC task communication patterns in main.rs
  - NAMING: usb_command_handler_task following RTIC conventions
  - DEPENDENCIES: Import usb_command_handler driver, bootloader_task spawning
  - INTEGRATION: Connect to existing USB HID infrastructure

Task 6: MODIFY src/main.rs
  - ADD: Bootloader task registration and shared resource setup
  - FIND pattern: Existing task definitions and shared resource structure
  - ADD: BootloaderState to Shared struct, task spawn calls
  - PRESERVE: All existing USB infrastructure and task scheduling
  - INTEGRATION: Connect USB command parsing to bootloader entry

Task 7: CREATE host_tools/bootloader_entry.py
  - IMPLEMENT: Python script using hidapi to send EnterBootloader command
  - FOLLOW pattern: host_tools/device_control.py structure
  - NAMING: descriptive function names like trigger_bootloader_entry()
  - DEPENDENCIES: hidapi, device detection by VID/PID from config/usb.rs
  - PLACEMENT: Host communication tools in host_tools/

Task 8: MODIFY host_tools/requirements.txt
  - ADD: hidapi dependency for USB HID communication
  - FOLLOW pattern: Existing requirement format
  - PRESERVE: All existing dependencies
```

### Implementation Patterns & Key Details

```rust
// USB HID Report Parsing Pattern
pub fn parse_hid_report(report: &[u8; 64]) -> Option<UsbCommand> {
    // PATTERN: First byte indicates command type
    match report[0] {
        0x01 => Some(UsbCommand::SetFrequency(u32::from_le_bytes([
            report[1], report[2], report[3], report[4]
        ]))),
        0x02 => Some(UsbCommand::SetDutyCycle(report[1])),
        0x03 => Some(UsbCommand::EnterBootloader), // Bootloader command
        _ => None,
    }
}

// RTIC Bootloader Task Pattern
#[task(shared = [bootloader_state], priority = 2)]
fn bootloader_entry_task(mut ctx: bootloader_entry_task::Context, config: BootloaderConfig) {
    ctx.shared.bootloader_state.lock(|state| {
        *state = BootloaderState::PrepareEntry;
    });
    
    // CRITICAL: Allow cleanup time before reset
    Timer::delay_ms(config.prep_delay_ms);
    
    // CRITICAL: Disable interrupts before ROM call
    cortex_m::interrupt::disable();
    
    // PATTERN: Use RP2040 ROM bootloader entry function
    unsafe {
        rp2040_hal::rom_data::reset_to_usb_boot(
            config.activity_pin_mask,
            config.disable_interface_mask
        );
    }
    // Note: This function does not return - device resets
}

// Python Host Tool Pattern  
def trigger_bootloader_entry(vendor_id=0xfade, product_id=0x1212):
    """Send EnterBootloader command via USB HID"""
    import hid
    
    # PATTERN: Device detection by VID/PID
    device = hid.device()
    device.open(vendor_id, product_id)
    
    # PATTERN: 64-byte HID report with command in first byte
    report = [0x03] + [0x00] * 63  # 0x03 = EnterBootloader command
    device.send_feature_report(report)
    device.close()
```

### Integration Points

```yaml
USB_INFRASTRUCTURE:
  - modify: src/main.rs shared resources
  - pattern: "Add BootloaderState to Shared struct following existing pattern"
  - preserve: "All existing USB device and HID class setup"

TASK_SCHEDULING:
  - add to: src/main.rs RTIC task definitions
  - pattern: "Follow existing task priority and context patterns"
  - integration: "Connect usb_command_task to bootloader_entry_task via message passing"

ERROR_HANDLING:
  - extend: src/types/errors.rs SystemError enum
  - pattern: "Add BootloaderError variant following existing error structure"
  - usage: "Return errors from bootloader preparation failures"

HOST_TOOLS:
  - extend: host_tools/ Python scripts
  - pattern: "Follow device_control.py structure for USB communication"
  - integration: "Use VID/PID constants from src/config/usb.rs"
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation - fix before proceeding
cargo check --target thumbv6m-none-eabi  # ARM Cortex-M0+ target check
cargo clippy --target thumbv6m-none-eabi # Lint checking
cargo fmt                                # Code formatting

# Expected: Zero errors. If errors exist, fix before proceeding.
```

### Level 2: Build Validation (Component Validation)

```bash
# Build firmware with bootloader functionality
cargo build --release --target thumbv6m-none-eabi

# Check binary size (should remain reasonable)
arm-none-eabi-size target/thumbv6m-none-eabi/release/firmware

# Validate host tools
cd host_tools && python -m py_compile bootloader_entry.py

# Expected: Clean build, reasonable binary size, no Python syntax errors.
```

### Level 3: Hardware Integration Testing (System Validation)

```bash
# Flash firmware with bootloader functionality
cargo run  # Uses flash.sh script

# Test USB enumeration (device should appear)
lsusb | grep "fada:1212"  # Check VID:PID from config/usb.rs

# Test bootloader entry command
cd host_tools && python bootloader_entry.py

# Verify bootloader mode entry (device should appear as RPI-RP2)
sleep 2 && lsusb | grep "2e8a:0003"  # Bootloader mode VID:PID

# Verify flash script detects bootloader mode
./flash.sh 2>&1 | grep "BOOTSEL mode detected"

# Expected: Device enumerates, bootloader command works, device enters BOOTSEL mode
```

### Level 4: Automation Testing

```bash
# Full automation test cycle
cd host_tools

# Test complete workflow
python << EOF
import bootloader_entry
import time
import subprocess

# Trigger bootloader entry
print("Triggering bootloader entry...")
bootloader_entry.trigger_bootloader_entry()

# Wait for bootloader mode
time.sleep(3)

# Test that device is in bootloader mode
result = subprocess.run(['lsusb'], capture_output=True, text=True)
if '2e8a:0003' in result.stdout:
    print("✅ Device successfully entered bootloader mode")
else:
    print("❌ Device failed to enter bootloader mode")

# Test flash script
flash_result = subprocess.run(['../flash.sh'], capture_output=True, text=True)
if 'Successfully flashed' in flash_result.stdout:
    print("✅ Flash script works with bootloader mode")
else:
    print("❌ Flash script failed")
EOF

# Expected: Complete automation cycle works end-to-end
```

## Final Validation Checklist

### Technical Validation

- [ ] Foremost, you *MUST* ensure that `cargo run` fully flashes the firmware to the device. No exceptions. You must run it successfully 3 times in a row
without any help or intervention before proceeding.
- [ ] No linting errors: `cargo clippy --target thumbv6m-none-eabi`
- [ ] Proper formatting: `cargo fmt --check`
- [ ] Python tools validate: `python -m py_compile host_tools/bootloader_entry.py`

### Feature Validation

- [ ] USB device enumerates with correct VID/PID (0xfade:0x1212)
- [ ] Host tool successfully sends bootloader entry command
- [ ] Device enters bootloader mode (appears as 2e8a:0003)
- [ ] Existing flash.sh script detects bootloader mode correctly
- [ ] Device recovers normally after firmware flashing

### Code Quality Validation

- [ ] Follows RTIC task patterns from existing main.rs
- [ ] Uses existing USB infrastructure without breaking enumeration
- [ ] Error handling follows src/types/errors.rs patterns
- [ ] Module organization matches existing src/ structure
- [ ] Host tools follow host_tools/ Python patterns

### Integration Validation

- [ ] No breaking changes to existing USB functionality
- [ ] Bootloader entry preserves device state appropriately
- [ ] Flash script integration works seamlessly
- [ ] Host automation can reliably trigger bootloader mode
- [ ] Complete firmware update cycle works end-to-end

---

## Anti-Patterns to Avoid

- ❌ Don't break existing USB enumeration - keep usb_poll_task running
- ❌ Don't forget cortex_m::interrupt::disable() before ROM reset call
- ❌ Don't skip the unsafe block for ROM function calls
- ❌ Don't modify USB device descriptors unnecessarily
- ❌ Don't ignore timing considerations for cleanup before reset
- ❌ Don't hardcode delays - use configurable values
- ❌ Don't bypass existing error handling patterns
