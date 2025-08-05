# Development Environment Setup Guide

This comprehensive guide covers setting up the complete development environment for the RP2040 pEMF/Battery Monitoring Device with automated testing capabilities.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Hardware Setup](#hardware-setup)
3. [Software Environment Setup](#software-environment-setup)
4. [Test Framework Installation](#test-framework-installation)
5. [Development Workflow](#development-workflow)
6. [Validation and Testing](#validation-and-testing)
7. [Troubleshooting](#troubleshooting)

## Prerequisites

### System Requirements

- **Operating System**: Windows 10/11, macOS 10.15+, or Linux (Ubuntu 20.04+ recommended)
- **RAM**: Minimum 4GB, 8GB recommended
- **Storage**: 2GB free space for development tools and dependencies
- **USB Ports**: At least 2 USB-A ports (one for device, one for debugger if used)
- **Internet Connection**: Required for downloading tools and dependencies

### Hardware Requirements

#### Essential Components
- Raspberry Pi Pico (RP2040-based microcontroller)
- USB-A to Micro-USB cable
- 3.7V LiPo battery (500mAh minimum)
- MOSFET driver module (logic-level, 3.3V compatible)
- Resistors: 10kΩ and 5.1kΩ (1% tolerance recommended)
- Breadboard or PCB for prototyping

#### Optional Components
- SWD debugger (Picoprobe or compatible)
- Oscilloscope for timing verification
- Multimeter for voltage measurements
- Logic analyzer for protocol debugging

### Knowledge Prerequisites

- Basic understanding of embedded systems
- Familiarity with Rust programming language
- Understanding of USB HID protocol (helpful)
- Experience with Python for test automation

## Hardware Setup

### Step 1: Basic Wiring

Follow the detailed wiring instructions in [WIRING_GUIDE.md](../hardware/WIRING_GUIDE.md) to connect:

1. **Power connections**: Battery to VSYS and GND
2. **Voltage divider**: 10kΩ and 5.1kΩ resistors for battery monitoring
3. **MOSFET driver**: GPIO15 to driver input, power connections
4. **ADC input**: Voltage divider output to GPIO26

### Step 2: Hardware Validation

Before proceeding with software setup, validate hardware connections:

```bash
# Use multimeter to verify:
# - Battery voltage: 3.0V - 4.2V
# - Voltage divider output: ~1/3 of battery voltage
# - No shorts between VCC and GND
# - Continuity of all signal connections
```

### Step 3: Initial Device Test

1. Connect Raspberry Pi Pico via USB
2. Verify it appears as a USB device
3. Test bootloader mode (hold BOOTSEL, connect USB, release BOOTSEL)
4. Confirm "RPI-RP2" drive appears

## Software Environment Setup

### Step 1: Install Rust Development Environment

#### Install Rust and Cargo

```bash
# Install rustup (Rust installer and version manager)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Restart terminal or source environment
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

#### Add ARM Cortex-M Target

```bash
# Add target for RP2040 (ARM Cortex-M0+)
rustup target add thumbv6m-none-eabi

# Verify target is installed
rustup target list --installed
```

### Step 2: Install Required Tools

#### Install elf2uf2-rs (UF2 Flashing Tool)

```bash
# Install UF2 conversion tool
cargo install elf2uf2-rs

# Verify installation
elf2uf2-rs --version
```

#### Install probe-rs (Optional, for Advanced Debugging)

```bash
# Install probe-rs for SWD debugging
cargo install probe-rs --features cli

# Verify installation
probe-rs --version
```

### Step 3: Platform-Specific Setup

#### Linux Setup

```bash
# Install required system packages
sudo apt update
sudo apt install build-essential pkg-config libudev-dev

# Add user to dialout group for USB device access
sudo usermod -a -G dialout $USER

# Install additional dependencies for HID support
sudo apt install libhidapi-dev libusb-1.0-0-dev

# Log out and back in for group changes to take effect
```

#### macOS Setup

```bash
# Install Xcode command line tools
xcode-select --install

# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install required packages
brew install libusb hidapi
```

#### Windows Setup

1. Install Visual Studio Build Tools or Visual Studio Community
2. Ensure "C++ build tools" workload is selected
3. Install Windows SDK
4. Install libusb drivers using Zadig tool (for HID support)

### Step 4: Clone and Setup Project

```bash
# Clone the repository
git clone <repository-url>
cd pico-pemf-device

# Verify project structure
ls -la

# Test build environment
cargo check
```

## Test Framework Installation

### Step 1: Python Environment Setup

#### Install Python and Virtual Environment

```bash
# Ensure Python 3.7+ is installed
python3 --version

# Create virtual environment for the project
python3 -m venv venv

# Activate virtual environment
# Linux/macOS:
source venv/bin/activate
# Windows:
venv\Scripts\activate
```

#### Install Python Dependencies

```bash
# Navigate to test framework directory
cd test_framework

# Install required packages
pip install -r requirements.txt

# Verify installation
python -c "import hid; print('HID library installed successfully')"
```

### Step 2: Test Framework Configuration

#### Configure Device Parameters

Edit `test_framework/config.py` if it exists, or create it:

```python
# Device identification
VENDOR_ID = 0x2E8A  # Raspberry Pi Foundation
PRODUCT_ID = 0x000A  # RP2040 USB HID
BOOTLOADER_PRODUCT_ID = 0x0003  # RP2040 Bootloader

# Communication settings
COMMAND_TIMEOUT = 5.0  # seconds
RESPONSE_TIMEOUT = 10.0  # seconds
DISCOVERY_INTERVAL = 1.0  # seconds

# Test parameters
DEFAULT_TEST_TIMEOUT = 30.0  # seconds
MAX_RETRY_COUNT = 3
```

#### Validate Test Framework Installation

```bash
# Run basic framework tests
python test_framework/run_tests.py

# Test device discovery (with device connected)
python test_framework/example_usage.py --discover-only
```

## Development Workflow

### Step 1: Firmware Development Cycle

#### Build and Flash Firmware

```bash
# Clean build (recommended for first build)
cargo clean
cargo build --release

# Flash to device (ensure device is in bootloader mode)
cargo run --release

# Alternative: Manual UF2 flashing
elf2uf2-rs target/thumbv6m-none-eabi/release/pico-pemf-device
# Copy generated .uf2 file to RPI-RP2 drive
```

#### Monitor Device Operation

```bash
# Monitor serial output (if available)
probe-rs attach --chip RP2040

# Monitor USB HID communication
python test_framework/enhanced_monitoring_demo.py
```

### Step 2: Automated Testing Workflow

#### Run Basic Validation Tests

```bash
# Run comprehensive test suite
python test_framework/comprehensive_test_runner.py

# Run specific test scenarios
python test_framework/test_scenarios.py --test pemf_timing_validation

# Generate detailed reports
python test_framework/report_generator.py --format all
```

#### Firmware Flashing and Testing

```bash
# Automated firmware flash and test cycle
python test_framework/firmware_flash_example.py --firmware path/to/firmware.uf2

# Run regression tests after firmware update
python test_framework/comprehensive_test_runner.py --regression-mode
```

### Step 3: Development Best Practices

#### Code Quality Checks

```bash
# Run Rust code formatting
cargo fmt

# Run Rust linting
cargo clippy

# Run Python code formatting
cd test_framework
black *.py
flake8 *.py
```

#### Testing Before Commit

```bash
# Build firmware in both debug and release modes
cargo build
cargo build --release

# Run all unit tests
cargo test

# Run integration tests with hardware
python test_framework/run_tests.py --integration

# Verify no compiler warnings
cargo build --release 2>&1 | grep -i warning
```

## Validation and Testing

### Step 1: Hardware Validation

#### Basic Functionality Test

```bash
# Test basic device communication
python test_framework/device_manager.py --test-connection

# Validate hardware setup
python scripts/validation/hardware_validation.py
```

#### Timing Accuracy Validation

```bash
# Test pEMF timing accuracy
python test_framework/test_scenarios.py --test pemf_timing_validation --duration 60

# Validate battery monitoring accuracy
python test_framework/test_scenarios.py --test battery_adc_calibration
```

### Step 2: Software Validation

#### Unit Test Execution

```bash
# Run Rust unit tests
cargo test --lib

# Run Python unit tests
cd test_framework
python -m pytest tests/ -v
```

#### Integration Test Execution

```bash
# Run hardware-in-the-loop tests
python test_framework/comprehensive_test_runner.py --hardware-tests

# Test multi-device scenarios (if multiple devices available)
python test_framework/comprehensive_test_runner.py --multi-device
```

### Step 3: Performance Validation

#### Memory and Resource Usage

```bash
# Check firmware binary size
cargo size --release

# Analyze memory usage
cargo bloat --release

# Monitor runtime resource usage
python test_framework/real_time_monitor.py --resource-monitoring
```

#### Timing Performance

```bash
# Validate real-time constraints
python test_framework/test_scenarios.py --test system_stress_test --duration 300

# Long-term stability testing
python test_framework/comprehensive_test_runner.py --stability-test --duration 3600
```

## Troubleshooting

### Common Setup Issues

#### Rust Installation Problems

**Issue**: "cargo: command not found"
```bash
# Solution: Restart terminal or source environment
source ~/.cargo/env

# Or reinstall Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Issue**: "linker `rust-lld` not found"
```bash
# Solution: Reinstall Rust toolchain
rustup toolchain install stable
rustup default stable
```

#### Hardware Connection Issues

**Issue**: Device not detected
```bash
# Check USB connection
lsusb | grep -i "2e8a"

# Check permissions (Linux)
ls -l /dev/ttyACM*
sudo chmod 666 /dev/ttyACM0  # Temporary fix
```

**Issue**: Bootloader mode not working
```bash
# Verify bootloader entry procedure:
# 1. Hold BOOTSEL button
# 2. Connect USB cable
# 3. Release BOOTSEL button
# 4. Check for RPI-RP2 drive
```

#### Python/Test Framework Issues

**Issue**: "No module named 'hid'"
```bash
# Install HID library dependencies
# Linux:
sudo apt install libhidapi-dev
pip install hidapi

# macOS:
brew install hidapi
pip install hidapi

# Windows:
# Use pre-compiled wheels or install Visual Studio Build Tools
```

**Issue**: Permission denied accessing USB device
```bash
# Linux: Add user to dialout group
sudo usermod -a -G dialout $USER
# Log out and back in

# Or create udev rule for HID devices
sudo nano /etc/udev/rules.d/99-hid.rules
# Add: SUBSYSTEM=="hidraw", ATTRS{idVendor}=="2e8a", MODE="0666"
sudo udevadm control --reload-rules
```

### Development Workflow Issues

#### Build Failures

**Issue**: "can't find crate for `core`"
```bash
# Ensure target is installed
rustup target add thumbv6m-none-eabi
rustup target list --installed
```

**Issue**: Linking errors
```bash
# Check memory.x file exists and is correct
cat memory.x

# Verify .cargo/config.toml is properly configured
cat .cargo/config.toml
```

#### Runtime Issues

**Issue**: Device resets unexpectedly
```bash
# Check power supply stability
# Verify battery voltage and capacity
# Add decoupling capacitors if needed
# Check for electromagnetic interference
```

**Issue**: Incorrect timing behavior
```bash
# Verify crystal oscillator (12MHz)
# Check clock configuration in software
# Use oscilloscope to measure actual timing
# Validate RTIC task priorities
```

### Test Framework Issues

#### Communication Problems

**Issue**: Command timeouts
```bash
# Increase timeout values in configuration
# Check USB connection stability
# Verify device is responding to commands
python test_framework/device_manager.py --ping-test
```

**Issue**: Inconsistent test results
```bash
# Check for electromagnetic interference
# Verify stable power supply
# Run tests with longer settling times
# Check for timing race conditions
```

### Getting Help

#### Documentation Resources

- [WIRING_GUIDE.md](../hardware/WIRING_GUIDE.md) - Detailed hardware setup
- [SOFTWARE_SETUP.md](SOFTWARE_SETUP.md) - Basic software installation
- [test_framework/README.md](../../test_framework/README.md) - Test framework documentation
- [USB_HID_TROUBLESHOOTING_GUIDE.md](../troubleshooting/USB_HID_TROUBLESHOOTING_GUIDE.md) - USB communication issues

#### Community Support

- Check project issues on repository
- Review existing documentation for similar problems
- Create detailed issue reports with:
  - System information (OS, versions)
  - Hardware configuration
  - Error messages and logs
  - Steps to reproduce

#### Debug Information Collection

When reporting issues, include:

```bash
# System information
uname -a
rustc --version
python --version

# Hardware information
lsusb
dmesg | tail -20

# Build information
cargo --version
cargo check 2>&1

# Test framework information
cd test_framework
pip list
python device_manager.py --version
```

---

This guide provides a comprehensive foundation for setting up the development environment. For specific implementation details, refer to the individual documentation files mentioned throughout this guide.