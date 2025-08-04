# Firmware Flashing and Bootloader Integration

This document describes the automated firmware flashing and bootloader mode triggering capabilities added to the test framework.

## Overview

The firmware flashing functionality enables fully automated testing workflows by providing:

- **Bootloader Mode Triggering**: Remotely trigger devices to enter bootloader mode via USB HID commands
- **Automated Firmware Flashing**: Flash firmware using external tools with full workflow automation
- **Device Reconnection Detection**: Automatically detect when devices reconnect after firmware updates
- **Multi-Device Support**: Flash multiple devices in parallel for efficient CI/CD workflows
- **Timeout Handling**: Comprehensive timeout protection for all operations
- **CI/CD Integration**: Headless operation with proper exit codes and standard report formats

## Components

### FirmwareFlasher

The main class that orchestrates firmware flashing operations.

```python
from test_framework import FirmwareFlasher, UsbHidDeviceManager, CommandHandler

# Initialize components
device_manager = UsbHidDeviceManager()
command_handler = CommandHandler(device_manager)
flasher = FirmwareFlasher(device_manager, command_handler)

# Flash firmware to a single device
operation = flasher.flash_firmware("device_serial_123", "firmware.uf2")
if operation.result == FlashResult.SUCCESS:
    print("Firmware flashing successful!")
```

### Key Features

#### 1. Bootloader Mode Triggering

```python
# Trigger bootloader mode on a specific device
success = flasher.trigger_bootloader_mode("device_serial_123", timeout_ms=5000)
if success:
    print("Device entered bootloader mode")
```

#### 2. Multi-Device Parallel Flashing

```python
# Flash multiple devices in parallel
device_firmware_map = {
    "device_1": "firmware.uf2",
    "device_2": "firmware.uf2",
    "device_3": "firmware.uf2"
}

results = flasher.flash_multiple_devices(
    device_firmware_map, 
    parallel=True, 
    max_parallel=4
)

for device, operation in results.items():
    print(f"{device}: {operation.result.value}")
```

#### 3. Flash Tool Auto-Detection

The system automatically detects available firmware flashing tools:

- `picotool` (Official Raspberry Pi tool)
- `uf2conv.py` (UF2 conversion tool)
- `rp2040load` (Alternative loader)

```python
# Verify flash tool availability
if flasher.verify_flash_tool_availability():
    print(f"Flash tool available: {flasher.flash_tool_path}")
    formats = flasher.get_supported_firmware_formats()
    print(f"Supported formats: {formats}")
```

## Usage Examples

### Basic Firmware Flashing

```bash
# Flash firmware to all connected devices
python test_framework/firmware_flash_example.py --firmware firmware.uf2

# Flash specific device
python test_framework/firmware_flash_example.py --firmware firmware.uf2 --device device_123

# Test bootloader triggering without flashing
python test_framework/firmware_flash_example.py --test-bootloader

# Enable verbose logging
python test_framework/firmware_flash_example.py --firmware firmware.uf2 --verbose
```

### CI/CD Integration

```bash
# Run CI test suite with firmware flashing
python test_framework/ci_integration_example.py \
    --firmware firmware.uf2 \
    --devices 2 \
    --timeout 300 \
    --junit-xml \
    --json-report \
    --output-dir test_results

# Headless operation for CI pipelines
python test_framework/ci_integration_example.py \
    --config ci_config.json \
    --firmware firmware.uf2 \
    --devices 4 \
    --junit-xml \
    --output-dir /tmp/test_results
echo "Exit code: $?"
```

### Configuration File Format

Create a JSON configuration file for CI testing:

```json
{
  "name": "Production Validation Suite",
  "description": "Comprehensive validation for production firmware",
  "parallel_execution": true,
  "max_parallel_devices": 8,
  "global_timeout": 600.0,
  "steps": [
    {
      "name": "communication_test",
      "test_type": 5,
      "parameters": {"message_count": 10, "timeout_ms": 1000},
      "timeout": 15.0,
      "required": true
    },
    {
      "name": "pemf_validation",
      "test_type": 1,
      "parameters": {"duration_ms": 5000, "tolerance_percent": 1.0},
      "timeout": 20.0,
      "required": true,
      "depends_on": ["communication_test"]
    }
  ]
}
```

## Workflow Details

### Complete Firmware Flashing Workflow

1. **Device Discovery**: Automatically discover connected devices
2. **Bootloader Entry**: Send USB HID command to trigger bootloader mode
3. **Device Disconnection**: Wait for device to disconnect from normal mode
4. **Bootloader Detection**: Detect device reappearing in bootloader mode
5. **Firmware Flashing**: Execute external flashing tool (picotool, etc.)
6. **Reconnection Wait**: Wait for device to reconnect in normal mode
7. **Connection Verification**: Verify device is operational

### Timeout Handling

- **Bootloader Entry Timeout**: Default 10 seconds
- **Firmware Flash Timeout**: Default 60 seconds per device
- **Reconnection Timeout**: Default 30 seconds
- **Global Test Timeout**: Configurable for entire test suite

### Error Recovery

The system provides comprehensive error handling:

- **Bootloader Entry Failed**: Device remains in normal operation
- **Flash Operation Failed**: Detailed error reporting with tool output
- **Reconnection Failed**: Timeout with diagnostic information
- **Multi-Device Failures**: Continue with remaining devices

## CI/CD Integration Features

### Exit Codes

- `0`: All operations successful
- `1`: Test failures or operational errors
- `130`: User cancellation (SIGINT)

### Report Formats

#### JUnit XML
```xml
<testsuites name="CI Validation Suite" tests="10" failures="0" time="45.123">
  <testsuite name="Device_123" tests="5" failures="0" time="22.456">
    <testcase classname="Device_123" name="communication_test" time="2.123"/>
    <testcase classname="Device_123" name="pemf_validation" time="5.234"/>
  </testsuite>
</testsuites>
```

#### JSON Report
```json
{
  "test_suite": {
    "name": "CI Validation Suite",
    "start_time": 1640995200.0,
    "end_time": 1640995245.123,
    "duration": 45.123
  },
  "devices": {
    "device_123": {
      "tests": [
        {
          "name": "communication_test",
          "status": "completed",
          "duration": 2.123,
          "error_message": null
        }
      ]
    }
  },
  "summary": {
    "total_tests": 10,
    "passed": 10,
    "failed": 0,
    "timeout": 0
  }
}
```

## Requirements Satisfied

This implementation satisfies the following requirements:

- **Requirement 4.3**: Framework triggers bootloader mode, flashes firmware, and verifies deployment
- **Requirement 4.4**: Collects diagnostic logs and system state when tests fail
- **Requirement 7.1**: Supports headless operation with proper exit codes for CI/CD
- **Requirement 7.2**: Supports parallel testing with multiple devices in CI environments

## Dependencies

### Python Packages
- `hidapi` or `hid` for USB HID communication
- `concurrent.futures` for parallel operations (built-in)
- `subprocess` for external tool execution (built-in)

### External Tools
- `picotool` (recommended) - Official Raspberry Pi RP2040 tool
- `uf2conv.py` - UF2 format conversion tool
- `rp2040load` - Alternative RP2040 loader

### Installation

```bash
# Install Python dependencies
pip install hidapi

# Install picotool (Ubuntu/Debian)
sudo apt install picotool

# Or build from source
git clone https://github.com/raspberrypi/picotool.git
cd picotool
mkdir build && cd build
cmake ..
make -j4
sudo make install
```

## Troubleshooting

### Common Issues

1. **No Flash Tool Detected**
   - Install picotool or specify path with `--flash-tool`
   - Verify tool is in PATH: `which picotool`

2. **Device Not Entering Bootloader Mode**
   - Check device firmware supports bootloader commands
   - Verify USB HID communication is working
   - Try manual BOOTSEL button test

3. **Firmware Flash Fails**
   - Check firmware file format (.uf2, .elf, .bin)
   - Verify file permissions and path
   - Check available disk space

4. **Device Reconnection Timeout**
   - Increase reconnection timeout
   - Check USB cable and connections
   - Verify firmware is valid and boots correctly

### Debug Mode

Enable verbose logging for detailed troubleshooting:

```bash
python test_framework/firmware_flash_example.py --firmware firmware.uf2 --verbose
```

This provides detailed logs of:
- USB HID communication
- External tool execution
- Device state transitions
- Timing information

## Integration with Existing Tests

The firmware flashing functionality integrates seamlessly with existing test infrastructure:

```python
from test_framework import (
    UsbHidDeviceManager, CommandHandler, TestSequencer, 
    FirmwareFlasher, TestConfiguration
)

# Initialize all components
device_manager = UsbHidDeviceManager()
command_handler = CommandHandler(device_manager)
test_sequencer = TestSequencer(device_manager, command_handler)
firmware_flasher = FirmwareFlasher(device_manager, command_handler)

# Discover devices
devices = device_manager.discover_devices()
connected_devices = [d.serial_number for d in devices if d.status.value == "connected"]

# Flash firmware
flash_results = firmware_flasher.flash_multiple_devices({
    device: "firmware.uf2" for device in connected_devices
})

# Run tests
test_config = test_sequencer.create_basic_validation_config()
test_results = test_sequencer.execute_test_sequence(test_config, connected_devices)

# Analyze results
for device, executions in test_results.items():
    print(f"Device {device}: {len([e for e in executions if e.status.value == 'completed'])} tests passed")
```