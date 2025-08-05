//! Test Execution Handler for USB HID Command Processing
//! 
//! This module handles test execution commands received via USB HID and coordinates
//! with the test framework and result serialization system to execute tests and
//! transmit results back to the host.
//! 
//! Requirements: 4.2, 6.1, 6.4

use heapless::{Vec, String};
use core::option::Option::{self, Some, None};
use core::result::Result::{self, Ok, Err};
use crate::command::parsing::{CommandReport, TestCommand, ErrorCode};
use crate::test_framework::{TestRunner, TestSuiteResult};
use crate::test_result_serializer::{TestResultCollector, TestResultStatus};
use crate::logging::LogReport;

/// Maximum number of test suites that can be registered
pub const MAX_TEST_SUITES: usize = 8;

/// Maximum length for test suite names
pub const MAX_SUITE_NAME_LENGTH: usize = 32;

/// Test execution parameters from USB command payload
#[derive(Debug, Clone)]
pub struct TestExecutionParams {
    /// Test suite name to execute (empty = run all)
    pub suite_name: String<MAX_SUITE_NAME_LENGTH>,
    /// Specific test name within suite (empty = run all in suite)
    pub test_name: String<MAX_SUITE_NAME_LENGTH>,
    /// Test timeout in milliseconds
    pub timeout_ms: u32,
    /// Test execution flags
    pub flags: TestExecutionFlags,
}

/// Test execution flags
#[derive(Debug, Clone, Copy)]
pub struct TestExecutionFlags {
    /// Run tests in parallel (if supported)
    pub parallel: bool,
    /// Stop on first failure
    pub stop_on_failure: bool,
    /// Collect detailed timing information
    pub collect_timing: bool,
    /// Enable verbose logging during test execution
    pub verbose_logging: bool,
}

impl Default for TestExecutionFlags {
    fn default() -> Self {
        Self {
            parallel: false,
            stop_on_failure: false,
            collect_timing: true,
            verbose_logging: false,
        }
    }
}

/// Test execution status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TestExecutionStatus {
    Idle,
    Running,
    Completed,
    Failed,
    Timeout,
}

/// Test execution handler for processing USB HID test commands
pub struct TestExecutionHandler {
    /// Registered test suites
    test_suites: Vec<(&'static str, fn() -> TestRunner), MAX_TEST_SUITES>,
    /// Test result collector for batching and transmission
    result_collector: TestResultCollector,
    /// Current execution status
    execution_status: TestExecutionStatus,
    /// Current execution parameters
    current_params: Option<TestExecutionParams>,
    /// Execution statistics
    total_executions: u32,
    successful_executions: u32,
    failed_executions: u32,
}

impl Default for TestExecutionHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl TestExecutionHandler {
    /// Create a new test execution handler
    pub const fn new() -> Self {
        Self {
            test_suites: Vec::new(),
            result_collector: TestResultCollector::new(),
            execution_status: TestExecutionStatus::Idle,
            current_params: None,
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
        }
    }

    /// Register a test suite with the handler
    pub fn register_test_suite(&mut self, name: &'static str, suite_factory: fn() -> TestRunner) -> Result<(), &'static str> {
        self.test_suites.push((name, suite_factory)).map_err(|_| "Test suite registry full")
    }

    /// Process a test execution command from USB HID
    pub fn process_test_command(&mut self, command: &CommandReport) -> Result<Vec<LogReport, 8>, ErrorCode> {
        match command.get_test_command() {
            Some(TestCommand::RunTestSuite) => self.handle_run_test_suite(command),
            Some(TestCommand::GetTestResults) => self.handle_get_test_results(command),
            Some(TestCommand::ClearTestResults) => self.handle_clear_test_results(command),
            Some(TestCommand::ExecuteTest) => self.handle_execute_single_test(command),
            _ => Err(ErrorCode::UnsupportedCommand),
        }
    }

    /// Handle RunTestSuite command
    fn handle_run_test_suite(&mut self, command: &CommandReport) -> Result<Vec<LogReport, 8>, ErrorCode> {
        // Parse execution parameters from command payload
        let params = self.parse_execution_params(&command.payload)?;
        
        // Check if already running
        if self.execution_status == TestExecutionStatus::Running {
            return Err(ErrorCode::SystemNotReady);
        }

        // Start test execution
        self.execution_status = TestExecutionStatus::Running;
        self.current_params = Some(params.clone());
        self.total_executions += 1;

        // Execute the requested test suite(s)
        let execution_result = if params.suite_name.is_empty() {
            // Run all test suites
            self.execute_all_suites(&params)
        } else {
            // Run specific test suite
            self.execute_specific_suite(&params.suite_name, &params)
        };

        // Update execution status based on result
        match execution_result {
            Ok(_) => {
                self.execution_status = TestExecutionStatus::Completed;
                self.successful_executions += 1;
            }
            Err(_) => {
                self.execution_status = TestExecutionStatus::Failed;
                self.failed_executions += 1;
            }
        }

        // Create response reports
        let mut responses = Vec::new();
        
        // Add acknowledgment
        let ack_response = CommandReport::success_response(command.command_id, &[0x00])
            .map_err(|_| ErrorCode::SystemNotReady)?;
        let ack_report = LogReport::try_from(ack_response.serialize())
            .map_err(|_| ErrorCode::SystemNotReady)?;
        responses.push(ack_report).map_err(|_| ErrorCode::SystemNotReady)?;

        // Add status update
        if let Ok(status_report) = self.result_collector.get_serializer_mut().create_status_update(
            TestResultStatus::Running,
            "Test suite execution started"
        ) {
            let _ = responses.push(status_report);
        }

        Ok(responses)
    }

    /// Handle GetTestResults command
    fn handle_get_test_results(&mut self, command: &CommandReport) -> Result<Vec<LogReport, 8>, ErrorCode> {
        let mut responses = Vec::new();

        // Add acknowledgment
        let ack_response = CommandReport::success_response(command.command_id, &[0x00])
            .map_err(|_| ErrorCode::SystemNotReady)?;
        let ack_report = LogReport::try_from(ack_response.serialize())
            .map_err(|_| ErrorCode::SystemNotReady)?;
        responses.push(ack_report).map_err(|_| ErrorCode::SystemNotReady)?;

        // Get next batch of test results
        if let Some(batch_reports) = self.result_collector.get_next_batch() {
            for report in batch_reports {
                if responses.push(report).is_err() {
                    break; // Response vector is full
                }
            }
        }

        // Get suite summaries
        while let Some(suite_report) = self.result_collector.get_next_suite_summary() {
            if responses.push(suite_report).is_err() {
                break; // Response vector is full
            }
        }

        Ok(responses)
    }

    /// Handle ClearTestResults command
    fn handle_clear_test_results(&mut self, command: &CommandReport) -> Result<Vec<LogReport, 8>, ErrorCode> {
        // Clear all pending results
        self.result_collector.clear();
        self.execution_status = TestExecutionStatus::Idle;
        self.current_params = None;

        // Create acknowledgment response
        let mut responses = Vec::new();
        let ack_response = CommandReport::success_response(command.command_id, &[0x00])
            .map_err(|_| ErrorCode::SystemNotReady)?;
        let ack_report = LogReport::try_from(ack_response.serialize())
            .map_err(|_| ErrorCode::SystemNotReady)?;
        responses.push(ack_report).map_err(|_| ErrorCode::SystemNotReady)?;

        Ok(responses)
    }

    /// Handle ExecuteTest command (single test execution)
    fn handle_execute_single_test(&mut self, command: &CommandReport) -> Result<Vec<LogReport, 8>, ErrorCode> {
        let params = self.parse_execution_params(&command.payload)?;
        
        if params.suite_name.is_empty() || params.test_name.is_empty() {
            return Err(ErrorCode::InvalidFormat);
        }

        // Execute the specific test
        let execution_result = self.execute_single_test(&params.suite_name, &params.test_name, &params);

        // Create response
        let mut responses = Vec::new();
        let ack_response = CommandReport::success_response(command.command_id, &[0x00])
            .map_err(|_| ErrorCode::SystemNotReady)?;
        let ack_report = LogReport::try_from(ack_response.serialize())
            .map_err(|_| ErrorCode::SystemNotReady)?;
        responses.push(ack_report).map_err(|_| ErrorCode::SystemNotReady)?;

        // Add test result if available
        if execution_result.is_ok() {
            if let Some(batch_reports) = self.result_collector.get_next_batch() {
                for report in batch_reports.into_iter().take(responses.capacity() - responses.len()) {
                    let _ = responses.push(report);
                }
            }
        }

        Ok(responses)
    }

    /// Execute all registered test suites
    fn execute_all_suites(&mut self, params: &TestExecutionParams) -> Result<(), &'static str> {
        // Clone the test suite registry to avoid borrowing issues
        let mut suites_to_run: Vec<(&'static str, fn() -> TestRunner), MAX_TEST_SUITES> = Vec::new();
        for (suite_name, suite_factory) in &self.test_suites {
            if suites_to_run.push((*suite_name, *suite_factory)).is_err() {
                break; // Registry is full
            }
        }

        for (suite_name, suite_factory) in suites_to_run {
            let suite_result = self.execute_suite_internal(suite_name, &suite_factory, params)?;
            self.result_collector.add_suite_result(suite_result)
                .map_err(|_| "Failed to collect suite result")?;
        }
        Ok(())
    }

    /// Execute a specific test suite by name
    fn execute_specific_suite(&mut self, suite_name: &str, params: &TestExecutionParams) -> Result<(), &'static str> {
        // Find the suite factory first
        let mut target_factory: Option<fn() -> TestRunner> = None;
        for (name, suite_factory) in &self.test_suites {
            if *name == suite_name {
                target_factory = Some(*suite_factory);
                break;
            }
        }

        if let Some(suite_factory) = target_factory {
            let suite_result = self.execute_suite_internal(suite_name, &suite_factory, params)?;
            self.result_collector.add_suite_result(suite_result)
                .map_err(|_| "Failed to collect suite result")?;
            Ok(())
        } else {
            Err("Test suite not found")
        }
    }

    /// Execute a single test within a suite
    fn execute_single_test(&mut self, suite_name: &str, test_name: &str, _params: &TestExecutionParams) -> Result<(), &'static str> {
        // Find the suite factory first
        let mut target_factory: Option<fn() -> TestRunner> = None;
        for (name, suite_factory) in &self.test_suites {
            if *name == suite_name {
                target_factory = Some(*suite_factory);
                break;
            }
        }

        if let Some(suite_factory) = target_factory {
            let runner = suite_factory();
            if let Some(test_result) = runner.run_test(test_name) {
                self.result_collector.add_test_result(test_result)
                    .map_err(|_| "Failed to collect test result")?;
                Ok(())
            } else {
                Err("Test not found in suite")
            }
        } else {
            Err("Test suite not found")
        }
    }

    /// Internal suite execution with error handling
    fn execute_suite_internal(&mut self, _suite_name: &str, suite_factory: &fn() -> TestRunner, _params: &TestExecutionParams) -> Result<TestSuiteResult, &'static str> {
        let runner = suite_factory();
        let suite_result = runner.run_all();
        
        // Add individual test results to collector
        for test_result in &suite_result.test_results {
            self.result_collector.add_test_result(test_result.clone())
                .map_err(|_| "Failed to collect individual test result")?;
        }

        Ok(suite_result)
    }

    /// Parse test execution parameters from command payload
    fn parse_execution_params(&self, payload: &[u8]) -> Result<TestExecutionParams, ErrorCode> {
        if payload.is_empty() {
            // Default parameters
            return Ok(TestExecutionParams {
                suite_name: String::new(),
                test_name: String::new(),
                timeout_ms: 30000, // 30 second default timeout
                flags: TestExecutionFlags::default(),
            });
        }

        // Parse payload format: [timeout:4][flags:1][suite_name_len:1][suite_name:N][test_name_len:1][test_name:M]
        if payload.len() < 6 {
            return Err(ErrorCode::InvalidFormat);
        }

        // Parse timeout (bytes 0-3)
        let timeout_bytes = [payload[0], payload[1], payload[2], payload[3]];
        let timeout_ms = u32::from_le_bytes(timeout_bytes);

        // Parse flags (byte 4)
        let flags_byte = payload[4];
        let flags = TestExecutionFlags {
            parallel: (flags_byte & 0x01) != 0,
            stop_on_failure: (flags_byte & 0x02) != 0,
            collect_timing: (flags_byte & 0x04) != 0,
            verbose_logging: (flags_byte & 0x08) != 0,
        };

        // Parse suite name
        let suite_name_len = payload[5] as usize;
        let mut suite_name = String::new();
        if suite_name_len > 0 && payload.len() > 6 + suite_name_len {
            let suite_name_bytes = &payload[6..6 + suite_name_len];
            if let Ok(name_str) = core::str::from_utf8(suite_name_bytes) {
                let _ = suite_name.push_str(name_str);
            }
        }

        // Parse test name
        let mut test_name = String::new();
        let test_name_start = 6 + suite_name_len;
        if payload.len() > test_name_start {
            let test_name_len = payload[test_name_start] as usize;
            if test_name_len > 0 && payload.len() > test_name_start + 1 + test_name_len {
                let test_name_bytes = &payload[test_name_start + 1..test_name_start + 1 + test_name_len];
                if let Ok(name_str) = core::str::from_utf8(test_name_bytes) {
                    let _ = test_name.push_str(name_str);
                }
            }
        }

        Ok(TestExecutionParams {
            suite_name,
            test_name,
            timeout_ms,
            flags,
        })
    }

    /// Get current execution status
    pub fn get_execution_status(&self) -> TestExecutionStatus {
        self.execution_status
    }

    /// Check if there are pending test results to transmit
    pub fn has_pending_results(&self) -> bool {
        self.result_collector.has_pending_results()
    }

    /// Get execution statistics
    pub fn get_stats(&self) -> TestExecutionStats {
        TestExecutionStats {
            total_executions: self.total_executions,
            successful_executions: self.successful_executions,
            failed_executions: self.failed_executions,
            registered_suites: self.test_suites.len(),
            execution_status: self.execution_status,
            collector_stats: self.result_collector.get_stats(),
        }
    }

}

/// Test execution statistics
#[derive(Debug, Clone, Copy)]
pub struct TestExecutionStats {
    pub total_executions: u32,
    pub successful_executions: u32,
    pub failed_executions: u32,
    pub registered_suites: usize,
    pub execution_status: TestExecutionStatus,
    pub collector_stats: crate::test_result_serializer::CollectorStats,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_framework::{TestResult, TestCase};

    fn create_sample_test_suite() -> TestRunner {
        let mut runner = TestRunner::new("sample_suite");
        let _ = runner.register_test("test1", || TestResult::pass());
        let _ = runner.register_test("test2", || TestResult::fail("test error"));
        runner
    }

    #[test]
    fn test_register_test_suite() {
        let mut handler = TestExecutionHandler::new();
        assert!(handler.register_test_suite("sample", create_sample_test_suite).is_ok());
        assert_eq!(handler.test_suites.len(), 1);
    }

    #[test]
    fn test_parse_execution_params() {
        let handler = TestExecutionHandler::new();
        
        // Test empty payload (default params)
        let params = handler.parse_execution_params(&[]).unwrap();
        assert_eq!(params.timeout_ms, 30000);
        assert!(params.suite_name.is_empty());
        
        // Test with timeout and flags
        let payload = [
            0x10, 0x27, 0x00, 0x00, // timeout: 10000ms
            0x05, // flags: parallel + stop_on_failure + collect_timing
            0x04, // suite name length: 4
            b't', b'e', b's', b't', // suite name: "test"
            0x00, // test name length: 0
        ];
        
        let params = handler.parse_execution_params(&payload).unwrap();
        assert_eq!(params.timeout_ms, 10000);
        assert!(params.flags.parallel);
        assert!(params.flags.stop_on_failure);
        assert!(params.flags.collect_timing);
        assert_eq!(params.suite_name.as_str(), "test");
    }
}