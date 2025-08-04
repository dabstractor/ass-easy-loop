//! Demo of the no_std test framework
//!
//! This example shows how to use the custom test framework in a no_std environment.
//! Run with: cargo run --example test_framework_demo

use ass_easy_loop::test_framework::*;
use ass_easy_loop::{assert_eq_no_std, assert_no_std, register_tests};

// Example test functions using the no_std test framework
fn test_basic_assertion() -> TestResult {
    let value = 42;
    assert_no_std!(value == 42);
    TestResult::pass()
}

fn test_equality_assertion() -> TestResult {
    let a = 10;
    let b = 10;
    assert_eq_no_std!(a, b);
    TestResult::pass()
}

fn test_failing_case() -> TestResult {
    TestResult::fail("This test demonstrates a failure")
}

fn test_skipped_case() -> TestResult {
    TestResult::skip("This test is skipped for demonstration")
}

fn test_string_operations() -> TestResult {
    let test_str = "hello";
    assert_no_std!(test_str.len() == 5);
    assert_no_std!(test_str.starts_with("he"));
    TestResult::pass()
}

fn main() {
    println!("No-std Test Framework Demo");
    println!("==========================");

    // Create a test runner
    let mut runner = TestRunner::new("Demo Test Suite");

    // Register tests using the macro
    register_tests!(
        runner,
        test_basic_assertion,
        test_equality_assertion,
        test_failing_case,
        test_skipped_case,
        test_string_operations
    );

    println!("Registered {} tests", runner.test_count());
    println!();

    // Run all tests
    let results = runner.run_all();

    // Display results
    println!("Test Results for '{}':", results.suite_name);
    println!("  Total tests: {}", results.stats.total_tests);
    println!("  Passed: {}", results.stats.passed);
    println!("  Failed: {}", results.stats.failed);
    println!("  Skipped: {}", results.stats.skipped);
    println!("  Success rate: {}%", results.stats.success_rate());
    println!();

    // Display individual test results
    println!("Individual Test Results:");
    for test_result in &results.test_results {
        let status = match &test_result.result {
            TestResult::Pass => "PASS",
            TestResult::Fail(_) => "FAIL",
            TestResult::Skip(_) => "SKIP",
        };

        print!("  {} - {}", test_result.test_name, status);

        match &test_result.result {
            TestResult::Fail(msg) => println!(" ({})", msg),
            TestResult::Skip(reason) => println!(" ({})", reason),
            TestResult::Pass => println!(),
        }
    }

    println!();

    // Demonstrate running a specific test
    println!("Running specific test 'test_basic_assertion':");
    if let Some(result) = runner.run_test("test_basic_assertion") {
        match result.result {
            TestResult::Pass => println!("  ✓ Test passed"),
            TestResult::Fail(msg) => println!("  ✗ Test failed: {}", msg),
            TestResult::Skip(reason) => println!("  - Test skipped: {}", reason),
        }
    } else {
        println!("  Test not found");
    }

    println!();

    // Demonstrate the utility function
    println!("Using create_test_suite utility:");
    let utility_tests = [
        ("util_test_1", test_basic_assertion as fn() -> TestResult),
        ("util_test_2", test_equality_assertion as fn() -> TestResult),
    ];

    let utility_runner = create_test_suite("Utility Suite", &utility_tests);
    let utility_results = utility_runner.run_all();

    println!(
        "  Utility suite results: {}/{} passed",
        utility_results.stats.passed, utility_results.stats.total_tests
    );

    // Final summary
    println!();
    if results.stats.all_passed() {
        println!("🎉 All tests passed!");
    } else if results.stats.has_failures() {
        println!("❌ Some tests failed");
    } else {
        println!("⚠️  Some tests were skipped");
    }
}
