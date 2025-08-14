//! Test Result Serialization for USB HID Communication
//!
//! This module provides serialization capabilities for test results to be transmitted
//! via USB HID reports. It converts test framework results into standardized 64-byte
//! HID reports that can be processed by the Python test framework.
//!
//! Requirements: 4.2, 6.1, 6.4

use crate::logging::LogReport;
use crate::test_framework::{TestExecutionResult, TestResult, TestSuiteResult, TestSuiteStats};
use core::convert::From;
use core::default::Default;
use core::option::Option::{self, None, Some};
use core::result::Result::{self, Err, Ok};
use heapless::{String, Vec};

/// Maximum number of test results that can be batched in a single transmission
pub const MAX_BATCH_SIZE: usize = 8;

/// Maximum length for serialized test names and error messages
pub const MAX_SERIALIZED_NAME_LENGTH: usize = 32;
pub const MAX_SERIALIZED_ERROR_LENGTH: usize = 40;

/// Test result report types for USB HID transmission
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum TestReportType {
    /// Individual test result
    TestResult = 0x92,
    /// Test suite summary
    SuiteSummary = 0x93,
    /// Test execution status update
    StatusUpdate = 0x94,
    /// Test batch start marker
    BatchStart = 0x95,
    /// Test batch end marker
    BatchEnd = 0x96,
}

/// Serialized test result for USB HID transmission
/// Format: [Type:1][TestID:1][Status:1][Reserved:1][Name:32][Error:28] = 64 bytes
#[derive(Debug, Clone)]
pub struct SerializedTestResult {
    /// Report type identifier
    pub report_type: TestReportType,
    /// Test identifier (sequence number)
    pub test_id: u8,
    /// Test result status
    pub status: TestResultStatus,
    /// Test name (truncated to fit)
    pub test_name: String<MAX_SERIALIZED_NAME_LENGTH>,
    /// Error message (if test failed)
    pub error_message: String<MAX_SERIALIZED_ERROR_LENGTH>,
    /// Execution time in milliseconds
    pub execution_time_ms: u32,
}

/// Test result status codes for serialization
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum TestResultStatus {
    Pass = 0x00,
    Fail = 0x01,
    Skip = 0x02,
    Running = 0x03,
    Timeout = 0x04,
    Error = 0x05,
}

impl From<&TestResult> for TestResultStatus {
    fn from(result: &TestResult) -> Self {
        match result {
            TestResult::Pass => TestResultStatus::Pass,
            TestResult::Fail(_) => TestResultStatus::Fail,
            TestResult::Skip(_) => TestResultStatus::Skip,
        }
    }
}

/// Serialized test suite summary for USB HID transmission
/// Format: [Type:1][SuiteID:1][TotalTests:2][Passed:2][Failed:2][Skipped:2][ExecTime:4][Name:32][Reserved:16] = 64 bytes
#[derive(Debug, Clone)]
pub struct SerializedSuiteSummary {
    /// Report type identifier
    pub report_type: TestReportType,
    /// Suite identifier
    pub suite_id: u8,
    /// Test statistics
    pub stats: TestSuiteStats,
    /// Suite name (truncated to fit)
    pub suite_name: String<MAX_SERIALIZED_NAME_LENGTH>,
}

/// Test result serializer for converting test framework results to USB HID reports
#[derive(Debug)]
pub struct TestResultSerializer {
    /// Current test ID counter for sequencing
    test_id_counter: u8,
    /// Current suite ID counter for sequencing
    suite_id_counter: u8,
    /// Batch tracking
    current_batch_size: usize,
    /// Statistics
    serialized_count: u32,
    error_count: u32,
}

impl Default for TestResultSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl TestResultSerializer {
    /// Create a new test result serializer
    pub const fn new() -> Self {
        Self {
            test_id_counter: 0,
            suite_id_counter: 0,
            current_batch_size: 0,
            serialized_count: 0,
            error_count: 0,
        }
    }

    /// Serialize a single test execution result to USB HID report
    pub fn serialize_test_result(
        &mut self,
        result: &TestExecutionResult,
    ) -> Result<LogReport, &'static str> {
        // Create serialized test result
        let mut test_name = String::new();
        let _ = test_name.push_str(&result.test_name);

        let mut error_message = String::new();
        if let TestResult::Fail(ref msg) = result.result {
            let _ = error_message.push_str(msg);
        } else if let TestResult::Skip(ref msg) = result.result {
            let _ = error_message.push_str(msg);
        }

        let serialized = SerializedTestResult {
            report_type: TestReportType::TestResult,
            test_id: self.get_next_test_id(),
            status: TestResultStatus::from(&result.result),
            test_name,
            error_message,
            execution_time_ms: result.execution_time_us.unwrap_or(0) / 1000, // Convert to ms
        };

        // Convert to USB HID report
        self.serialize_to_hid_report(&serialized)
    }

    /// Serialize a test suite summary to USB HID report
    pub fn serialize_suite_summary(
        &mut self,
        suite_result: &TestSuiteResult,
    ) -> Result<LogReport, &'static str> {
        let mut suite_name = String::new();
        let _ = suite_name.push_str(&suite_result.suite_name);

        let serialized = SerializedSuiteSummary {
            report_type: TestReportType::SuiteSummary,
            suite_id: self.get_next_suite_id(),
            stats: suite_result.stats.clone(),
            suite_name,
        };

        self.serialize_suite_to_hid_report(&serialized)
    }

    /// Serialize multiple test results as a batch
    pub fn serialize_test_batch(
        &mut self,
        results: &[TestExecutionResult],
    ) -> Result<Vec<LogReport, 16>, &'static str> {
        let mut reports = Vec::new();

        // Add batch start marker
        let batch_start = self.create_batch_marker(TestReportType::BatchStart)?;
        reports
            .push(batch_start)
            .map_err(|_| "Batch reports vector full")?;

        // Serialize individual test results
        for result in results.iter().take(MAX_BATCH_SIZE) {
            match self.serialize_test_result(result) {
                Ok(report) => {
                    if reports.push(report).is_err() {
                        break; // Vector is full
                    }
                    self.current_batch_size += 1;
                }
                Err(_) => {
                    self.error_count += 1;
                    continue; // Skip failed serialization
                }
            }
        }

        // Add batch end marker
        let batch_end = self.create_batch_marker(TestReportType::BatchEnd)?;
        reports
            .push(batch_end)
            .map_err(|_| "Batch reports vector full")?;

        self.current_batch_size = 0; // Reset batch size
        Ok(reports)
    }

    /// Create a status update report for test execution progress
    pub fn create_status_update(
        &mut self,
        status: TestResultStatus,
        message: &str,
    ) -> Result<LogReport, &'static str> {
        let mut status_message = String::new();
        let _ = status_message.push_str(message);

        let serialized = SerializedTestResult {
            report_type: TestReportType::StatusUpdate,
            test_id: self.get_next_test_id(),
            status,
            test_name: String::new(), // Empty for status updates
            error_message: status_message,
            execution_time_ms: 0,
        };

        self.serialize_to_hid_report(&serialized)
    }

    /// Get statistics for the serializer
    pub fn get_stats(&self) -> SerializerStats {
        SerializerStats {
            serialized_count: self.serialized_count,
            error_count: self.error_count,
            current_test_id: self.test_id_counter,
            current_suite_id: self.suite_id_counter,
        }
    }

    /// Reset serializer state
    pub fn reset(&mut self) {
        self.test_id_counter = 0;
        self.suite_id_counter = 0;
        self.current_batch_size = 0;
        self.serialized_count = 0;
        self.error_count = 0;
    }

    /// Get next test ID and increment counter
    fn get_next_test_id(&mut self) -> u8 {
        let id = self.test_id_counter;
        self.test_id_counter = self.test_id_counter.wrapping_add(1);
        id
    }

    /// Get next suite ID and increment counter
    fn get_next_suite_id(&mut self) -> u8 {
        let id = self.suite_id_counter;
        self.suite_id_counter = self.suite_id_counter.wrapping_add(1);
        id
    }

    /// Convert SerializedTestResult to USB HID report
    fn serialize_to_hid_report(
        &mut self,
        result: &SerializedTestResult,
    ) -> Result<LogReport, &'static str> {
        let mut buffer = [0u8; 64];

        // Header: [Type:1][TestID:1][Status:1][Reserved:1] = 4 bytes
        buffer[0] = result.report_type as u8;
        buffer[1] = result.test_id;
        buffer[2] = result.status as u8;
        buffer[3] = 0; // Reserved

        // Test name: bytes 4-35 (32 bytes)
        let name_bytes = result.test_name.as_bytes();
        let name_len = core::cmp::min(name_bytes.len(), MAX_SERIALIZED_NAME_LENGTH);
        buffer[4..4 + name_len].copy_from_slice(&name_bytes[..name_len]);

        // Error message: bytes 36-63 (28 bytes)
        let error_bytes = result.error_message.as_bytes();
        let error_len = core::cmp::min(error_bytes.len(), MAX_SERIALIZED_ERROR_LENGTH - 4); // Reserve 4 bytes for execution time
        buffer[36..36 + error_len].copy_from_slice(&error_bytes[..error_len]);

        // Execution time: bytes 60-63 (4 bytes, little-endian)
        let time_bytes = result.execution_time_ms.to_le_bytes();
        buffer[60..64].copy_from_slice(&time_bytes);

        self.serialized_count += 1;
        LogReport::try_from(buffer)
    }

    /// Convert SerializedSuiteSummary to USB HID report
    fn serialize_suite_to_hid_report(
        &mut self,
        summary: &SerializedSuiteSummary,
    ) -> Result<LogReport, &'static str> {
        let mut buffer = [0u8; 64];

        // Header: [Type:1][SuiteID:1][Reserved:2] = 4 bytes
        buffer[0] = summary.report_type as u8;
        buffer[1] = summary.suite_id;
        buffer[2] = 0; // Reserved
        buffer[3] = 0; // Reserved

        // Statistics: [TotalTests:2][Passed:2][Failed:2][Skipped:2] = 8 bytes
        let total_bytes = summary.stats.total_tests.to_le_bytes();
        buffer[4..6].copy_from_slice(&total_bytes);

        let passed_bytes = summary.stats.passed.to_le_bytes();
        buffer[6..8].copy_from_slice(&passed_bytes);

        let failed_bytes = summary.stats.failed.to_le_bytes();
        buffer[8..10].copy_from_slice(&failed_bytes);

        let skipped_bytes = summary.stats.skipped.to_le_bytes();
        buffer[10..12].copy_from_slice(&skipped_bytes);

        // Execution time: bytes 12-15 (4 bytes)
        if let Some(exec_time) = summary.stats.execution_time_ms {
            let time_bytes = exec_time.to_le_bytes();
            buffer[12..16].copy_from_slice(&time_bytes);
        }

        // Suite name: bytes 16-47 (32 bytes)
        let name_bytes = summary.suite_name.as_bytes();
        let name_len = core::cmp::min(name_bytes.len(), MAX_SERIALIZED_NAME_LENGTH);
        buffer[16..16 + name_len].copy_from_slice(&name_bytes[..name_len]);

        // Remaining bytes 48-63 are reserved/padding (already zeroed)

        self.serialized_count += 1;
        LogReport::try_from(buffer)
    }

    /// Create a batch marker report
    fn create_batch_marker(
        &mut self,
        marker_type: TestReportType,
    ) -> Result<LogReport, &'static str> {
        let mut buffer = [0u8; 64];

        buffer[0] = marker_type as u8;
        buffer[1] = self.current_batch_size as u8; // Include batch size in marker
                                                   // Remaining bytes are zero (padding)

        LogReport::try_from(buffer)
    }
}

/// Statistics for the test result serializer
#[derive(Debug, Clone, Copy)]
pub struct SerializerStats {
    pub serialized_count: u32,
    pub error_count: u32,
    pub current_test_id: u8,
    pub current_suite_id: u8,
}

/// Test result collection and batching system
#[derive(Debug)]
pub struct TestResultCollector {
    /// Collected test results waiting for transmission
    pending_results: Vec<TestExecutionResult, 32>,
    /// Completed test suites
    completed_suites: Vec<TestSuiteResult, 8>,
    /// Serializer instance
    serializer: TestResultSerializer,
    /// Collection statistics
    collected_count: u32,
    transmitted_count: u32,
}

impl Default for TestResultCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl TestResultCollector {
    /// Create a new test result collector
    pub const fn new() -> Self {
        Self {
            pending_results: Vec::new(),
            completed_suites: Vec::new(),
            serializer: TestResultSerializer::new(),
            collected_count: 0,
            transmitted_count: 0,
        }
    }

    /// Add a test result to the collection
    pub fn add_test_result(&mut self, result: TestExecutionResult) -> Result<(), &'static str> {
        self.pending_results
            .push(result)
            .map_err(|_| "Result collection full")?;
        self.collected_count += 1;
        Ok(())
    }

    /// Add a completed test suite
    pub fn add_suite_result(&mut self, suite: TestSuiteResult) -> Result<(), &'static str> {
        self.completed_suites
            .push(suite)
            .map_err(|_| "Suite collection full")?;
        Ok(())
    }

    /// Get the next batch of serialized test results for transmission
    pub fn get_next_batch(&mut self) -> Option<Vec<LogReport, 16>> {
        if self.pending_results.is_empty() {
            return None;
        }

        // Take up to MAX_BATCH_SIZE results
        let batch_size = core::cmp::min(self.pending_results.len(), MAX_BATCH_SIZE);
        let mut batch_results: Vec<TestExecutionResult, MAX_BATCH_SIZE> = Vec::new();

        // Extract results for the batch
        for _ in 0..batch_size {
            if let Some(result) = self.pending_results.pop() {
                if batch_results.push(result).is_err() {
                    break; // Batch is full
                }
            }
        }

        // Serialize the batch
        match self.serializer.serialize_test_batch(&batch_results) {
            Ok(reports) => {
                self.transmitted_count += batch_results.len() as u32;
                Some(reports)
            }
            Err(_) => None,
        }
    }

    /// Get the next suite summary for transmission
    pub fn get_next_suite_summary(&mut self) -> Option<LogReport> {
        if let Some(suite) = self.completed_suites.pop() {
            match self.serializer.serialize_suite_summary(&suite) {
                Ok(report) => Some(report),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    /// Check if there are pending results to transmit
    pub fn has_pending_results(&self) -> bool {
        !self.pending_results.is_empty() || !self.completed_suites.is_empty()
    }

    /// Get collection statistics
    pub fn get_stats(&self) -> CollectorStats {
        CollectorStats {
            collected_count: self.collected_count,
            transmitted_count: self.transmitted_count,
            pending_results: self.pending_results.len(),
            pending_suites: self.completed_suites.len(),
            serializer_stats: self.serializer.get_stats(),
        }
    }

    /// Clear all pending results and reset state
    pub fn clear(&mut self) {
        self.pending_results.clear();
        self.completed_suites.clear();
        self.serializer.reset();
        self.collected_count = 0;
        self.transmitted_count = 0;
    }

    /// Get mutable reference to the serializer for creating status updates
    pub fn get_serializer_mut(&mut self) -> &mut TestResultSerializer {
        &mut self.serializer
    }
}

/// Statistics for the test result collector
#[derive(Debug, Clone, Copy)]
pub struct CollectorStats {
    pub collected_count: u32,
    pub transmitted_count: u32,
    pub pending_results: usize,
    pub pending_suites: usize,
    pub serializer_stats: SerializerStats,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_framework::{TestExecutionResult, TestResult, TestSuiteResult, TestSuiteStats};

    #[test]
    fn test_serialize_test_result() {
        let mut serializer = TestResultSerializer::new();

        let test_result = TestExecutionResult::new("sample_test", TestResult::pass());
        let report = serializer.serialize_test_result(&test_result).unwrap();

        // Verify report structure
        let data = report.as_bytes();
        assert_eq!(data[0], TestReportType::TestResult as u8);
        assert_eq!(data[2], TestResultStatus::Pass as u8);
    }

    #[test]
    fn test_serialize_suite_summary() {
        let mut serializer = TestResultSerializer::new();

        let mut suite_result = TestSuiteResult::new("test_suite");
        suite_result.stats.total_tests = 5;
        suite_result.stats.passed = 3;
        suite_result.stats.failed = 1;
        suite_result.stats.skipped = 1;

        let report = serializer.serialize_suite_summary(&suite_result).unwrap();

        // Verify report structure
        let data = report.as_bytes();
        assert_eq!(data[0], TestReportType::SuiteSummary as u8);

        // Check statistics
        let total = u16::from_le_bytes([data[4], data[5]]);
        assert_eq!(total, 5);
    }

    #[test]
    fn test_result_collector() {
        let mut collector = TestResultCollector::new();

        let test_result = TestExecutionResult::new("test1", TestResult::pass());
        collector.add_test_result(test_result).unwrap();

        assert!(collector.has_pending_results());

        let batch = collector.get_next_batch().unwrap();
        assert!(!batch.is_empty());
    }
}
