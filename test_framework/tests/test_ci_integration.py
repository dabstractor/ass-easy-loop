#!/usr/bin/env python3
"""
Integration Tests for CI/CD Pipeline Integration

Tests the comprehensive CI/CD integration capabilities including:
- Headless operation with proper exit codes
- Parallel testing support
- Standard test result formats
- Automated device setup and cleanup
- Environment detection and configuration
"""

import unittest
import tempfile
import json
import os
import time
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock
from typing import Dict, List, Any

import sys
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    from ..ci_integration import (
        CIIntegration, CIEnvironmentInfo, CITestConfiguration, CITestResult,
        create_ci_integration
    )
    from ..device_manager import DeviceInfo, DeviceStatus
    from ..test_sequencer import TestConfiguration, TestStep, TestType
    from ..result_collector import TestSuiteResult, TestMetrics
except ImportError:
    # Fallback for direct execution
    from ci_integration import (
        CIIntegration, CIEnvironmentInfo, CITestConfiguration, CITestResult,
        create_ci_integration
    )
    from device_manager import DeviceInfo, DeviceStatus
    from test_sequencer import TestConfiguration, TestStep, TestType
    from result_collector import TestSuiteResult, TestMetrics


class TestCIIntegration(unittest.TestCase):
    """Test cases for CI/CD integration functionality"""
    
    def setUp(self):
        """Set up test environment"""
        self.temp_dir = tempfile.mkdtemp()
        self.ci = CIIntegration(output_dir=self.temp_dir, verbose=False)
        
        # Mock devices
        self.mock_devices = [
            DeviceInfo(
                vendor_id=0x2e8a,
                product_id=0x0003,
                serial_number="TEST_DEVICE_001",
                manufacturer="Test Manufacturer",
                product="Test Device",
                path=b"/dev/hidraw0",
                status=DeviceStatus.CONNECTED,
                last_seen=time.time()
            ),
            DeviceInfo(
                vendor_id=0x2e8a,
                product_id=0x0003,
                serial_number="TEST_DEVICE_002", 
                manufacturer="Test Manufacturer",
                product="Test Device",
                path=b"/dev/hidraw1",
                status=DeviceStatus.CONNECTED,
                last_seen=time.time()
            )
        ]
    
    def tearDown(self):
        """Clean up test environment"""
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)
    
    def test_ci_environment_detection(self):
        """Test CI environment detection for different systems"""
        # Test GitHub Actions detection
        with patch.dict(os.environ, {
            'GITHUB_ACTIONS': 'true',
            'GITHUB_RUN_NUMBER': '123',
            'GITHUB_REF_NAME': 'main',
            'GITHUB_SHA': 'abc123'
        }):
            env_info = self.ci.detect_ci_environment()
            self.assertEqual(env_info.ci_system, 'github_actions')
            self.assertEqual(env_info.build_number, '123')
            self.assertEqual(env_info.branch_name, 'main')
            self.assertEqual(env_info.commit_hash, 'abc123')
        
        # Test Jenkins detection
        with patch.dict(os.environ, {
            'JENKINS_URL': 'http://jenkins.example.com',
            'BUILD_NUMBER': '456',
            'GIT_BRANCH': 'develop',
            'GIT_COMMIT': 'def456'
        }):
            env_info = self.ci.detect_ci_environment()
            self.assertEqual(env_info.ci_system, 'jenkins')
            self.assertEqual(env_info.build_number, '456')
            self.assertEqual(env_info.branch_name, 'develop')
            self.assertEqual(env_info.commit_hash, 'def456')
        
        # Test GitLab CI detection
        with patch.dict(os.environ, {
            'GITLAB_CI': 'true',
            'CI_PIPELINE_ID': '789',
            'CI_COMMIT_REF_NAME': 'feature-branch',
            'CI_COMMIT_SHA': 'ghi789'
        }):
            env_info = self.ci.detect_ci_environment()
            self.assertEqual(env_info.ci_system, 'gitlab_ci')
            self.assertEqual(env_info.build_number, '789')
            self.assertEqual(env_info.branch_name, 'feature-branch')
            self.assertEqual(env_info.commit_hash, 'ghi789')
    
    def test_default_ci_configuration(self):
        """Test default CI configuration generation"""
        config_data = self.ci.get_default_ci_configuration()
        
        self.assertIn('test_config', config_data)
        self.assertIn('required_devices', config_data)
        self.assertIn('max_parallel_devices', config_data)
        self.assertIn('timeout_seconds', config_data)
        
        # Verify test configuration structure
        test_config = config_data['test_config']
        self.assertIn('name', test_config)
        self.assertIn('steps', test_config)
        self.assertTrue(len(test_config['steps']) > 0)
        
        # Verify required test steps are present
        step_names = [step['name'] for step in test_config['steps']]
        self.assertIn('device_communication_test', step_names)
        self.assertIn('pemf_timing_validation', step_names)
        self.assertIn('battery_monitoring_test', step_names)
    
    def test_configuration_loading(self):
        """Test loading CI configuration from file"""
        # Create test configuration file
        test_config = {
            "test_config": {
                "name": "Custom CI Test",
                "description": "Custom test configuration",
                "steps": [
                    {
                        "name": "basic_test",
                        "test_type": "USB_COMMUNICATION_TEST",
                        "parameters": {"message_count": 1},
                        "timeout": 5.0,
                        "required": True
                    }
                ],
                "parallel_execution": False,
                "global_timeout": 60.0
            },
            "required_devices": 2,
            "max_parallel_devices": 2,
            "timeout_seconds": 120.0,
            "fail_fast": False
        }
        
        config_file = Path(self.temp_dir) / "test_config.json"
        with open(config_file, 'w') as f:
            json.dump(test_config, f)
        
        # Load configuration
        loaded_config = self.ci.load_ci_configuration(str(config_file))
        
        self.assertEqual(loaded_config.required_devices, 2)
        self.assertEqual(loaded_config.max_parallel_devices, 2)
        self.assertEqual(loaded_config.timeout_seconds, 120.0)
        self.assertFalse(loaded_config.fail_fast)
        self.assertEqual(loaded_config.test_config.name, "Custom CI Test")
    
    @patch('test_framework.ci_integration.UsbHidDeviceManager')
    def test_device_discovery_and_setup(self, mock_device_manager_class):
        """Test device discovery and setup with retry logic"""
        # Mock device manager
        mock_device_manager = Mock()
        mock_device_manager_class.return_value = mock_device_manager
        
        # Test successful discovery
        mock_device_manager.discover_devices.return_value = self.mock_devices
        mock_device_manager.connect_device.return_value = True
        
        devices, success = self.ci.discover_and_setup_devices(required_count=2)
        
        self.assertTrue(success)
        self.assertEqual(len(devices), 2)
        self.assertIn("TEST_DEVICE_001", devices)
        self.assertIn("TEST_DEVICE_002", devices)
        
        # Verify connection attempts
        self.assertEqual(mock_device_manager.connect_device.call_count, 2)
    
    @patch('test_framework.ci_integration.UsbHidDeviceManager')
    def test_device_discovery_insufficient_devices(self, mock_device_manager_class):
        """Test device discovery with insufficient devices"""
        mock_device_manager = Mock()
        mock_device_manager_class.return_value = mock_device_manager
        
        # Return only one device when two are required
        mock_device_manager.discover_devices.return_value = [self.mock_devices[0]]
        mock_device_manager.connect_device.return_value = True
        
        devices, success = self.ci.discover_and_setup_devices(required_count=2, max_attempts=1)
        
        self.assertFalse(success)
        self.assertEqual(len(devices), 1)
    
    @patch('test_framework.ci_integration.FirmwareFlasher')
    def test_parallel_firmware_flashing(self, mock_flasher_class):
        """Test parallel firmware flashing functionality"""
        # Create mock firmware file
        firmware_file = Path(self.temp_dir) / "test_firmware.uf2"
        firmware_file.write_bytes(b"mock firmware data")
        
        # Mock firmware flasher
        mock_flasher = Mock()
        mock_flasher_class.return_value = mock_flasher
        
        # Mock successful flash results
        from firmware_flasher import FlashResult, FlashOperation
        mock_flash_results = {
            "TEST_DEVICE_001": FlashOperation(
                device_serial="TEST_DEVICE_001",
                firmware_path=str(firmware_file),
                result=FlashResult.SUCCESS,
                total_duration=5.0,
                error_message=None
            ),
            "TEST_DEVICE_002": FlashOperation(
                device_serial="TEST_DEVICE_002", 
                firmware_path=str(firmware_file),
                result=FlashResult.SUCCESS,
                total_duration=4.5,
                error_message=None
            )
        }
        mock_flasher.flash_multiple_devices.return_value = mock_flash_results
        
        # Test parallel flashing
        devices = ["TEST_DEVICE_001", "TEST_DEVICE_002"]
        flash_results, success = self.ci.flash_firmware_parallel(
            devices, str(firmware_file), max_parallel=2
        )
        
        self.assertTrue(success)
        self.assertEqual(len(flash_results), 2)
        
        # Verify flasher was called correctly
        mock_flasher.flash_multiple_devices.assert_called_once()
        call_args = mock_flasher.flash_multiple_devices.call_args
        self.assertTrue(call_args[1]['parallel'])
        self.assertEqual(call_args[1]['max_parallel'], 2)
    
    @patch('test_framework.ci_integration.TestSequencer')
    @patch('test_framework.ci_integration.ResultCollector')
    def test_parallel_test_execution(self, mock_collector_class, mock_sequencer_class):
        """Test parallel test execution"""
        # Mock test sequencer
        mock_sequencer = Mock()
        mock_sequencer_class.return_value = mock_sequencer
        
        # Mock result collector
        mock_collector = Mock()
        mock_collector_class.return_value = mock_collector
        
        # Create test configuration
        config = CITestConfiguration(
            test_config=TestConfiguration(
                name="Test Suite",
                description="Test description",
                steps=[],
                parallel_execution=True,
                global_timeout=60.0
            ),
            required_devices=2,
            max_parallel_devices=2,
            firmware_path=None,
            timeout_seconds=120.0,
            retry_attempts=1,
            fail_fast=True,
            generate_artifacts=True,
            artifact_retention_days=30
        )
        
        # Mock execution results
        mock_execution_results = {
            "TEST_DEVICE_001": [],
            "TEST_DEVICE_002": []
        }
        mock_sequencer.execute_test_sequence.return_value = mock_execution_results
        
        # Mock suite result
        mock_suite_result = TestSuiteResult(
            suite_name="Test Suite",
            description="Test description",
            start_time=time.time(),
            end_time=time.time() + 10,
            duration=10.0,
            device_results={},
            aggregate_metrics=TestMetrics(
                total_tests=4, passed_tests=4, failed_tests=0,
                skipped_tests=0, success_rate=100.0
            ),
            performance_trends=[],
            artifacts=[],
            environment_info={}
        )
        mock_collector.collect_results.return_value = mock_suite_result
        
        # Test parallel execution
        devices = ["TEST_DEVICE_001", "TEST_DEVICE_002"]
        suite_result, success = self.ci.run_parallel_tests(config, devices)
        
        self.assertTrue(success)
        self.assertEqual(suite_result.aggregate_metrics.failed_tests, 0)
        
        # Verify sequencer was called
        mock_sequencer.execute_test_sequence.assert_called_once()
    
    @patch('test_framework.ci_integration.ReportGenerator')
    def test_ci_report_generation(self, mock_generator_class):
        """Test CI report generation in multiple formats"""
        # Mock report generator
        mock_generator = Mock()
        mock_generator_class.return_value = mock_generator
        
        # Mock generated report files
        mock_report_files = {
            'json': str(Path(self.temp_dir) / 'report.json'),
            'junit': str(Path(self.temp_dir) / 'report.xml'),
            'html': str(Path(self.temp_dir) / 'report.html'),
            'csv': str(Path(self.temp_dir) / 'report.csv')
        }
        mock_generator.generate_comprehensive_report.return_value = mock_report_files
        
        # Create mock suite result
        suite_result = TestSuiteResult(
            suite_name="CI Test Suite",
            description="CI test description",
            start_time=time.time(),
            end_time=time.time() + 30,
            duration=30.0,
            device_results={},
            aggregate_metrics=TestMetrics(
                total_tests=5, passed_tests=4, failed_tests=1,
                skipped_tests=0, success_rate=80.0
            ),
            performance_trends=[],
            artifacts=[],
            environment_info={}
        )
        
        # Create test configuration
        config = CITestConfiguration(
            test_config=TestConfiguration(name="Test", description="", steps=[], 
                                        parallel_execution=True, global_timeout=60.0),
            required_devices=1,
            max_parallel_devices=1,
            firmware_path=None,
            timeout_seconds=120.0,
            retry_attempts=1,
            fail_fast=True,
            generate_artifacts=True,
            artifact_retention_days=30
        )
        
        # Test report generation
        generated_files = self.ci.generate_ci_reports(suite_result, config)
        
        self.assertEqual(len(generated_files), 4)
        
        # Verify report generator was called with correct formats
        mock_generator.generate_comprehensive_report.assert_called_once()
        call_args = mock_generator.generate_comprehensive_report.call_args
        formats = call_args[0][1]  # Second argument is formats list
        self.assertIn('json', formats)
        self.assertIn('junit', formats)
        self.assertIn('html', formats)
        self.assertIn('csv', formats)
    
    def test_tap_report_generation(self):
        """Test TAP (Test Anything Protocol) report generation"""
        # Create mock suite result with test results
        from result_collector import DeviceResult, TestExecution, ExecutionStatus, TestMetrics
        from test_sequencer import TestStep, TestType
        
        # Mock test executions
        test_step = TestStep(
            name="test_communication",
            test_type=TestType.USB_COMMUNICATION_TEST,
            parameters={},
            timeout=10.0
        )
        
        execution_passed = TestExecution(
            step=test_step,
            status=ExecutionStatus.COMPLETED,
            start_time=time.time(),
            end_time=time.time() + 2,
            duration=2.0,
            retry_attempt=0,
            error_message=None,
            response=None
        )
        
        execution_failed = TestExecution(
            step=TestStep(name="test_failed", test_type=TestType.BATTERY_ADC_CALIBRATION,
                         parameters={}, timeout=10.0),
            status=ExecutionStatus.FAILED,
            start_time=time.time(),
            end_time=time.time() + 1,
            duration=1.0,
            retry_attempt=0,
            error_message="Test failed",
            response=None
        )
        
        device_result = DeviceResult(
            device_serial="TEST_DEVICE_001",
            overall_status=ExecutionStatus.COMPLETED,
            start_time=time.time(),
            end_time=time.time() + 10,
            executions=[execution_passed, execution_failed],
            metrics=TestMetrics(
                total_tests=2, passed_tests=1, failed_tests=1,
                skipped_tests=0, success_rate=50.0
            )
        )
        
        suite_result = TestSuiteResult(
            suite_name="TAP Test Suite",
            description="Test TAP generation",
            start_time=time.time(),
            end_time=time.time() + 10,
            duration=10.0,
            device_results={"TEST_DEVICE_001": device_result},
            aggregate_metrics=TestMetrics(
                total_tests=2, passed_tests=1, failed_tests=1,
                skipped_tests=0, success_rate=50.0
            ),
            performance_trends=[],
            artifacts=[],
            environment_info={}
        )
        
        # Generate TAP report
        tap_file = self.ci.generate_tap_report(suite_result)
        
        # Verify TAP file was created
        self.assertTrue(Path(tap_file).exists())
        
        # Verify TAP content
        with open(tap_file, 'r') as f:
            content = f.read()
        
        self.assertIn("TAP version 13", content)
        self.assertIn("1..2", content)  # Plan for 2 tests
        self.assertIn("ok 1 - TEST_DEVICE_001.test_communication", content)
        self.assertIn("not ok 2 - TEST_DEVICE_001.test_failed", content)
        self.assertIn("message: Test failed", content)
    
    @patch('test_framework.ci_integration.UsbHidDeviceManager')
    @patch('test_framework.ci_integration.TestSequencer')
    @patch('test_framework.ci_integration.ResultCollector')
    def test_complete_ci_pipeline(self, mock_collector_class, mock_sequencer_class, 
                                 mock_device_manager_class):
        """Test complete CI pipeline execution"""
        # Mock device manager
        mock_device_manager = Mock()
        mock_device_manager_class.return_value = mock_device_manager
        mock_device_manager.discover_devices.return_value = [self.mock_devices[0]]
        mock_device_manager.connect_device.return_value = True
        
        # Mock test sequencer
        mock_sequencer = Mock()
        mock_sequencer_class.return_value = mock_sequencer
        mock_sequencer.execute_test_sequence.return_value = {"TEST_DEVICE_001": []}
        
        # Mock result collector
        mock_collector = Mock()
        mock_collector_class.return_value = mock_collector
        
        # Mock successful suite result
        mock_suite_result = TestSuiteResult(
            suite_name="CI Pipeline Test",
            description="Complete pipeline test",
            start_time=time.time(),
            end_time=time.time() + 20,
            duration=20.0,
            device_results={},
            aggregate_metrics=TestMetrics(
                total_tests=3, passed_tests=3, failed_tests=0,
                skipped_tests=0, success_rate=100.0
            ),
            performance_trends=[],
            artifacts=[],
            environment_info={}
        )
        mock_collector.collect_results.return_value = mock_suite_result
        
        # Run complete CI pipeline
        result = self.ci.run_ci_pipeline()
        
        # Verify successful execution
        self.assertTrue(result.success)
        self.assertEqual(result.exit_code, 0)
        self.assertEqual(result.total_tests, 3)
        self.assertEqual(result.passed_tests, 3)
        self.assertEqual(result.failed_tests, 0)
        self.assertEqual(len(result.devices_tested), 1)
        self.assertIsNone(result.error_summary)
    
    @patch('test_framework.ci_integration.UsbHidDeviceManager')
    def test_ci_pipeline_device_setup_failure(self, mock_device_manager_class):
        """Test CI pipeline with device setup failure"""
        # Mock device manager with no devices
        mock_device_manager = Mock()
        mock_device_manager_class.return_value = mock_device_manager
        mock_device_manager.discover_devices.return_value = []
        
        # Run CI pipeline
        result = self.ci.run_ci_pipeline()
        
        # Verify failure result
        self.assertFalse(result.success)
        self.assertEqual(result.exit_code, 2)  # Device setup failure
        self.assertEqual(result.total_tests, 0)
        self.assertEqual(len(result.devices_tested), 0)
        self.assertIn("Failed to discover and setup required devices", result.error_summary)
    
    def test_exit_code_mapping(self):
        """Test proper exit code mapping for different failure scenarios"""
        # Test success
        result = CITestResult(
            success=True, exit_code=0, total_tests=5, passed_tests=5,
            failed_tests=0, skipped_tests=0, duration_seconds=30.0,
            devices_tested=["TEST_DEVICE_001"], environment_info=self.ci.environment_info,
            artifacts_generated=[], error_summary=None
        )
        self.assertEqual(result.exit_code, 0)
        
        # Test test failures
        result = CITestResult(
            success=False, exit_code=1, total_tests=5, passed_tests=3,
            failed_tests=2, skipped_tests=0, duration_seconds=30.0,
            devices_tested=["TEST_DEVICE_001"], environment_info=self.ci.environment_info,
            artifacts_generated=[], error_summary="Test failures detected"
        )
        self.assertEqual(result.exit_code, 1)
        
        # Test device setup failure
        result = CITestResult(
            success=False, exit_code=2, total_tests=0, passed_tests=0,
            failed_tests=0, skipped_tests=0, duration_seconds=5.0,
            devices_tested=[], environment_info=self.ci.environment_info,
            artifacts_generated=[], error_summary="Failed to discover devices"
        )
        self.assertEqual(result.exit_code, 2)
    
    def test_factory_function(self):
        """Test factory function for creating CI integration"""
        ci = create_ci_integration(output_dir=self.temp_dir, verbose=True)
        
        self.assertIsInstance(ci, CIIntegration)
        self.assertEqual(str(ci.output_dir), self.temp_dir)
        self.assertTrue(ci.verbose)
    
    def test_cleanup_old_artifacts(self):
        """Test cleanup of old test artifacts"""
        # Create some old files
        old_file1 = Path(self.temp_dir) / "old_report.json"
        old_file2 = Path(self.temp_dir) / "old_results.xml"
        recent_file = Path(self.temp_dir) / "recent_report.json"
        
        # Create files with different timestamps
        old_time = time.time() - (35 * 24 * 3600)  # 35 days ago
        recent_time = time.time() - (5 * 24 * 3600)  # 5 days ago
        
        for file_path in [old_file1, old_file2, recent_file]:
            file_path.write_text("test content")
        
        # Set file modification times
        os.utime(old_file1, (old_time, old_time))
        os.utime(old_file2, (old_time, old_time))
        os.utime(recent_file, (recent_time, recent_time))
        
        # Run cleanup
        self.ci.cleanup_old_artifacts(retention_days=30)
        
        # Verify old files were removed and recent file remains
        self.assertFalse(old_file1.exists())
        self.assertFalse(old_file2.exists())
        self.assertTrue(recent_file.exists())


class TestCIIntegrationCommandLine(unittest.TestCase):
    """Test command line interface for CI integration"""
    
    def setUp(self):
        """Set up test environment"""
        self.temp_dir = tempfile.mkdtemp()
    
    def tearDown(self):
        """Clean up test environment"""
        import shutil
        shutil.rmtree(self.temp_dir, ignore_errors=True)
    
    @patch('test_framework.ci_integration.CIIntegration')
    def test_command_line_argument_parsing(self, mock_ci_class):
        """Test command line argument parsing"""
        # Mock CI integration
        mock_ci = Mock()
        mock_ci_class.return_value = mock_ci
        
        # Mock successful pipeline result
        mock_result = CITestResult(
            success=True, exit_code=0, total_tests=3, passed_tests=3,
            failed_tests=0, skipped_tests=0, duration_seconds=15.0,
            devices_tested=["TEST_DEVICE_001"], 
            environment_info=CIEnvironmentInfo(
                ci_system="test", build_number=None, branch_name=None,
                commit_hash=None, pull_request=None, workspace_path="/test",
                environment_variables={}
            ),
            artifacts_generated=[], error_summary=None
        )
        mock_ci.run_ci_pipeline.return_value = mock_result
        
        # Test command line execution
        from ci_integration import main
        
        # Mock sys.argv
        test_args = [
            'ci_integration.py',
            '--devices', '2',
            '--parallel', '2',
            '--timeout', '180',
            '--output-dir', self.temp_dir,
            '--verbose',
            '--fail-fast'
        ]
        
        with patch('sys.argv', test_args):
            with patch('sys.exit') as mock_exit:
                main()
                mock_exit.assert_called_with(0)  # Successful exit
        
        # Verify CI integration was created with correct parameters
        mock_ci_class.assert_called_once_with(output_dir=self.temp_dir, verbose=True)


if __name__ == '__main__':
    # Run tests
    unittest.main(verbosity=2)