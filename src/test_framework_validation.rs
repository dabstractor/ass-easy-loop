//! Validation module to demonstrate the no_std test framework works correctly
//! This module contains simple tests that verify the test framework functionality

use crate::test_framework::*;
use crate::{assert_no_std, assert_eq_no_std};

// Simple test functions to validate the framework
fn validation_test_pass() -> TestResult {
    TestResult::pass()
}

fn validation_test_fail() -> TestResult {
    TestResult::fail("Expected failure for validation")
}

fn validation_test_skip() -> TestResult {
    TestResult::skip("Skipped for validation")
}

fn validation_test_assertion() -> TestResult {
    let value = 42;
    assert_no_std!(value == 42);
    TestResult::pass()
}

fn validation_test_eq_assertion() -> TestResult {
    let a = 10;
    let b = 10;
    assert_eq_no_std!(a, b);
    TestResult::pass()
}

/// Function to validate the test framework works correctly
/// Returns true if all validations pass
pub fn validate_test_framework() -> bool {
    // Create a test runner
    let mut runner = TestRunner::new("Validation Suite");
    
    // Register tests manually (since we can't use the macro in this context)
    let _ = runner.register_test("pass_test", validation_test_pass);
    let _ = runner.register_test("fail_test", validation_test_fail);
    let _ = runner.register_test("skip_test", validation_test_skip);
    let _ = runner.register_test("assertion_test", validation_test_assertion);
    let _ = runner.register_test("eq_assertion_test", validation_test_eq_assertion);
    
    // Verify test count
    if runner.test_count() != 5 {
        return false;
    }
    
    // Run all tests
    let results = runner.run_all();
    
    // Validate results
    if results.stats.total_tests != 5 {
        return false;
    }
    
    if results.stats.passed != 3 {  // pass_test, assertion_test, eq_assertion_test
        return false;
    }
    
    if results.stats.failed != 1 {  // fail_test
        return false;
    }
    
    if results.stats.skipped != 1 {  // skip_test
        return false;
    }
    
    // Test individual test execution
    if let Some(result) = runner.run_test("pass_test") {
        if !result.result.is_success() {
            return false;
        }
    } else {
        return false;
    }
    
    // Test utility function
    let utility_tests = [
        ("util_pass", validation_test_pass as fn() -> TestResult),
        ("util_fail", validation_test_fail as fn() -> TestResult),
    ];
    
    let utility_runner = create_test_suite("Utility Validation", &utility_tests);
    let utility_results = utility_runner.run_all();
    
    if utility_results.stats.total_tests != 2 {
        return false;
    }
    
    if utility_results.stats.passed != 1 {
        return false;
    }
    
    if utility_results.stats.failed != 1 {
        return false;
    }
    
    // All validations passed
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_framework_validation() {
        // This test will only run in std environment (for development)
        // The actual no_std validation happens through the validate_test_framework function
        assert!(validate_test_framework());
    }
}