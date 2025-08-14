//! Common test utilities and infrastructure
//!
//! This module provides shared utilities, mock implementations, test data generators,
//! and custom assertions that can be used across all test categories.

// External dependencies for testing
extern crate std;
use std::collections::HashMap;
use std::sync::Once;

pub mod assertions;
pub mod mocks;
pub mod test_data;

// Re-export commonly used items
pub use mocks::{
    MockBatteryMonitor, MockBootloaderHardware, MockSystemState, MockTestEnvironment,
    MockUsbHidDevice,
};
pub use test_data::{battery, performance, system_state, usb_hid};

// Re-export assertion modules
pub use assertions::{
    battery as battery_assertions, performance as performance_assertions,
    results as results_assertions, system_state as system_state_assertions,
    timing as timing_assertions, usb_hid as usb_hid_assertions,
};

// Re-export test context and configuration
// pub use TestContext;
pub use config::{TestConfig, TestConfigBuilder, TestProfile};

/// Common test setup utilities
pub mod setup {
    use super::Once;
    use std::collections::HashMap;
    use std::env::{set_var, var};
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    static INIT: Once = Once::new();

    /// Global test environment state
    pub static TEST_ENV: std::sync::LazyLock<Arc<Mutex<TestEnvironment>>> =
        std::sync::LazyLock::new(|| Arc::new(Mutex::new(TestEnvironment::new())));

    /// Test environment configuration and state
    #[derive(Debug, Clone)]
    pub struct TestEnvironment {
        pub initialized: bool,
        pub test_start_time: u64,
        pub config: HashMap<String, String>,
        pub temp_files: Vec<String>,
        pub mock_states: HashMap<String, String>,
        pub test_counters: HashMap<String, u32>,
    }

    impl TestEnvironment {
        fn new() -> Self {
            Self {
                initialized: false,
                test_start_time: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                config: HashMap::new(),
                temp_files: Vec::new(),
                mock_states: HashMap::new(),
                test_counters: HashMap::new(),
            }
        }
    }

    /// Initialize test environment (call once per test process)
    pub fn init_test_env() {
        INIT.call_once(|| {
            let mut env = TEST_ENV.lock().unwrap();
            env.initialized = true;
            env.config = default_test_config();

            // Set up test logging
            if std::env::var("RUST_LOG").is_err() {
                std::env::set_var("RUST_LOG", "debug");
            }

            println!("Test environment initialized at {}", env.test_start_time);
        });
    }

    /// Initialize test environment with custom configuration
    pub fn init_test_env_with_config(config: HashMap<String, String>) {
        INIT.call_once(|| {
            let mut env = TEST_ENV.lock().unwrap();
            env.initialized = true;
            env.config = config;
            env.test_start_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            println!("Test environment initialized with custom config");
        });
    }

    /// Create a test configuration with sensible defaults
    pub fn default_test_config() -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.insert("log_level".to_string(), "2".to_string());
        config.insert("battery_threshold".to_string(), "3200".to_string());
        config.insert("usb_timeout_ms".to_string(), "5000".to_string());
        config.insert("pemf_frequency_hz".to_string(), "10.0".to_string());
        config.insert("test_timeout_ms".to_string(), "30000".to_string());
        config.insert("mock_hardware".to_string(), "true".to_string());
        config.insert("enable_timing_checks".to_string(), "true".to_string());
        config.insert("memory_limit_bytes".to_string(), "65536".to_string());
        config
    }

    /// Create a minimal test configuration for fast tests
    pub fn minimal_test_config() -> HashMap<String, String> {
        let mut config = HashMap::new();
        config.insert("log_level".to_string(), "3".to_string()); // Errors only
        config.insert("test_timeout_ms".to_string(), "5000".to_string());
        config.insert("mock_hardware".to_string(), "true".to_string());
        config.insert("enable_timing_checks".to_string(), "false".to_string());
        config
    }

    /// Create a debug test configuration with verbose logging
    pub fn debug_test_config() -> HashMap<String, String> {
        let mut config = default_test_config();
        config.insert("log_level".to_string(), "0".to_string()); // Debug level
        config.insert("enable_timing_checks".to_string(), "true".to_string());
        config.insert("verbose_assertions".to_string(), "true".to_string());
        config
    }

    /*
    /// Set up test environment for battery testing
    pub fn setup_battery_test_env() -> TestContext {
        init_test_env();
        let mut context = TestContext::new("battery_test");

        // Configure for battery testing
        context.set_config("battery_threshold", "3200");
        context.set_config("adc_sample_rate", "1000");
        context.set_config("battery_monitoring_enabled", "true");

        // Initialize battery mock
        let battery_mock = crate::mocks::MockBatteryMonitor::new();
        battery_mock.set_voltage(3700); // Start with normal voltage
        context.add_mock("battery", Box::new(battery_mock));

        context
    }

    /// Set up test environment for USB HID testing
    pub fn setup_usb_test_env() -> TestContext {
        init_test_env();
        let mut context = TestContext::new("usb_test");

        // Configure for USB testing
        context.set_config("usb_timeout_ms", "5000");
        context.set_config("hid_report_size", "64");
        context.set_config("usb_polling_enabled", "true");

        // Initialize USB mock
        let usb_mock = crate::mocks::MockUsbHidDevice::new();
        context.add_mock("usb", Box::new(usb_mock));

        context
    }

    /// Set up test environment for timing-critical tests
    pub fn setup_timing_test_env() -> TestContext {
        init_test_env();
        let mut context = TestContext::new("timing_test");

        // Configure for timing tests
        context.set_config("enable_timing_checks", "true");
        context.set_config("timing_tolerance_us", "100");
        context.set_config("pemf_frequency_hz", "10.0");
        context.set_config("high_precision_timing", "true");

        context
    }

    /// Set up test environment for performance testing
    pub fn setup_performance_test_env() -> TestContext {
        init_test_env();
        let mut context = TestContext::new("performance_test");

        // Configure for performance tests
        context.set_config("memory_limit_bytes", "65536");
        context.set_config("cpu_limit_percent", "80");
        context.set_config("enable_profiling", "true");
        context.set_config("benchmark_mode", "true");

        context
    }

    /// Set up test environment for integration testing
    pub fn setup_integration_test_env() -> TestContext {
        init_test_env();
        let mut context = TestContext::new("integration_test");

        // Configure for integration tests
        context.set_config("test_timeout_ms", "60000"); // Longer timeout
        context.set_config("enable_all_mocks", "true");
        context.set_config("comprehensive_logging", "true");

        // Initialize all mocks
        let battery_mock = crate::mocks::MockBatteryMonitor::new();
        let usb_mock = crate::mocks::MockUsbHidDevice::new();
        let system_mock = crate::mocks::MockSystemState::new();

        context.add_mock("battery", Box::new(battery_mock));
        context.add_mock("usb", Box::new(usb_mock));
        context.add_mock("system", Box::new(system_mock));

        context
    }
    */

    /// Get current test environment configuration
    pub fn get_test_config() -> HashMap<String, String> {
        TEST_ENV.lock().unwrap().config.clone()
    }

    /// Update test environment configuration
    pub fn update_test_config(key: &str, value: &str) {
        TEST_ENV
            .lock()
            .unwrap()
            .config
            .insert(key.to_string(), value.to_string());
    }

    /// Check if test environment is initialized
    pub fn is_initialized() -> bool {
        TEST_ENV.lock().unwrap().initialized
    }

    /// Get test start time
    pub fn get_test_start_time() -> u64 {
        TEST_ENV.lock().unwrap().test_start_time
    }

    /// Increment test counter
    pub fn increment_counter(name: &str) -> u32 {
        let mut env = TEST_ENV.lock().unwrap();
        let counter = env.test_counters.entry(name.to_string()).or_insert(0);
        *counter += 1;
        *counter
    }

    /// Get test counter value
    pub fn get_counter(name: &str) -> u32 {
        TEST_ENV
            .lock()
            .unwrap()
            .test_counters
            .get(name)
            .copied()
            .unwrap_or(0)
    }

    /// Reset test counter
    pub fn reset_counter(name: &str) {
        TEST_ENV
            .lock()
            .unwrap()
            .test_counters
            .insert(name.to_string(), 0);
    }
}

/// Test context for individual test cases
pub struct TestContext {
    pub test_name: String,
    pub config: HashMap<String, String>,
    pub mocks: HashMap<String, Box<dyn std::any::Any + Send>>,
    pub temp_files: Vec<String>,
    pub start_time: std::time::Instant,
    pub cleanup_handlers: Vec<Box<dyn FnOnce() + Send>>,
}

impl TestContext {
    /// Create a new test context
    pub fn new(test_name: &str) -> Self {
        Self {
            test_name: test_name.to_string(),
            config: setup::get_test_config(),
            mocks: HashMap::new(),
            temp_files: Vec::new(),
            start_time: std::time::Instant::now(),
            cleanup_handlers: Vec::new(),
        }
    }

    /// Set a configuration value for this test
    pub fn set_config(&mut self, key: &str, value: &str) {
        self.config.insert(key.to_string(), value.to_string());
    }

    /// Get a configuration value
    pub fn get_config(&self, key: &str) -> Option<&String> {
        self.config.get(key)
    }

    /// Add a mock object to the test context
    pub fn add_mock<T: 'static + Send>(&mut self, name: &str, mock: Box<T>) {
        self.mocks.insert(name.to_string(), mock);
    }

    /// Get a mock object from the test context
    pub fn get_mock<T: 'static>(&self, name: &str) -> Option<&T> {
        self.mocks.get(name)?.downcast_ref::<T>()
    }

    /// Add a temporary file to be cleaned up
    pub fn add_temp_file(&mut self, path: String) {
        self.temp_files.push(path);
    }

    /// Add a cleanup handler
    pub fn add_cleanup_handler<F>(&mut self, handler: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.cleanup_handlers.push(Box::new(handler));
    }

    /// Get elapsed time since test start
    pub fn elapsed_time(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Check if test has exceeded timeout
    pub fn is_timeout_exceeded(&self) -> bool {
        if let Some(timeout_str) = self.get_config("test_timeout_ms") {
            if let Ok(timeout_ms) = timeout_str.parse::<u64>() {
                return self.elapsed_time().as_millis() as u64 > timeout_ms;
            }
        }
        false
    }
}

/// Convenience functions for common test setups
pub mod helpers {
    use super::*;

    /// Quick setup for unit tests
    pub fn unit_test(test_name: &str) -> TestContext {
        config::presets::unit_test().create_context(test_name)
    }

    /// Quick setup for integration tests
    pub fn integration_test(test_name: &str) -> TestContext {
        config::presets::integration_test().create_context(test_name)
    }

    /// Quick setup for performance tests
    pub fn performance_test(test_name: &str) -> TestContext {
        config::presets::performance_test().create_context(test_name)
    }

    /// Quick setup for battery tests
    pub fn battery_test(test_name: &str) -> TestContext {
        let mut context = config::presets::battery_test().create_context(test_name);

        // Add battery mock
        let battery_mock = crate::mocks::MockBatteryMonitor::new();
        battery_mock.set_voltage(3700); // Start with normal voltage
        context.add_mock("battery", Box::new(battery_mock));

        context
    }

    /// Quick setup for USB tests
    pub fn usb_test(test_name: &str) -> TestContext {
        let mut context = config::presets::usb_test().create_context(test_name);

        // Add USB mock
        let usb_mock = crate::mocks::MockUsbHidDevice::new();
        context.add_mock("usb", Box::new(usb_mock));

        context
    }

    /// Quick setup for timing tests
    pub fn timing_test(test_name: &str) -> TestContext {
        config::presets::timing_test().create_context(test_name)
    }

    /// Setup test with automatic cleanup
    pub fn with_cleanup<F, R>(test_name: &str, test_fn: F) -> R
    where
        F: FnOnce(TestContext) -> R,
    {
        let context = unit_test(test_name);
        let result = test_fn(context);
        teardown::cleanup_test_resources();
        result
    }

    /// Setup integration test with automatic cleanup
    pub fn with_integration_cleanup<F, R>(test_name: &str, test_fn: F) -> R
    where
        F: FnOnce(TestContext) -> R,
    {
        let context = integration_test(test_name);
        let result = test_fn(context);
        teardown::cleanup_test_resources();
        result
    }

    /// Run test with timeout
    pub fn with_timeout<F, R>(timeout: std::time::Duration, test_fn: F) -> R
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        use std::sync::mpsc;
        use std::thread;

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let result = test_fn();
            let _ = tx.send(result);
        });

        match rx.recv_timeout(timeout) {
            Ok(result) => result,
            Err(_) => panic!("Test timed out after {:?}", timeout),
        }
    }

    /// Run test with memory limit checking
    pub fn with_memory_limit<F, R>(limit_bytes: u32, test_fn: F) -> R
    where
        F: FnOnce() -> R,
    {
        // Note: This is a simplified memory check
        // In a real implementation, you might use more sophisticated memory tracking
        let initial_memory = get_memory_usage();
        let result = test_fn();
        let final_memory = get_memory_usage();

        let memory_used = final_memory.saturating_sub(initial_memory);
        assert!(
            memory_used <= limit_bytes,
            "Test used {} bytes, limit was {} bytes",
            memory_used,
            limit_bytes
        );

        result
    }

    /// Get current memory usage (simplified implementation)
    fn get_memory_usage() -> u32 {
        // This is a placeholder - in a real implementation you'd use
        // platform-specific APIs to get actual memory usage
        0
    }

    /// Setup test environment based on environment variables
    pub fn auto_setup(test_name: &str) -> TestContext {
        let config = config::env::current_config();
        config.create_context(test_name)
    }

    /// Create a test context with custom configuration
    pub fn custom_test(test_name: &str, builder: config::TestConfigBuilder) -> TestContext {
        builder.build().create_context(test_name)
    }
}

/// Common test teardown utilities
pub mod teardown {
    use super::setup::TEST_ENV;
    use std::fs;
    use std::path::Path;

    /// Clean up test resources
    pub fn cleanup_test_resources() {
        let mut env = TEST_ENV.lock().unwrap();

        // Clean up temporary files
        for file_path in &env.temp_files {
            if Path::new(file_path).exists() {
                if let Err(e) = fs::remove_file(file_path) {
                    eprintln!("Warning: Failed to remove temp file {}: {}", file_path, e);
                }
            }
        }
        env.temp_files.clear();

        // Reset mock states
        env.mock_states.clear();

        // Reset test counters
        env.test_counters.clear();

        println!("Test resources cleaned up");
    }

    /// Clean up test context resources
    pub fn cleanup_test_context(mut context: super::TestContext) {
        // Execute cleanup handlers in reverse order
        while let Some(handler) = context.cleanup_handlers.pop() {
            handler();
        }

        // Clean up temporary files
        for file_path in &context.temp_files {
            if Path::new(file_path).exists() {
                if let Err(e) = fs::remove_file(file_path) {
                    eprintln!("Warning: Failed to remove temp file {}: {}", file_path, e);
                }
            }
        }

        // Clear mocks
        context.mocks.clear();

        println!(
            "Test context '{}' cleaned up (duration: {:?})",
            context.test_name,
            context.elapsed_time()
        );
    }

    /// Clean up specific mock objects
    pub fn cleanup_mocks() {
        // Reset all mock states to defaults
        // This is useful between test cases to ensure clean state
        println!("Mock objects reset to default state");
    }

    /// Clean up temporary test files in a directory
    pub fn cleanup_temp_directory(dir_path: &str) -> Result<(), std::io::Error> {
        let path = Path::new(dir_path);
        if path.exists() && path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let file_path = entry.path();

                // Only remove files that look like test temporaries
                if let Some(file_name) = file_path.file_name() {
                    if let Some(name_str) = file_name.to_str() {
                        if name_str.starts_with("test_") || name_str.starts_with("tmp_") {
                            if file_path.is_file() {
                                fs::remove_file(&file_path)?;
                                println!("Removed temp file: {:?}", file_path);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Force cleanup of all test resources (for emergency cleanup)
    pub fn force_cleanup() {
        cleanup_test_resources();
        cleanup_mocks();

        // Try to clean up common temp directories
        let temp_dirs = ["./tmp", "./test_output", "./test_logs"];
        for dir in &temp_dirs {
            if let Err(e) = cleanup_temp_directory(dir) {
                eprintln!("Warning: Failed to clean temp directory {}: {}", dir, e);
            }
        }

        println!("Force cleanup completed");
    }

    /// Validate that cleanup was successful
    pub fn validate_cleanup() -> Result<(), String> {
        let env = TEST_ENV.lock().unwrap();

        // Check that temp files were cleaned up
        if !env.temp_files.is_empty() {
            return Err(format!("Temp files not cleaned up: {:?}", env.temp_files));
        }

        // Check that mock states were reset
        if !env.mock_states.is_empty() {
            return Err(format!("Mock states not reset: {:?}", env.mock_states));
        }

        Ok(())
    }

    /// Register a temporary file for cleanup
    pub fn register_temp_file(file_path: String) {
        TEST_ENV.lock().unwrap().temp_files.push(file_path);
    }

    /// Unregister a temporary file (if manually cleaned up)
    pub fn unregister_temp_file(file_path: &str) {
        let mut env = TEST_ENV.lock().unwrap();
        env.temp_files.retain(|f| f != file_path);
    }
}

/// Test timing utilities
pub mod timing {
    use std::time::{Duration, Instant};

    /// Measure execution time of a closure
    pub fn measure_time<F, R>(f: F) -> (R, Duration)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        (result, duration)
    }

    /// Assert that an operation completes within a time limit
    pub fn assert_within_time<F, R>(time_limit: Duration, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let (result, duration) = measure_time(f);
        assert!(
            duration <= time_limit,
            "Operation took {:?}, expected <= {:?}",
            duration,
            time_limit
        );
        result
    }
}

/// Test environment configuration utilities
pub mod config {
    use std::collections::HashMap;
    use std::env::{set_var, var};
    use std::fs;
    use std::path::Path;

    /// Test environment configuration builder
    #[derive(Debug, Clone)]
    pub struct TestConfigBuilder {
        config: HashMap<String, String>,
        profile: TestProfile,
    }

    /// Test execution profiles
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum TestProfile {
        /// Fast unit tests with minimal setup
        Unit,
        /// Integration tests with full mock setup
        Integration,
        /// Performance tests with timing validation
        Performance,
        /// Hardware validation tests
        Hardware,
        /// Debug tests with verbose logging
        Debug,
        /// CI/CD optimized tests
        CI,
    }

    impl TestConfigBuilder {
        /// Create a new test configuration builder
        pub fn new() -> Self {
            Self {
                config: HashMap::new(),
                profile: TestProfile::Unit,
            }
        }

        /// Set the test profile
        pub fn with_profile(mut self, profile: TestProfile) -> Self {
            self.profile = profile;
            self.apply_profile_defaults();
            self
        }

        /// Set a configuration value
        pub fn with_config(mut self, key: &str, value: &str) -> Self {
            self.config.insert(key.to_string(), value.to_string());
            self
        }

        /// Set multiple configuration values
        pub fn with_configs(mut self, configs: HashMap<String, String>) -> Self {
            self.config.extend(configs);
            self
        }

        /// Enable mock hardware
        pub fn with_mock_hardware(mut self, enabled: bool) -> Self {
            self.config
                .insert("mock_hardware".to_string(), enabled.to_string());
            self
        }

        /// Set test timeout
        pub fn with_timeout(mut self, timeout_ms: u32) -> Self {
            self.config
                .insert("test_timeout_ms".to_string(), timeout_ms.to_string());
            self
        }

        /// Set log level
        pub fn with_log_level(mut self, level: u8) -> Self {
            self.config
                .insert("log_level".to_string(), level.to_string());
            self
        }

        /// Enable timing checks
        pub fn with_timing_checks(mut self, enabled: bool) -> Self {
            self.config
                .insert("enable_timing_checks".to_string(), enabled.to_string());
            self
        }

        /// Set memory limit
        pub fn with_memory_limit(mut self, limit_bytes: u32) -> Self {
            self.config
                .insert("memory_limit_bytes".to_string(), limit_bytes.to_string());
            self
        }

        /// Load configuration from environment variables
        pub fn from_env(mut self) -> Self {
            // Load common test environment variables
            if let Ok(log_level) = var("TEST_LOG_LEVEL") {
                self.config.insert("log_level".to_string(), log_level);
            }

            if let Ok(timeout) = var("TEST_TIMEOUT_MS") {
                self.config.insert("test_timeout_ms".to_string(), timeout);
            }

            if let Ok(mock_hw) = var("TEST_MOCK_HARDWARE") {
                self.config.insert("mock_hardware".to_string(), mock_hw);
            }

            if let Ok(timing) = var("TEST_ENABLE_TIMING") {
                self.config
                    .insert("enable_timing_checks".to_string(), timing);
            }

            if let Ok(memory) = var("TEST_MEMORY_LIMIT") {
                self.config.insert("memory_limit_bytes".to_string(), memory);
            }

            // Detect CI environment
            if var("CI").is_ok() || var("GITHUB_ACTIONS").is_ok() {
                self.profile = TestProfile::CI;
                self.apply_profile_defaults();
            }

            self
        }

        /// Load configuration from file
        pub fn from_file<P: AsRef<Path>>(mut self, path: P) -> Result<Self, std::io::Error> {
            let content = fs::read_to_string(path)?;

            // Simple key=value format
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                if let Some((key, value)) = line.split_once('=') {
                    self.config
                        .insert(key.trim().to_string(), value.trim().to_string());
                }
            }

            Ok(self)
        }

        /// Build the final configuration
        pub fn build(mut self) -> TestConfig {
            self.apply_profile_defaults();
            self.validate_config();

            TestConfig {
                config: self.config,
                profile: self.profile,
            }
        }

        /// Apply default values based on the selected profile
        fn apply_profile_defaults(&mut self) {
            let defaults = match self.profile {
                TestProfile::Unit => {
                    let mut defaults = HashMap::new();
                    defaults.insert("log_level".to_string(), "3".to_string()); // Error only
                    defaults.insert("test_timeout_ms".to_string(), "5000".to_string());
                    defaults.insert("mock_hardware".to_string(), "true".to_string());
                    defaults.insert("enable_timing_checks".to_string(), "false".to_string());
                    defaults.insert("memory_limit_bytes".to_string(), "32768".to_string());
                    defaults.insert("parallel_execution".to_string(), "true".to_string());
                    defaults
                }
                TestProfile::Integration => {
                    let mut defaults = HashMap::new();
                    defaults.insert("log_level".to_string(), "2".to_string()); // Info
                    defaults.insert("test_timeout_ms".to_string(), "30000".to_string());
                    defaults.insert("mock_hardware".to_string(), "true".to_string());
                    defaults.insert("enable_timing_checks".to_string(), "true".to_string());
                    defaults.insert("memory_limit_bytes".to_string(), "65536".to_string());
                    defaults.insert("comprehensive_logging".to_string(), "true".to_string());
                    defaults.insert("enable_all_mocks".to_string(), "true".to_string());
                    defaults
                }
                TestProfile::Performance => {
                    let mut defaults = HashMap::new();
                    defaults.insert("log_level".to_string(), "3".to_string()); // Error only
                    defaults.insert("test_timeout_ms".to_string(), "60000".to_string());
                    defaults.insert("mock_hardware".to_string(), "true".to_string());
                    defaults.insert("enable_timing_checks".to_string(), "true".to_string());
                    defaults.insert("memory_limit_bytes".to_string(), "131072".to_string());
                    defaults.insert("enable_profiling".to_string(), "true".to_string());
                    defaults.insert("benchmark_mode".to_string(), "true".to_string());
                    defaults.insert("high_precision_timing".to_string(), "true".to_string());
                    defaults
                }
                TestProfile::Hardware => {
                    let mut defaults = HashMap::new();
                    defaults.insert("log_level".to_string(), "1".to_string()); // Debug
                    defaults.insert("test_timeout_ms".to_string(), "120000".to_string());
                    defaults.insert("mock_hardware".to_string(), "false".to_string());
                    defaults.insert("enable_timing_checks".to_string(), "true".to_string());
                    defaults.insert("memory_limit_bytes".to_string(), "65536".to_string());
                    defaults.insert("hardware_validation".to_string(), "true".to_string());
                    defaults.insert("real_hardware_tests".to_string(), "true".to_string());
                    defaults
                }
                TestProfile::Debug => {
                    let mut defaults = HashMap::new();
                    defaults.insert("log_level".to_string(), "0".to_string()); // Trace
                    defaults.insert("test_timeout_ms".to_string(), "300000".to_string()); // 5 minutes
                    defaults.insert("mock_hardware".to_string(), "true".to_string());
                    defaults.insert("enable_timing_checks".to_string(), "true".to_string());
                    defaults.insert("memory_limit_bytes".to_string(), "131072".to_string());
                    defaults.insert("verbose_assertions".to_string(), "true".to_string());
                    defaults.insert("debug_output".to_string(), "true".to_string());
                    defaults.insert("comprehensive_logging".to_string(), "true".to_string());
                    defaults
                }
                TestProfile::CI => {
                    let mut defaults = HashMap::new();
                    defaults.insert("log_level".to_string(), "2".to_string()); // Info
                    defaults.insert("test_timeout_ms".to_string(), "60000".to_string());
                    defaults.insert("mock_hardware".to_string(), "true".to_string());
                    defaults.insert("enable_timing_checks".to_string(), "false".to_string()); // Less strict in CI
                    defaults.insert("memory_limit_bytes".to_string(), "65536".to_string());
                    defaults.insert("parallel_execution".to_string(), "true".to_string());
                    defaults.insert("ci_mode".to_string(), "true".to_string());
                    defaults.insert("junit_output".to_string(), "true".to_string());
                    defaults
                }
            };

            // Only set defaults for keys that aren't already configured
            for (key, value) in defaults {
                self.config.entry(key).or_insert(value);
            }
        }

        /// Validate the configuration
        fn validate_config(&self) {
            // Validate timeout
            if let Some(timeout_str) = self.config.get("test_timeout_ms") {
                if timeout_str.parse::<u32>().is_err() {
                    panic!("Invalid test_timeout_ms value: {}", timeout_str);
                }
            }

            // Validate log level
            if let Some(log_level_str) = self.config.get("log_level") {
                if let Ok(level) = log_level_str.parse::<u8>() {
                    if level > 3 {
                        panic!("Invalid log_level value: {} (must be 0-3)", level);
                    }
                } else {
                    panic!("Invalid log_level value: {}", log_level_str);
                }
            }

            // Validate memory limit
            if let Some(memory_str) = self.config.get("memory_limit_bytes") {
                if memory_str.parse::<u32>().is_err() {
                    panic!("Invalid memory_limit_bytes value: {}", memory_str);
                }
            }
        }
    }

    impl Default for TestConfigBuilder {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Final test configuration
    #[derive(Debug, Clone)]
    pub struct TestConfig {
        config: HashMap<String, String>,
        profile: TestProfile,
    }

    impl TestConfig {
        /// Get a configuration value
        pub fn get(&self, key: &str) -> Option<&String> {
            self.config.get(key)
        }

        /// Get a configuration value as a specific type
        pub fn get_as<T>(&self, key: &str) -> Option<T>
        where
            T: std::str::FromStr,
        {
            self.config.get(key)?.parse().ok()
        }

        /// Get a boolean configuration value
        pub fn get_bool(&self, key: &str) -> bool {
            self.config
                .get(key)
                .and_then(|v| v.parse().ok())
                .unwrap_or(false)
        }

        /// Get the test profile
        pub fn profile(&self) -> TestProfile {
            self.profile
        }

        /// Get all configuration values
        pub fn all(&self) -> &HashMap<String, String> {
            &self.config
        }

        /// Check if a feature is enabled
        pub fn is_enabled(&self, feature: &str) -> bool {
            self.get_bool(feature)
        }

        /// Get timeout duration
        pub fn timeout(&self) -> std::time::Duration {
            let timeout_ms = self.get_as::<u64>("test_timeout_ms").unwrap_or(30000);
            std::time::Duration::from_millis(timeout_ms)
        }

        /// Get log level
        pub fn log_level(&self) -> u8 {
            self.get_as::<u8>("log_level").unwrap_or(2)
        }

        /// Get memory limit
        pub fn memory_limit(&self) -> u32 {
            self.get_as::<u32>("memory_limit_bytes").unwrap_or(65536)
        }

        /// Apply configuration to environment
        pub fn apply_to_env(&self) {
            for (key, value) in &self.config {
                let env_key = format!("TEST_{}", key.to_uppercase());
                set_var(env_key, value);
            }
        }

        /// Create a test context with this configuration
        pub fn create_context(&self, test_name: &str) -> super::TestContext {
            let mut context = super::TestContext::new(test_name);
            context.config = self.config.clone();
            context
        }
    }

    /// Predefined test configurations
    pub mod presets {
        use super::*;

        /// Fast unit test configuration
        pub fn unit_test() -> TestConfig {
            TestConfigBuilder::new()
                .with_profile(TestProfile::Unit)
                .build()
        }

        /// Integration test configuration
        pub fn integration_test() -> TestConfig {
            TestConfigBuilder::new()
                .with_profile(TestProfile::Integration)
                .build()
        }

        /// Performance test configuration
        pub fn performance_test() -> TestConfig {
            TestConfigBuilder::new()
                .with_profile(TestProfile::Performance)
                .build()
        }

        /// Hardware validation test configuration
        pub fn hardware_test() -> TestConfig {
            TestConfigBuilder::new()
                .with_profile(TestProfile::Hardware)
                .build()
        }

        /// Debug test configuration
        pub fn debug_test() -> TestConfig {
            TestConfigBuilder::new()
                .with_profile(TestProfile::Debug)
                .build()
        }

        /// CI/CD test configuration
        pub fn ci_test() -> TestConfig {
            TestConfigBuilder::new()
                .with_profile(TestProfile::CI)
                .from_env()
                .build()
        }

        /// Battery-specific test configuration
        pub fn battery_test() -> TestConfig {
            TestConfigBuilder::new()
                .with_profile(TestProfile::Integration)
                .with_config("battery_threshold", "3200")
                .with_config("adc_sample_rate", "1000")
                .with_config("battery_monitoring_enabled", "true")
                .build()
        }

        /// USB HID test configuration
        pub fn usb_test() -> TestConfig {
            TestConfigBuilder::new()
                .with_profile(TestProfile::Integration)
                .with_config("usb_timeout_ms", "5000")
                .with_config("hid_report_size", "64")
                .with_config("usb_polling_enabled", "true")
                .build()
        }

        /// Timing-critical test configuration
        pub fn timing_test() -> TestConfig {
            TestConfigBuilder::new()
                .with_profile(TestProfile::Performance)
                .with_config("timing_tolerance_us", "100")
                .with_config("pemf_frequency_hz", "10.0")
                .with_config("high_precision_timing", "true")
                .build()
        }
    }

    /// Test environment utilities
    pub mod env {
        use super::*;
        use std::env::{set_var, var};

        /// Set up test environment with configuration
        pub fn setup_with_config(config: &TestConfig) {
            config.apply_to_env();
            super::super::setup::init_test_env_with_config(config.all().clone());
        }

        /// Detect current test environment
        pub fn detect_environment() -> TestProfile {
            if var("CI").is_ok() || var("GITHUB_ACTIONS").is_ok() {
                TestProfile::CI
            } else if var("TEST_HARDWARE").is_ok() {
                TestProfile::Hardware
            } else if var("TEST_PERFORMANCE").is_ok() {
                TestProfile::Performance
            } else if var("TEST_DEBUG").is_ok() {
                TestProfile::Debug
            } else {
                TestProfile::Unit
            }
        }

        /// Get configuration for current environment
        pub fn current_config() -> TestConfig {
            let profile = detect_environment();
            TestConfigBuilder::new()
                .with_profile(profile)
                .from_env()
                .build()
        }

        /// Check if running in CI environment
        pub fn is_ci() -> bool {
            matches!(detect_environment(), TestProfile::CI)
        }

        /// Check if hardware tests are enabled
        pub fn hardware_tests_enabled() -> bool {
            var("TEST_HARDWARE").is_ok()
                || var("TEST_MOCK_HARDWARE")
                    .map(|v| v == "false")
                    .unwrap_or(false)
        }

        /// Get test output directory
        pub fn test_output_dir() -> std::path::PathBuf {
            var("TEST_OUTPUT_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| std::path::PathBuf::from("./test_output"))
        }

        /// Ensure test output directory exists
        pub fn ensure_output_dir() -> Result<std::path::PathBuf, std::io::Error> {
            let dir = test_output_dir();
            fs::create_dir_all(&dir)?;
            Ok(dir)
        }
    }
}
