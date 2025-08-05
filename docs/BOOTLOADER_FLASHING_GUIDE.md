# RP2040 Bootloader Mode and Firmware Flashing Guide

This guide provides detailed instructions for entering bootloader mode, flashing firmware, and troubleshooting common issues with the RP2040 pEMF/Battery monitoring device.

## Table of Contents

1. [Understanding RP2040 Boot Modes](#understanding-rp2040-boot-modes)
2. [Entering Bootloader Mode](#entering-bootloader-mode)
3. [Firmware Flashing Methods](#firmware-flashing-methods)
4. [Verification and Testing](#verification-and-testing)
5. [Troubleshooting Common Issues](#troubleshooting-common-issues)
6. [Advanced Flashing Techniques](#advanced-flashing-techniques)

## Understanding RP2040 Boot Modes

The RP2040 microcontroller has two primary operating modes:

### 1. Application Mode (Normal Operation)
- **Purpose**: Runs your custom firmware
- **USB Behavior**: Device appears with custom VID:PID (e.g., 1234:5678)
- **Device Type**: USB HID device (for logging firmware)
- **LED Behavior**: Controlled by your firmware
- **When Active**: After successful firmware flash and reset

### 2. Bootloader Mode (BOOTSEL Mode)
- **Purpose**: Allows firmware flashing via USB mass storage
- **USB Behavior**: Device appears as "RPI-RP2" mass storage device
- **VID:PID**: 2e8a:0003 (Raspberry Pi Foundation)
- **LED Behavior**: Usually solid on or breathing pattern
- **When Active**: When BOOTSEL button is held during power-on/reset

## Entering Bootloader Mode

### Method 1: BOOTSEL Button (Standard Method)

This is the most common and reliable method:

#### Step-by-Step Instructions

1. **Prepare the Device**
   ```bash
   # Ensure device is disconnected from USB
   # Locate the BOOTSEL button on your RP2040 board
   # Have USB cable ready
   ```

2. **Enter Bootloader Sequence**
   ```bash
   # 1. Hold down the BOOTSEL button (small button on RP2040 board)
   # 2. While holding BOOTSEL, connect USB cable to computer
   # 3. Keep holding BOOTSEL for 2-3 seconds after USB connection
   # 4. Release BOOTSEL button
   ```

3. **Verify Bootloader Mode**
   ```bash
   # Check USB enumeration
   lsusb | grep "2e8a:0003"
   # Expected output: Bus XXX Device XXX: ID 2e8a:0003 Raspberry Pi RP2 Boot
   
   # Check for mass storage device
   ls /media/*/RPI-RP2 2>/dev/null || ls /run/media/*/RPI-RP2 2>/dev/null
   # Expected: Directory listing of RPI-RP2 drive
   
   # Alternative check with dmesg
   dmesg | tail -10 | grep -i "rpi-rp2"
   ```

#### Visual Indicators

- **LED**: Usually solid on or breathing pattern (depends on board)
- **USB Sound**: Computer may play USB connection sound
- **File Manager**: "RPI-RP2" drive should appear in file manager

### Method 2: Reset Button + BOOTSEL (If Available)

Some boards have both RESET and BOOTSEL buttons:

```bash
# 1. Hold down BOOTSEL button
# 2. Press and release RESET button while holding BOOTSEL
# 3. Release BOOTSEL button after 2-3 seconds
# 4. Verify bootloader mode as above
```

### Method 3: Software Reset to Bootloader (Advanced)

If your firmware supports it, you can reset to bootloader mode programmatically:

```rust
// In your Rust firmware
use rp2040_hal::rom_data;

// Function to reset to bootloader
fn reset_to_bootloader() -> ! {
    rom_data::reset_to_usb_boot(0, 0);
    // This function never returns
    loop {}
}
```

### Method 4: Power Cycle Method

If BOOTSEL button is difficult to access:

```bash
# 1. Disconnect USB cable
# 2. Hold BOOTSEL button
# 3. Connect power (USB or external)
# 4. Keep holding BOOTSEL for 3-5 seconds
# 5. Release BOOTSEL
# 6. Connect USB cable (if using external power)
```

## Firmware Flashing Methods

### Method 1: Cargo Run (Recommended for Development)

This is the easiest method for Rust development:

#### Prerequisites
```bash
# Ensure development environment is set up
rustc --version
cargo --version
elf2uf2-rs --version

# Navigate to project directory
cd ~/rp2040-projects/ass-easy-loop
```

#### Flashing Process
```bash
# 1. Enter bootloader mode (see above)
# 2. Verify bootloader mode
lsusb | grep "2e8a:0003"

# 3. Build and flash in one command
cargo run --release

# What happens:
# - Compiles Rust code for thumbv6m-none-eabi target
# - Links with memory.x layout and boot2 bootloader
# - Converts ELF binary to UF2 format
# - Automatically finds RPI-RP2 drive
# - Copies UF2 file to device
# - Device automatically resets and starts new firmware
```

#### Expected Output
```bash
$ cargo run --release
   Compiling ass-easy-loop v0.1.0 (/home/user/rp2040-projects/ass-easy-loop)
    Finished release [optimized] target(s) in 12.34s
     Running `elf2uf2-rs -d target/thumbv6m-none-eabi/release/ass-easy-loop`
Found pico uf2 disk /media/user/RPI-RP2
Copying target/thumbv6m-none-eabi/release/ass-easy-loop to /media/user/RPI-RP2/ass-easy-loop.uf2
```

### Method 2: Manual UF2 Flashing

For more control over the flashing process:

#### Build Firmware
```bash
# Clean build (recommended)
cargo clean
cargo build --release

# Verify build succeeded
ls -la target/thumbv6m-none-eabi/release/ass-easy-loop
```

#### Convert to UF2 Format
```bash
# Convert ELF to UF2
elf2uf2-rs target/thumbv6m-none-eabi/release/ass-easy-loop

# Verify UF2 file created
ls -la ass-easy-loop.uf2
file ass-easy-loop.uf2
# Expected: ass-easy-loop.uf2: data
```

#### Copy to Device
```bash
# Find RPI-RP2 mount point
MOUNT_POINT=$(findmnt -n -o TARGET -S LABEL=RPI-RP2)
echo "RPI-RP2 mounted at: $MOUNT_POINT"

# Copy UF2 file
cp ass-easy-loop.uf2 "$MOUNT_POINT/"

# Ensure write completes
sync

# Verify copy
ls -la "$MOUNT_POINT/"
```

#### Alternative Copy Methods
```bash
# Method 1: Direct copy
cp ass-easy-loop.uf2 /media/user/RPI-RP2/

# Method 2: Using dd for reliability
dd if=ass-easy-loop.uf2 of=/media/user/RPI-RP2/firmware.uf2 bs=1M

# Method 3: Using rsync
rsync -av ass-easy-loop.uf2 /media/user/RPI-RP2/
```

### Method 3: Drag-and-Drop (GUI Method)

For users who prefer graphical interface:

1. **Open File Manager**
   - Navigate to project directory
   - Locate `ass-easy-loop.uf2` file

2. **Drag and Drop**
   - Drag UF2 file to RPI-RP2 drive
   - Wait for copy to complete
   - Device will automatically reset

3. **Verify Success**
   - RPI-RP2 drive should disappear
   - Device should start running new firmware

### Method 4: Command Line Tools

#### Using picotool (Official Raspberry Pi Tool)

```bash
# Install picotool (if available)
git clone https://github.com/raspberrypi/picotool.git
cd picotool
mkdir build && cd build
cmake ..
make -j4
sudo make install

# Flash firmware
picotool load ass-easy-loop.uf2
picotool reboot
```

#### Using OpenOCD (Advanced)

```bash
# Install OpenOCD with RP2040 support
sudo pacman -S openocd

# Create OpenOCD configuration
cat > rp2040.cfg << 'EOF'
source [find interface/cmsis-dap.cfg]
source [find target/rp2040.cfg]
adapter speed 5000
EOF

# Flash firmware
openocd -f rp2040.cfg -c "program ass-easy-loop.uf2 verify reset exit"
```

## Verification and Testing

### Verify Firmware Flash Success

#### Check USB Enumeration
```bash
# Device should no longer appear as bootloader
lsusb | grep "2e8a:0003"
# Should return nothing

# Device should appear with custom VID:PID
lsusb | grep "1234:5678"
# Expected: Bus XXX Device XXX: ID 1234:5678 Custom Electronics pEMF Battery Monitor
```

#### Check HID Device Creation
```bash
# List HID raw devices
ls -la /dev/hidraw*

# Check HID device properties
udevadm info --query=all --name=/dev/hidraw0 | grep -E "(ID_VENDOR_ID|ID_MODEL_ID)"
```

#### Test Basic Communication
```bash
# Test HID communication
python3 -c "
import hid
devices = hid.enumerate(0x1234, 0x5678)
if devices:
    print(f'Found {len(devices)} device(s)')
    for d in devices:
        print(f'  Path: {d[\"path\"]}')
        print(f'  Product: {d[\"product_string\"]}')
else:
    print('No devices found')
"
```

#### Monitor Device Logs
```bash
# Start log monitoring
python3 hidlog.py --level INFO

# Expected output:
# Connected to device:
#   Manufacturer: Custom Electronics
#   Product: pEMF Battery Monitor
#   Serial: 123456789
#   VID:PID: 1234:5678
# 
# [000.001s] [INFO ] [SYSTEM ] RP2040 boot started, firmware v1.2.3
# [000.010s] [INFO ] [SYSTEM ] Clock configuration: 125MHz system, 48MHz USB
```

### Verify Hardware Functionality

#### Test pEMF Output
```bash
# Monitor pEMF timing logs
python3 hidlog.py --module PEMF --level INFO

# Expected output:
# [000.789s] [INFO ] [PEMF   ] Pulse generation started, frequency: 2Hz
# [001.289s] [DEBUG] [PEMF   ] Pulse timing: HIGH=2.0ms, LOW=498.0ms
```

#### Test Battery Monitoring
```bash
# Monitor battery status
python3 hidlog.py --module BATTERY --level INFO

# Expected output:
# [000.456s] [INFO ] [BATTERY] ADC initialization complete
# [001.456s] [INFO ] [BATTERY] State: Normal, Voltage: 3.45V, ADC: 1612
```

#### Test LED Control
```bash
# Monitor LED status changes
python3 hidlog.py --module SYSTEM --level INFO

# Change battery state (connect/disconnect charger) and observe:
# [010.123s] [INFO ] [SYSTEM ] LED state changed: OFF -> CHARGING
```

## Troubleshooting Common Issues

### Issue 1: Cannot Enter Bootloader Mode

#### Symptoms
- Device doesn't appear as RPI-RP2
- `lsusb` doesn't show 2e8a:0003
- No mass storage device appears

#### Diagnostic Steps
```bash
# Check USB connection
lsusb
dmesg | tail -10

# Check USB cable
# Try different USB cable (ensure it's a data cable, not power-only)

# Check USB port
# Try different USB port on computer
```

#### Solutions

**Solution 1: Timing Issues**
```bash
# Ensure proper timing:
# 1. Disconnect USB completely
# 2. Hold BOOTSEL button BEFORE connecting USB
# 3. Connect USB while holding BOOTSEL
# 4. Hold BOOTSEL for 3-5 seconds after connection
# 5. Release BOOTSEL
```

**Solution 2: Hardware Issues**
```bash
# Check BOOTSEL button
# - Ensure button is making proper contact
# - Try pressing firmly
# - Check for physical damage

# Check power supply
# - Ensure device is receiving power
# - Check for power LED indicators
# - Try external power source if available
```

**Solution 3: USB Cable/Port Issues**
```bash
# Test USB cable
# - Try known-good USB data cable
# - Avoid USB extension cables or hubs
# - Test cable with other devices

# Test USB port
# - Try different USB ports on computer
# - Avoid USB 3.0 ports if having issues (try USB 2.0)
# - Check USB port power output
```

### Issue 2: Firmware Flash Fails

#### Symptoms
- UF2 file copies but device doesn't reset
- Device remains in bootloader mode after flash
- Flash process hangs or errors

#### Diagnostic Steps
```bash
# Check UF2 file integrity
file ass-easy-loop.uf2
ls -la ass-easy-loop.uf2

# Check mount point permissions
ls -la /media/*/RPI-RP2/
mount | grep RPI-RP2

# Check available space
df -h /media/*/RPI-RP2/
```

#### Solutions

**Solution 1: File System Issues**
```bash
# Unmount and remount
sudo umount /media/*/RPI-RP2
# Disconnect and reconnect USB to remount

# Check file system
sudo fsck.fat /dev/sdX1  # Replace X with appropriate device
```

**Solution 2: Permission Issues**
```bash
# Check mount permissions
mount | grep RPI-RP2

# Remount with proper permissions
sudo umount /media/*/RPI-RP2
sudo mkdir -p /mnt/rpi-rp2
sudo mount -o uid=$USER,gid=$USER /dev/sdX1 /mnt/rpi-rp2
cp ass-easy-loop.uf2 /mnt/rpi-rp2/
sync
sudo umount /mnt/rpi-rp2
```

**Solution 3: Corrupted UF2 File**
```bash
# Rebuild UF2 file
cargo clean
cargo build --release
elf2uf2-rs target/thumbv6m-none-eabi/release/ass-easy-loop

# Verify UF2 file size (should be reasonable, not 0 bytes)
ls -la ass-easy-loop.uf2
```

### Issue 3: Device Doesn't Start After Flash

#### Symptoms
- Flash appears successful but device doesn't enumerate
- No USB device appears after reset
- Device appears to be "dead"

#### Diagnostic Steps
```bash
# Check if device is still in bootloader mode
lsusb | grep "2e8a:0003"

# Check for any USB enumeration
lsusb
dmesg | tail -20

# Check power indicators
# Look for LED activity on device
```

#### Solutions

**Solution 1: Firmware Issues**
```bash
# Try flashing known-good firmware
# Use a minimal test firmware first

# Check build configuration
cargo check
cargo build --release --verbose

# Verify memory layout
cat memory.x
```

**Solution 2: Hardware Reset**
```bash
# Try hardware reset (if RESET button available)
# Press and release RESET button

# Try power cycle
# Disconnect USB, wait 10 seconds, reconnect

# Try entering bootloader mode again
# Flash a minimal test firmware
```

**Solution 3: Recovery Flash**
```bash
# Create minimal recovery firmware
cat > src/main.rs << 'EOF'
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    loop {
        // Minimal firmware that just runs
        cortex_m::asm::nop();
    }
}
EOF

# Build and flash recovery firmware
cargo build --release
elf2uf2-rs target/thumbv6m-none-eabi/release/ass-easy-loop
# Flash recovery firmware, then flash main firmware
```

### Issue 4: Intermittent Flash Failures

#### Symptoms
- Flash works sometimes but not always
- Inconsistent behavior
- Random failures during development

#### Solutions

**Solution 1: Improve Reliability**
```bash
# Add delays in flash process
sleep 2  # After entering bootloader mode
cp ass-easy-loop.uf2 /media/*/RPI-RP2/
sync
sleep 2  # Before checking results

# Use more reliable copy method
dd if=ass-easy-loop.uf2 of=/media/*/RPI-RP2/firmware.uf2 bs=1M conv=fsync
```

**Solution 2: Automate Flash Process**
```bash
# Create reliable flash script
cat > flash.sh << 'EOF'
#!/bin/bash
set -e

echo "Building firmware..."
cargo build --release

echo "Converting to UF2..."
elf2uf2-rs target/thumbv6m-none-eabi/release/ass-easy-loop

echo "Waiting for bootloader mode..."
while ! lsusb | grep -q "2e8a:0003"; do
    echo "Please enter bootloader mode (hold BOOTSEL while connecting USB)"
    sleep 2
done

echo "Device in bootloader mode, flashing..."
MOUNT_POINT=$(findmnt -n -o TARGET -S LABEL=RPI-RP2)
if [ -z "$MOUNT_POINT" ]; then
    echo "ERROR: RPI-RP2 not mounted"
    exit 1
fi

cp ass-easy-loop.uf2 "$MOUNT_POINT/"
sync

echo "Flash complete, waiting for device reset..."
sleep 3

if lsusb | grep -q "1234:5678"; then
    echo "SUCCESS: Device running new firmware"
else
    echo "WARNING: Device not detected, may need manual reset"
fi
EOF

chmod +x flash.sh
```

## Advanced Flashing Techniques

### Using SWD Debug Interface

For development with hardware debugger:

#### Hardware Setup
```bash
# Connect SWD debugger (Picoprobe, J-Link, etc.)
# Connections:
# - SWDIO (GPIO 2 on Picoprobe) -> SWDIO on target
# - SWCLK (GPIO 3 on Picoprobe) -> SWCLK on target  
# - GND -> GND
# - 3V3 -> 3V3 (optional, for power)
```

#### Software Setup
```bash
# Install probe-rs
cargo install probe-rs --features cli

# List available probes
probe-rs list

# Flash via SWD
probe-rs run --chip RP2040 target/thumbv6m-none-eabi/release/ass-easy-loop
```

#### Debugging Session
```bash
# Start GDB server
probe-rs gdb --chip RP2040 target/thumbv6m-none-eabi/release/ass-easy-loop &

# Connect with GDB
arm-none-eabi-gdb target/thumbv6m-none-eabi/release/ass-easy-loop
(gdb) target remote :1337
(gdb) load
(gdb) continue
```

### Automated Testing and Flashing

#### CI/CD Integration
```yaml
# .github/workflows/flash-test.yml
name: Flash Test
on: [push, pull_request]

jobs:
  flash-test:
    runs-on: self-hosted  # Requires hardware
    steps:
    - uses: actions/checkout@v3
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: thumbv6m-none-eabi
    - name: Build firmware
      run: cargo build --release
    - name: Flash and test
      run: |
        # Automated flash and test script
        ./scripts/automated_flash_test.sh
```

#### Automated Flash Script
```bash
#!/bin/bash
# automated_flash_test.sh

set -e

echo "=== Automated Flash and Test ==="

# Build firmware
echo "Building firmware..."
cargo build --release

# Wait for bootloader mode
echo "Waiting for device in bootloader mode..."
timeout 30 bash -c 'while ! lsusb | grep -q "2e8a:0003"; do sleep 1; done'

# Flash firmware
echo "Flashing firmware..."
cargo run --release

# Wait for application mode
echo "Waiting for application mode..."
timeout 30 bash -c 'while ! lsusb | grep -q "1234:5678"; do sleep 1; done'

# Test basic functionality
echo "Testing basic functionality..."
timeout 10 python3 hidlog.py --level INFO > test_output.log &
sleep 5
kill %1 2>/dev/null || true

if [ -s test_output.log ]; then
    echo "SUCCESS: Device responding to HID communication"
    echo "Log messages received: $(wc -l < test_output.log)"
else
    echo "ERROR: No log messages received"
    exit 1
fi

echo "=== Flash and test completed successfully ==="
```

This comprehensive guide should help users successfully enter bootloader mode and flash firmware to their RP2040 devices, with detailed troubleshooting for common issues.