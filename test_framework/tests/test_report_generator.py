"""
Unit Tests for Report Generator

Tests the advanced report generation functionality including multiple formats,
performance trend analysis, and regression detection.
"""

import unittest
import tempfile
import json
import os
import time
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock

# Import the modules to test
import sys
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from test_framework.report_generator import ReportGenerator
from test_framework.result_collector import (
    TestSuiteResult, DeviceTestResult, TestMetrics, TestArtifact,
    PerformanceTrend, ResultCollector
)
from test_framework.test_sequencer import TestExecution, TestStatus, TestStep
from test_framework.command_handler import TestType


class TestReportGenerator(unittest.TestCase):
    """Test cases for ReportGenerator class"""
    
    def setUp(self):
        """Set up test fixtures"""
        self.temp_dir = tempfile.mkdtemp()
        self.report_generator = ReportGenerator(self.temp_dir)
        
        # Create mock test data
        self.mock_suite_result = self._create_mock_suite_result()
    
    def tearDown(self):
        """Clean up test fixtures"""
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)
    
    def _create_mock_suite_result(self) -> TestSuiteResult:
        """Create a mock TestSuiteResult for testing"""
        # Create mock test executions
        mock_step1 = TestStep(
            name="test_pemf_timing",
            test_type=TestType.PEMF_TIMING_VALIDATION,
            parameters={"duration_ms": 5000},
            timeout=30.0,
            required=True
        )
        
        mock_step2 = TestStep(
            name="test_battery_adc",
            test_type=TestType.BATTERY_ADC_CALIBRATION,
            parameters={"reference_voltage": 3.3},
            timeout=15.0,
            required=True
        )
        
        mock_execution1 = TestExecution(
            step=mock_step1,
            status=TestStatus.COMPLETED,
            start_time=time.time() - 100,
            end_time=time.time() - 95,
            duration=5.0,
            retry_attempt=0,
            error_message=None,
            response=None
        )
        
        mock_execution2 = TestExecution(
            step=mock_step2,
            status=TestStatus.FAILED,
            start_time=time.time() - 90,
            end_time=time.time() - 85,
            duration=5.0,
            retry_attempt=1,
            error_message="ADC calibration failed",
            response=None
        )
        
        # Create mock device results
        mock_metrics = TestMetrics(
            total_tests=2,
            passed_tests=1,
            failed_tests=1,
            skipped_tests=0,
            timeout_tests=0,
            total_duration=10.0,
            average_duration=5.0,
            success_rate=50.0
        )
        
        mock_device_result = DeviceTestResult(
            device_serial="TEST_DEVICE_001",
            executions=[mock_execution1, mock_execution2],
            metrics=mock_metrics,
            start_time=time.time() - 100,
            end_time=time.time() - 85,
            overall_status=TestStatus.FAILED
        )
        
        # Create mock artifacts
        mock_artifacts = [
            TestArtifact(
                name="timing_data",
                type="timing",
                content={"test1": 5.0, "test2": 5.0},
                timestamp=time.time(),
                size_bytes=100,
                metadata={"test_count": 2}
            ),
            TestArtifact(
                name="error_reports",
                type="error",
                content=[{"test": "test_battery_adc", "error": "ADC calibration failed"}],
                timestamp=time.time(),
                size_bytes=200,
                metadata={"error_count": 1}
            )
        ]
        
        # Create mock performance trends
        mock_trends = [
            PerformanceTrend(
                metric_name="response_time",
                historical_values=[1.0, 1.1, 1.2, 1.0, 1.1],
                current_value=1.5,
                trend_direction="degrading",
                regression_detected=True,
                confidence_level=0.8
            ),
            PerformanceTrend(
                metric_name="throughput",
                historical_values=[100.0, 101.0, 99.0, 102.0, 100.5],
                current_value=100.2,
                trend_direction="stable",
                regression_detected=False,
                confidence_level=0.3
            )
        ]
        
        # Create mock environment info
        mock_env_info = {
            "platform": "Linux-5.4.0-test",
            "python_version": "3.9.0",
            "timestamp": time.time(),
            "hostname": "test-host"
        }
        
        return TestSuiteResult(
            suite_name="Test Suite",
            description="Mock test suite for unit testing",
            device_results={"TEST_DEVICE_001": mock_device_result},
            aggregate_metrics=mock_metrics,
            start_time=time.time() - 100,
            end_time=time.time() - 85,
            duration=15.0,
            artifacts=mock_artifacts,
            performance_trends=mock_trends,
            environment_info=mock_env_info
        )
    
    def test_generate_comprehensive_report_default_formats(self):
        """Test generating comprehensive report with default formats"""
        report_files = self.report_generator.generate_comprehensive_report(
            self.mock_suite_result
        )
        
        # Check that default formats are generated
        expected_formats = ['html', 'json', 'junit', 'csv', 'index']
        for format_name in expected_formats:
            self.assertIn(format_name, report_files)
            self.assertTrue(os.path.exists(report_files[format_name]))
    
    def test_generate_comprehensive_report_custom_formats(self):
        """Test generating comprehensive report with custom formats"""
        custom_formats = ['json', 'junit']
        report_files = self.report_generator.generate_comprehensive_report(
            self.mock_suite_result,
            formats=custom_formats
        )
        
        # Check that only requested formats are generated
        for format_name in custom_formats:
            self.assertIn(format_name, report_files)
            self.assertTrue(os.path.exists(report_files[format_name]))
        
        # Check that index is always generated
        self.assertIn('index', report_files)
        self.assertTrue(os.path.exists(report_files['index']))
    
    def test_generate_json_report(self):
        """Test JSON report generation"""
        timestamp = "20240101_120000"
        json_file = self.report_generator._generate_json_report(
            self.mock_suite_result, timestamp
        )
        
        # Verify file exists
        self.assertTrue(os.path.exists(json_file))
        
        # Verify JSON content
        with open(json_file, 'r') as f:
            report_data = json.load(f)
        
        # Check required sections
        self.assertIn('metadata', report_data)
        self.assertIn('summary', report_data)
        self.assertIn('device_results', report_data)
        self.assertIn('performance_trends', report_data)
        self.assertIn('artifacts', report_data)
        self.assertIn('environment_info', report_data)
        self.assertIn('analysis', report_data)
        
        # Verify summary data
        summary = report_data['summary']
        self.assertEqual(summary['suite_name'], "Test Suite")
        self.assertEqual(summary['total_devices'], 1)
        
        # Verify device results
        device_results = report_data['device_results']
        self.assertIn('TEST_DEVICE_001', device_results)
        device_data = device_results['TEST_DEVICE_001']
        self.assertEqual(len(device_data['executions']), 2)
    
    def test_generate_junit_report(self):
        """Test JUnit XML report generation"""
        timestamp = "20240101_120000"
        junit_file = self.report_generator._generate_junit_report(
            self.mock_suite_result, timestamp
        )
        
        # Verify file exists
        self.assertTrue(os.path.exists(junit_file))
        
        # Verify XML content structure
        with open(junit_file, 'r') as f:
            xml_content = f.read()
        
        # Check for required XML elements
        self.assertIn('<testsuites', xml_content)
        self.assertIn('<testsuite', xml_content)
        self.assertIn('<testcase', xml_content)
        self.assertIn('<failure', xml_content)  # Should have failure from mock data
        self.assertIn('TEST_DEVICE_001', xml_content)
    
    def test_generate_csv_report(self):
        """Test CSV report generation"""
        timestamp = "20240101_120000"
        csv_file = self.report_generator._generate_csv_report(
            self.mock_suite_result, timestamp
        )
        
        # Verify file exists
        self.assertTrue(os.path.exists(csv_file))
        
        # Verify CSV content
        import csv
        with open(csv_file, 'r') as f:
            reader = csv.DictReader(f)
            rows = list(reader)
        
        # Should have 2 rows (one for each test execution)
        self.assertEqual(len(rows), 2)
        
        # Check required columns
        expected_columns = [
            'suite_name', 'device_serial', 'test_name', 'test_type',
            'status', 'duration', 'error_message'
        ]
        for column in expected_columns:
            self.assertIn(column, rows[0].keys())
        
        # Verify data content
        self.assertEqual(rows[0]['suite_name'], "Test Suite")
        self.assertEqual(rows[0]['device_serial'], "TEST_DEVICE_001")
        self.assertEqual(rows[1]['status'], "failed")
    
    def test_generate_html_report(self):
        """Test HTML report generation"""
        timestamp = "20240101_120000"
        html_file = self.report_generator._generate_html_report(
            self.mock_suite_result, timestamp
        )
        
        # Verify file exists
        self.assertTrue(os.path.exists(html_file))
        
        # Verify HTML content
        with open(html_file, 'r') as f:
            html_content = f.read()
        
        # Check for required HTML elements
        self.assertIn('<!DOCTYPE html>', html_content)
        self.assertIn('<title>', html_content)
        self.assertIn('Test Suite', html_content)
        self.assertIn('TEST_DEVICE_001', html_content)
    
    @patch('test_framework.report_generator.SimpleDocTemplate')
    def test_generate_pdf_report_without_reportlab(self, mock_doc):
        """Test PDF report generation when reportlab is not available"""
        with patch.dict('sys.modules', {'reportlab': None}):
            with self.assertRaises(ImportError):
                self.report_generator._generate_pdf_report(
                    self.mock_suite_result, "20240101_120000"
                )
    
    def test_generate_report_index(self):
        """Test report index generation"""
        report_files = {
            'html': '/path/to/report.html',
            'json': '/path/to/report.json',
            'junit': '/path/to/report.xml'
        }
        
        index_file = self.report_generator._generate_report_index(
            self.mock_suite_result, report_files, "20240101_120000"
        )
        
        # Verify file exists
        self.assertTrue(os.path.exists(index_file))
        
        # Verify HTML content
        with open(index_file, 'r') as f:
            html_content = f.read()
        
        # Check for required elements
        self.assertIn('Test Report Index', html_content)
        self.assertIn('Test Suite', html_content)
        self.assertIn('Interactive HTML Report', html_content)
        self.assertIn('JSON Data Export', html_content)
        self.assertIn('JUnit XML Report', html_content)
    
    def test_analysis_data_generation(self):
        """Test analysis data generation"""
        analysis = self.report_generator._generate_analysis_data(self.mock_suite_result)
        
        # Check required analysis sections
        self.assertIn('failure_analysis', analysis)
        self.assertIn('performance_analysis', analysis)
        self.assertIn('device_comparison', analysis)
        self.assertIn('trend_analysis', analysis)
        
        # Verify failure analysis
        failure_analysis = analysis['failure_analysis']
        self.assertIn('failure_patterns', failure_analysis)
        self.assertIn('device_failures', failure_analysis)
        self.assertIn('most_common_failures', failure_analysis)
        
        # Verify trend analysis
        trend_analysis = analysis['trend_analysis']
        self.assertEqual(trend_analysis['regression_count'], 1)
        self.assertIn('response_time', trend_analysis['degrading_metrics'])
        self.assertIn('throughput', trend_analysis['stable_metrics'])


class TestResultCollectorEnhancements(unittest.TestCase):
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
        mock_executions = self._create_mock_executions()
        
        artifacts = self.result_collector._collect_test_artifacts(
            mock_executions, "test_suite"
        )
        
        # Should collect timing, error, and performance artifacts
        artifact_types = [a.type for a in artifacts]
        self.assertIn('timing', artifact_types)
        self.assertIn('error', artifact_types)
        
        # Verify timing artifact
        timing_artifact = next(a for a in artifacts if a.type == 'timing')
        self.assertEqual(timing_artifact.name, "test_suite_timing_data")
        self.assertIsInstance(timing_artifact.content, list)
        self.assertGreater(timing_artifact.size_bytes, 0)
    
    def test_performance_trend_analysis(self):
        """Test performance trend analysis"""
        # Create mock executions with performance data
        mock_executions = self._create_mock_executions_with_performance()
        
        trends = self.result_collector._analyze_performance_trends(mock_executions)
        
        # Should detect trends for performance metrics
        self.assertGreater(len(trends), 0)
        
        # Verify trend structure
        for trend in trends:
            self.assertIsInstance(trend.metric_name, str)
            self.assertIsInstance(trend.current_value, (int, float))
            self.assertIn(trend.trend_direction, ['improving', 'degrading', 'stable'])
            self.assertIsInstance(trend.regression_detected, bool)
            self.assertIsInstance(trend.confidence_level, float)
    
    def test_regression_detection(self):
        """Test performance regression detection"""
        # Test with clear regression
        historical_values = [1.0, 1.1, 1.0, 1.2, 1.1]
        current_value = 2.0  # Significant increase
        
        trend_direction, regression_detected, confidence = \
            self.result_collector._detect_performance_regression(
                historical_values, current_value
            )
        
        self.assertEqual(trend_direction, "degrading")
        self.assertTrue(regression_detected)
        self.assertGreater(confidence, 0.5)
        
        # Test with stable performance
        historical_values = [1.0, 1.1, 1.0, 1.2, 1.1]
        current_value = 1.05  # Within normal range
        
        trend_direction, regression_detected, confidence = \
            self.result_collector._detect_performance_regression(
                historical_values, current_value
            )
        
        self.assertFalse(regression_detected)
    
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
    
    def test_html_report_export(self):
        """Test HTML report export"""
        # Create a mock suite result
        mock_suite_result = self._create_simple_mock_suite_result()
        
        # Export HTML report
        output_path = self.result_collector.export_html_report(mock_suite_result)
        
        # Verify file exists
        self.assertTrue(os.path.exists(output_path))
        
        # Verify HTML content
        with open(output_path, 'r') as f:
            html_content = f.read()
        
        self.assertIn('<!DOCTYPE html>', html_content)
        self.assertIn('Test Suite', html_content)
    
    def test_csv_report_export(self):
        """Test CSV report export"""
        # Create a mock suite result
        mock_suite_result = self._create_simple_mock_suite_result()
        
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
    
    def test_artifacts_save_to_disk(self):
        """Test saving artifacts to disk"""
        # Create a mock suite result with artifacts
        mock_suite_result = self._create_simple_mock_suite_result()
        
        # Save artifacts
        artifact_paths = self.result_collector.save_artifacts_to_disk(mock_suite_result)
        
        # Verify files were created
        for artifact_name, filepath in artifact_paths.items():
            self.assertTrue(os.path.exists(filepath))
            
            # Verify JSON content
            with open(filepath, 'r') as f:
                artifact_data = json.load(f)
            
            self.assertIn('artifact_info', artifact_data)
            self.assertIn('content', artifact_data)
    
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
    
    def _create_mock_executions(self):
        """Create mock test executions for testing"""
        mock_step = TestStep(
            name="test_example",
            test_type=TestType.USB_COMMUNICATION_TEST,
            parameters={"param1": "value1"},
            timeout=30.0,
            required=True
        )
        
        mock_execution = TestExecution(
            step=mock_step,
            status=TestStatus.FAILED,
            start_time=time.time() - 10,
            end_time=time.time() - 5,
            duration=5.0,
            retry_attempt=0,
            error_message="Test failed for testing",
            response=None
        )
        
        return [mock_execution]
    
    def _create_mock_executions_with_performance(self):
        """Create mock executions with performance data"""
        mock_response = Mock()
        mock_response.data = {
            'performance_metrics': {
                'response_time': 1.5,
                'throughput': 100.0,
                'cpu_usage': 45.2
            }
        }
        mock_response.timestamp = time.time()
        
        mock_step = TestStep(
            name="perf_test",
            test_type=TestType.SYSTEM_STRESS_TEST,
            parameters={},
            timeout=30.0,
            required=True
        )
        
        mock_execution = TestExecution(
            step=mock_step,
            status=TestStatus.COMPLETED,
            start_time=time.time() - 10,
            end_time=time.time() - 5,
            duration=5.0,
            retry_attempt=0,
            error_message=None,
            response=mock_response
        )
        
        return [mock_execution]
    
    def _create_simple_mock_suite_result(self):
        """Create a simple mock suite result for testing"""
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
        
        mock_execution = TestExecution(
            step=TestStep("test", TestType.USB_COMMUNICATION_TEST, {}, 30.0, True),
            status=TestStatus.COMPLETED,
            start_time=time.time() - 10,
            end_time=time.time() - 5,
            duration=5.0,
            retry_attempt=0,
            error_message=None,
            response=None
        )
        
        mock_device_result = DeviceTestResult(
            device_serial="TEST_DEVICE",
            executions=[mock_execution],
            metrics=mock_metrics,
            start_time=time.time() - 10,
            end_time=time.time() - 5,
            overall_status=TestStatus.COMPLETED
        )
        
        mock_artifact = TestArtifact(
            name="test_artifact",
            type="timing",
            content={"test": "data"},
            timestamp=time.time(),
            size_bytes=50,
            metadata={}
        )
        
        return TestSuiteResult(
            suite_name="Test Suite",
            description="Simple test suite",
            device_results={"TEST_DEVICE": mock_device_result},
            aggregate_metrics=mock_metrics,
            start_time=time.time() - 10,
            end_time=time.time() - 5,
            duration=5.0,
            artifacts=[mock_artifact],
            performance_trends=[],
            environment_info={"platform": "test"}
        )
    
    def _create_mock_suite_result_with_regressions(self):
        """Create mock suite result with performance regressions"""
        mock_suite_result = self._create_simple_mock_suite_result()
        
        # Add performance trends with regressions
        regression_trend = PerformanceTrend(
            metric_name="response_time",
            historical_values=[1.0, 1.1, 1.0, 1.2],
            current_value=2.0,
            trend_direction="degrading",
            regression_detected=True,
            confidence_level=0.9
        )
        
        mock_suite_result.performance_trends = [regression_trend]
        return mock_suite_result


if __name__ == '__main__':
    unittest.main()