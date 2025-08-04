//! No-std test framework for embedded unit testing
//!
//! This module provides a custom test framework that works in no_std environments,
//! specifically designed for the thumbv6m-none-eabi target. It includes:
//! - Custom test runner with result collection
//! - No-std compatible assertion macros
//! - Test registration system using const arrays
//! - Integration with USB HID for result reporting

use core::default::Default;
use core::option::Option::{self, None, Some};
use core::result::Result::{self};
use heapless::{String, Vec};

/// Maximum number of tests that can be registered in a single test suite
pub const MAX_TESTS_PER_SUITE: usize = 64;

/// Maximum number of test suites that can be registered
pub const MAX_TEST_SUITES: usize = 16;

/// Maximum length for test names and error messages
pub const MAX_NAME_LENGTH: usize = 64;

/// Maximum length for error messages
pub const MAX_ERROR_MESSAGE_LENGTH: usize = 128;

/// Result of a single test execution
#[derive(Debug, Clone, PartialEq)]
pub enum TestResult {
    /// Test passed successfully
    Pass,
    /// Test failed with an error message
    Fail(String<MAX_ERROR_MESSAGE_LENGTH>),
    /// Test was skipped with a reason
    Skip(String<MAX_ERROR_MESSAGE_LENGTH>),
}

impl TestResult {
    /// Create a new Pass result
    pub fn pass() -> Self {
        TestResult::Pass
    }

    /// Create a new Fail result with a message
    pub fn fail(msg: &str) -> Self {
        let mut error_msg = String::new();
        let _ = error_msg.push_str(msg);
        TestResult::Fail(error_msg)
    }

    /// Create a new Skip result with a reason
    pub fn skip(reason: &str) -> Self {
        let mut skip_msg = String::new();
        let _ = skip_msg.push_str(reason);
        TestResult::Skip(skip_msg)
    }

    /// Check if the test result indicates success
    pub fn is_success(&self) -> bool {
        matches!(self, TestResult::Pass)
    }

    /// Check if the test result indicates failure
    pub fn is_failure(&self) -> bool {
        matches!(self, TestResult::Fail(_))
    }

    /// Check if the test result indicates the test was skipped
    pub fn is_skipped(&self) -> bool {
        matches!(self, TestResult::Skip(_))
    }
}

/// A single test case with its metadata
#[derive(Debug, Clone)]
pub struct TestCase {
    /// Name of the test
    pub name: String<MAX_NAME_LENGTH>,
    /// Function pointer to the test implementation
    pub test_fn: fn() -> TestResult,
}

impl TestCase {
    /// Create a new test case
    pub fn new(name: &str, test_fn: fn() -> TestResult) -> Self {
        let mut test_name = String::new();
        let _ = test_name.push_str(name);
        TestCase {
            name: test_name,
            test_fn,
        }
    }

    /// Execute this test case
    pub fn run(&self) -> TestResult {
        (self.test_fn)()
    }
}

/// Statistics for a test suite execution
#[derive(Debug, Clone, Default)]
pub struct TestSuiteStats {
    /// Total number of tests executed
    pub total_tests: u16,
    /// Number of tests that passed
    pub passed: u16,
    /// Number of tests that failed
    pub failed: u16,
    /// Number of tests that were skipped
    pub skipped: u16,
    /// Execution time in milliseconds (if timing is available)
    pub execution_time_ms: Option<u32>,
}

impl TestSuiteStats {
    /// Create new empty statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Update statistics with a test result
    pub fn update(&mut self, result: &TestResult) {
        self.total_tests += 1;
        match result {
            TestResult::Pass => self.passed += 1,
            TestResult::Fail(_) => self.failed += 1,
            TestResult::Skip(_) => self.skipped += 1,
        }
    }

    /// Check if all tests passed (no failures or skips)
    pub fn all_passed(&self) -> bool {
        self.failed == 0 && self.skipped == 0 && self.total_tests > 0
    }

    /// Check if any tests failed
    pub fn has_failures(&self) -> bool {
        self.failed > 0
    }

    /// Get success rate as a percentage (0-100)
    pub fn success_rate(&self) -> u8 {
        if self.total_tests == 0 {
            return 0;
        }
        ((self.passed as u32 * 100) / self.total_tests as u32) as u8
    }
}

/// Individual test execution result with metadata
#[derive(Debug, Clone)]
pub struct TestExecutionResult {
    /// Name of the executed test
    pub test_name: String<MAX_NAME_LENGTH>,
    /// Result of the test execution
    pub result: TestResult,
    /// Execution time in microseconds (if available)
    pub execution_time_us: Option<u32>,
}

impl TestExecutionResult {
    /// Create a new test execution result
    pub fn new(test_name: &str, result: TestResult) -> Self {
        let mut name = String::new();
        let _ = name.push_str(test_name);
        TestExecutionResult {
            test_name: name,
            result,
            execution_time_us: None,
        }
    }

    /// Create a new test execution result with timing
    pub fn with_timing(test_name: &str, result: TestResult, execution_time_us: u32) -> Self {
        let mut name = String::new();
        let _ = name.push_str(test_name);
        TestExecutionResult {
            test_name: name,
            result,
            execution_time_us: Some(execution_time_us),
        }
    }
}

/// Test suite execution results
#[derive(Debug, Clone)]
pub struct TestSuiteResult {
    /// Name of the test suite
    pub suite_name: String<MAX_NAME_LENGTH>,
    /// Overall statistics for the suite
    pub stats: TestSuiteStats,
    /// Individual test results
    pub test_results: Vec<TestExecutionResult, MAX_TESTS_PER_SUITE>,
}

impl TestSuiteResult {
    /// Create a new test suite result
    pub fn new(suite_name: &str) -> Self {
        let mut name = String::new();
        let _ = name.push_str(suite_name);
        TestSuiteResult {
            suite_name: name,
            stats: TestSuiteStats::new(),
            test_results: Vec::new(),
        }
    }

    /// Add a test result to this suite
    pub fn add_test_result(&mut self, result: TestExecutionResult) {
        self.stats.update(&result.result);
        let _ = self.test_results.push(result);
    }
}

/// Custom test runner for no_std environments
#[derive(Debug)]
pub struct TestRunner {
    /// Collection of test cases to execute
    tests: Vec<TestCase, MAX_TESTS_PER_SUITE>,
    /// Name of this test suite
    suite_name: String<MAX_NAME_LENGTH>,
}

impl TestRunner {
    /// Create a new test runner for a named test suite
    pub fn new(suite_name: &str) -> Self {
        let mut name = String::new();
        let _ = name.push_str(suite_name);
        TestRunner {
            tests: Vec::new(),
            suite_name: name,
        }
    }

    /// Register a test case with the runner
    pub fn register_test(
        &mut self,
        name: &str,
        test_fn: fn() -> TestResult,
    ) -> Result<(), &'static str> {
        let test_case = TestCase::new(name, test_fn);
        self.tests.push(test_case).map_err(|_| "Test registry full")
    }

    /// Run all registered tests and return results
    pub fn run_all(&self) -> TestSuiteResult {
        let mut suite_result = TestSuiteResult::new(&self.suite_name);

        for test_case in &self.tests {
            let result = test_case.run();
            let test_result = TestExecutionResult::new(&test_case.name, result);
            suite_result.add_test_result(test_result);
        }

        suite_result
    }

    /// Run a specific test by name
    pub fn run_test(&self, test_name: &str) -> Option<TestExecutionResult> {
        for test_case in &self.tests {
            if test_case.name.as_str() == test_name {
                let result = test_case.run();
                return Some(TestExecutionResult::new(&test_case.name, result));
            }
        }
        None
    }

    /// Get the number of registered tests
    pub fn test_count(&self) -> usize {
        self.tests.len()
    }

    /// Get the names of all registered tests
    pub fn test_names(&self) -> Vec<&str, MAX_TESTS_PER_SUITE> {
        let mut names = Vec::new();
        for test_case in &self.tests {
            let _ = names.push(test_case.name.as_str());
        }
        names
    }
}

/// Macro for creating a test registry using const arrays
/// This replaces the standard #[test] attribute approach
#[macro_export]
macro_rules! register_tests {
    ($runner:expr, $($test_name:ident),* $(,)?) => {
        $(
            let _ = $runner.register_test(stringify!($test_name), $test_name);
        )*
    };
}

/// Custom assertion macro that works in no_std environment
#[macro_export]
macro_rules! assert_no_std {
    ($cond:expr) => {
        if !($cond) {
            return $crate::test_framework::TestResult::fail(concat!(
                "Assertion failed: ",
                stringify!($cond)
            ));
        }
    };
    ($cond:expr, $msg:expr) => {
        if !($cond) {
            return $crate::test_framework::TestResult::fail($msg);
        }
    };
}

/// Custom equality assertion macro for no_std environment
#[macro_export]
macro_rules! assert_eq_no_std {
    ($left:expr, $right:expr) => {
        let left_val = $left;
        let right_val = $right;
        if left_val != right_val {
            return $crate::test_framework::TestResult::fail(concat!(
                "Assertion failed: ",
                stringify!($left),
                " == ",
                stringify!($right)
            ));
        }
    };
    ($left:expr, $right:expr, $msg:expr) => {
        let left_val = $left;
        let right_val = $right;
        if left_val != right_val {
            return $crate::test_framework::TestResult::fail($msg);
        }
    };
}

/// Custom not-equal assertion macro for no_std environment
#[macro_export]
macro_rules! assert_ne_no_std {
    ($left:expr, $right:expr) => {
        let left_val = $left;
        let right_val = $right;
        if left_val == right_val {
            return $crate::test_framework::TestResult::fail(concat!(
                "Assertion failed: ",
                stringify!($left),
                " != ",
                stringify!($right)
            ));
        }
    };
    ($left:expr, $right:expr, $msg:expr) => {
        let left_val = $left;
        let right_val = $right;
        if left_val == right_val {
            return $crate::test_framework::TestResult::fail($msg);
        }
    };
}

/// Macro to create a test function that returns TestResult
#[macro_export]
macro_rules! test_case {
    ($name:ident, $body:block) => {
        fn $name() -> $crate::test_framework::TestResult {
            $body
            $crate::test_framework::TestResult::pass()
        }
    };
}

/// Utility function to create a test runner and register tests from a const array
pub fn create_test_suite(suite_name: &str, tests: &[(&str, fn() -> TestResult)]) -> TestRunner {
    let mut runner = TestRunner::new(suite_name);

    for (name, test_fn) in tests {
        let _ = runner.register_test(name, *test_fn);
    }

    runner
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::option::Option::{None, Some};
    use core::result::Result::{Err, Ok};

    fn sample_passing_test() -> TestResult {
        TestResult::pass()
    }

    fn sample_failing_test() -> TestResult {
        TestResult::fail("This test always fails")
    }

    fn sample_skipped_test() -> TestResult {
        TestResult::skip("This test is skipped")
    }

    #[test]
    fn test_result_creation() {
        let pass = TestResult::pass();
        assert!(pass.is_success());
        assert!(!pass.is_failure());
        assert!(!pass.is_skipped());

        let fail = TestResult::fail("error");
        assert!(!fail.is_success());
        assert!(fail.is_failure());
        assert!(!fail.is_skipped());

        let skip = TestResult::skip("reason");
        assert!(!skip.is_success());
        assert!(!skip.is_failure());
        assert!(skip.is_skipped());
    }

    #[test]
    fn test_case_creation_and_execution() {
        let test_case = TestCase::new("sample_test", sample_passing_test);
        assert_eq!(test_case.name.as_str(), "sample_test");

        let result = test_case.run();
        assert!(result.is_success());
    }

    #[test]
    fn test_runner_registration_and_execution() {
        let mut runner = TestRunner::new("test_suite");

        assert!(runner
            .register_test("pass_test", sample_passing_test)
            .is_ok());
        assert!(runner
            .register_test("fail_test", sample_failing_test)
            .is_ok());
        assert!(runner
            .register_test("skip_test", sample_skipped_test)
            .is_ok());

        assert_eq!(runner.test_count(), 3);

        let results = runner.run_all();
        assert_eq!(results.stats.total_tests, 3);
        assert_eq!(results.stats.passed, 1);
        assert_eq!(results.stats.failed, 1);
        assert_eq!(results.stats.skipped, 1);
    }

    #[test]
    fn test_suite_stats() {
        let mut stats = TestSuiteStats::new();

        stats.update(&TestResult::pass());
        stats.update(&TestResult::fail("error"));
        stats.update(&TestResult::skip("reason"));

        assert_eq!(stats.total_tests, 3);
        assert_eq!(stats.passed, 1);
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.skipped, 1);
        assert!(!stats.all_passed());
        assert!(stats.has_failures());
        assert_eq!(stats.success_rate(), 33); // 1/3 * 100 = 33%
    }

    #[test]
    fn test_create_test_suite_utility() {
        let tests = [
            ("pass_test", sample_passing_test as fn() -> TestResult),
            ("fail_test", sample_failing_test as fn() -> TestResult),
        ];

        let runner = create_test_suite("utility_suite", &tests);
        assert_eq!(runner.test_count(), 2);

        let results = runner.run_all();
        assert_eq!(results.stats.total_tests, 2);
        assert_eq!(results.stats.passed, 1);
        assert_eq!(results.stats.failed, 1);
    }
}
