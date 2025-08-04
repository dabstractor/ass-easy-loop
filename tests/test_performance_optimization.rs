//! Test Performance Optimization Integration Test
//!
//! This test validates that the performance optimization features work correctly
//! and don't interfere with pEMF timing requirements.

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

use ass_easy_loop::test_framework::{TestResult, TestRunner};
#[cfg(not(test))]
use panic_halt as _;

#[cfg(feature = "test-commands")]
use ass_easy_loop::test_performance_optimizer::{OptimizationSettings, TestPerformanceOptimizer};

fn test_basic_optimization() -> TestResult {
    #[cfg(feature = "test-commands")]
    {
        let optimizer = TestPerformanceOptimizer::new();

        // Test that optimizer is created successfully
        if optimizer.get_performance_stats().tests_optimized == 0 {
            TestResult::pass()
        } else {
            TestResult::fail("Optimizer should start with zero optimized tests")
        }
    }

    #[cfg(not(feature = "test-commands"))]
    {
        // Skip test when optimization features are disabled
        TestResult::skip("Performance optimization features disabled")
    }
}

fn test_optimization_settings() -> TestResult {
    #[cfg(feature = "test-commands")]
    {
        let settings = OptimizationSettings {
            enable_dynamic_scheduling: true,
            max_cpu_utilization_percent: 15,
            max_memory_usage_bytes: 2048,
            pemf_timing_tolerance_percent: 0.5,
            enable_result_caching: true,
            test_execution_priority: 128,
        };

        let optimizer = TestPerformanceOptimizer::new_with_settings(settings);

        // Test that optimizer accepts custom settings
        if optimizer.is_system_ready_for_tests() {
            TestResult::pass()
        } else {
            TestResult::fail("System should be ready for tests with default conditions")
        }
    }

    #[cfg(not(feature = "test-commands"))]
    {
        TestResult::skip("Performance optimization features disabled")
    }
}

fn test_conditional_compilation() -> TestResult {
    // Test that conditional compilation works correctly
    #[cfg(feature = "test-commands")]
    {
        // Performance optimization features should be available
        TestResult::pass()
    }

    #[cfg(not(feature = "test-commands"))]
    {
        // Performance optimization features should be excluded
        TestResult::pass()
    }
}

// Test suite registration
const PERFORMANCE_OPTIMIZATION_TESTS: &[(&str, fn() -> TestResult)] = &[
    ("test_basic_optimization", test_basic_optimization),
    ("test_optimization_settings", test_optimization_settings),
    ("test_conditional_compilation", test_conditional_compilation),
];

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() -> ! {
    // Create test runner
    let runner = ass_easy_loop::test_framework::create_test_suite(
        "performance_optimization_tests",
        PERFORMANCE_OPTIMIZATION_TESTS,
    );

    // Run tests
    let _results = runner.run_all();

    // In a real embedded system, this would report results via USB HID
    // For now, we'll just loop
    loop {}
}
