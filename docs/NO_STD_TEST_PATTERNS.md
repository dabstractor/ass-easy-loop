# No-std Testing Patterns and Best Practices

## Overview

This document provides comprehensive guidance for developing and maintaining unit tests in the no_std embedded environment. The patterns described here have been validated through the conversion of 30+ test files from std to no_std compatibility.

## Core Principles

### 1. No Standard Library Dependencies
All tests must be compatible with `#![no_std]` and the `thumbv6m-none-eabi` target:

```rust
#![no_std]
#![no_main]

use panic_halt as _; // Required panic handler
```

### 2. Custom Test Framework
Since `#[test]` attribute is not available in no_std, use the custom test framework:

```rust
use crate::test_framework::{TestRunner, TestCase, TestResult};

// Register tests using const arrays
const TESTS: &[TestCase] = &[
    TestCase {
        name: "test_system_state_initialization",
        test_fn: test_system_state_initialization,
    },
    TestCase {
        name: "test_command_processing",
        test_fn: test_command_processing,
    },
];

fn test_system_state_initialization() -> TestResult {
    // Test implementation
    TestResult::Pass
}
```

### 3. Heapless Collections
Replace std collections with heapless alternatives:

```rust
// Instead of std::collections::HashMap
use heapless::FnvIndexMap;
let mut state_map: FnvIndexMap<u8, SystemState, 16> = FnvIndexMap::new();

// Instead of std::vec::Vec
use heapless::Vec;
let mut results: Vec<TestResult, 32> = Vec::new();
```

## Testing Patterns

### Pattern 1: Basic Unit Test Structure

```rust
fn test_component_functionality() -> TestResult {
    // Setup
    let mut component = Component::new();
    
    // Execute
    let result = component.process_data(test_input);
    
    // Verify
    if result == expected_output {
        TestResult::Pass
    } else {
        TestResult::Fail("Component processing failed")
    }
}
```

### Pattern 2: Mock Component Usage

```rust
use crate::test_mocks::{MockUsbHidDevice, MockSystemState};

fn test_usb_communication() -> TestResult {
    let mut mock_usb = MockUsbHidDevice::new();
    mock_usb.set_expected_data(&[0x01, 0x02, 0x03]);
    
    let result = send_test_data(&mut mock_usb);
    
    if mock_usb.verify_expectations() {
        TestResult::Pass
    } else {
        TestResult::Fail("USB communication expectations not met")
    }
}
```

### Pattern 3: Error Handling Tests

```rust
fn test_error_recovery() -> TestResult {
    let mut system = SystemComponent::new();
    
    // Inject error condition
    system.simulate_hardware_failure();
    
    // Test recovery
    match system.recover_from_error() {
        Ok(_) => TestResult::Pass,
        Err(e) => TestResult::Fail("Error recovery failed"),
    }
}
```

### Pattern 4: Timing-Sensitive Tests

```rust
fn test_pemf_timing_accuracy() -> TestResult {
    let start_time = get_system_time_ms();
    
    // Execute timing-critical operation
    execute_pemf_pulse();
    
    let elapsed = get_system_time_ms() - start_time;
    let expected_duration = 1000; // 1 second
    let tolerance = expected_duration / 100; // 1% tolerance
    
    if (elapsed >= expected_duration - tolerance) && 
       (elapsed <= expected_duration + tolerance) {
        TestResult::Pass
    } else {
        TestResult::Fail("Timing outside acceptable tolerance")
    }
}
```

### Pattern 5: Resource Management Tests

```rust
fn test_memory_usage() -> TestResult {
    let initial_free = get_free_memory();
    
    {
        // Allocate resources
        let buffer: Vec<u8, 1024> = Vec::new();
        // Use buffer...
    } // buffer goes out of scope
    
    let final_free = get_free_memory();
    
    if final_free == initial_free {
        TestResult::Pass
    } else {
        TestResult::Fail("Memory leak detected")
    }
}
```

## Best Practices

### 1. Test Organization
- Group related tests in the same file
- Use descriptive test names that explain what is being tested
- Keep tests focused on single functionality

### 2. Mock Usage
- Use mocks for hardware dependencies
- Verify mock expectations to ensure proper interaction
- Keep mocks simple and focused

### 3. Assertion Patterns
```rust
// Use custom assertion macros
macro_rules! assert_no_std {
    ($cond:expr, $msg:expr) => {
        if !($cond) {
            return TestResult::Fail($msg);
        }
    };
}

// Or explicit conditional logic
if actual != expected {
    return TestResult::Fail("Values do not match");
}
```

### 4. Test Data Management
```rust
// Use const arrays for test data
const TEST_BATTERY_READINGS: &[u16] = &[3300, 3250, 3200, 3150];

// Use heapless collections for dynamic data
let mut test_results: Vec<TestResult, 64> = Vec::new();
```

### 5. Performance Considerations
- Keep tests lightweight to avoid impacting device operation
- Use timeouts to prevent hanging tests
- Batch test execution when possible

## Common Pitfalls and Solutions

### Pitfall 1: Forgetting no_std Attribute
**Problem**: Test compiles but fails at link time
**Solution**: Always add `#![no_std]` at the top of test files

### Pitfall 2: Using std Collections
**Problem**: Compilation errors about missing std
**Solution**: Replace with heapless alternatives and specify capacity

### Pitfall 3: Infinite Test Loops
**Problem**: Test hangs device
**Solution**: Implement timeouts and watchdog mechanisms

### Pitfall 4: Resource Exhaustion
**Problem**: Tests fail due to memory limits
**Solution**: Use appropriate collection sizes and clean up resources

### Pitfall 5: Timing Dependencies
**Problem**: Tests fail intermittently due to timing
**Solution**: Use proper synchronization and tolerance ranges

## Integration with Test Framework

### Test Registration
```rust
// In each test file, export test array
pub const SYSTEM_STATE_TESTS: &[TestCase] = &[
    TestCase { name: "test_init", test_fn: test_init },
    TestCase { name: "test_update", test_fn: test_update },
];

// In main test runner, collect all tests
use system_state_tests::SYSTEM_STATE_TESTS;
use command_tests::COMMAND_TESTS;

let all_tests = [SYSTEM_STATE_TESTS, COMMAND_TESTS].concat();
```

### Result Collection
```rust
pub fn run_test_suite(tests: &[TestCase]) -> TestSuiteResult {
    let mut results = Vec::new();
    let mut passed = 0;
    let mut failed = 0;
    
    for test in tests {
        let result = (test.test_fn)();
        match result {
            TestResult::Pass => passed += 1,
            TestResult::Fail(_) => failed += 1,
            TestResult::Skip(_) => {},
        }
        results.push(IndividualTestResult {
            test_name: test.name,
            result,
            execution_time_ms: 0, // Implement timing if needed
            memory_usage: None,
        }).ok(); // Handle Vec capacity limits
    }
    
    TestSuiteResult {
        suite_name: "Test Suite",
        total_tests: tests.len() as u16,
        passed,
        failed,
        skipped: (tests.len() as u16) - passed - failed,
        execution_time_ms: 0,
        individual_results: results,
    }
}
```

## Future Development Guidelines

### Adding New Tests
1. Follow the established patterns in this document
2. Use the custom test framework consistently
3. Include appropriate mocks for hardware dependencies
4. Test both success and failure cases
5. Document any special requirements or limitations

### Maintaining Existing Tests
1. Keep tests up to date with code changes
2. Regularly review test coverage
3. Optimize test performance when needed
4. Update documentation when patterns change

### Integration Testing
1. Ensure tests work with the automated testing infrastructure
2. Validate USB HID communication for test results
3. Test bootloader integration workflows
4. Verify CI/CD pipeline compatibility

This document serves as the definitive guide for no_std testing in this project and should be updated as new patterns emerge or requirements change.