# USB HID Log Receiver Usage Guide

This document explains how to use the `hidlog.py` utility to receive and display log messages from the RP2040 pEMF/Battery monitoring device.

## Installation

1. Install Python dependencies:
```bash
pip3 install -r requirements.txt
```

2. On Arch Linux, you may also need:
```bash
sudo pacman -S python-hid libusb hidapi
```

## Basic Usage

### List Available Devices
```bash
python3 hidlog.py --list
```

### Monitor All Log Messages
```bash
python3 hidlog.py
```

### Filter by Log Level
```bash
# Show only INFO level and above (INFO, WARN, ERROR)
python3 hidlog.py --level INFO

# Show only errors
python3 hidlog.py --level ERROR
```

### Filter by Module
```bash
# Show only battery-related logs
python3 hidlog.py --module BATTERY

# Show only pEMF-related logs
python3 hidlog.py --module PEMF
```

### Connect to Specific Device
```bash
# Connect to specific device path
python3 hidlog.py --device /dev/hidraw0

# Connect to device by serial number
python3 hidlog.py --serial 123456789

# Use custom VID/PID
python3 hidlog.py --vid 0x1234 --pid 0x5678
```

### Save Logs to File
```bash
# Save logs to file with timestamps
python3 hidlog.py --log-file logs/device.log

# Combine with filtering
python3 hidlog.py --level ERROR --log-file error.log
```

### JSON Output
```bash
# Output logs in JSON format for programmatic processing
python3 hidlog.py --json-output

# Save JSON logs to file
python3 hidlog.py --json-output --log-file logs.json
```

### Debug Mode
```bash
# Show raw message data for debugging
python3 hidlog.py --raw

# Inspect raw HID reports for low-level debugging
python3 hidlog.py --inspect-hid
```

## Log Message Format

Log messages are displayed in the following format:
```
[TIMESTAMP] [LEVEL] [MODULE ] MESSAGE
```

Where:
- **TIMESTAMP**: Time in seconds since device boot (e.g., `012.345s`)
- **LEVEL**: Log level (`DEBUG`, `INFO`, `WARN`, `ERROR`)
- **MODULE**: Source module (e.g., `BATTERY`, `PEMF`, `SYSTEM`, `USB`)
- **MESSAGE**: The actual log message content

## Example Output

```
Connected to device:
  Manufacturer: Custom Electronics
  Product: pEMF Battery Monitor
  Serial: 123456789
  VID:PID: 1234:5678

Monitoring logs (min level: DEBUG, module filter: none)
Press Ctrl+C to stop
--------------------------------------------------------------------------------
[000.123s] [INFO ] [SYSTEM ] System boot complete, initializing peripherals
[000.456s] [DEBUG] [BATTERY] ADC reading: 1650 (3.21V)
[000.789s] [INFO ] [PEMF   ] Pulse generation started, frequency: 2Hz
[001.234s] [WARN ] [USB    ] Log queue 75% full, consider reducing verbosity
[002.567s] [ERROR] [PEMF   ] Pulse timing deviation detected: +2.1ms
```

## Troubleshooting

### Device Not Found
- Ensure the RP2040 device is connected via USB
- Check that the device has enumerated correctly: `lsusb | grep 1234:5678`
- Try listing devices with `--list` to see available options
- Verify the device is not in bootloader mode (should be running normal firmware)

### Permission Issues
On Linux, you may need to add udev rules or run with appropriate permissions:
```bash
# Add user to dialout group (logout/login required)
sudo usermod -a -G dialout $USER

# Or run with sudo (not recommended for regular use)
sudo python3 hidlog.py
```

### No Log Messages
- Verify the device firmware includes USB HID logging functionality
- Check that logging is enabled in the device configuration
- Try different log levels with `--level DEBUG`
- Use `--raw` mode to see if any data is being received

### Connection Issues
- Try specifying a specific device path with `--device`
- Verify VID/PID matches the device configuration
- Check USB cable and connection
- Try disconnecting and reconnecting the device

## Development and Testing

### Running Tests
```bash
python3 scripts/testing/test_hidlog.py
```

### Testing Without Hardware
The test script (`scripts/testing/test_hidlog.py`) validates the log message parsing and formatting functionality without requiring actual hardware.

### Integration with Device Development
1. Flash the RP2040 with firmware that includes USB HID logging
2. Connect the device via USB (normal mode, not bootloader)
3. Run `hidlog.py` to monitor real-time log messages
4. Trigger various device states to generate different types of log messages

## Command-Line Reference

```
usage: hidlog.py [-h] [--vid VID] [--pid PID] [--device DEVICE] [--serial SERIAL]
                 [--level {DEBUG,INFO,WARN,ERROR}] [--module MODULE] [--list] [--raw] 
                 [--log-file LOG_FILE] [--inspect-hid] [--json-output]

Options:
  --vid VID             USB Vendor ID (default: 0x1234)
  --pid PID             USB Product ID (default: 0x5678)
  --device DEVICE       Specific device path (e.g., /dev/hidraw0)
  --serial SERIAL       Connect to device with specific serial number
  --level LEVEL         Minimum log level to display
  --module MODULE       Filter logs by module name (case-insensitive)
  --list                List available devices and exit
  --raw                 Show raw message data for debugging
  --log-file LOG_FILE   Save logs to file with timestamps
  --inspect-hid         Show raw HID report data for debugging
  --json-output         Output logs in JSON format
```