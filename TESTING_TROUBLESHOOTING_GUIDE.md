# Testing Troubleshooting Guide

This guide provides comprehensive troubleshooting information for common issues encountered during automated testing of the RP2040 pEMF/Battery Monitoring Device.

## Table of Contents

1. [Quick Diagnostic Checklist](#quick-diagnostic-checklist)
2. [Hardware-Related Issues](#hardware-related-issues)
3. [Software and Firmware Issues](#software-and-firmware-issues)
4. [Communication Problems](#communication-problems)
5. [Test Framework Issues](#test-framework-issues)
6. [Performance and Timing Issues](#performance-and-timing-issues)
7. [Environment and Setup Issues](#environment-and-setup-issues)
8. [Advanced Debugging Techniques](#advanced-debugging-techniques)

## Quick Diagnostic Checklist

### Before Starting Troubleshooting

Run this quick checklist to identify the most common issues:

```bash
# 1. Hardware Status Check
echo "=== Hardware Status Check ==="
lsusb | grep -i "2e8a" && echo "✓ Device detected" || echo "✗ Device not detected"

# 2. Battery Status Check
echo "=== Battery Status Check ==="
# Measure battery voltage with multimeter
# Should be 3.0V - 4.2V

# 3. Test Framework Status Check
echo "=== Test Framework Status Check ==="
cd test_framework
python -c "import hid; print('✓ HID library available')" 2>/dev/null || echo "✗ HID library missing"

# 4. Basic Communication Test
echo "=== Communication Test ==="
python device_manager.py --ping-test 2>/dev/null && echo "✓ Communication OK" || echo "✗ Communication failed"

# 5. Firmware Status Check
echo "=== Firmware Status Check ==="
# Check if device responds to basic commands
python command_handler.py --system-health 2>/dev/null && echo "✓ Firmware responding" || echo "✗ Firmware not responding"
```

### Common Issue Categories

| Symptom | Likely Category | Quick Fix |
|---------|----------------|-----------|
| Device not detected | Hardware/USB | Check connections, try different USB port |
| Communication timeout | Firmware/Protocol | Restart device, check firmware version |
| Test failures | Test Framework | Update test parameters, check device state |
| Timing inaccuracy | Hardware/Firmware | Check crystal oscillator, verify timing code |
| Inconsistent results | Environment | Check power supply, reduce EMI |

## Hardware-Related Issues

### Device Detection Problems

#### Issue: Device Not Detected by Host

**Symptoms:**
- `lsusb` doesn't show device (Linux)
- Device Manager shows unknown device (Windows)
- Test framework can't find device

**Diagnostic Steps:**
```bash
# Check USB connection
lsusb | grep -i "2e8a"
# Expected: Bus XXX Device XXX: ID 2e8a:000a Raspberry Pi RP2 Boot

# Check device manager (Windows)
# Look for "RP2040" or unknown devices

# Try different USB port
# Try different USB cable (ensure data-capable)
```

**Solutions:**
1. **Hardware Connection Issues:**
   ```bash
   # Check physical connections
   # - USB cable firmly connected
   # - No damage to USB connector
   # - Try different USB cable
   # - Try different USB port
   ```

2. **Power Supply Issues:**
   ```bash
   # Verify power supply
   # - Battery voltage >3.0V
   # - VSYS pin receiving power
   # - No short circuits
   ```

3. **Driver Issues (Windows):**
   ```bash
   # Install libusb drivers using Zadig
   # 1. Download Zadig from zadig.akeo.ie
   # 2. Run as administrator
   # 3. Select RP2040 device
   # 4. Install WinUSB driver
   ```

#### Issue: Device Detected but Not Accessible

**Symptoms:**
- Device appears in `lsusb` but permission denied
- Test framework can't open device
- "Access denied" or "Permission denied" errors

**Solutions:**
1. **Linux Permission Issues:**
   ```bash
   # Add user to dialout group
   sudo usermod -a -G dialout $USER
   
   # Create udev rule for HID devices
   sudo nano /etc/udev/rules.d/99-hid.rules
   # Add: SUBSYSTEM=="hidraw", ATTRS{idVendor}=="2e8a", MODE="0666"
   
   # Reload udev rules
   sudo udevadm control --reload-rules
   sudo udevadm trigger
   
   # Log out and back in for group changes
   ```

2. **Windows Permission Issues:**
   ```bash
   # Run test framework as administrator
   # Or install proper HID drivers using Zadig
   ```

### Power and Battery Issues

#### Issue: Inconsistent Power Supply

**Symptoms:**
- Device resets during testing
- Inconsistent test results
- Battery monitoring inaccurate

**Diagnostic Steps:**
```bash
# Measure voltages with multimeter
echo "Battery voltage: $(measure_battery_voltage)V"  # Should be 3.0-4.2V
echo "VSYS voltage: $(measure_vsys_voltage)V"        # Should equal battery
echo "3V3 voltage: $(measure_3v3_voltage)V"         # Should be 3.3V ±0.1V

# Check voltage under load
# Measure during pEMF pulse generation
```

**Solutions:**
1. **Battery Issues:**
   ```bash
   # Replace weak battery
   # Check battery capacity (should be >500mAh)
   # Verify battery protection circuit
   # Check for battery swelling or damage
   ```

2. **Power Supply Decoupling:**
   ```bash
   # Add decoupling capacitors
   # 10µF electrolytic near VSYS
   # 100nF ceramic near 3V3
   # Check existing capacitor values
   ```

#### Issue: Battery Monitoring Inaccuracy

**Symptoms:**
- ADC readings don't match actual battery voltage
- Battery state detection incorrect
- Voltage readings drift over time

**Diagnostic Steps:**
```bash
# Measure voltage divider components
echo "R1 (10kΩ): $(measure_r1_resistance)Ω"  # Should be 9.9-10.1kΩ
echo "R2 (5.1kΩ): $(measure_r2_resistance)Ω"  # Should be 5.05-5.15kΩ

# Calculate actual scaling factor
scaling_factor = R2 / (R1 + R2)
echo "Scaling factor: $scaling_factor"  # Should be ~0.337

# Measure ADC input voltage
echo "GPIO26 voltage: $(measure_gpio26_voltage)V"
```

**Solutions:**
1. **Calibration Issues:**
   ```bash
   # Recalibrate ADC scaling in firmware
   # Update VOLTAGE_SCALE_FACTOR constant
   # Account for actual resistor values
   ```

2. **Hardware Issues:**
   ```bash
   # Replace resistors with 1% tolerance parts
   # Add filtering capacitor (100nF) at GPIO26
   # Check for loose connections
   # Verify ADC reference voltage (3.3V)
   ```

### MOSFET Driver Issues

#### Issue: No pEMF Output

**Symptoms:**
- No signal on MOSFET driver output
- Electromagnetic load not activating
- GPIO15 shows correct signal but no load response

**Diagnostic Steps:**
```bash
# Check GPIO15 signal with oscilloscope
# Should show 2Hz square wave, 2ms pulse width

# Check MOSFET driver power
echo "Driver VCC: $(measure_driver_vcc)V"  # Should be 3.3V
echo "Driver GND: Connected to system ground"

# Check driver input signal
echo "Driver IN: Should match GPIO15 signal"
```

**Solutions:**
1. **Driver Power Issues:**
   ```bash
   # Verify VCC connection to 3V3 or VSYS
   # Check GND connection
   # Ensure adequate current capacity
   ```

2. **Signal Issues:**
   ```bash
   # Check GPIO15 to driver IN connection
   # Add series resistor (100Ω) if needed
   # Verify driver enable pins (if present)
   ```

3. **Load Issues:**
   ```bash
   # Check load connections
   # Verify load current rating
   # Test with resistive load first
   # Check for short circuits in load
   ```

## Software and Firmware Issues

### Firmware Communication Problems

#### Issue: Device Not Responding to Commands

**Symptoms:**
- Commands sent but no response received
- Timeout errors in test framework
- Device appears to be running but not communicating

**Diagnostic Steps:**
```bash
# Test basic USB HID communication
python test_framework/device_manager.py --test-connection

# Check firmware version and status
python test_framework/command_handler.py --system-health

# Monitor USB traffic (Linux)
sudo usbmon
# Look for HID report traffic
```

**Solutions:**
1. **Firmware Issues:**
   ```bash
   # Reflash firmware
   cargo clean
   cargo build --release
   cargo run --release
   
   # Verify firmware version
   # Check for firmware corruption
   ```

2. **Protocol Issues:**
   ```bash
   # Verify command format
   # Check authentication tokens
   # Validate command parameters
   # Update test framework if needed
   ```

#### Issue: Firmware Crashes or Resets

**Symptoms:**
- Device resets during testing
- Panic messages in logs
- Inconsistent behavior

**Diagnostic Steps:**
```bash
# Enable debug logging in firmware
# Monitor for panic messages
# Check memory usage
cargo size --release

# Look for stack overflow
# Check for infinite loops
# Verify interrupt handling
```

**Solutions:**
1. **Memory Issues:**
   ```bash
   # Reduce memory usage
   # Check for memory leaks
   # Optimize data structures
   # Increase stack size if needed
   ```

2. **Timing Issues:**
   ```bash
   # Review RTIC task priorities
   # Check for priority inversion
   # Verify interrupt handling
   # Add proper synchronization
   ```

### Build and Compilation Issues

#### Issue: Compilation Errors

**Symptoms:**
- `cargo build` fails
- Linker errors
- Missing dependencies

**Common Solutions:**
```bash
# Update Rust toolchain
rustup update

# Clean and rebuild
cargo clean
cargo build --release

# Check target installation
rustup target add thumbv6m-none-eabi

# Verify dependencies
cargo check
```

#### Issue: Flashing Problems

**Symptoms:**
- UF2 file not copying to device
- Device not entering bootloader mode
- Flash verification failures

**Solutions:**
```bash
# Verify bootloader mode entry
# 1. Hold BOOTSEL button
# 2. Connect USB
# 3. Release BOOTSEL
# 4. Check for RPI-RP2 drive

# Try manual UF2 flashing
elf2uf2-rs target/thumbv6m-none-eabi/release/firmware
cp firmware.uf2 /path/to/RPI-RP2/

# Use probe-rs for SWD flashing
probe-rs run --chip RP2040 target/thumbv6m-none-eabi/release/firmware
```

## Communication Problems

### USB HID Communication Issues

#### Issue: Communication Timeouts

**Symptoms:**
- Commands timeout waiting for response
- Intermittent communication failures
- Long delays in response

**Diagnostic Steps:**
```bash
# Test communication latency
python test_framework/command_handler.py --latency-test

# Monitor USB traffic
# Linux: usbmon, Wireshark
# Windows: USBPcap, Wireshark

# Check for USB errors
dmesg | grep -i usb
```

**Solutions:**
1. **Timeout Configuration:**
   ```python
   # Increase timeout values in test framework
   COMMAND_TIMEOUT = 10.0  # Increase from 5.0
   RESPONSE_TIMEOUT = 20.0  # Increase from 10.0
   ```

2. **USB Issues:**
   ```bash
   # Try different USB port
   # Use shorter USB cable
   # Check for USB hub issues
   # Test on different computer
   ```

#### Issue: Data Corruption

**Symptoms:**
- Invalid responses received
- Checksum failures
- Garbled data

**Solutions:**
1. **Protocol Validation:**
   ```python
   # Enable verbose logging
   logging.basicConfig(level=logging.DEBUG)
   
   # Verify command format
   # Check response parsing
   # Validate checksums
   ```

2. **Hardware Issues:**
   ```bash
   # Check USB cable quality
   # Test with different cable
   # Verify USB connector integrity
   # Check for electromagnetic interference
   ```

### Multi-Device Communication

#### Issue: Device Identification Problems

**Symptoms:**
- Wrong device selected for testing
- Commands sent to incorrect device
- Device enumeration failures

**Solutions:**
```python
# Improve device identification
def identify_device(device_info):
    # Use serial number for unique identification
    # Verify device capabilities
    # Check firmware version
    return device_info.serial_number

# Implement device validation
def validate_device(device):
    # Send identification command
    # Verify expected response
    # Check device capabilities
    pass
```

## Test Framework Issues

### Test Execution Problems

#### Issue: Test Failures

**Symptoms:**
- Tests fail unexpectedly
- Inconsistent test results
- False positive/negative results

**Diagnostic Approach:**
```python
# Enable detailed logging
import logging
logging.basicConfig(level=logging.DEBUG)

# Run single test in isolation
python test_framework/test_scenarios.py --test single_test --verbose

# Check test parameters
# Verify expected vs actual results
# Review test logic
```

**Solutions:**
1. **Test Parameter Issues:**
   ```python
   # Adjust test parameters for hardware
   test_params = {
       'duration_ms': 5000,  # Increase duration
       'tolerance_percent': 2.0,  # Relax tolerance
       'retry_count': 3  # Add retries
   }
   ```

2. **Environmental Issues:**
   ```bash
   # Reduce electromagnetic interference
   # Ensure stable power supply
   # Control ambient temperature
   # Minimize vibration
   ```

#### Issue: Test Framework Crashes

**Symptoms:**
- Python exceptions during test execution
- Framework hangs or becomes unresponsive
- Memory errors

**Solutions:**
```python
# Add exception handling
try:
    result = execute_test(test_config)
except Exception as e:
    logging.error(f"Test execution failed: {e}")
    # Implement recovery logic

# Add timeout protection
import signal
def timeout_handler(signum, frame):
    raise TimeoutError("Test execution timeout")

signal.signal(signal.SIGALRM, timeout_handler)
signal.alarm(test_timeout)
```

### Result Collection Issues

#### Issue: Incomplete or Missing Results

**Symptoms:**
- Test results not saved
- Missing performance data
- Report generation failures

**Solutions:**
```python
# Implement robust result collection
class ResultCollector:
    def __init__(self):
        self.results = []
        self.backup_file = "test_results_backup.json"
    
    def collect_result(self, result):
        self.results.append(result)
        # Immediately backup to file
        self.save_backup()
    
    def save_backup(self):
        with open(self.backup_file, 'w') as f:
            json.dump(self.results, f, indent=2)
```

## Performance and Timing Issues

### Timing Accuracy Problems

#### Issue: pEMF Timing Inaccuracy

**Symptoms:**
- Measured frequency not 2.00Hz
- Pulse width not 2.0ms
- Timing drift over time

**Diagnostic Steps:**
```bash
# Measure with oscilloscope
# Check frequency: Should be 2.00Hz ± 0.02Hz
# Check pulse width: Should be 2.0ms ± 0.02ms
# Monitor for drift over time

# Check crystal oscillator
# Verify 12MHz external crystal
# Check crystal load capacitors
# Measure crystal frequency accuracy
```

**Solutions:**
1. **Hardware Issues:**
   ```bash
   # Replace crystal oscillator
   # Check crystal load capacitors (typically 22pF)
   # Verify crystal connections
   # Check for electromagnetic interference
   ```

2. **Software Issues:**
   ```rust
   // Verify timer configuration
   const TIMER_FREQUENCY: u32 = 1_000_000; // 1MHz
   const PULSE_HIGH_TICKS: u32 = 2_000;    // 2ms
   const PULSE_LOW_TICKS: u32 = 498_000;   // 498ms
   
   // Check RTIC task priorities
   // Ensure timing-critical tasks have highest priority
   ```

#### Issue: System Performance Degradation

**Symptoms:**
- Slower response times
- Increased latency
- Resource exhaustion

**Diagnostic Steps:**
```bash
# Monitor resource usage
python test_framework/real_time_monitor.py --resource-monitoring

# Check memory usage
cargo size --release
cargo bloat --release

# Profile performance
# Use timing measurements
# Monitor task execution times
```

**Solutions:**
1. **Optimization:**
   ```rust
   // Optimize critical paths
   // Reduce memory allocations
   // Use more efficient algorithms
   // Profile and optimize hot spots
   ```

2. **Resource Management:**
   ```rust
   // Implement proper resource cleanup
   // Use static allocation where possible
   // Monitor stack usage
   // Optimize interrupt handling
   ```

## Environment and Setup Issues

### Development Environment Problems

#### Issue: Inconsistent Build Environment

**Symptoms:**
- Builds work on one machine but not another
- Different behavior between debug and release
- Version compatibility issues

**Solutions:**
```bash
# Standardize environment
# Use specific Rust version
rustup install 1.70.0
rustup default 1.70.0

# Pin dependency versions in Cargo.toml
[dependencies]
rp2040-hal = "=0.11.0"
rtic = "=2.2.0"

# Document exact versions used
rustc --version > build_environment.txt
cargo --version >> build_environment.txt
```

#### Issue: Platform-Specific Issues

**Symptoms:**
- Works on Linux but not Windows
- Different USB behavior on different platforms
- Path or permission issues

**Solutions:**
```python
# Add platform detection
import platform
import os

def get_platform_config():
    system = platform.system()
    if system == "Linux":
        return LinuxConfig()
    elif system == "Windows":
        return WindowsConfig()
    elif system == "Darwin":
        return MacOSConfig()
    else:
        raise UnsupportedPlatformError(f"Platform {system} not supported")
```

### CI/CD Integration Issues

#### Issue: Tests Pass Locally but Fail in CI

**Symptoms:**
- Local tests pass but CI fails
- Timing-sensitive tests fail in CI
- Resource constraints in CI environment

**Solutions:**
```yaml
# Adjust CI configuration
# .github/workflows/ci.yml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - name: Setup hardware simulation
      run: |
        # Setup virtual hardware or mocking
        # Adjust timeouts for CI environment
        export CI_ENVIRONMENT=true
        export TEST_TIMEOUT_MULTIPLIER=2.0
```

## Advanced Debugging Techniques

### Hardware Debugging

#### Using Oscilloscope for Signal Analysis

```bash
# Key signals to monitor:
# 1. GPIO15 (pEMF control) - Should show 2Hz square wave
# 2. GPIO26 (ADC input) - Should show stable DC voltage
# 3. Power rails (3V3, VSYS) - Should be clean and stable
# 4. Crystal oscillator - Should show 12MHz sine wave

# Oscilloscope settings:
# Timebase: 100ms/div for pEMF signal
# Voltage: 1V/div for digital signals
# Trigger: Rising edge on GPIO15
# Measurements: Frequency, pulse width, rise/fall times
```

#### Using Logic Analyzer for Protocol Analysis

```bash
# Monitor USB HID communication:
# 1. Connect logic analyzer to USB D+ and D- lines
# 2. Set sample rate to 12MHz minimum
# 3. Decode USB protocol
# 4. Look for HID reports and timing

# Key things to check:
# - USB enumeration sequence
# - HID report structure
# - Command/response timing
# - Error conditions
```

### Software Debugging

#### Using probe-rs for Firmware Debugging

```bash
# Start GDB server
probe-rs gdb --chip RP2040 target/thumbv6m-none-eabi/release/firmware

# In another terminal, connect with GDB
arm-none-eabi-gdb target/thumbv6m-none-eabi/release/firmware
(gdb) target remote :1337
(gdb) load
(gdb) break main
(gdb) continue

# Useful GDB commands:
# info registers - Show CPU registers
# backtrace - Show call stack
# print variable - Show variable value
# step - Single step execution
```

#### Using RTT for Real-time Logging

```rust
// Add to Cargo.toml
[dependencies]
rtt-target = "0.4"

// In firmware code
use rtt_target::{rprintln, rtt_init_print};

#[rtic::app(device = rp2040_hal::pac, peripherals = true)]
mod app {
    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();
        rprintln!("Firmware starting...");
        // ... rest of init
    }
}
```

```bash
# Monitor RTT output
probe-rs attach --chip RP2040
# RTT output will appear in terminal
```

### Test Framework Debugging

#### Verbose Logging and Tracing

```python
# Enable comprehensive logging
import logging
import sys

# Configure logging
logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('test_debug.log'),
        logging.StreamHandler(sys.stdout)
    ]
)

# Add timing information
import time
import functools

def timing_decorator(func):
    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        start_time = time.time()
        result = func(*args, **kwargs)
        end_time = time.time()
        logging.debug(f"{func.__name__} took {end_time - start_time:.3f} seconds")
        return result
    return wrapper

# Apply to critical functions
@timing_decorator
def send_command(self, command):
    # ... implementation
```

#### Network and USB Traffic Analysis

```bash
# Monitor USB traffic on Linux
sudo modprobe usbmon
sudo tcpdump -i usbmon1 -w usb_traffic.pcap

# Analyze with Wireshark
wireshark usb_traffic.pcap

# Look for:
# - USB enumeration
# - HID report transfers
# - Error conditions
# - Timing issues
```

### Creating Reproducible Test Cases

#### Minimal Reproduction Scripts

```python
#!/usr/bin/env python3
"""
Minimal reproduction script for issue #XXX
"""

import sys
import logging
from test_framework import UsbHidDeviceManager, CommandHandler

def reproduce_issue():
    """Reproduce the specific issue with minimal code"""
    
    # Setup logging
    logging.basicConfig(level=logging.DEBUG)
    
    # Initialize components
    device_manager = UsbHidDeviceManager()
    command_handler = CommandHandler(device_manager)
    
    # Discover device
    devices = device_manager.discover_devices()
    if not devices:
        print("No devices found")
        return False
    
    # Connect to first device
    device = devices[0]
    success = device_manager.connect_device(device.serial_number)
    if not success:
        print(f"Failed to connect to device {device.serial_number}")
        return False
    
    # Reproduce the issue
    try:
        # Add specific steps that reproduce the issue
        command = command_handler.create_system_state_query('system_health')
        response = command_handler.send_command_and_wait(device.serial_number, command)
        print(f"Response: {response}")
        return True
        
    except Exception as e:
        print(f"Issue reproduced: {e}")
        return False

if __name__ == "__main__":
    success = reproduce_issue()
    sys.exit(0 if success else 1)
```

### Documentation for Issue Reports

#### Issue Report Template

```markdown
## Issue Description
Brief description of the problem

## Environment
- OS: [Linux/Windows/macOS version]
- Python version: [version]
- Rust version: [version]
- Hardware: [Raspberry Pi Pico version]
- Test framework version: [version]

## Steps to Reproduce
1. Step 1
2. Step 2
3. Step 3

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Logs and Error Messages
```
[Include relevant logs]
```

## Additional Information
- Oscilloscope traces (if applicable)
- USB traffic captures (if applicable)
- Hardware photos (if applicable)

## Attempted Solutions
- What has been tried
- Results of attempted fixes
```

---

This troubleshooting guide provides comprehensive coverage of common issues and their solutions. When encountering problems, start with the quick diagnostic checklist and then dive into the specific category that matches your symptoms. Remember to document any new issues and solutions for future reference.