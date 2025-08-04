# Test Result Communication via USB HID

This document describes the test result communication system that enables transmission of test execution results from the embedded device to the host via USB HID reports.

## Overview

The test result communication system consists of several key components:

1. **TestResultSerializer** - Converts test framework results into standardized USB HID reports
2. **TestResultCollector** - Batches and manages test results for efficient transmission
3. **TestExecutionHandler** - Processes USB HID commands for test execution and coordinates result transmission
4. **USB HID Integration** - Extends existing USB HID infrastructure to support test result reports

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Test Runner   │───▶│ TestResultCollector │───▶│ USB HID Reports │
│                 │    │                  │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
                       ┌──────────────────┐    ┌─────────────────┐
                       │TestResultSerializer│    │  Python Test    │
                       │                  │    │   Framework     │
                       └──────────────────┘    └─────────────────┘
```

## USB HID Report Format

All test result reports use the standard 64-byte USB HID report format:

### Test Result Report (0x92)
```
Byte 0:    Report Type (0x92)
Byte 1:    Test ID (sequence number)
Byte 2:    Status (0x00=Pass, 0x01=Fail, 0x02=Skip, 0x03=Running, 0x04=Timeout, 0x05=Error)
Byte 3:    Reserved
Bytes 4-35: Test Name (32 bytes, null-terminated)
Bytes 36-59: Error Message (24 bytes, null-terminated)
Bytes 60-63: Execution Time (4 bytes, little-endian, milliseconds)
```

### Suite Summary Report (0x93)
```
Byte 0:    Report Type (0x93)
Byte 1:    Suite ID (sequence number)
Bytes 2-3: Reserved
Bytes 4-5: Total Tests (2 bytes, little-endian)
Bytes 6-7: Passed Tests (2 bytes, little-endian)
Bytes 8-9: Failed Tests (2 bytes, little-endian)
Bytes 10-11: Skipped Tests (2 bytes, little-endian)
Bytes 12-15: Execution Time (4 bytes, little-endian, milliseconds)
Bytes 16-47: Suite Name (32 bytes, null-terminated)
Bytes 48-63: Reserved
```

### Status Update Report (0x94)
```
Byte 0:    Report Type (0x94)
Byte 1:    Update ID (sequence number)
Byte 2:    Status (same as test result status)
Byte 3:    Reserved
Bytes 4-35: Reserved (empty for status updates)
Bytes 36-59: Status Message (24 bytes, null-terminated)
Bytes 60-63: Reserved
```

### Batch Markers (0x95, 0x96)
```
Byte 0:    Report Type (0x95=BatchStart, 0x96=BatchEnd)
Byte 1:    Batch Size (number of test results in batch)
Bytes 2-63: Reserved (all zeros)
```

## Test Execution Commands

The system supports several USB HID commands for test execution:

### RunTestSuite (0x85)
Executes one or more test suites and collects results.

**Payload Format:**
```
Bytes 0-3: Timeout (4 bytes, little-endian, milliseconds)
Byte 4:    Flags (bit 0=parallel, bit 1=stop_on_failure, bit 2=collect_timing, bit 3=verbose)
Byte 5:    Suite Name Length
Bytes 6-N: Suite Name (UTF-8)
Byte N+1:  Test Name Length (0 = run all tests in suite)
Bytes N+2-M: Test Name (UTF-8, optional)
```

### GetTestResults (0x86)
Retrieves pending test results from the device.

**Response:** Batch of test result reports or suite summary reports.

### ClearTestResults (0x87)
Clears all pending test results and resets the test execution state.

### ExecuteTest (0x82)
Executes a single test within a specified suite.

**Payload Format:** Same as RunTestSuite, but both suite name and test name are required.

## Usage Examples

### Registering Test Suites

```rust
use ass_easy_loop::{TestExecutionHandler, TestRunner, TestResult};

// Create test execution handler
let mut handler = TestExecutionHandler::new();

// Define a test suite factory function
fn create_my_test_suite() -> TestRunner {
    let mut runner = TestRunner::new("my_suite");
    runner.register_test("test1", || TestResult::pass());
    runner.register_test("test2", || TestResult::fail("error"));
    runner
}

// Register the test suite
handler.register_test_suite("my_suite", create_my_test_suite);
```

### Processing Test Commands

```rust
use ass_easy_loop::{UsbCommandHandler, CommandReport};

let mut command_handler = UsbCommandHandler::new();

// Register test suites with the command handler
command_handler.register_test_suite("system_tests", create_system_test_suite);

// Process incoming USB HID command
let command = CommandReport::parse(&usb_report_data);
if let ParseResult::Valid(cmd) = command {
    let responses = command_handler.process_test_command(&cmd);
    
    // Transmit response reports via USB HID
    for response in responses {
        // Send response.as_bytes() via USB HID
    }
}
```

### Collecting and Serializing Results

```rust
use ass_easy_loop::{TestResultCollector, TestResultSerializer};

let mut collector = TestResultCollector::new();
let mut serializer = TestResultSerializer::new();

// Execute tests and collect results
let runner = create_test_suite();
let suite_result = runner.run_all();

// Add results to collector
for test_result in &suite_result.test_results {
    collector.add_test_result(test_result.clone());
}
collector.add_suite_result(suite_result);

// Get batched results for transmission
if let Some(batch_reports) = collector.get_next_batch() {
    for report in batch_reports {
        // Transmit report.as_bytes() via USB HID
    }
}
```

## Integration with Python Test Framework

The Python test framework can communicate with the embedded test system using the following workflow:

1. **Send RunTestSuite command** with suite name and parameters
2. **Receive acknowledgment** confirming command was accepted
3. **Poll for results** using GetTestResults command
4. **Process batch reports** containing individual test results
5. **Receive suite summary** with overall statistics
6. **Clear results** using ClearTestResults when done

### Python Example

```python
import usb.core
import struct

class EmbeddedTestClient:
    def __init__(self, device):
        self.device = device
    
    def run_test_suite(self, suite_name, timeout_ms=30000):
        # Construct RunTestSuite command
        payload = struct.pack('<I', timeout_ms)  # timeout
        payload += bytes([0x04])  # flags: collect_timing
        payload += bytes([len(suite_name)])  # suite name length
        payload += suite_name.encode('utf-8')  # suite name
        payload += bytes([0])  # test name length (0 = run all)
        
        command = self.create_command(0x85, payload)  # RunTestSuite
        self.device.write(command)
        
        # Wait for acknowledgment
        ack = self.device.read()
        return self.parse_response(ack)
    
    def get_test_results(self):
        command = self.create_command(0x86, b'')  # GetTestResults
        self.device.write(command)
        
        results = []
        while True:
            response = self.device.read()
            if response[0] == 0x96:  # BatchEnd
                break
            elif response[0] == 0x92:  # TestResult
                results.append(self.parse_test_result(response))
        
        return results
```

## Performance Considerations

- **Batching**: Test results are batched to minimize USB communication overhead
- **Memory Usage**: The collector uses fixed-size heapless collections to prevent memory allocation
- **Timing Impact**: Test execution and result transmission are designed to minimize impact on critical system timing
- **Error Handling**: Robust error handling ensures system stability even if test communication fails

## Error Handling

The system includes comprehensive error handling:

- **Command Validation**: All incoming commands are validated for format and authentication
- **Resource Limits**: Fixed-size collections prevent memory exhaustion
- **Timeout Protection**: Test execution includes timeout mechanisms
- **Graceful Degradation**: System continues operating even if test communication fails

## Requirements Satisfied

This implementation satisfies the following requirements:

- **4.2**: Test result transmission via USB HID with standardized report format
- **6.1**: Integration with existing automated testing infrastructure
- **6.4**: Efficient batching and communication protocols for test result transmission

## Future Enhancements

Potential future improvements include:

- **Compression**: Compress test result data for more efficient transmission
- **Streaming**: Support for streaming large test outputs
- **Filtering**: Allow filtering of test results based on criteria
- **Encryption**: Add encryption for secure test result transmission