# Automated Test Framework for RP2040 pEMF/Battery Monitoring Device

This Python framework provides comprehensive automated testing capabilities for the RP2040 pEMF/battery monitoring device, enabling remote bootloader entry, test command execution, and system validation.

## Features

- **USB HID Device Discovery**: Automatic detection and connection management for multiple devices
- **Command Transmission**: Bidirectional communication protocol for sending test commands and receiving responses
- **Test Sequencing**: Orchestrated test execution with dependency management and retry logic
- **Result Collection**: Comprehensive result analysis with multiple output formats (JSON, JUnit XML)
- **Multi-Device Support**: Parallel testing across multiple connected devices
- **CI/CD Integration**: Standard test result formats for continuous integration pipelines

## Architecture

The framework consists of four main components:

1. **UsbHidDeviceManager**: Handles device discovery, connection, and communication
2. **CommandHandler**: Manages command transmission and response processing
3. **TestSequencer**: Orchestrates test execution sequences with dependency management
4. **ResultCollector**: Collects, analyzes, and formats test results

## Installation

### Prerequisites

- Python 3.7 or higher
- USB HID library support

### Install Dependencies

```bash
pip install -r requirements.txt
```

### Required System Libraries

On Linux, you may need to install additional system libraries:

```bash
# Ubuntu/Debian
sudo apt-get install libhidapi-dev

# Fedora/CentOS
sudo yum install hidapi-devel
```

## Quick Start

### Basic Usage

```python
from test_framework import (
    UsbHidDeviceManager, CommandHandler, TestSequencer, ResultCollector
)

# Initialize framework components
device_manager = UsbHidDeviceManager()
command_handler = CommandHandler(device_manager)
test_sequencer = TestSequencer(device_manager, command_handler)
result_collector = ResultCollector()

# Discover and connect to devices
devices = device_manager.discover_devices()
if devices:
    device_manager.connect_device(devices[0].serial_number)
    
    # Run basic validation tests
    config = test_sequencer.create_basic_validation_config()
    results = test_sequencer.execute_test_sequence(config)
    
    # Collect and analyze results
    suite_result = result_collector.collect_results(
        "Basic Validation", "Device functionality test", results, 
        start_time, end_time
    )
    
    # Generate reports
    json_report = result_collector.export_json(suite_result)
    junit_xml = result_collector.export_junit_xml(suite_result)
```

### Running the Example

```bash
python test_framework/example_usage.py
```

## API Reference

### UsbHidDeviceManager

Manages USB HID device discovery and connections.

```python
manager = UsbHidDeviceManager(connection_timeout=5.0, discovery_interval=1.0)

# Discover available devices
devices = manager.discover_devices()

# Connect to a specific device
success = manager.connect_device(serial_number)

# Get connected devices
connected = manager.get_connected_devices()

# Wait for device to become available
available = manager.wait_for_device(serial_number, timeout=10.0)
```

### CommandHandler

Handles command transmission and response processing.

```python
handler = CommandHandler(device_manager, response_timeout=5.0)

# Create commands
bootloader_cmd = handler.create_bootloader_command(timeout_ms=5000)
state_query = handler.create_system_state_query('system_health')
test_cmd = handler.create_test_command(TestType.PEMF_TIMING_VALIDATION, parameters)

# Send commands
success = handler.send_command(serial_number, command)
response = handler.send_command_and_wait(serial_number, command)

# Read responses
responses = handler.read_responses(serial_number)
```

### TestSequencer

Orchestrates test sequence execution.

```python
sequencer = TestSequencer(device_manager, command_handler)

# Create test configuration
config = TestConfiguration(
    name="Custom Test Suite",
    description="Custom validation tests",
    steps=[
        TestStep(
            name="communication_test",
            test_type=TestType.USB_COMMUNICATION_TEST,
            parameters={"message_count": 10},
            timeout=10.0
        )
    ]
)

# Execute tests
results = sequencer.execute_test_sequence(config, target_devices)

# Monitor execution
status = sequencer.get_execution_status(device_serial)
sequencer.cancel_execution(device_serial)
```

### ResultCollector

Collects and analyzes test results.

```python
collector = ResultCollector()

# Collect results
suite_result = collector.collect_results(
    suite_name, description, execution_results, start_time, end_time
)

# Generate reports
summary = collector.generate_summary_report(suite_result)
detailed = collector.generate_detailed_report(suite_result)

# Export formats
json_output = collector.export_json(suite_result, detailed=True)
junit_xml = collector.export_junit_xml(suite_result)

# Analysis
failure_analysis = collector.get_failure_analysis(suite_result)
performance_analysis = collector.get_performance_analysis(suite_result)
```

## Test Types

The framework supports the following test types:

- **USB_COMMUNICATION_TEST**: Validates bidirectional USB HID communication
- **PEMF_TIMING_VALIDATION**: Tests pEMF pulse generation timing accuracy
- **BATTERY_ADC_CALIBRATION**: Validates battery voltage ADC readings
- **LED_FUNCTIONALITY**: Tests LED control patterns and timing
- **SYSTEM_STRESS_TEST**: Validates system behavior under high load

## Configuration

### Test Configuration

```python
config = TestConfiguration(
    name="Test Suite Name",
    description="Test suite description",
    steps=[
        TestStep(
            name="test_name",
            test_type=TestType.USB_COMMUNICATION_TEST,
            parameters={"message_count": 10, "timeout_ms": 1000},
            timeout=10.0,
            retry_count=2,
            required=True,
            depends_on=["prerequisite_test"]
        )
    ],
    parallel_execution=False,
    max_parallel_devices=4,
    global_timeout=300.0,
    setup_commands=[],
    teardown_commands=[]
)
```

### Device Configuration

```python
# Device identification constants
VENDOR_ID = 0x2E8A  # Raspberry Pi Foundation
PRODUCT_ID = 0x000A  # RP2040 USB HID
BOOTLOADER_PRODUCT_ID = 0x0003  # RP2040 Bootloader
```

## Testing

Run the unit tests to verify framework functionality:

```bash
python test_framework/run_tests.py
```

The test suite includes:
- Device manager functionality tests
- Command handler communication tests
- Test sequencer orchestration tests
- Result collector analysis tests

## CI/CD Integration

The framework generates standard test result formats for CI integration:

### JUnit XML Output
```xml
<testsuite name="Test Suite" tests="5" failures="1" skipped="0" time="45.123">
    <testcase classname="TestSuite.DEVICE123" name="communication_test" time="2.456"/>
    <testcase classname="TestSuite.DEVICE123" name="timing_test" time="5.789">
        <failure message="Timing tolerance exceeded">Timing deviation: 2.1%</failure>
    </testcase>
</testsuite>
```

### JSON Output
```json
{
    "suite_name": "Test Suite",
    "device_count": 2,
    "aggregate_metrics": {
        "total_tests": 10,
        "passed_tests": 8,
        "failed_tests": 2,
        "success_rate": 80.0
    }
}
```

## Troubleshooting

### Common Issues

1. **Device Not Found**: Ensure device is connected and in the correct mode
2. **Permission Denied**: On Linux, add user to `dialout` group or use `sudo`
3. **Import Errors**: Verify all dependencies are installed correctly
4. **Communication Timeouts**: Check USB connection and device responsiveness

### Debug Logging

Enable verbose logging for troubleshooting:

```python
import logging
logging.basicConfig(level=logging.DEBUG)
```

## Contributing

1. Follow PEP 8 style guidelines
2. Add unit tests for new functionality
3. Update documentation for API changes
4. Ensure all tests pass before submitting

## License

This framework is part of the RP2040 pEMF/battery monitoring device project.