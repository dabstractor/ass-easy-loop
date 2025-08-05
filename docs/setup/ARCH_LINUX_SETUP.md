# Arch Linux Development Environment Setup Guide

This comprehensive guide covers setting up the complete development environment for the RP2040 pEMF/Battery Monitor Device on Arch Linux, including all tools needed for hardware validation testing.

## Table of Contents

1. [System Requirements](#system-requirements)
2. [Base Development Environment](#base-development-environment)
3. [USB HID Development Tools](#usb-hid-development-tools)
4. [Hardware Testing Tools](#hardware-testing-tools)
5. [Python Environment Setup](#python-environment-setup)
6. [Firmware Development Tools](#firmware-development-tools)
7. [Hardware Validation Setup](#hardware-validation-setup)
8. [Troubleshooting](#troubleshooting)
9. [Verification Steps](#verification-steps)

## System Requirements

### Minimum System Requirements
- **OS**: Arch Linux (current)
- **RAM**: 4GB minimum, 8GB recommended
- **Storage**: 2GB free space for development tools
- **USB**: USB 2.0 or higher port for device connection
- **Network**: Internet connection for package downloads

### Hardware Requirements
- Raspberry Pi Pico (RP2040) with USB HID logging firmware
- USB-A to Micro-USB cable
- Optional: Logic analyzer or oscilloscope for timing validation

## Base Development Environment

### Step 1: Update System

```bash
# Update package database and system
sudo pacman -Syu

# Install base development tools
sudo pacman -S base-devel git curl wget
```

### Step 2: Install Rust Development Environment

```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Source the environment
source ~/.cargo/env

# Add ARM Cortex-M target for RP2040
rustup target add thumbv6m-none-eabi

# Install embedded development tools
cargo install elf2uf2-rs
cargo install probe-rs --features cli

# Verify installation
rustc --version
cargo --version
probe-rs --version
```

### Step 3: Install System Libraries

```bash
# Install essential system libraries
sudo pacman -S pkg-config libudev systemd-libs

# Install USB development libraries
sudo pacman -S libusb hidapi

# Install Python development headers
sudo pacman -S python python-pip python-setuptools
```

## USB HID Development Tools

### Step 4: Install USB Utilities

```bash
# Install USB debugging and monitoring tools
sudo pacman -S usbutils lsusb

# Install HID-specific tools
sudo pacman -S hidapi python-hid

# Install additional USB debugging tools
sudo pacman -S usb-modeswitch usbview

# Optional: GUI USB analyzer
sudo pacman -S wireshark-qt  # For USB packet analysis
```

### Step 5: Configure USB Permissions

```bash
# Add user to required groups
sudo usermod -a -G uucp,lock,dialout $USER

# Create udev rules for RP2040 device access
sudo tee /etc/udev/rules.d/99-rp2040-hid.rules << 'EOF'
# RP2040 USB HID Logging Device
# Replace 1234:5678 with your actual VID:PID
SUBSYSTEM=="usb", ATTRS{idVendor}=="1234", ATTRS{idProduct}=="5678", MODE="0666", GROUP="dialout"
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="1234", ATTRS{idProduct}=="5678", MODE="0666", GROUP="dialout"

# RP2040 Bootloader (for flashing)
SUBSYSTEM=="usb", ATTRS{idVendor}=="2e8a", ATTRS{idProduct}=="0003", MODE="0666", GROUP="dialout"
EOF

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger

# Log out and back in for group changes to take effect
echo "Please log out and back in for group membership changes to take effect"
```

## Hardware Testing Tools

### Step 6: Install Testing and Measurement Tools

```bash
# Install Python testing framework
sudo pacman -S python-pytest python-pytest-cov

# Install scientific Python packages for data analysis
sudo pacman -S python-numpy python-scipy python-matplotlib

# Install serial communication tools
sudo pacman -S minicom screen picocom

# Install logic analyzer software (if using logic analyzer)
yay -S pulseview sigrok-cli  # Requires AUR helper

# Install oscilloscope software (if using USB oscilloscope)
yay -S openhantek6022  # For Hantek USB oscilloscopes
```

### Step 7: Install AUR Helper (if not already installed)

```bash
# Install yay AUR helper for additional packages
git clone https://aur.archlinux.org/yay.git
cd yay
makepkg -si
cd ..
rm -rf yay
```

## Python Environment Setup

### Step 8: Create Python Virtual Environment

```bash
# Create project directory
mkdir -p ~/rp2040-development
cd ~/rp2040-development

# Create virtual environment for the project
python -m venv venv

# Activate virtual environment
source venv/bin/activate

# Upgrade pip and install wheel
pip install --upgrade pip wheel setuptools
```

### Step 9: Install Python Dependencies

```bash
# Install HID communication libraries
pip install hidapi hid

# Install USB communication libraries
pip install pyusb pyserial

# Install testing and development tools
pip install pytest pytest-cov pytest-timeout

# Install data analysis tools
pip install numpy scipy matplotlib pandas

# Install JSON and configuration tools
pip install pyyaml toml

# Install development utilities
pip install black flake8 mypy

# Create requirements.txt for reproducibility
pip freeze > requirements.txt
```

## Firmware Development Tools

### Step 10: Install Embedded Development Tools

```bash
# Install cross-compilation tools
sudo pacman -S arm-none-eabi-gcc arm-none-eabi-gdb arm-none-eabi-newlib

# Install debugging tools
sudo pacman -S gdb-multiarch openocd

# Install additional embedded tools
cargo install cargo-binutils
rustup component add llvm-tools-preview

# Install memory analysis tools
cargo install cargo-bloat cargo-size
```

### Step 11: Configure Development Environment

```bash
# Create project workspace
mkdir -p ~/rp2040-projects
cd ~/rp2040-projects

# Clone the project repository (replace with actual URL)
git clone <repository-url> ass-easy-loop
cd ass-easy-loop

# Verify project builds
cargo check

# Test build for target
cargo build --release
```

## Hardware Validation Setup

### Step 12: Install Hardware Testing Dependencies

```bash
# Activate Python virtual environment
source ~/rp2040-development/venv/bin/activate

# Install additional testing libraries
pip install timeout-decorator psutil

# Install hardware interface libraries
pip install RPi.GPIO gpiozero  # If using Raspberry Pi for testing
pip install pyftdi  # For FTDI-based test equipment

# Install measurement and analysis tools
pip install sigrok  # For logic analyzer integration
```

### Step 13: Configure Hardware Test Environment

```bash
# Create hardware test configuration
mkdir -p ~/.config/rp2040-testing

# Create test configuration file
cat > ~/.config/rp2040-testing/config.toml << 'EOF'
[device]
vendor_id = 0x1234
product_id = 0x5678
timeout_seconds = 30

[timing]
pemf_frequency_hz = 2.0
pemf_high_duration_ms = 2
pemf_low_duration_ms = 498
tolerance_percent = 1.0

[battery]
low_threshold_adc = 1425
charging_threshold_adc = 1675
voltage_divider_ratio = 0.337

[testing]
log_capture_duration_s = 15
performance_test_duration_s = 30
connection_stability_cycles = 5
EOF

# Set up test data directory
mkdir -p ~/rp2040-testing/{logs,results,captures}
```

### Step 14: Install Hardware Validation Scripts

```bash
# Copy hardware validation test files to project
cd ~/rp2040-projects/ass-easy-loop

# Ensure test files are executable
chmod +x tests/hardware_validation_tests.rs

# Install Python test utilities
cat > hardware_test_runner.py << 'EOF'
#!/usr/bin/env python3
"""
Hardware Test Runner for RP2040 Device Validation
Provides command-line interface for running hardware tests
"""

import argparse
import subprocess
import sys
import time
from pathlib import Path

def run_rust_tests():
    """Run Rust hardware validation tests"""
    print("Running Rust hardware validation tests...")
    
    cmd = ["cargo", "test", "--test", "hardware_validation_tests", "--", "--nocapture"]
    result = subprocess.run(cmd, capture_output=False)
    
    return result.returncode == 0

def run_python_hid_test():
    """Run Python HID communication test"""
    print("Running Python HID communication test...")
    
    python_test = '''
import hid
import sys

try:
    # Try to enumerate HID devices
    devices = hid.enumerate(0x1234, 0x5678)
    if devices:
        print(f"Found {len(devices)} matching device(s)")
        for device in devices:
            print(f"  Path: {device['path']}")
            print(f"  Manufacturer: {device['manufacturer_string']}")
            print(f"  Product: {device['product_string']}")
        sys.exit(0)
    else:
        print("No matching HID devices found")
        sys.exit(1)
except Exception as e:
    print(f"HID test failed: {e}")
    sys.exit(1)
'''
    
    result = subprocess.run([sys.executable, "-c", python_test])
    return result.returncode == 0

def main():
    parser = argparse.ArgumentParser(description="RP2040 Hardware Test Runner")
    parser.add_argument("--rust-tests", action="store_true", help="Run Rust hardware tests")
    parser.add_argument("--python-tests", action="store_true", help="Run Python HID tests")
    parser.add_argument("--all", action="store_true", help="Run all tests")
    
    args = parser.parse_args()
    
    if not any([args.rust_tests, args.python_tests, args.all]):
        parser.print_help()
        return 1
    
    success = True
    
    if args.python_tests or args.all:
        success &= run_python_hid_test()
    
    if args.rust_tests or args.all:
        success &= run_rust_tests()
    
    if success:
        print("\n✓ All tests passed!")
        return 0
    else:
        print("\n✗ Some tests failed!")
        return 1

if __name__ == "__main__":
    sys.exit(main())
EOF

chmod +x hardware_test_runner.py
```

## Troubleshooting

### Common Issues and Solutions

#### Issue: "Permission denied" when accessing USB device

**Solution:**
```bash
# Check if user is in correct groups
groups $USER

# If not in dialout group:
sudo usermod -a -G dialout $USER

# Check udev rules
ls -la /etc/udev/rules.d/99-rp2040-hid.rules

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger

# Log out and back in
```

#### Issue: "hidapi not found" or import errors

**Solution:**
```bash
# Reinstall hidapi
sudo pacman -S hidapi python-hid

# Or in virtual environment:
pip uninstall hid hidapi
pip install hidapi hid

# Check library linking
python -c "import hid; print(hid.__file__)"
```

#### Issue: Device not found in lsusb

**Solution:**
```bash
# Check USB connection
lsusb

# Check if device is in bootloader mode
lsusb | grep "2e8a:0003"  # RP2040 bootloader

# If in bootloader mode, flash firmware:
# 1. Build firmware: cargo build --release
# 2. Convert to UF2: elf2uf2-rs target/thumbv6m-none-eabi/release/ass-easy-loop
# 3. Copy to device: cp ass-easy-loop.uf2 /media/RPI-RP2/
```

#### Issue: Rust compilation errors

**Solution:**
```bash
# Update Rust toolchain
rustup update

# Reinstall target
rustup target remove thumbv6m-none-eabi
rustup target add thumbv6m-none-eabi

# Clean and rebuild
cargo clean
cargo build --release
```

#### Issue: Python virtual environment issues

**Solution:**
```bash
# Remove and recreate virtual environment
rm -rf ~/rp2040-development/venv
cd ~/rp2040-development
python -m venv venv
source venv/bin/activate
pip install --upgrade pip
pip install -r requirements.txt
```

### System-Specific Issues

#### Wayland Display Server Issues

If using Wayland, some GUI tools may not work properly:

```bash
# Install X11 compatibility
sudo pacman -S xorg-xwayland

# Set environment variable for GUI tools
export QT_QPA_PLATFORM=wayland
export GDK_BACKEND=wayland
```

#### Firewall Issues

If having network connectivity issues:

```bash
# Check firewall status
sudo systemctl status ufw

# If needed, allow USB debugging
sudo ufw allow from 127.0.0.1
```

## Verification Steps

### Step 15: Verify Complete Installation

Run these commands to verify everything is installed correctly:

```bash
# 1. Check Rust environment
echo "=== Rust Environment ==="
rustc --version
cargo --version
rustup target list --installed | grep thumbv6m-none-eabi

# 2. Check USB tools
echo "=== USB Tools ==="
lsusb --version
which hidapi-info || echo "hidapi-info not found (optional)"

# 3. Check Python environment
echo "=== Python Environment ==="
cd ~/rp2040-development
source venv/bin/activate
python --version
pip list | grep -E "(hid|usb|pytest)"

# 4. Check system permissions
echo "=== System Permissions ==="
groups $USER | grep -E "(dialout|uucp)"
ls -la /etc/udev/rules.d/99-rp2040-hid.rules

# 5. Check project build
echo "=== Project Build ==="
cd ~/rp2040-projects/ass-easy-loop
cargo check

# 6. Check hardware connection (if device connected)
echo "=== Hardware Connection ==="
lsusb | grep -E "(1234:5678|2e8a:0003)" || echo "RP2040 device not connected"
```

### Step 16: Run Initial Hardware Tests

```bash
# Navigate to project directory
cd ~/rp2040-projects/ass-easy-loop

# Activate Python environment
source ~/rp2040-development/venv/bin/activate

# Run basic connection test
python3 hardware_test_runner.py --python-tests

# If device is connected and firmware is flashed, run full tests
python3 hardware_test_runner.py --all

# Run Rust hardware validation tests
cargo test --test hardware_validation_tests -- --nocapture
```

## Development Workflow

### Daily Development Setup

Create a setup script for daily use:

```bash
cat > ~/setup-rp2040-dev.sh << 'EOF'
#!/bin/bash
# RP2040 Development Environment Setup Script

echo "Setting up RP2040 development environment..."

# Activate Python virtual environment
source ~/rp2040-development/venv/bin/activate

# Navigate to project directory
cd ~/rp2040-projects/ass-easy-loop

# Check device connection
echo "Checking for RP2040 device..."
if lsusb | grep -q "1234:5678"; then
    echo "✓ RP2040 device found"
else
    echo "⚠ RP2040 device not found"
    echo "Connect device and ensure it's running USB HID firmware"
fi

# Show available commands
echo ""
echo "Available commands:"
echo "  cargo build --release          # Build firmware"
echo "  cargo test                     # Run unit tests"
echo "  python3 scripts/utilities/hidlog.py              # Monitor device logs"
echo "  python3 hardware_test_runner.py --all  # Run hardware tests"
echo ""
echo "Development environment ready!"
EOF

chmod +x ~/setup-rp2040-dev.sh
```

### Usage

```bash
# Run setup script
~/setup-rp2040-dev.sh

# Or manually:
source ~/rp2040-development/venv/bin/activate
cd ~/rp2040-projects/ass-easy-loop
```

## Next Steps

After completing this setup:

1. **Flash Firmware**: Build and flash the USB HID logging firmware to your RP2040
2. **Run Hardware Tests**: Execute the hardware validation test suite
3. **Develop and Test**: Use the development environment for firmware development
4. **Monitor Logs**: Use `hidlog.py` to monitor real-time device logs
5. **Validate Timing**: Use hardware tests to verify pEMF timing accuracy

## Additional Resources

- [RP2040 Datasheet](https://datasheets.raspberrypi.org/rp2040/rp2040-datasheet.pdf)
- [Rust Embedded Book](https://docs.rust-embedded.org/book/)
- [RTIC Framework Documentation](https://rtic.rs/)
- [USB HID Specification](https://www.usb.org/hid)
- [Arch Linux Wiki - USB](https://wiki.archlinux.org/title/USB)

---

This setup guide provides a complete development environment for RP2040 USB HID logging development on Arch Linux, including all tools needed for hardware validation testing.