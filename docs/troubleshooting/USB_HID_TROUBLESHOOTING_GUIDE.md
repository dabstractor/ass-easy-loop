# USB HID Logging Troubleshooting Guide

This comprehensive troubleshooting guide addresses common issues encountered when using the USB HID logging functionality with the RP2040 pEMF/Battery monitoring device.

## Table of Contents

1. [Quick Diagnostic Checklist](#quick-diagnostic-checklist)
2. [Device Detection Issues](#device-detection-issues)
3. [Communication Problems](#communication-problems)
4. [Performance Issues](#performance-issues)
5. [Firmware-Related Problems](#firmware-related-problems)
6. [Host System Issues](#host-system-issues)
7. [Advanced Debugging Techniques](#advanced-debugging-techniques)
8. [Common Error Messages](#common-error-messages)

## Quick Diagnostic Checklist

Before diving into detailed troubleshooting, run through this quick checklist:

### Basic System Check
```bash
# 1. Check if device is connected and powered
lsusb | grep -E "(1234:5678|2e8a:0003)"

# 2. Check USB HID logging utility
python3 hidlog.py --list

# 3. Check system permissions
groups $USER | grep dialout

# 4. Check udev rules
ls -la /etc/udev/rules.d/99-rp2040-hid.rules

# 5. Check Python environment
python3 -c "import hid; print('HID library available')"
```

### Expected Results
- **lsusb**: Should show device with VID:PID 1234:5678 (not 2e8a:0003)
- **hidlog.py --list**: Should show at least one HID device
- **groups**: Should include "dialout" group
- **udev rules**: Should exist and be readable
- **Python HID**: Should import without errors

If any of these fail, see the relevant sections below.

## Device Detection Issues

### Issue: Device Not Detected by System

#### Symptoms
- `lsusb` doesn't show any RP2040 device
- No USB enumeration sounds/notifications
- Device appears completely unresponsive

#### Diagnostic Steps
```bash
# Check USB connection
lsusb -v | grep -A 10 -B 5 "2e8a\|1234"

# Check kernel messages
dmesg | tail -20 | grep -i usb

# Check USB device tree
lsusb -t

# Check power management
cat /sys/bus/usb/devices/*/power/control
```

#### Solutions

**Solution 1: Hardware Connection Issues**
```bash
# Check physical connections:
# 1. Ensure USB cable is fully inserted
# 2. Try different USB cable (data cable, not power-only)
# 3. Try different USB port on computer
# 4. Avoid USB hubs - connect directly to computer

# Test USB cable with another device
# Test USB port with another device
```

**Solution 2: Power Supply Issues**
```bash
# Check device power indicators
# - LED should be on when powered
# - Check for dim or flickering LED (insufficient power)

# Try external power source if available
# Check USB port power output:
lsusb -v | grep -A 5 "MaxPower"
```

**Solution 3: Device Firmware Issues**
```bash
# Check if device is in bootloader mode
lsusb | grep "2e8a:0003"

# If in bootloader mode, flash application firmware:
cd ~/rp2040-projects/ass-easy-loop
cargo run --release

# If not detected at all, try entering bootloader mode manually
# (Hold BOOTSEL while connecting USB)
```

### Issue: Device Detected as Bootloader Instead of Application

#### Symptoms
- `lsusb` shows VID:PID 2e8a:0003 (RP2040 bootloader)
- Device appears as "RPI-RP2" mass storage device
- No HID functionality available

#### Solutions
```bash
# This indicates firmware is not running or corrupted
# Flash the USB HID logging firmware:

# 1. Verify device is in bootloader mode
lsusb | grep "2e8a:0003"
ls /media/*/RPI-RP2 || ls /run/media/*/RPI-RP2

# 2. Build and flash firmware
cd ~/rp2040-projects/ass-easy-loop
cargo clean
cargo build --release
cargo run --release

# 3. Verify application mode after flash
sleep 5
lsusb | grep "1234:5678"
```

### Issue: Wrong VID:PID Detected

#### Symptoms
- Device detected but with unexpected VID:PID
- `hidlog.py` can't find device with default VID:PID

#### Solutions
```bash
# 1. Check actual VID:PID
lsusb | grep -i "rp2040\|pico\|custom"

# 2. Use correct VID:PID with hidlog.py
python3 hidlog.py --vid 0xXXXX --pid 0xYYYY

# 3. Update firmware configuration if needed
# Edit src/main.rs or config file to set correct VID:PID

# 4. Update udev rules for new VID:PID
sudo sed -i 's/1234:5678/XXXX:YYYY/g' /etc/udev/rules.d/99-rp2040-hid.rules
sudo udevadm control --reload-rules
```

## Communication Problems

### Issue: Device Detected But No Log Messages

#### Symptoms
- `hidlog.py --list` shows device
- Connection appears successful
- No log messages displayed

#### Diagnostic Steps
```bash
# 1. Check raw HID communication
python3 hidlog.py --raw

# 2. Check log level filtering
python3 hidlog.py --level DEBUG

# 3. Check module filtering
python3 hidlog.py --module ""

# 4. Test with different device path
python3 hidlog.py --device /dev/hidraw0

# 5. Check HID device permissions
ls -la /dev/hidraw*
```

#### Solutions

**Solution 1: Firmware Not Sending Logs**
```bash
# Check if firmware includes USB HID logging
# Verify firmware was built with logging enabled

# Check firmware configuration
grep -r "USB_HID\|LOG" src/

# Reflash firmware with logging enabled
cargo build --release --features usb-hid-logging
cargo run --release
```

**Solution 2: Message Format Issues**
```bash
# Check raw HID reports
python3 hidlog.py --inspect-hid

# Expected: Should show non-zero data in HID reports
# If all zeros: Firmware not sending data
# If garbage data: Message format mismatch

# Update hidlog.py or firmware to match message format
```

**Solution 3: USB Communication Issues**
```bash
# Try different USB port/cable
# Check for electromagnetic interference

# Reset USB subsystem
sudo modprobe -r usbhid
sudo modprobe usbhid

# Restart device
# Disconnect USB, wait 10 seconds, reconnect
```

### Issue: Intermittent Message Loss

#### Symptoms
- Some log messages appear but others are missing
- Gaps in timestamp sequences
- "Log queue full" warnings

#### Solutions
```bash
# 1. Reduce log verbosity
python3 hidlog.py --level INFO  # Instead of DEBUG

# 2. Filter to specific modules
python3 hidlog.py --module BATTERY

# 3. Increase USB polling frequency (firmware change required)
# Modify USB task priority or polling interval

# 4. Check system load
htop  # Look for high CPU usage
iostat 1  # Check disk I/O if logging to file
```

### Issue: Corrupted or Garbled Messages

#### Symptoms
- Messages contain random characters
- Incorrect timestamps
- Malformed log entries

#### Diagnostic Steps
```bash
# 1. Check raw HID data
python3 hidlog.py --raw --inspect-hid

# 2. Check message parsing
python3 -c "
import hid
device = hid.device()
device.open(0x1234, 0x5678)
data = device.read(64)
print('Raw data:', [hex(b) for b in data])
device.close()
"

# 3. Check USB communication integrity
dmesg | grep -i "usb.*error"
```

#### Solutions
```bash
# 1. Check message format compatibility
# Ensure hidlog.py version matches firmware message format

# 2. Check for USB interference
# Try different USB port
# Move away from sources of electromagnetic interference

# 3. Reset communication
# Disconnect and reconnect device
# Restart hidlog.py

# 4. Update firmware and host software
# Ensure both are using same message protocol version
```

## Performance Issues

### Issue: High CPU Usage

#### Symptoms
- `hidlog.py` uses excessive CPU resources
- System becomes slow during log monitoring
- High interrupt rate in system monitor

#### Solutions
```bash
# 1. Reduce update frequency
# Modify hidlog.py to add delays between reads

# 2. Use file output instead of terminal
python3 hidlog.py --log-file device.log

# 3. Filter messages to reduce processing
python3 hidlog.py --level ERROR --module BATTERY

# 4. Use JSON output for automated processing
python3 hidlog.py --json-output --log-file logs.json

# 5. Check system resources
top -p $(pgrep -f hidlog.py)
```

### Issue: Memory Usage Growth

#### Symptoms
- `hidlog.py` memory usage increases over time
- System runs out of memory during long monitoring sessions
- Python process becomes unresponsive

#### Solutions
```bash
# 1. Restart hidlog.py periodically
# Use a wrapper script to restart every hour

# 2. Limit log file size
python3 hidlog.py --log-file device.log | head -10000

# 3. Use log rotation
logrotate -f /etc/logrotate.d/hidlog

# 4. Monitor memory usage
watch -n 5 'ps aux | grep hidlog'
```

### Issue: USB Communication Timeouts

#### Symptoms
- "Timeout reading from device" errors
- Intermittent connection drops
- Device appears to freeze

#### Solutions
```bash
# 1. Increase timeout values
# Modify hidlog.py timeout parameters

# 2. Check USB power management
echo 'on' | sudo tee /sys/bus/usb/devices/*/power/control

# 3. Disable USB autosuspend
echo -1 | sudo tee /sys/module/usbcore/parameters/autosuspend

# 4. Check USB hub issues
# Connect device directly to computer, not through hub
```

## Firmware-Related Problems

### Issue: Device Resets or Crashes

#### Symptoms
- Device disappears from USB enumeration
- Log messages stop abruptly
- Device requires power cycle to recover

#### Diagnostic Steps
```bash
# 1. Check for panic messages before crash
python3 hidlog.py --level ERROR | tail -20

# 2. Monitor system messages
dmesg -w | grep -i usb

# 3. Check device power stability
# Use oscilloscope to monitor power supply if available

# 4. Check for memory issues
python3 hidlog.py --module SYSTEM --level WARN
```

#### Solutions
```bash
# 1. Check firmware for bugs
# Review panic handler implementation
# Add more error checking in firmware

# 2. Check power supply stability
# Ensure stable 3.3V supply
# Add power supply filtering capacitors

# 3. Reduce firmware complexity
# Disable non-essential features
# Reduce log queue size

# 4. Flash debug firmware
# Build with debug symbols and additional logging
cargo build --features debug-logging
```

### Issue: Timing Issues with pEMF Generation

#### Symptoms
- pEMF timing deviates from specification
- "Timing deviation" error messages
- Inconsistent pulse generation

#### Solutions
```bash
# 1. Check USB logging impact on timing
python3 hidlog.py --module PEMF --level WARN

# 2. Reduce USB logging verbosity
# Modify firmware to log less frequently
# Increase USB task priority if needed

# 3. Verify clock configuration
# Check external crystal oscillator
# Verify PLL configuration in firmware

# 4. Monitor timing with oscilloscope
# Measure actual pulse timing on GPIO 15
# Compare with logged timing values
```

## Host System Issues

### Issue: Permission Denied Errors

#### Symptoms
- "Permission denied" when running `hidlog.py`
- Can see device with `lsusb` but can't access it
- HID device files not accessible

#### Solutions
```bash
# 1. Check user groups
groups $USER
# Should include "dialout" group

# If not in dialout group:
sudo usermod -a -G dialout $USER
# Log out and back in

# 2. Check udev rules
ls -la /etc/udev/rules.d/99-rp2040-hid.rules

# If missing, create udev rules:
sudo tee /etc/udev/rules.d/99-rp2040-hid.rules << 'EOF'
SUBSYSTEM=="usb", ATTRS{idVendor}=="1234", ATTRS{idProduct}=="5678", MODE="0666", GROUP="dialout"
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="1234", ATTRS{idProduct}=="5678", MODE="0666", GROUP="dialout"
EOF

# 3. Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger

# 4. Check device permissions
ls -la /dev/hidraw*
# Should show group ownership as dialout
```

### Issue: Python Library Issues

#### Symptoms
- "ModuleNotFoundError: No module named 'hid'"
- Import errors when running `hidlog.py`
- Incompatible library versions

#### Solutions
```bash
# 1. Check Python environment
python3 --version
which python3

# 2. Install/reinstall HID libraries
pip3 install --upgrade hidapi hid

# On Arch Linux:
sudo pacman -S python-hid hidapi

# 3. Check library installation
python3 -c "import hid; print(hid.__file__)"

# 4. Use virtual environment
python3 -m venv ~/rp2040-venv
source ~/rp2040-venv/bin/activate
pip install hidapi hid
```

### Issue: System-Specific Problems

#### Wayland Display Server Issues
```bash
# Some GUI tools may not work with Wayland
export QT_QPA_PLATFORM=wayland
export GDK_BACKEND=wayland

# Or force X11 compatibility
export QT_QPA_PLATFORM=xcb
export GDK_BACKEND=x11
```

#### Firewall/Security Issues
```bash
# Check if firewall is blocking USB communication
sudo systemctl status ufw
sudo ufw status

# Allow local USB communication if needed
sudo ufw allow from 127.0.0.1
```

#### SELinux Issues (if applicable)
```bash
# Check SELinux status
sestatus

# If SELinux is enforcing, may need policy updates
# Consult distribution documentation for USB device policies
```

## Advanced Debugging Techniques

### USB Traffic Analysis

#### Using usbmon
```bash
# Load usbmon module
sudo modprobe usbmon

# Monitor USB traffic
sudo cat /sys/kernel/debug/usb/usbmon/0u | grep 1234:5678

# Or use tcpdump-like interface
sudo tcpdump -i usbmon0 -w usb_capture.pcap
```

#### Using Wireshark
```bash
# Install Wireshark with USB capture support
sudo pacman -S wireshark-qt

# Add user to wireshark group
sudo usermod -a -G wireshark $USER

# Capture USB traffic
# Start Wireshark, select USBPcap interface
# Filter by device VID:PID
```

### Low-Level HID Debugging

#### Raw HID Report Analysis
```bash
# Create HID debugging script
cat > debug_hid.py << 'EOF'
#!/usr/bin/env python3
import hid
import time
import struct

def debug_hid_device(vid=0x1234, pid=0x5678):
    try:
        device = hid.device()
        device.open(vid, pid)
        device.set_nonblocking(True)
        
        print(f"Connected to device {vid:04x}:{pid:04x}")
        print("Raw HID reports (Ctrl+C to stop):")
        print("-" * 80)
        
        while True:
            data = device.read(64, timeout_ms=1000)
            if data:
                # Print raw bytes
                hex_data = ' '.join(f'{b:02x}' for b in data)
                print(f"Raw: {hex_data}")
                
                # Try to parse as log message
                try:
                    timestamp = struct.unpack('<I', bytes(data[57:61]))[0]
                    level = data[0]
                    module = bytes(data[1:9]).decode('ascii', errors='ignore').rstrip('\x00')
                    message = bytes(data[9:57]).decode('ascii', errors='ignore').rstrip('\x00')
                    
                    print(f"Parsed: [{timestamp/1000:.3f}s] [{level}] [{module}] {message}")
                except Exception as e:
                    print(f"Parse error: {e}")
                
                print("-" * 80)
            
            time.sleep(0.1)
            
    except KeyboardInterrupt:
        print("\nStopped by user")
    except Exception as e:
        print(f"Error: {e}")
    finally:
        try:
            device.close()
        except:
            pass

if __name__ == "__main__":
    debug_hid_device()
EOF

chmod +x debug_hid.py
python3 debug_hid.py
```

#### HID Descriptor Analysis
```bash
# Analyze HID descriptor
python3 -c "
import hid
device = hid.device()
device.open(0x1234, 0x5678)
print('Manufacturer:', device.get_manufacturer_string())
print('Product:', device.get_product_string())
print('Serial:', device.get_serial_number_string())
device.close()
"

# Use system tools
lsusb -v -d 1234:5678 | grep -A 20 "HID Device Descriptor"
```

### Firmware Debugging

#### Adding Debug Output to Firmware
```rust
// Add to firmware for debugging
use rtt_target::{rprintln, rtt_init_print};

#[rtic::app(device = rp2040_hal::pac, peripherals = true)]
mod app {
    // ... existing code ...
    
    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();
        rprintln!("Firmware debug output initialized");
        
        // ... rest of init ...
    }
    
    // Add debug prints in USB tasks
    #[task(shared = [hid_class], local = [log_queue], priority = 1)]
    async fn usb_hid_task(ctx: usb_hid_task::Context) {
        rprintln!("USB HID task started");
        // ... task implementation ...
    }
}
```

#### Using RTT for Debug Output
```bash
# Install probe-rs with RTT support
cargo install probe-rs --features cli

# Connect with RTT
probe-rs attach --chip RP2040 --protocol swd

# Monitor RTT output in separate terminal
probe-rs rtt --chip RP2040
```

## Common Error Messages

### "No HID devices found"
**Cause**: Device not detected or wrong VID:PID
**Solution**: Check device connection, verify VID:PID, check permissions

### "Permission denied"
**Cause**: Insufficient permissions to access HID device
**Solution**: Add user to dialout group, check udev rules

### "Device or resource busy"
**Cause**: Another process is using the HID device
**Solution**: Close other applications, check for multiple hidlog.py instances

### "Timeout reading from device"
**Cause**: Device not responding or USB communication issues
**Solution**: Check USB connection, try different port/cable

### "ModuleNotFoundError: No module named 'hid'"
**Cause**: Python HID library not installed
**Solution**: Install hidapi and hid packages

### "USB device disconnected"
**Cause**: Device reset, power issue, or firmware crash
**Solution**: Check power supply, review firmware for bugs

### "Invalid HID report format"
**Cause**: Message format mismatch between firmware and host
**Solution**: Update firmware or hidlog.py to match protocol

### "Log queue overflow"
**Cause**: Too many log messages, queue full
**Solution**: Reduce log verbosity, increase queue size in firmware

This comprehensive troubleshooting guide should help users diagnose and resolve most issues encountered with the USB HID logging system.