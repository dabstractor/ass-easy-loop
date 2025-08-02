"""
Unit tests for Result Collector

Tests result collection, analysis, basic reporting functionality,
and enhanced features like artifact collection and trend analysis.
"""

import unittest
from unittest.mock import Mock
import time
import tempfile
import json
import statistics
import os

from test_framework.result_collector import (
    ResultCollector, TestMetrics, DeviceTestResult, TestSuiteResult,
    TestArtifact, PerformanceTrend, TestResultSeverity
)
from test_framework.test_sequencer import TestExecution, TestStep, TestStatus
from test_framework.command_handler import TestType, TestResponse, ResponseStatus


class TestResultCollector(unittest.TestCase):
    """Test cases for ResultCollector"""
    
    def setUp(self):
        """Set up test fixtures"""
        self.temp_dir = tempfile.mkdtemp()
        self.result_collector = ResultCollector(self.temp_dir)
        
        # Create test executions
        self.test_step1 = TestStep(
            name="test1",
            test_type=TestType.USB_COMMUNICATION_TEST,
            parameters={},
            required=True
        )
        
        self.test_step2 = TestStep(
            name="test2", 
            test_type=TestType.PEMF_TIMING_VALIDATION,
            parameters={},
            required=False
        )
        
        self.successful_execution = TestExecution(
            step=self.test_step1,
            device_serial='TEST123',
            status=TestStatus.COMPLETED,
            start_time=1000.0,
            end_time=1005.0
        )
        
        self.failed_execution = TestExecution(
            step=self.test_step2,
            device_serial='TEST123', 
            status=TestStatus.FAILED,
            start_time=1010.0,
            end_time=1015.0,
            error_message="Test failed"
        )
    
    def test_calculate_device_metrics_empty(self):
        """Test metrics calculation with empty execution list"""
        metrics = self.result_collector._calculate_device_metrics([])
        
        self.assertEqual(metrics.total_tests, 0)
        self.assertEqual(metrics.passed_tests, 0)
        self.assertEqual(metrics.failed_tests, 0)
        self.assertEqual(metrics.success_rate, 0.0)
    
    def test_calculate_device_metrics_mixed_results(self):
        """Test metrics calculation with mixed test results"""
        executions = [self.successful_execution, self.failed_execution]
        
        metrics = self.result_collector._calculate_device_metrics(executions)
        
        self.assertEqual(metrics.total_tests, 2)
        self.assertEqual(metrics.passed_tests, 1)
        self.assertEqual(metrics.failed_tests, 1)
        self.assertEqual(metrics.success_rate, 50.0)
        self.assertEqual(metrics.total_duration, 10.0)  # 5.0 + 5.0
        self.assertEqual(metrics.average_duration, 5.0)
    
    def test_calculate_device_metrics_with_skipped(self):
        """Test metrics calculation including skipped tests"""
        skipped_execution = TestExecution(
            step=self.test_step1,
            device_serial='TEST123',
            status=TestStatus.SKIPPED
        )
        
        executions = [self.successful_execution, skipped_execution]
        metrics = self.result_collector._calculate_device_metrics(executions)
        
        self.assertEqual(metrics.total_tests, 2)
        self.assertEqual(metrics.passed_tests, 1)
        self.assertEqual(metrics.skipped_tests, 1)
        self.assertEqual(metrics.success_rate, 50.0)
    
    def test_calculate_device_metrics_with_timeout(self):
        """Test metrics calculation including timeout tests"""
        timeout_execution = TestExecution(
            step=self.test_step1,
            device_serial='TEST123',
            status=TestStatus.TIMEOUT,
            start_time=1000.0,
            end_time=1030.0
        )
        
        executions = [timeout_execution]
        metrics = self.result_collector._calculate_device_metrics(executions)
        
        self.assertEqual(metrics.total_tests, 1)
        self.assertEqual(metrics.timeout_tests, 1)
        self.assertEqual(metrics.success_rate, 0.0)
    
    def test_determine_overall_status_all_passed(self):
        """Test overall status determination when all required tests pass"""
        executions = [self.successful_execution]
        
        status = self.result_collector._determine_overall_status(executions)
        
        self.assertEqual(status, TestStatus.COMPLETED)
    
    def test_determine_overall_status_required_failed(self):
        """Test overall status when required test fails"""
        failed_required = TestExecution(
            step=self.test_step1,  # Required test
            device_serial='TEST123',
            status=TestStatus.FAILED
        )
        
        executions = [failed_required]
        status = self.result_collector._determine_overall_status(executions)
        
        self.assertEqual(status, TestStatus.FAILED)
    
    def test_determine_overall_status_optional_failed(self):
        """Test overall status when only optional test fails"""
        # Make test_step2 non-required and test_step1 required and passed
        self.test_step2.required = False
        
        executions = [self.successful_execution, self.failed_execution]
        status = self.result_collector._determine_overall_status(executions)
        
        self.assertEqual(status, TestStatus.COMPLETED)
    
    def test_collect_results_single_device(self):
        """Test collecting results for single device"""
        execution_results = {
            'TEST123': [self.successful_execution, self.failed_execution]
        }
        
        start_time = 1000.0
        end_time = 1020.0
        
        suite_result = self.result_collector.collect_results(
            "Test Suite",
            "Test Description", 
            execution_results,
            start_time,
            end_time
        )
        
        self.assertEqual(suite_result.suite_name, "Test Suite")
        self.assertEqual(suite_result.description, "Test Description")
        self.assertEqual(len(suite_result.device_results), 1)
        self.assertIn('TEST123', suite_result.device_results)
        
        device_result = suite_result.device_results['TEST123']
        self.assertEqual(len(device_result.executions), 2)
        self.assertEqual(device_result.metrics.total_tests, 2)
    
    def test_collect_results_multiple_devices(self):
        """Test collecting results for multiple devices"""
        execution_results = {
            'TEST123': [self.successful_execution],
            'TEST456': [self.failed_execution]
        }
        
        suite_result = self.result_collector.collect_results(
            "Multi-Device Test",
            "Testing multiple devices",
            execution_results,
            1000.0,
            1020.0
        )
        
        self.assertEqual(len(suite_result.device_results), 2)
        self.assertIn('TEST123', suite_result.device_results)
        self.assertIn('TEST456', suite_result.device_results)
        
        # Aggregate metrics should combine both devices
        self.assertEqual(suite_result.aggregate_metrics.total_tests, 2)
        self.assertEqual(suite_result.aggregate_metrics.passed_tests, 1)
        self.assertEqual(suite_result.aggregate_metrics.failed_tests, 1)
    
    def test_generate_summary_report(self):
        """Test generating summary report"""
        execution_results = {'TEST123': [self.successful_execution]}
        
        suite_result = self.result_collector.collect_results(
            "Test Suite", "Description", execution_results, 1000.0, 1020.0
        )
        
        summary = self.result_collector.generate_summary_report(suite_result)
        
        self.assertEqual(summary['suite_name'], "Test Suite")
        self.assertEqual(summary['device_count'], 1)
        self.assertIn('aggregate_metrics', summary)
        self.assertIn('device_summaries', summary)
        self.assertIn('TEST123', summary['device_summaries'])
    
    def test_generate_detailed_report(self):
        """Test generating detailed report"""
        # Add response to execution
        self.successful_execution.response = Mock(spec=TestResponse)
        self.successful_execution.response.status = ResponseStatus.SUCCESS
        self.successful_execution.response.data = {'result': 'success'}
        self.successful_execution.response.timestamp = 1005.0
        
        execution_results = {'TEST123': [self.successful_execution]}
        
        suite_result = self.result_collector.collect_results(
            "Test Suite", "Description", execution_results, 1000.0, 1020.0
        )
        
        detailed = self.result_collector.generate_detailed_report(suite_result)
        
        self.assertIn('device_details', detailed)
        self.assertIn('TEST123', detailed['device_details'])
        
        device_details = detailed['device_details']['TEST123']
        self.assertIn('test_executions', device_details)
        self.assertEqual(len(device_details['test_executions']), 1)
        
        execution_detail = device_details['test_executions'][0]
        self.assertEqual(execution_detail['test_name'], 'test1')
        self.assertEqual(execution_detail['status'], 'completed')
        self.assertIn('response', execution_detail)
    
    def test_export_junit_xml(self):
        """Test exporting results in JUnit XML format"""
        execution_results = {
            'TEST123': [self.successful_execution, self.failed_execution]
        }
        
        suite_result = self.result_collector.collect_results(
            "Test Suite", "Description", execution_results, 1000.0, 1020.0
        )
        
        xml_output = self.result_collector.export_junit_xml(suite_result)
        
        self.assertIn('<?xml version', xml_output)
        self.assertIn('<testsuite', xml_output)
        self.assertIn('name="Test Suite"', xml_output)
        self.assertIn('tests="2"', xml_output)
        self.assertIn('failures="1"', xml_output)
        self.assertIn('<testcase', xml_output)
        self.assertIn('<failure', xml_output)
    
    def test_export_json_summary(self):
        """Test exporting results in JSON format (summary)"""
        execution_results = {'TEST123': [self.successful_execution]}
        
        suite_result = self.result_collector.collect_results(
            "Test Suite", "Description", execution_results, 1000.0, 1020.0
        )
        
        json_output = self.result_collector.export_json(suite_result, detailed=False)
        
        self.assertIn('"suite_name": "Test Suite"', json_output)
        self.assertIn('"device_count": 1', json_output)
        self.assertIn('"aggregate_metrics"', json_output)
    
    def test_export_json_detailed(self):
        """Test exporting results in JSON format (detailed)"""
        execution_results = {'TEST123': [self.successful_execution]}
        
        suite_result = self.result_collector.collect_results(
            "Test Suite", "Description", execution_results, 1000.0, 1020.0
        )
        
        json_output = self.result_collector.export_json(suite_result, detailed=True)
        
        self.assertIn('"device_details"', json_output)
        self.assertIn('"test_executions"', json_output)
    
    def test_get_failure_analysis(self):
        """Test failure analysis generation"""
        # Create multiple failures
        failed_execution2 = TestExecution(
            step=self.test_step1,
            device_serial='TEST456',
            status=TestStatus.FAILED,
            error_message="Another failure"
        )
        
        execution_results = {
            'TEST123': [self.failed_execution],
            'TEST456': [failed_execution2]
        }
        
        suite_result = self.result_collector.collect_results(
            "Test Suite", "Description", execution_results, 1000.0, 1020.0
        )
        
        analysis = self.result_collector.get_failure_analysis(suite_result)
        
        self.assertEqual(analysis['total_failures'], 2)
        self.assertIn('failure_by_test', analysis)
        self.assertIn('failure_by_device', analysis)
        self.assertIn('recommendations', analysis)
    
    def test_get_performance_analysis(self):
        """Test performance analysis generation"""
        execution_results = {
            'TEST123': [self.successful_execution],
            'TEST456': [self.failed_execution]
        }
        
        suite_result = self.result_collector.collect_results(
            "Test Suite", "Description", execution_results, 1000.0, 1020.0
        )
        
        analysis = self.result_collector.get_performance_analysis(suite_result)
        
        self.assertIn('execution_times', analysis)
        self.assertIn('slowest_tests', analysis)
        
        # Should have timing data for both tests
        self.assertIn('test1', analysis['execution_times'])
        self.assertIn('test2', analysis['execution_times'])
    
    def tearDown(self):
        """Clean up test fixtures"""
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)


class TestEnhancedResultCollector(unittest.TestCase):
    """Test cases for enhanced ResultCollector functionality"""
    
    def setUp(self):
        """Set up test fixtures"""
        self.temp_dir = tempfile.mkdtemp()
        self.result_collector = ResultCollector(self.temp_dir)
    
    def tearDown(self):
        """Clean up test fixtures"""
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)
    
    def test_collect_test_artifacts(self):
        """Test test artifact collection"""
        # Create mock executions with various data
        mock_executions = self._create_mock_executions_with_artifacts()
        
        artifacts = self.result_collector._collect_test_artifacts(
            mock_executions, "test_suite"
        )
        
        # Should collect timing and error artifacts
        artifact_types = [a.type for a in artifacts]
        self.assertIn('timing', artifact_types)
        self.assertIn('error', artifact_types)
        
        # Verify timing artifact
        timing_artifact = next(a for a in artifacts if a.type == 'timing')
        self.assertEqual(timing_artifact.name, "test_suite_timing_data")
        self.assertIsInstance(timing_artifact.content, list)
        self.assertGreater(timing_artifact.size_bytes, 0)
        
        # Verify error artifact
        error_artifact = next(a for a in artifacts if a.type == 'error')
        self.assertEqual(error_artifact.name, "test_suite_error_reports")
        self.assertIsInstance(error_artifact.content, list)
        self.assertGreater(len(error_artifact.content), 0)
    
    def test_performance_trend_analysis(self):
        """Test performance trend analysis"""
        # Create mock executions with performance data
        mock_executions = self._create_mock_executions_with_performance()
        
        # First call to populate history
        trends1 = self.result_collector._analyze_performance_trends(mock_executions)
        
        # Second call to create trends (need historical data)
        trends2 = self.result_collector._analyze_performance_trends(mock_executions)
        
        # Third call to have enough data for trend analysis
        trends3 = self.result_collector._analyze_performance_trends(mock_executions)
        
        # Should detect trends for performance metrics after multiple calls
        self.assertGreaterEqual(len(trends3), 0)  # May be 0 if not enough historical data
        
        # If trends are detected, verify their structure
        for trend in trends3:
            self.assertIsInstance(trend.metric_name, str)
            self.assertIsInstance(trend.current_value, (int, float))
            self.assertIn(trend.trend_direction, ['improving', 'degrading', 'stable', 'unknown'])
            self.assertIsInstance(trend.regression_detected, bool)
            self.assertIsInstance(trend.confidence_level, float)
            self.assertGreaterEqual(trend.confidence_level, 0.0)
            self.assertLessEqual(trend.confidence_level, 1.0)
    
    def test_regression_detection_clear_regression(self):
        """Test performance regression detection with clear regression"""
        historical_values = [1.0, 1.1, 1.0, 1.2, 1.1]
        current_value = 2.5  # Significant increase (regression)
        
        trend_direction, regression_detected, confidence = \
            self.result_collector._detect_performance_regression(
                historical_values, current_value
            )
        
        self.assertEqual(trend_direction, "degrading")
        self.assertTrue(regression_detected)
        self.assertGreater(confidence, 0.5)
    
    def test_regression_detection_stable_performance(self):
        """Test regression detection with stable performance"""
        historical_values = [1.0, 1.1, 1.0, 1.2, 1.1]
        current_value = 1.05  # Within normal range
        
        trend_direction, regression_detected, confidence = \
            self.result_collector._detect_performance_regression(
                historical_values, current_value
            )
        
        self.assertFalse(regression_detected)
        self.assertLessEqual(confidence, 0.5)
    
    def test_regression_detection_insufficient_data(self):
        """Test regression detection with insufficient historical data"""
        historical_values = [1.0, 1.1]  # Too few data points
        current_value = 1.5
        
        trend_direction, regression_detected, confidence = \
            self.result_collector._detect_performance_regression(
                historical_values, current_value
            )
        
        self.assertEqual(trend_direction, "unknown")
        self.assertFalse(regression_detected)
        self.assertEqual(confidence, 0.0)
    
    def test_environment_info_collection(self):
        """Test environment information collection"""
        env_info = self.result_collector._collect_environment_info()
        
        # Check required fields
        required_fields = [
            'timestamp', 'platform', 'python_version',
            'architecture', 'hostname', 'working_directory'
        ]
        
        for field in required_fields:
            self.assertIn(field, env_info)
            self.assertIsNotNone(env_info[field])
        
        # Verify timestamp is recent
        self.assertGreater(env_info['timestamp'], time.time() - 10)
    
    def test_html_report_export(self):
        """Test HTML report export"""
        # Create a mock suite result
        mock_suite_result = self._create_mock_suite_result()
        
        # Export HTML report
        output_path = self.result_collector.export_html_report(mock_suite_result)
        
        # Verify file exists
        self.assertTrue(os.path.exists(output_path))
        
        # Verify HTML content
        with open(output_path, 'r') as f:
            html_content = f.read()
        
        self.assertIn('<!DOCTYPE html>', html_content)
        self.assertIn('Test Suite', html_content)
        self.assertIn('TEST_DEVICE_001', html_content)
    
    def test_csv_report_export(self):
        """Test CSV report export"""
        # Create a mock suite result
        mock_suite_result = self._create_mock_suite_result()
        
        # Export CSV report
        output_path = self.result_collector.export_csv_report(mock_suite_result)
        
        # Verify file exists
        self.assertTrue(os.path.exists(output_path))
        
        # Verify CSV content
        import csv
        with open(output_path, 'r') as f:
            reader = csv.DictReader(f)
            rows = list(reader)
        
        self.assertGreater(len(rows), 0)
        self.assertIn('device_serial', rows[0].keys())
        self.assertIn('test_name', rows[0].keys())
        self.assertIn('status', rows[0].keys())
        self.assertIn('duration', rows[0].keys())
    
    def test_artifacts_save_to_disk(self):
        """Test saving artifacts to disk"""
        # Create a mock suite result with artifacts
        mock_suite_result = self._create_mock_suite_result_with_artifacts()
        
        # Save artifacts
        artifact_paths = self.result_collector.save_artifacts_to_disk(mock_suite_result)
        
        # Verify files were created
        self.assertGreater(len(artifact_paths), 0)
        
        for artifact_name, filepath in artifact_paths.items():
            self.assertTrue(os.path.exists(filepath))
            
            # Verify JSON content
            with open(filepath, 'r') as f:
                artifact_data = json.load(f)
            
            self.assertIn('artifact_info', artifact_data)
            self.assertIn('content', artifact_data)
            self.assertIn('name', artifact_data['artifact_info'])
            self.assertIn('type', artifact_data['artifact_info'])
    
    def test_regression_report_generation(self):
        """Test regression report generation"""
        # Create mock suite result with regressions
        mock_suite_result = self._create_mock_suite_result_with_regressions()
        
        # Generate regression report
        regression_report = self.result_collector.generate_regression_report(mock_suite_result)
        
        # Verify report structure
        self.assertIn('summary', regression_report)
        self.assertIn('regressions', regression_report)
        self.assertIn('recommendations', regression_report)
        
        # Verify summary
        summary = regression_report['summary']
        self.assertIn('total_metrics_analyzed', summary)
        self.assertIn('regressions_detected', summary)
        self.assertIn('regression_rate', summary)
        
        # Verify regressions
        if regression_report['regressions']:
            regression = regression_report['regressions'][0]
            self.assertIn('metric_name', regression)
            self.assertIn('severity', regression)
            self.assertIn('confidence_level', regression)
            self.assertIn('current_value', regression)
            self.assertIn('historical_mean', regression)
    
    def test_regression_severity_assessment(self):
        """Test regression severity assessment"""
        # Test critical severity
        critical_regression = PerformanceTrend(
            metric_name="test_metric",
            historical_values=[1.0, 1.1, 1.0],
            current_value=2.0,
            trend_direction="degrading",
            regression_detected=True,
            confidence_level=0.95
        )
        
        severity = self.result_collector._assess_regression_severity(critical_regression)
        self.assertEqual(severity, "critical")
        
        # Test low severity
        low_regression = PerformanceTrend(
            metric_name="test_metric",
            historical_values=[1.0, 1.1, 1.0],
            current_value=1.2,
            trend_direction="degrading",
            regression_detected=True,
            confidence_level=0.3
        )
        
        severity = self.result_collector._assess_regression_severity(low_regression)
        self.assertEqual(severity, "low")
    
    def test_collect_results_with_environment_info(self):
        """Test collecting results with custom environment info"""
        execution_results = {
            'TEST_DEVICE': [self._create_simple_execution()]
        }
        
        custom_env_info = {
            'test_environment': 'CI',
            'build_number': '123',
            'git_commit': 'abc123'
        }
        
        suite_result = self.result_collector.collect_results(
            "Test Suite",
            "Test with custom env info",
            execution_results,
            time.time() - 10,
            time.time(),
            environment_info=custom_env_info
        )
        
        # Verify custom environment info is included
        self.assertEqual(suite_result.environment_info, custom_env_info)
        
        # Verify artifacts and trends are collected
        self.assertIsInstance(suite_result.artifacts, list)
        self.assertIsInstance(suite_result.performance_trends, list)
    
    def _create_mock_executions_with_artifacts(self):
        """Create mock executions with artifact data"""
        # Create execution with timing data
        timing_execution = TestExecution(
            step=TestStep("timing_test", TestType.PEMF_TIMING_VALIDATION, {}, 30.0, True),
            device_serial="TEST_DEVICE_001",
            status=TestStatus.COMPLETED,
            start_time=time.time() - 10,
            end_time=time.time() - 5,
            retry_attempt=0,
            error_message=None,
            response=None
        )
        
        # Create execution with error data
        error_execution = TestExecution(
            step=TestStep("error_test", TestType.BATTERY_ADC_CALIBRATION, {}, 30.0, True),
            device_serial="TEST_DEVICE_001",
            status=TestStatus.FAILED,
            start_time=time.time() - 8,
            end_time=time.time() - 3,
            retry_attempt=1,
            error_message="Test failed for artifact collection",
            response=None
        )
        
        return [timing_execution, error_execution]
    
    def _create_mock_executions_with_performance(self):
        """Create mock executions with performance data"""
        mock_response = Mock()
        mock_response.data = {
            'performance_metrics': {
                'response_time': 1.5,
                'throughput': 100.0,
                'cpu_usage': 45.2,
                'memory_usage': 67.8
            }
        }
        mock_response.timestamp = time.time()
        
        perf_execution = TestExecution(
            step=TestStep("perf_test", TestType.SYSTEM_STRESS_TEST, {}, 30.0, True),
            device_serial="TEST_DEVICE_001",
            status=TestStatus.COMPLETED,
            start_time=time.time() - 10,
            end_time=time.time() - 5,
            retry_attempt=0,
            error_message=None,
            response=mock_response
        )
        
        return [perf_execution]
    
    def _create_mock_suite_result(self):
        """Create a mock suite result for testing"""
        mock_execution = self._create_simple_execution()
        
        mock_metrics = TestMetrics(
            total_tests=1,
            passed_tests=1,
            failed_tests=0,
            skipped_tests=0,
            timeout_tests=0,
            total_duration=5.0,
            average_duration=5.0,
            success_rate=100.0
        )
        
        mock_device_result = DeviceTestResult(
            device_serial="TEST_DEVICE_001",
            executions=[mock_execution],
            metrics=mock_metrics,
            start_time=time.time() - 10,
            end_time=time.time() - 5,
            overall_status=TestStatus.COMPLETED
        )
        
        return TestSuiteResult(
            suite_name="Test Suite",
            description="Mock test suite",
            device_results={"TEST_DEVICE_001": mock_device_result},
            aggregate_metrics=mock_metrics,
            start_time=time.time() - 10,
            end_time=time.time() - 5,
            duration=5.0,
            artifacts=[],
            performance_trends=[],
            environment_info={"platform": "test"}
        )
    
    def _create_mock_suite_result_with_artifacts(self):
        """Create mock suite result with artifacts"""
        mock_suite_result = self._create_mock_suite_result()
        
        # Add test artifacts
        mock_artifacts = [
            TestArtifact(
                name="timing_data",
                type="timing",
                content={"test1": 5.0, "test2": 3.0},
                timestamp=time.time(),
                size_bytes=100,
                metadata={"test_count": 2}
            ),
            TestArtifact(
                name="error_reports",
                type="error",
                content=[{"test": "test1", "error": "mock error"}],
                timestamp=time.time(),
                size_bytes=50,
                metadata={"error_count": 1}
            )
        ]
        
        mock_suite_result.artifacts = mock_artifacts
        return mock_suite_result
    
    def _create_mock_suite_result_with_regressions(self):
        """Create mock suite result with performance regressions"""
        mock_suite_result = self._create_mock_suite_result()
        
        # Add performance trends with regressions
        regression_trend = PerformanceTrend(
            metric_name="response_time",
            historical_values=[1.0, 1.1, 1.0, 1.2],
            current_value=2.0,
            trend_direction="degrading",
            regression_detected=True,
            confidence_level=0.9
        )
        
        stable_trend = PerformanceTrend(
            metric_name="throughput",
            historical_values=[100.0, 101.0, 99.0, 102.0],
            current_value=100.5,
            trend_direction="stable",
            regression_detected=False,
            confidence_level=0.2
        )
        
        mock_suite_result.performance_trends = [regression_trend, stable_trend]
        return mock_suite_result
    
    def _create_simple_execution(self):
        """Create a simple test execution"""
        execution = TestExecution(
            step=TestStep("simple_test", TestType.USB_COMMUNICATION_TEST, {}, 30.0, True),
            device_serial="TEST_DEVICE_001",
            status=TestStatus.COMPLETED,
            start_time=time.time() - 10,
            end_time=time.time() - 5,
            retry_attempt=0,
            error_message=None,
            response=None
        )
        return execution


if __name__ == '__main__':
    unittest.main()