# No-std Testing Troubleshooting Guide

## Overview

This guide provides solutions to common issues encountered when developing, running, and maintaining no_std unit tests. It covers compilation errors, runtime issues, integration problems, and performance concerns.

## Compilation Issues

### Issue 1: "can't find crate for `std`"

**Symptoms:**
```
error[E0463]: can't find crate for `std`
 --> tests/my_test.rs:1:1
  |
1 | use std::collections::HashMap;
  | ^^^ can't find crate for `std`
```

**Root Cause:** Test file is missing `#![no_std]` attribute or trying to use std library.

**Solution:**
```rust
// Add at the top of every test file
#![no_std]
#![no_main]

// Replace std imports
// Before:
use std::collections::HashMap;

// After:
use heapless::FnvIndexMap;
```

**Prevention:** Always start new test files with the no_std template.

### Issue 2: "cannot find attribute `test`"

**Symptoms:**
```
error: cannot find attribute `test` in this scope
 --> tests/my_test.rs:5:3
  |
5 | #[test]
  | ^^^^
```

**Root Cause:** The `#[test]` attribute is not available in no_std environment.

**Solution:**
```rust
// Before:
#[test]
fn my_test() {
    assert_eq!(1 + 1, 2);
}

// After:
fn my_test() -> TestResult {
    if 1 + 1 == 2 {
        TestResult::Pass
    } else {
        TestResult::Fail("Math is broken")
    }
}

// Register test in const array
const TESTS: &[TestCase] = &[
    TestCase {
        name: "my_test",
        test_fn: my_test,
    },
];
```

**Prevention:** Use the custom test framework consistently.

### Issue 3: "cannot find macro `vec`"

**Symptoms:**
```
error: cannot find macro `vec` in this scope
 --> tests/my_test.rs:8:17
  |
8 |     let data = vec![1, 2, 3];
  |                 ^^^
```

**Root Cause:** The `vec!` macro is not available in no_std.

**Solution:**
```rust
// Before:
let data = vec![1, 2, 3];

// After:
use heapless::Vec;
let mut data: Vec<i32, 8> = Vec::new();
data.push(1).ok();
data.push(2).ok();
data.push(3).ok();

// Or use arrays for fixed data:
let data = [1, 2, 3];
```

**Prevention:** Use heapless collections with appropriate capacity limits.

### Issue 4: "cannot find macro `assert_eq`"

**Symptoms:**
```
error: cannot find macro `assert_eq` in this scope
 --> tests/my_test.rs:10:5
  |
10 |     assert_eq!(result, expected);
   |     ^^^^^^^^^
```

**Root Cause:** Standard assertion macros are not available in no_std.

**Solution:**
```rust
// Create custom assertion macros
macro_rules! assert_eq_no_std {
    ($left:expr, $right:expr) => {
        if $left != $right {
            return TestResult::Fail("Assertion failed: values not equal");
        }
    };
}

// Or use explicit comparison
fn my_test() -> TestResult {
    let result = calculate_something();
    let expected = 42;
    
    if result == expected {
        TestResult::Pass
    } else {
        TestResult::Fail("Values do not match")
    }
}
```

**Prevention:** Use custom assertion macros or explicit comparisons.

### Issue 5: "linking with `cc` failed"

**Symptoms:**
```
error: linking with `cc` failed: exit code: 1
note: /usr/bin/ld: cannot find -lgcc
```

**Root Cause:** Missing embedded toolchain or incorrect target configuration.

**Solution:**
```bash
# Install correct toolchain
rustup target add thumbv6m-none-eabi

# Verify .cargo/config.toml
[build]
target = "thumbv6m-none-eabi"

[target.thumbv6m-none-eabi]
runner = "probe-run --chip RP2040"
rustflags = [
  "-C", "linker=flip-link",
  "-C", "link-arg=--nmagic",
  "-C", "link-arg=-Tlink.x",
  "-C", "link-arg=-Tdefmt.x",
]
```

**Prevention:** Maintain proper toolchain setup and configuration.

## Runtime Issues

### Issue 6: Test Hangs or Times Out

**Symptoms:** Test execution never completes, device becomes unresponsive.

**Root Cause:** Infinite loop, blocking operation, or resource deadlock.

**Solution:**
```rust
// Add timeout mechanism
fn test_with_timeout() -> TestResult {
    let start_time = get_system_time_ms();
    let timeout_ms = 5000; // 5 second timeout
    
    while condition_not_met() {
        if get_system_time_ms() - start_time > timeout_ms {
            return TestResult::Fail("Test timeout");
        }
        
        // Test logic here
        cortex_m::asm::nop(); // Prevent tight loop
    }
    
    TestResult::Pass
}
```

**Prevention:** Always include timeout mechanisms in tests with loops.

### Issue 7: Memory Exhaustion

**Symptoms:** Tests fail with allocation errors or device resets.

**Root Cause:** Insufficient memory allocation for heapless collections.

**Solution:**
```rust
// Before: Collection too small
let mut buffer: Vec<u8, 16> = Vec::new();

// After: Appropriate size
let mut buffer: Vec<u8, 256> = Vec::new();

// Or use streaming approach
fn process_large_data() -> TestResult {
    const CHUNK_SIZE: usize = 64;
    let mut chunk: [u8; CHUNK_SIZE] = [0; CHUNK_SIZE];
    
    for i in 0..total_data_size / CHUNK_SIZE {
        // Process chunk by chunk
        process_chunk(&mut chunk);
    }
    
    TestResult::Pass
}
```

**Prevention:** Profile memory usage and size collections appropriately.

### Issue 8: Panic During Test Execution

**Symptoms:** Device resets unexpectedly during tests.

**Root Cause:** Unhandled panic or assertion failure.

**Solution:**
```rust
// Add panic handler for tests
#[cfg(feature = "embedded_tests")]
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    // Log panic information
    log_panic_info(info);
    
    // Send panic report via USB
    send_panic_report();
    
    // Reset device
    cortex_m::peripheral::SCB::sys_reset();
}

// Use safe operations
fn safe_array_access() -> TestResult {
    let data = [1, 2, 3, 4, 5];
    
    // Before: Potential panic
    // let value = data[10];
    
    // After: Safe access
    match data.get(10) {
        Some(value) => TestResult::Pass,
        None => TestResult::Fail("Index out of bounds"),
    }
}
```

**Prevention:** Use safe operations and proper error handling.

## Integration Issues

### Issue 9: USB Communication Failures

**Symptoms:** Test results not received by Python framework.

**Root Cause:** USB HID communication problems or protocol mismatch.

**Solution:**
```rust
// Add retry mechanism
fn send_test_results_with_retry(results: &TestSuiteResult) -> bool {
    const MAX_RETRIES: u8 = 3;
    
    for attempt in 0..MAX_RETRIES {
        if send_test_results(results) {
            return true;
        }
        
        // Wait before retry
        delay_ms(100);
    }
    
    false
}

// Validate USB connection
fn check_usb_connection() -> TestResult {
    if usb_device_is_connected() {
        TestResult::Pass
    } else {
        TestResult::Skip("USB not connected")
    }
}
```

**Python side debugging:**
```python
# Add USB debugging
def debug_usb_communication():
    try:
        device = hid.device()
        device.open(VENDOR_ID, PRODUCT_ID)
        
        # Send test command
        command = [0x10, 0x01, 0x00] + [0] * 61
        device.write(command)
        
        # Wait for response with timeout
        response = device.read(64, timeout_ms=5000)
        print(f"Received: {response}")
        
    except Exception as e:
        print(f"USB communication error: {e}")
```

**Prevention:** Implement robust communication protocols with error handling.

### Issue 10: Bootloader Flashing Issues

**Symptoms:** Test firmware fails to flash or device doesn't enter bootloader mode.

**Root Cause:** Bootloader entry sequence problems or firmware compatibility.

**Solution:**
```python
# Robust bootloader entry
def enter_bootloader_with_retry():
    max_attempts = 5
    
    for attempt in range(max_attempts):
        try:
            # Try bootloader entry sequence
            send_bootloader_command()
            time.sleep(0.5)
            
            if device_in_bootloader_mode():
                return True
                
        except Exception as e:
            print(f"Bootloader entry attempt {attempt + 1} failed: {e}")
            time.sleep(1)
    
    raise BootloaderError("Failed to enter bootloader mode")

# Validate firmware before flashing
def validate_firmware(firmware_path):
    if not os.path.exists(firmware_path):
        raise FileNotFoundError(f"Firmware not found: {firmware_path}")
    
    # Check firmware size
    size = os.path.getsize(firmware_path)
    if size > MAX_FIRMWARE_SIZE:
        raise ValueError(f"Firmware too large: {size} bytes")
    
    return True
```

**Prevention:** Implement robust bootloader protocols and firmware validation.

## Performance Issues

### Issue 11: Slow Test Execution

**Symptoms:** Tests take too long to complete, impacting development workflow.

**Root Cause:** Inefficient test implementation or excessive USB communication.

**Solution:**
```rust
// Batch test results
fn run_test_suite_optimized(tests: &[TestCase]) -> TestSuiteResult {
    let mut results = Vec::new();
    let start_time = get_system_time_ms();
    
    // Run tests without individual reporting
    for test in tests {
        let test_start = get_system_time_ms();
        let result = (test.test_fn)();
        let test_duration = get_system_time_ms() - test_start;
        
        results.push(IndividualTestResult {
            test_name: test.name,
            result,
            execution_time_ms: test_duration as u16,
            memory_usage: None,
        }).ok();
    }
    
    // Send batched results
    let total_time = get_system_time_ms() - start_time;
    TestSuiteResult {
        suite_name: "Optimized Suite",
        total_tests: tests.len() as u16,
        passed: results.iter().filter(|r| matches!(r.result, TestResult::Pass)).count() as u16,
        failed: results.iter().filter(|r| matches!(r.result, TestResult::Fail(_))).count() as u16,
        skipped: results.iter().filter(|r| matches!(r.result, TestResult::Skip(_))).count() as u16,
        execution_time_ms: total_time,
        individual_results: results,
    }
}
```

**Prevention:** Design tests for efficiency and batch operations when possible.

### Issue 12: pEMF Timing Interference

**Symptoms:** pEMF timing accuracy degrades during test execution.

**Root Cause:** Test execution interfering with critical timing operations.

**Solution:**
```rust
// Suspend pEMF during tests
fn run_tests_with_pemf_suspension() -> TestSuiteResult {
    // Suspend pEMF operations
    suspend_pemf_generation();
    
    // Run tests
    let results = run_test_suite(TEST_SUITE);
    
    // Resume pEMF operations
    resume_pemf_generation();
    
    results
}

// Test timing validation
fn validate_pemf_timing_after_tests() -> TestResult {
    let timing_accuracy = measure_pemf_timing_accuracy();
    let tolerance = 0.01; // 1% tolerance
    
    if timing_accuracy <= tolerance {
        TestResult::Pass
    } else {
        TestResult::Fail("pEMF timing degraded after tests")
    }
}
```

**Prevention:** Design tests to minimize impact on critical system operations.

## Debug Tools and Utilities

### Debug Macros
```rust
// Debug output for tests
macro_rules! test_debug {
    ($($arg:tt)*) => {
        #[cfg(feature = "test_debug")]
        {
            use crate::logging::log_info;
            log_info!($($arg)*);
        }
    };
}

// Memory usage tracking
fn track_memory_usage() -> u32 {
    #[cfg(feature = "memory_tracking")]
    {
        get_free_memory()
    }
    #[cfg(not(feature = "memory_tracking"))]
    {
        0
    }
}
```

### Test Validation Scripts
```python
# Validate test environment
def validate_test_environment():
    checks = [
        ("Rust toolchain", check_rust_toolchain),
        ("USB device", check_usb_device),
        ("Bootloader", check_bootloader),
        ("Test firmware", check_test_firmware),
    ]
    
    for name, check_fn in checks:
        try:
            check_fn()
            print(f"✓ {name}")
        except Exception as e:
            print(f"✗ {name}: {e}")
            return False
    
    return True

# Monitor test execution
def monitor_test_execution():
    start_time = time.time()
    
    while True:
        try:
            # Check device status
            status = get_device_status()
            elapsed = time.time() - start_time
            
            print(f"[{elapsed:.1f}s] Device status: {status}")
            
            if status == "test_complete":
                break
                
            time.sleep(1)
            
        except KeyboardInterrupt:
            print("Monitoring stopped")
            break
```

## Best Practices for Troubleshooting

### 1. Systematic Debugging
- Start with compilation errors before runtime issues
- Use minimal test cases to isolate problems
- Check one component at a time

### 2. Logging and Monitoring
- Add debug output to track test execution
- Monitor memory usage during tests
- Log USB communication for debugging

### 3. Environment Validation
- Verify toolchain setup before debugging tests
- Check hardware connections and device status
- Validate firmware compatibility

### 4. Documentation
- Document known issues and solutions
- Keep troubleshooting logs for future reference
- Update this guide with new issues and solutions

This troubleshooting guide should be updated as new issues are discovered and resolved. Regular maintenance ensures it remains a valuable resource for developers working with no_std tests.