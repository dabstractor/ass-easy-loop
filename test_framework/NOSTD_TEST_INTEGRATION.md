# No-std Test Integration

This document describes the integration between the Python test framework and the embedded no_std test framework running on the RP2040 device.

## Overview

The no_std test integration allows the Python test framework to:
- Flash test firmware to RP2040 devices
- Execute embedded no_std test suites and individual tests
- Collect test results via USB HID reports
- Integrate results with existing test reporting pipeline
- Support CI/CD automation for embedded tests

## Architecture

```
┌─────────────────────┐    ┌──────────────────────┐    ┌─────────────────────┐
│   Python Test      │    │   USB HID            │    │   Embedded Device   │
│   Framework         │    │   Communication      │    │   (RP2040)          │
│                     │    │                      │    │                     │
│ ┌─────────────────┐ │    │ ┌──────────────────┐ │    │ ┌─────────────────┐ │
│ │ NoStdTest       │ │◄──►│ │ Command Handler  │ │◄──►│ │ Test Framework  │ │
│ │ Integration     │ │    │ │                  │ │    │ │ (no_std)        │ │
│ └─────────────────┘ │    │ └──────────────────┘ │    │ └─────────────────┘ │
│                     │    │                      │    │                     │
│ ┌─────────────────┐ │    │ ┌──────────────────┐ │    │ ┌─────────────────┐ │
│ │ Firmware        │ │    │ │ Result Parser    │ │    │ │ Result          │ │
│ │ Flasher         │ │    │ │                  │ │    │ │ Serializer      │ │
│ └─────────────────┘ │    │ └──────────────────┘ │    │ └─────────────────┘ │
└─────────────────────┘    └──────────────────────┘    └─────────────────────┘
```

## Components

### NoStdTestIntegration

Main integration class that coordinates between Python and embedded components.

**Key Methods:**
- `flash_test_firmware()` - Flash test firmware to device
- `execute_nostd_test_suite()` - Execute complete test suite
- `execute_nostd_test()` - Execute single test
- `list_available_tests()` - Get available tests from device
- `reset_test_framework()` - Reset embedded test framework

### FirmwareFlasher

Handles flashing firmware to RP2040 devices in bootloader mode.

**Supported Formats:**
- `.uf2` files (Universal Flash Format)
- `.bin` files (raw binary, requires picotool)
- `.elf` files (ELF format, requires picotool)

**Flashing Methods:**
- `picotool` - Command-line tool for RP2040 programming
- Mass storage - Copy UF2 files to mounted bootloader device

### USB HID Communication Protocol

#### Command Format (Host to Device)

```
Byte 0:    Command Type (0x85 for no_std tests)
Byte 1:    Command ID (sequence number)
Byte 2:    Payload Length
Byte 3:    Checksum
Bytes 4-63: Payload Data
```

#### No-std Command Payload Format

```
command_type:suite_name:test_name

Examples:
- "run_suite:system_tests"
- "run_test:system_tests:battery_test"
- "list_tests"
- "reset_framework"
```

#### Response Format (Device to Host)

Test results are transmitted as 64-byte USB HID reports:

**Test Result Report (0x92):**
```
Byte 0:     Report Type (0x92)
Byte 1:     Test ID
Byte 2:     Status (0x00=Pass, 0x01=Fail, 0x02=Skip, etc.)
Byte 3:     Reserved
Bytes 4-35: Test Name (null-terminated)
Bytes 36-59: Error Message (null-terminated)
Bytes 60-63: Execution Time (ms, little-endian)
```

**Suite Summary Report (0x93):**
```
Byte 0:     Report Type (0x93)
Byte 1:     Suite ID
Bytes 2-3:  Reserved
Bytes 4-5:  Total Tests (little-endian)
Bytes 6-7:  Passed Tests (little-endian)
Bytes 8-9:  Failed Tests (little-endian)
Bytes 10-11: Skipped Tests (little-endian)
Bytes 12-15: Execution Time (ms, little-endian)
Bytes 16-47: Suite Name (null-terminated)
Bytes 48-63: Reserved
```

## Usage Examples

### Basic Test Execution

```python
from test_framework.comprehensive_test_runner import ComprehensiveTestRunner

# Initialize test runner
runner = ComprehensiveTestRunner()

# Discover and connect to devices
devices = runner.discover_and_connect_devices()

# Execute no_std test suite
result = runner.run_nostd_test_suite(
    devices, 
    "system_tests",
    firmware_path="path/to/test_firmware.uf2"
)

print(f"Overall success: {result['overall_success']}")
```

### Command Line Usage

```bash
# List available no_std tests
python -m test_framework.comprehensive_test_runner --list-nostd-tests

# Run specific test suite
python -m test_framework.comprehensive_test_runner \
    --nostd-suite system_tests \
    --firmware-path test_firmware.uf2

# Run single test
python -m test_framework.comprehensive_test_runner \
    --nostd-test system_tests battery_test \
    --devices DEVICE_001
```

### Standalone Usage

```python
from test_framework.nostd_test_integration import NoStdTestIntegration
from test_framework.device_manager import UsbHidDeviceManager
from test_framework.command_handler import CommandHandler
from test_framework.firmware_flasher import FirmwareFlasher

# Initialize components
device_manager = UsbHidDeviceManager()
command_handler = CommandHandler(device_manager)
firmware_flasher = FirmwareFlasher()

integration = NoStdTestIntegration(
    device_manager, 
    command_handler, 
    firmware_flasher
)

# Connect to device
devices = device_manager.discover_devices()
device_manager.connect_device(devices[0].serial_number)

# Flash test firmware
integration.flash_test_firmware(
    devices[0].serial_number, 
    "test_firmware.uf2"
)

# Execute tests
suite_result = integration.execute_nostd_test_suite(
    devices[0].serial_number, 
    "system_tests"
)
```

## Integration with Existing Infrastructure

### Test Reporting

No_std test results are automatically converted to the standard `TestSuiteResult` format used by the existing test framework, ensuring compatibility with:

- HTML report generation
- JSON export
- JUnit XML output
- CI/CD pipeline integration
- Performance trend analysis

### Bootloader Integration

The integration leverages the existing bootloader flashing validation infrastructure:

- Uses established bootloader entry commands
- Integrates with device discovery and connection management
- Supports the same device identification and status tracking
- Maintains compatibility with existing firmware flashing workflows

### CI/CD Integration

No_std tests integrate seamlessly with existing CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
- name: Run No-std Tests
  run: |
    python -m test_framework.comprehensive_test_runner \
      --nostd-suite comprehensive_validation \
      --firmware-path artifacts/test_firmware.uf2 \
      --output-dir test_results
```

## Error Handling

### Common Error Scenarios

1. **Firmware Flashing Failures**
   - Device not in bootloader mode
   - Firmware file not found or corrupted
   - Flash tool (picotool) not available

2. **Test Execution Failures**
   - Device not responding to commands
   - Test timeout exceeded
   - Communication errors during result collection

3. **Result Collection Issues**
   - Malformed USB HID reports
   - Missing test results or suite summaries
   - USB communication timeouts

### Error Recovery

The integration includes robust error recovery mechanisms:

- Automatic retry for transient communication failures
- Graceful degradation when partial results are available
- Comprehensive error logging and diagnostic information
- Fallback to alternative communication methods when possible

## Performance Considerations

### Timing Requirements

No_std tests must not interfere with critical device timing:
- pEMF timing accuracy maintained within ±1% tolerance
- Test execution scheduled during low-activity periods
- Minimal impact on real-time system performance

### Resource Usage

- Test framework uses heapless collections to minimize memory usage
- Result batching reduces USB communication overhead
- Efficient serialization minimizes transmission time
- Conditional compilation excludes test infrastructure from production builds

## Development Guidelines

### Adding New No_std Tests

1. **Create Test Functions:**
   ```rust
   fn my_new_test() -> TestResult {
       // Test implementation
       assert_no_std!(condition);
       TestResult::pass()
   }
   ```

2. **Register with Test Runner:**
   ```rust
   let mut runner = TestRunner::new("my_test_suite");
   runner.register_test("my_new_test", my_new_test)?;
   ```

3. **Update Python Integration:**
   - No changes needed - tests are discovered automatically
   - Update documentation if new test suites are added

### Best Practices

1. **Test Design:**
   - Keep tests focused and atomic
   - Use descriptive test names
   - Include appropriate error messages
   - Minimize test execution time

2. **Error Handling:**
   - Use custom assertion macros for consistent error reporting
   - Provide meaningful error messages
   - Handle resource constraints gracefully

3. **Performance:**
   - Avoid blocking operations in tests
   - Use efficient data structures
   - Minimize memory allocations
   - Profile test execution impact

## Troubleshooting

### Common Issues

1. **Device Not Found:**
   - Check USB connection
   - Verify device is in correct mode (not bootloader)
   - Check device permissions (Linux/macOS)

2. **Firmware Flashing Fails:**
   - Ensure picotool is installed and in PATH
   - Verify device is in bootloader mode
   - Check firmware file format and integrity

3. **Test Results Not Received:**
   - Check USB HID communication
   - Verify test framework is properly initialized on device
   - Look for timeout or communication errors in logs

4. **Tests Fail Unexpectedly:**
   - Check device logs for error messages
   - Verify test firmware matches expected version
   - Ensure device is in proper state for testing

### Debug Mode

Enable debug logging for detailed troubleshooting:

```python
import logging
logging.basicConfig(level=logging.DEBUG)

# Run tests with verbose output
runner.run_nostd_test_suite(devices, "test_suite", timeout=60.0)
```

### Log Analysis

Key log messages to monitor:
- `"Flashing test firmware"` - Firmware deployment status
- `"Executing no_std test suite"` - Test execution start
- `"Test result batch completed"` - Result collection progress
- `"No_std test suite completed"` - Successful completion

## Future Enhancements

### Planned Features

1. **Enhanced Test Discovery:**
   - Automatic test suite detection
   - Test metadata and documentation extraction
   - Dependency analysis and ordering

2. **Advanced Result Analysis:**
   - Performance regression detection
   - Test execution profiling
   - Resource usage monitoring

3. **Improved Integration:**
   - Real-time test progress monitoring
   - Interactive test execution control
   - Enhanced error diagnostics

4. **Scalability Improvements:**
   - Parallel test execution across multiple devices
   - Distributed test coordination
   - Load balancing and resource management

### Contributing

To contribute to the no_std test integration:

1. Follow the existing code style and patterns
2. Add comprehensive tests for new functionality
3. Update documentation for any API changes
4. Ensure compatibility with existing infrastructure
5. Test with real hardware before submitting changes

For questions or issues, refer to the main project documentation or create an issue in the project repository.