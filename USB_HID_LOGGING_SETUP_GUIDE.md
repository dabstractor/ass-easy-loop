# USB HID Logging Setup and Usage Guide

This comprehensive guide covers setting up, using, and troubleshooting the USB HID logging functionality for the RP2040 pEMF/Battery monitoring device.

## Table of Contents

1. [Overview](#overview)
2. [Arch Linux Development Environment Setup](#arch-linux-development-environment-setup)
3. [Firmware Flashing Process](#firmware-flashing-process)
4. [Usage Examples](#usage-examples)
5. [Troubleshooting Guide](#troubleshooting-guide)
6. [Advanced Configuration](#advanced-configuration)

## Overview

The USB HID logging system provides real-time debugging and monitoring capabilities for the RP2040 device. It allows developers to:

- Monitor battery voltage readings and state changes
- Track pEMF pulse generation timing and accuracy
- Debug system initialization and error conditions
- Analyze performance metrics and resource usage

The system consists of:
- **Device-side**: USB HID logging firmware running on RP2040
- **Host-side**: Python utilities for receiving and displaying log messages
- **Development tools**: Build system and debugging utilities

## Arch Linux Development Environment Setup

### Prerequisites

- Arch Linux (current version)
- USB port for device connection
- Internet connection for package downloads
- 4GB RAM minimum, 8GB recommended
- 2GB free disk space

### Step 1: Install Base Development Tools

```bash
# Update system packages
sudo pacman -Syu

# Install essential development tools
sudo pacman -S base-devel git curl wget pkg-config

# Install USB and HID development libraries
sudo pacman -S libusb hidapi usbutils

# Install Python development environment
sudo pacman -S python python-pip python-setuptools python-hid
```

### Step 2: Install Rust Embedded Toolchain

```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Add ARM Cortex-M target for RP2040
rustup target add thumbv6m-none-eabi

# Install embedded development tools
cargo install elf2uf2-rs probe-rs --features cli

# Verify installation
rustc --version
cargo --version
probe-rs --version
```

### Step 3: Configure USB Device Permissions

```bash
# Add user to required groups for USB access
sudo usermod -a -G uucp,lock,dialout $USER

# Create udev rules for RP2040 device access
sudo tee /etc/udev/rules.d/99-rp2040-hid.rules << 'EOF'
# RP2040 USB HID Logging Device
SUBSYSTEM=="usb", ATTRS{idVendor}=="1234", ATTRS{idProduct}=="5678", MODE="0666", GROUP="dialout"
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="1234", ATTRS{idProduct}=="5678", MODE="0666", GROUP="dialout"

# RP2040 Bootloader (for firmware flashing)
SUBSYSTEM=="usb", ATTRS{idVendor}=="2e8a", ATTRS{idProduct}=="0003", MODE="0666", GROUP="dialout"
EOF

# Reload udev rules and trigger device detection
sudo udevadm control --reload-rules
sudo udevadm trigger

# Note: Log out and back in for group membership changes to take effect
echo "Please log out and back in for USB permissions to take effect"
```

### Step 4: Install Python Dependencies

```bash
# Create project directory and virtual environment
mkdir -p ~/rp2040-development
cd ~/rp2040-development
python -m venv venv
source venv/bin/activate

# Install required Python packages
pip install --upgrade pip wheel setuptools
pip install hidapi hid pyusb pyserial
pip install pytest pytest-cov pytest-timeout
pip install numpy scipy matplotlib pandas

# Save requirements for reproducibility
pip freeze > requirements.txt
```

### Step 5: Install Additional Development Tools

```bash
# Install hardware testing and analysis tools
sudo pacman -S minicom screen picocom

# Install logic analyzer software (optional)
yay -S pulseview sigrok-cli  # Requires AUR helper

# Install cross-compilation tools for advanced debugging
sudo pacman -S arm-none-eabi-gcc arm-none-eabi-gdb arm-none-eabi-newlib
sudo pacman -S gdb-multiarch openocd
```

### Step 6: Verify Installation

```bash
# Check Rust environment
echo "=== Rust Environment ==="
rustc --version
cargo --version
rustup target list --installed | grep thumbv6m-none-eabi

# Check USB tools
echo "=== USB Tools ==="
lsusb --version
python -c "import hid; print('HID library available')"

# Check system permissions
echo "=== System Permissions ==="
groups $USER | grep -E "(dialout|uucp)"
ls -la /etc/udev/rules.d/99-rp2040-hid.rules

echo "Development environment setup complete!"
```

## Firmware Flashing Process

### Understanding RP2040 Boot Modes

The RP2040 has two main operating modes:

1. **Bootloader Mode (BOOTSEL)**:
   - Used for flashing new firmware
   - Device appears as "RPI-RP2" USB mass storage device
   - VID:PID = 2e8a:0003

2. **Application Mode**:
   - Normal operation with your firmware
   - Device appears as USB HID device (with USB logging firmware)
   - VID:PID = 1234:5678 (configurable)

### Method 1: UF2 Bootloader Flashing (Recommended)

This is the easiest method for most users:

#### Step 1: Enter Bootloader Mode

1. **Disconnect** the USB cable from the RP2040
2. **Hold down** the BOOTSEL button on the RP2040 board
3. **Connect** the USB cable while holding BOOTSEL
4. **Release** the BOOTSEL button
5. **Verify** the device appears as "RPI-RP2" drive:
   ```bash
   lsusb | grep "2e8a:0003"  # Should show RP2040 bootloader
   ls /media/*/RPI-RP2 || ls /run/media/*/RPI-RP2  # Should show mounted drive
   ```

#### Step 2: Build and Flash Firmware

```bash
# Navigate to project directory
cd ~/rp2040-projects/ass-easy-loop

# Clean build (recommended for first build)
cargo clean

# Build firmware in release mode
cargo build --release

# Flash using cargo run (automatic UF2 conversion and copying)
cargo run --release
```

**What happens during `cargo run --release`:**
1. Builds the project for thumbv6m-none-eabi target
2. Converts ELF binary to UF2 format using elf2uf2-rs
3. Automatically detects RPI-RP2 drive
4. Copies UF2 file to the device
5. Device automatically resets and starts running new firmware

#### Step 3: Verify Firmware Installation

```bash
# Device should now appear as HID device (not bootloader)
lsusb | grep "1234:5678"  # Should show your custom VID:PID

# Check HID device enumeration
ls /dev/hidraw* | head -5  # Should show hidraw devices

# Test basic HID communication
python -c "
import hid
devices = hid.enumerate(0x1234, 0x5678)
if devices:
    print(f'Found {len(devices)} HID device(s)')
    for d in devices:
        print(f'  Path: {d[\"path\"]}')
else:
    print('No HID devices found')
"
```

### Method 2: Manual UF2 Flashing

If automatic flashing doesn't work:

```bash
# Build firmware
cargo build --release

# Convert ELF to UF2 manually
elf2uf2-rs target/thumbv6m-none-eabi/release/ass-easy-loop

# Find the RPI-RP2 mount point
MOUNT_POINT=$(findmnt -n -o TARGET -S LABEL=RPI-RP2)
echo "RPI-RP2 mounted at: $MOUNT_POINT"

# Copy UF2 file to device
cp ass-easy-loop.uf2 "$MOUNT_POINT/"

# Verify copy completed
sync
echo "Firmware flashed successfully"
```

### Method 3: SWD Debugging Interface (Advanced)

For development with hardware debugger:

```bash
# Connect SWD debugger (Picoprobe, J-Link, etc.)
# Wire connections: SWDIO, SWCLK, GND, optionally RESET

# List available debug probes
probe-rs list

# Flash firmware via SWD
probe-rs run --chip RP2040 target/thumbv6m-none-eabi/release/ass-easy-loop

# Start debugging session
probe-rs gdb --chip RP2040 target/thumbv6m-none-eabi/release/ass-easy-loop
```

### Troubleshooting Firmware Flashing

#### Issue: Device Not Entering Bootloader Mode

**Symptoms:**
- No "RPI-RP2" drive appears
- `lsusb` doesn't show 2e8a:0003

**Solutions:**
1. **Check BOOTSEL button timing:**
   ```bash
   # Ensure you hold BOOTSEL BEFORE connecting USB
   # Keep holding until USB is fully connected
   # Only then release BOOTSEL
   ```

2. **Try different USB cable:**
   - Some cables are power-only (no data)
   - Use a known-good USB data cable

3. **Check USB port:**
   - Try different USB ports
   - Avoid USB hubs if possible

4. **Manual reset sequence:**
   ```bash
   # If device is already connected:
   # 1. Hold BOOTSEL
   # 2. Press and release RESET (if available)
   # 3. Release BOOTSEL
   ```

#### Issue: Build Failures

**Symptoms:**
- Cargo build errors
- Missing dependencies
- Linker errors

**Solutions:**
1. **Update Rust toolchain:**
   ```bash
   rustup update
   rustup target add thumbv6m-none-eabi
   ```

2. **Clean and rebuild:**
   ```bash
   cargo clean
   rm -rf target/
   cargo build --release
   ```

3. **Check dependencies:**
   ```bash
   # Verify Cargo.toml has correct dependencies
   cargo check
   ```

#### Issue: UF2 Flashing Fails

**Symptoms:**
- UF2 file doesn't copy
- Device doesn't reset after copying
- Permission denied errors

**Solutions:**
1. **Check mount permissions:**
   ```bash
   # Find mount point
   findmnt -n -o TARGET -S LABEL=RPI-RP2
   
   # Check permissions
   ls -la /path/to/RPI-RP2/
   
   # If needed, remount with proper permissions
   sudo umount /path/to/RPI-RP2
   sudo mount -o uid=$USER,gid=$USER /dev/sdX /path/to/RPI-RP2
   ```

2. **Manual sync and unmount:**
   ```bash
   # After copying UF2 file
   sync
   sudo umount /path/to/RPI-RP2
   ```

3. **Try different file copy method:**
   ```bash
   # Instead of cp, try dd
   dd if=ass-easy-loop.uf2 of=/path/to/RPI-RP2/firmware.uf2 bs=1M
   sync
   ```

## Usage Examples

### Basic Log Monitoring

#### Example 1: Monitor All Log Messages

```bash
# Activate Python environment
source ~/rp2040-development/venv/bin/activate

# Start basic log monitoring
python3 hidlog.py

# Expected output:
# Connected to device:
#   Manufacturer: Custom Electronics  
#   Product: pEMF Battery Monitor
#   Serial: 123456789
#   VID:PID: 1234:5678
# 
# Monitoring logs (min level: DEBUG, module filter: none)
# Press Ctrl+C to stop
# --------------------------------------------------------------------------------
# [000.123s] [INFO ] [SYSTEM ] System boot complete, initializing peripherals
# [000.456s] [DEBUG] [BATTERY] ADC reading: 1650 (3.21V)
# [000.789s] [INFO ] [PEMF   ] Pulse generation started, frequency: 2Hz
```

#### Example 2: Filter by Log Level

```bash
# Show only important messages (INFO and above)
python3 hidlog.py --level INFO

# Show only errors
python3 hidlog.py --level ERROR

# Expected output for ERROR level:
# [002.567s] [ERROR] [PEMF   ] Pulse timing deviation detected: +2.1ms
# [005.123s] [ERROR] [BATTERY] ADC read timeout, using last known value
```

#### Example 3: Filter by Module

```bash
# Monitor only battery-related logs
python3 hidlog.py --module BATTERY

# Expected output:
# [000.456s] [DEBUG] [BATTERY] ADC reading: 1650 (3.21V)
# [000.556s] [DEBUG] [BATTERY] State: Normal, Voltage: 3.21V
# [001.456s] [INFO ] [BATTERY] State changed: Normal -> Charging
# [002.456s] [WARN ] [BATTERY] Voltage drop detected: 3.21V -> 3.05V

# Monitor only pEMF-related logs  
python3 hidlog.py --module PEMF

# Expected output:
# [000.789s] [INFO ] [PEMF   ] Pulse generation started, frequency: 2Hz
# [001.289s] [DEBUG] [PEMF   ] Pulse timing: HIGH=2.0ms, LOW=498.0ms
# [002.567s] [ERROR] [PEMF   ] Pulse timing deviation detected: +2.1ms
```

### Advanced Usage Examples

#### Example 4: Save Logs to File

```bash
# Save all logs to timestamped file
python3 hidlog.py --log-file logs/device_$(date +%Y%m%d_%H%M%S).log

# Save only errors to file
python3 hidlog.py --level ERROR --log-file error.log

# Monitor in terminal AND save to file
python3 hidlog.py --log-file device.log | tee terminal_output.log
```

#### Example 5: JSON Output for Analysis

```bash
# Output logs in JSON format
python3 hidlog.py --json-output

# Expected JSON output:
# {"timestamp": 0.123, "level": "INFO", "module": "SYSTEM", "message": "System boot complete"}
# {"timestamp": 0.456, "level": "DEBUG", "module": "BATTERY", "message": "ADC reading: 1650 (3.21V)"}

# Save JSON logs for analysis
python3 hidlog.py --json-output --log-file logs.json

# Process JSON logs with jq
python3 hidlog.py --json-output | jq 'select(.level == "ERROR")'
```

#### Example 6: Multiple Device Management

```bash
# List all available devices
python3 hidlog.py --list

# Expected output:
# Available HID devices:
# Device 1:
#   Path: /dev/hidraw0
#   VID:PID: 1234:5678
#   Manufacturer: Custom Electronics
#   Product: pEMF Battery Monitor
#   Serial: 123456789
# 
# Device 2:
#   Path: /dev/hidraw1  
#   VID:PID: 1234:5678
#   Manufacturer: Custom Electronics
#   Product: pEMF Battery Monitor
#   Serial: 987654321

# Connect to specific device by path
python3 hidlog.py --device /dev/hidraw0

# Connect to specific device by serial number
python3 hidlog.py --serial 123456789
```

### Battery Monitoring Examples

#### Example 7: Battery State Monitoring

```bash
# Monitor battery state changes
python3 hidlog.py --module BATTERY --level INFO

# Simulate battery state changes by connecting/disconnecting charger:
# 
# Normal operation (3.1V - 3.6V):
# [010.123s] [INFO ] [BATTERY] State: Normal, Voltage: 3.45V, ADC: 1612
# 
# Connect charger (>3.6V):
# [015.456s] [INFO ] [BATTERY] State changed: Normal -> Charging
# [015.457s] [INFO ] [BATTERY] State: Charging, Voltage: 3.78V, ADC: 1702
# 
# Low battery (<3.1V):
# [020.789s] [WARN ] [BATTERY] State changed: Normal -> Low
# [020.790s] [WARN ] [BATTERY] State: Low, Voltage: 3.05V, ADC: 1398
```

#### Example 8: Battery Calibration

```bash
# Monitor raw ADC values for calibration
python3 hidlog.py --module BATTERY --level DEBUG

# Use multimeter to measure actual battery voltage
# Compare with logged ADC values to calculate calibration factor:
# 
# [025.123s] [DEBUG] [BATTERY] Raw ADC: 1650, Calculated: 3.21V
# [025.223s] [DEBUG] [BATTERY] Raw ADC: 1651, Calculated: 3.21V
# 
# Calibration calculation:
# Actual voltage (multimeter): 3.25V
# Calculated voltage (device): 3.21V  
# Calibration factor: 3.25 / 3.21 = 1.012
```

### pEMF Timing Validation Examples

#### Example 9: Pulse Timing Monitoring

```bash
# Monitor pEMF pulse timing accuracy
python3 hidlog.py --module PEMF

# Expected output showing timing validation:
# [030.100s] [INFO ] [PEMF   ] Pulse generation started, target: 2Hz
# [030.200s] [DEBUG] [PEMF   ] Pulse timing: HIGH=2.0ms, LOW=498.0ms, Period=500.0ms
# [030.700s] [DEBUG] [PEMF   ] Pulse timing: HIGH=2.1ms, LOW=497.9ms, Period=500.0ms
# [031.200s] [WARN ] [PEMF   ] Timing deviation: HIGH +0.1ms (+5.0%)
# [031.700s] [DEBUG] [PEMF   ] Pulse timing: HIGH=2.0ms, LOW=498.0ms, Period=500.0ms
```

#### Example 10: Performance Impact Analysis

```bash
# Monitor system performance with USB logging active
python3 hidlog.py --module SYSTEM --level INFO

# Look for timing warnings or resource issues:
# [035.123s] [INFO ] [SYSTEM ] CPU usage: 15%, Free RAM: 180KB
# [035.623s] [INFO ] [SYSTEM ] Task timing: pEMF=0.1ms, Battery=2.3ms, USB=1.8ms
# [036.123s] [WARN ] [SYSTEM ] USB task overrun: 12.5ms (target: 10ms)
# [036.623s] [INFO ] [SYSTEM ] Log queue utilization: 45% (14/32 messages)
```

### System Diagnostics Examples

#### Example 11: Boot Sequence Monitoring

```bash
# Monitor system initialization
python3 hidlog.py --level INFO

# Power cycle the device and observe boot sequence:
# [000.001s] [INFO ] [SYSTEM ] RP2040 boot started, firmware v1.2.3
# [000.010s] [INFO ] [SYSTEM ] Clock configuration: 125MHz system, 48MHz USB
# [000.025s] [INFO ] [SYSTEM ] GPIO initialization complete
# [000.040s] [INFO ] [BATTERY] ADC initialization complete, calibration loaded
# [000.055s] [INFO ] [PEMF   ] Timer initialization complete, 2Hz target set
# [000.070s] [INFO ] [USB    ] HID device enumeration started
# [000.250s] [INFO ] [USB    ] HID device enumerated successfully
# [000.300s] [INFO ] [SYSTEM ] All systems operational, entering main loop
```

#### Example 12: Error Condition Monitoring

```bash
# Monitor for system errors and recovery
python3 hidlog.py --level WARN

# Trigger error conditions (disconnect battery, overload pEMF output, etc.):
# [040.123s] [WARN ] [BATTERY] ADC timeout, retrying...
# [040.223s] [ERROR] [BATTERY] ADC communication failed after 3 retries
# [040.323s] [INFO ] [BATTERY] Using last known voltage: 3.45V
# [041.123s] [WARN ] [PEMF   ] Load impedance changed, adjusting timing
# [041.223s] [ERROR] [PEMF   ] Timing correction failed, stopping pulse generation
# [041.323s] [INFO ] [PEMF   ] Pulse generation stopped for safety
```

## Troubleshooting Guide

### Device Connection Issues

#### Problem: Device Not Detected

**Symptoms:**
- `lsusb` doesn't show device with VID:PID 1234:5678
- `hidlog.py --list` shows no devices
- "No HID devices found" error

**Diagnostic Steps:**
```bash
# 1. Check if device is connected and powered
lsusb | grep -E "(1234:5678|2e8a:0003)"

# 2. Check if device is in bootloader mode (should NOT be)
lsusb | grep "2e8a:0003"
# If found, device is in bootloader mode - need to flash firmware

# 3. Check USB cable and port
# Try different USB cable (ensure it's a data cable, not power-only)
# Try different USB port

# 4. Check dmesg for USB events
dmesg | tail -20 | grep -i usb
```

**Solutions:**
1. **If device is in bootloader mode:**
   ```bash
   # Flash the USB HID logging firmware
   cd ~/rp2040-projects/ass-easy-loop
   cargo run --release
   ```

2. **If device is not detected at all:**
   ```bash
   # Check physical connections
   # Try different USB cable
   # Try different USB port
   # Check device power (LED should be on)
   ```

3. **If device appears but wrong VID:PID:**
   ```bash
   # Check firmware configuration
   # Verify correct firmware was flashed
   # Check VID:PID in source code matches hidlog.py expectations
   ```

#### Problem: Permission Denied

**Symptoms:**
- "Permission denied" when running hidlog.py
- Can see device with `lsusb` but can't access it

**Solutions:**
```bash
# 1. Check user groups
groups $USER | grep dialout
# If not in dialout group:
sudo usermod -a -G dialout $USER
# Then log out and back in

# 2. Check udev rules
ls -la /etc/udev/rules.d/99-rp2040-hid.rules
# If missing, recreate the udev rules (see setup section)

# 3. Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger

# 4. Check device permissions
ls -la /dev/hidraw*
# Should show group ownership as dialout or similar

# 5. Temporary workaround (not recommended for regular use)
sudo python3 hidlog.py
```

### Communication Issues

#### Problem: No Log Messages Received

**Symptoms:**
- hidlog.py connects successfully but shows no messages
- "Monitoring logs..." appears but no log entries

**Diagnostic Steps:**
```bash
# 1. Check if device is sending data
python3 hidlog.py --raw
# Should show raw HID report data if device is transmitting

# 2. Check log level filtering
python3 hidlog.py --level DEBUG
# Ensure you're not filtering out messages

# 3. Check module filtering
python3 hidlog.py --module ""
# Remove any module filters

# 4. Test with different device
python3 hidlog.py --list
# Try connecting to different device path if multiple available
```

**Solutions:**
1. **If no raw data:**
   ```bash
   # Device firmware may not be sending logs
   # Check firmware configuration
   # Verify USB HID logging is enabled in firmware
   ```

2. **If raw data but no parsed messages:**
   ```bash
   # Message format may be incorrect
   # Check firmware message format matches hidlog.py parser
   # Try updating hidlog.py or firmware
   ```

3. **If intermittent messages:**
   ```bash
   # USB communication issues
   # Try different USB port/cable
   # Check for USB power management issues
   ```

#### Problem: Garbled or Corrupted Messages

**Symptoms:**
- Messages contain random characters
- Timestamps are incorrect
- Message format is inconsistent

**Solutions:**
```bash
# 1. Check message format compatibility
python3 hidlog.py --inspect-hid
# Examine raw HID reports for format issues

# 2. Verify firmware version compatibility
# Ensure hidlog.py version matches firmware message format

# 3. Check USB communication integrity
# Try different USB cable/port
# Check for electromagnetic interference

# 4. Reset device and reconnect
# Power cycle the RP2040 device
# Restart hidlog.py
```

### Performance Issues

#### Problem: Missing Log Messages

**Symptoms:**
- Gaps in timestamp sequence
- "Log queue full" warnings
- Important messages not appearing

**Solutions:**
```bash
# 1. Reduce log verbosity
python3 hidlog.py --level INFO  # Instead of DEBUG

# 2. Filter to specific modules
python3 hidlog.py --module BATTERY  # Focus on specific subsystem

# 3. Check device log queue status
python3 hidlog.py --module SYSTEM --level WARN
# Look for queue utilization warnings

# 4. Increase USB polling frequency (firmware modification required)
# Modify USB task priority or polling interval in firmware
```

#### Problem: High CPU Usage

**Symptoms:**
- hidlog.py uses excessive CPU
- System becomes slow while monitoring logs
- High USB interrupt rate

**Solutions:**
```bash
# 1. Reduce update frequency
# Add delays in hidlog.py main loop (modify source)

# 2. Use log file instead of terminal output
python3 hidlog.py --log-file device.log
# Terminal output can be CPU-intensive

# 3. Filter messages to reduce processing
python3 hidlog.py --level ERROR --module BATTERY

# 4. Use JSON output for automated processing
python3 hidlog.py --json-output --log-file logs.json
```

### Firmware Issues

#### Problem: Device Resets or Stops Responding

**Symptoms:**
- Device disappears from USB enumeration
- Log messages stop abruptly
- Device requires power cycle to recover

**Diagnostic Steps:**
```bash
# 1. Check for panic messages
python3 hidlog.py --level ERROR
# Look for panic or crash messages before disconnect

# 2. Monitor system messages
dmesg | tail -20
# Check for USB disconnect/reconnect events

# 3. Check device power
# Verify stable power supply
# Check for power supply noise or voltage drops
```

**Solutions:**
1. **If firmware panics:**
   ```bash
   # Check firmware for bugs
   # Review panic handler implementation
   # Flash debug firmware with additional error checking
   ```

2. **If USB disconnects:**
   ```bash
   # Check USB cable and connections
   # Verify USB power supply stability
   # Check for electromagnetic interference
   ```

3. **If memory issues:**
   ```bash
   # Reduce log queue size in firmware
   # Check for memory leaks in firmware
   # Monitor memory usage messages
   ```

### Advanced Troubleshooting

#### Using Raw HID Inspection

```bash
# Inspect raw HID reports for low-level debugging
python3 hidlog.py --inspect-hid

# Expected output:
# Raw HID Report (64 bytes):
# 00: 01 00 00 7B 53 59 53 54 45 4D 00 00 53 79 73 74  ...{SYSTEM..Syst
# 10: 65 6D 20 62 6F 6F 74 20 63 6F 6D 70 6C 65 74 65  em boot complete
# 20: 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  ................
# 30: 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  ................
# 
# Parsed: [000.123s] [INFO ] [SYSTEM ] System boot complete
```

#### Using System Monitoring Tools

```bash
# Monitor USB traffic with usbmon
sudo modprobe usbmon
sudo cat /sys/kernel/debug/usb/usbmon/0u | grep 1234:5678

# Monitor HID events
sudo evtest  # If device creates input events

# Check USB device details
lsusb -v -d 1234:5678

# Monitor system resources
htop  # Check CPU/memory usage while running hidlog.py
```

#### Creating Debug Logs

```bash
# Create comprehensive debug log
python3 hidlog.py --level DEBUG --raw --log-file debug_$(date +%Y%m%d_%H%M%S).log

# Include system information
{
    echo "=== System Information ==="
    uname -a
    echo "=== USB Devices ==="
    lsusb
    echo "=== HID Devices ==="
    ls -la /dev/hidraw*
    echo "=== Log Output ==="
    python3 hidlog.py --level DEBUG --raw
} > system_debug_$(date +%Y%m%d_%H%M%S).log
```

## Advanced Configuration

### Customizing VID/PID

If you need to use different USB identifiers:

1. **Modify firmware configuration:**
   ```rust
   // In src/main.rs or config file
   const USB_VID: u16 = 0x1234;  // Your vendor ID
   const USB_PID: u16 = 0x5678;  // Your product ID
   ```

2. **Update udev rules:**
   ```bash
   sudo sed -i 's/1234:5678/XXXX:YYYY/g' /etc/udev/rules.d/99-rp2040-hid.rules
   sudo udevadm control --reload-rules
   ```

3. **Update hidlog.py default values:**
   ```python
   # In hidlog.py
   DEFAULT_VID = 0xXXXX
   DEFAULT_PID = 0xYYYY
   ```

### Performance Tuning

#### Firmware-Side Optimizations

```rust
// Adjust log queue size
const LOG_QUEUE_SIZE: usize = 64;  // Increase for more buffering

// Adjust USB task priority
#[task(priority = 2)]  // Increase priority if needed
async fn usb_hid_task(ctx: usb_hid_task::Context) {
    // ...
}

// Adjust message format for efficiency
struct LogMessage {
    timestamp: u32,
    level: u8,
    module: [u8; 4],    // Shorter module names
    message: [u8; 32],  // Shorter messages
}
```

#### Host-Side Optimizations

```python
# Increase HID read timeout for better reliability
device.set_nonblocking(False)
device.read(64, timeout_ms=5000)  # 5 second timeout

# Batch message processing
messages = []
while True:
    data = device.read(64, timeout_ms=100)
    if data:
        messages.append(parse_message(data))
        if len(messages) >= 10:  # Process in batches
            process_messages(messages)
            messages.clear()
```

### Integration with Development Workflow

#### Automated Testing Script

```bash
#!/bin/bash
# test_usb_logging.sh - Automated USB HID logging test

set -e

echo "Starting USB HID logging test..."

# Check device connection
if ! lsusb | grep -q "1234:5678"; then
    echo "ERROR: Device not found"
    exit 1
fi

# Test basic communication
timeout 10s python3 hidlog.py --level INFO > test_output.log &
PID=$!

sleep 5

# Trigger device events (if possible)
# This would depend on your specific device capabilities

# Stop logging
kill $PID 2>/dev/null || true

# Analyze results
if [ -s test_output.log ]; then
    echo "SUCCESS: Log messages received"
    echo "Message count: $(wc -l < test_output.log)"
else
    echo "ERROR: No log messages received"
    exit 1
fi

echo "USB HID logging test completed successfully"
```

#### Continuous Integration

```yaml
# .github/workflows/hardware-test.yml
name: Hardware Test
on: [push, pull_request]

jobs:
  hardware-test:
    runs-on: self-hosted  # Requires hardware runner
    steps:
    - uses: actions/checkout@v3
    - name: Setup Python
      uses: actions/setup-python@v4
      with:
        python-version: '3.11'
    - name: Install dependencies
      run: |
        pip install -r requirements.txt
    - name: Test USB HID communication
      run: |
        python3 hidlog.py --list
        timeout 30s python3 hidlog.py --level INFO > ci_test.log
        test -s ci_test.log  # Ensure log file is not empty
```

This comprehensive guide covers all aspects of setting up, using, and troubleshooting the USB HID logging system. The examples and troubleshooting steps should help users successfully implement and debug the logging functionality.