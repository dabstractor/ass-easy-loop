#!/usr/bin/env python3
"""
Integration Test for No-std Test Framework

This script tests the integration between the Python test framework and
the embedded no_std test framework.
"""

import unittest
import sys
import logging
from pathlib import Path
from unittest.mock import Mock, MagicMock, patch

# Add the test framework to the path
sys.path.insert(0, str(Path(__file__).parent))

from nostd_test_integration import (
    NoStdTestIntegration, NoStdTestResult, NoStdSuiteSummary,
    NoStdTestReportType, NoStdTestResultStatus
)
from device_manager import UsbHidDeviceManager, DeviceInfo, DeviceStatus
from command_handler import CommandHandler
from firmware_flasher import FirmwareFlasher


class TestNoStdIntegration(unittest.TestCase):
    """Test cases for no_std test integration"""
    
    def setUp(self):
        """Set up test fixtures"""
        self.device_manager = Mock(spec=UsbHidDeviceManager)
        self.command_handler = Mock(spec=CommandHandler)
        self.firmware_flasher = Mock(spec=FirmwareFlasher)
        
        self.integration = NoStdTestIntegration(
            self.device_manager,
            self.command_handler,
            self.firmware_flasher
        )
        
        # Mock device
        self.test_device_serial = "TEST_DEVICE_001"
        
    def test_parse_test_result_report(self):
        """Test parsing of test result reports"""
        # Create a mock test result report
        report_data = bytearray(64)
        report_data[0] = NoStdTestReportType.TEST_RESULT.value  # Report type
        report_data[1] = 42  # Test ID
        report_data[2] = NoStdTestResultStatus.PASS.value  # Status
        report_data[3] = 0  # Reserved
        
        # Test name (bytes 4-35)
        test_name = b"sample_test"
        report_data[4:4+len(test_name)] = test_name
        
        # Error message (bytes 36-59) - empty for passing test
        
        # Execution time (bytes 60-63) - 1500ms
        execution_time = (1500).to_bytes(4, 'little')
        report_data[60:64] = execution_time
        
        # Parse the report
        result = self.integration._parse_test_report(bytes(report_data))
        
        self.assertIsInstance(result, NoStdTestResult)
        self.assertEqual(result.test_id, 42)
        self.assertEqual(result.test_name, "sample_test")
        self.assertEqual(result.status, NoStdTestResultStatus.PASS)
        self.assertEqual(result.execution_time_ms, 1500)
        self.assertEqual(result.error_message, "")
    
    def test_parse_suite_summary_report(self):
        """Test parsing of suite summary reports"""
        # Create a mock suite summary report
        report_data = bytearray(64)
        report_data[0] = NoStdTestReportType.SUITE_SUMMARY.value  # Report type
        report_data[1] = 5  # Suite ID
        report_data[2] = 0  # Reserved
        report_data[3] = 0  # Reserved
        
        # Statistics (bytes 4-15)
        report_data[4:6] = (10).to_bytes(2, 'little')  # Total tests
        report_data[6:8] = (8).to_bytes(2, 'little')   # Passed
        report_data[8:10] = (1).to_bytes(2, 'little')  # Failed
        report_data[10:12] = (1).to_bytes(2, 'little') # Skipped
        report_data[12:16] = (5000).to_bytes(4, 'little') # Execution time
        
        # Suite name (bytes 16-47)
        suite_name = b"test_suite"
        report_data[16:16+len(suite_name)] = suite_name
        
        # Parse the report
        result = self.integration._parse_test_report(bytes(report_data))
        
        self.assertIsInstance(result, NoStdSuiteSummary)
        self.assertEqual(result.suite_id, 5)
        self.assertEqual(result.suite_name, "test_suite")
        self.assertEqual(result.total_tests, 10)
        self.assertEqual(result.passed, 8)
        self.assertEqual(result.failed, 1)
        self.assertEqual(result.skipped, 1)
        self.assertEqual(result.execution_time_ms, 5000)
    
    def test_flash_test_firmware(self):
        """Test firmware flashing functionality"""
        # Mock successful bootloader entry
        self.command_handler.create_bootloader_command.return_value = Mock()
        self.command_handler.send_command_and_wait.return_value = Mock(status=Mock(value=0))
        
        # Mock successful bootloader mode entry
        self.device_manager.wait_for_bootloader_mode.return_value = True
        
        # Mock successful firmware flashing
        self.firmware_flasher.flash_firmware.return_value = True
        
        # Mock successful device reconnection
        self.device_manager.wait_for_device_reconnection.return_value = True
        
        # Test firmware flashing
        result = self.integration.flash_test_firmware(
            self.test_device_serial, 
            "/path/to/test_firmware.uf2"
        )
        
        self.assertTrue(result)
        self.command_handler.create_bootloader_command.assert_called_once()
        self.firmware_flasher.flash_firmware.assert_called_once()
    
    def test_create_nostd_test_command(self):
        """Test creation of no_std test commands"""
        # Test run_suite command
        cmd = self.integration._create_nostd_test_command(
            "run_suite", 
            suite_name="test_suite"
        )
        
        self.assertEqual(cmd.command_type, "run_suite")
        self.assertEqual(cmd.suite_name, "test_suite")
        self.assertIsNone(cmd.test_name)
        
        # Test run_test command
        cmd = self.integration._create_nostd_test_command(
            "run_test",
            suite_name="test_suite",
            test_name="specific_test"
        )
        
        self.assertEqual(cmd.command_type, "run_test")
        self.assertEqual(cmd.suite_name, "test_suite")
        self.assertEqual(cmd.test_name, "specific_test")
    
    def test_convert_to_test_suite_result(self):
        """Test conversion of collected results to TestSuiteResult"""
        # Add some mock test results
        self.integration.collected_results[self.test_device_serial] = [
            NoStdTestResult(
                test_id=1,
                test_name="test1",
                status=NoStdTestResultStatus.PASS,
                error_message="",
                execution_time_ms=100
            ),
            NoStdTestResult(
                test_id=2,
                test_name="test2",
                status=NoStdTestResultStatus.FAIL,
                error_message="Test failed",
                execution_time_ms=200
            )
        ]
        
        # Convert to TestSuiteResult
        suite_result = self.integration._convert_to_test_suite_result(
            self.test_device_serial, 
            "test_suite"
        )
        
        self.assertIsNotNone(suite_result)
        self.assertEqual(suite_result.suite_name, "test_suite")
        self.assertEqual(suite_result.aggregate_metrics.total_tests, 2)
        self.assertEqual(suite_result.aggregate_metrics.passed_tests, 1)
        self.assertEqual(suite_result.aggregate_metrics.failed_tests, 1)
        self.assertEqual(suite_result.aggregate_metrics.success_rate, 50.0)
    
    @patch('time.time')
    def test_execute_nostd_test_suite(self, mock_time):
        """Test execution of no_std test suite"""
        mock_time.return_value = 1000.0
        
        # Mock successful command sending
        self.command_handler.send_command.return_value = True
        
        # Mock device handle for reading results
        mock_device_handle = Mock()
        mock_device_handle.read.side_effect = [
            # First call returns test result
            self._create_test_result_bytes("test1", NoStdTestResultStatus.PASS, 100),
            # Second call returns suite summary
            self._create_suite_summary_bytes("test_suite", 1, 1, 0, 0, 100),
            # Third call returns no data
            None
        ]
        self.device_manager.get_device_handle.return_value = mock_device_handle
        
        # Execute test suite
        result = self.integration.execute_nostd_test_suite(
            self.test_device_serial,
            "test_suite",
            timeout=10.0
        )
        
        self.assertIsNotNone(result)
        self.assertEqual(result.suite_name, "test_suite")
        self.assertEqual(result.aggregate_metrics.total_tests, 1)
        self.assertEqual(result.aggregate_metrics.passed_tests, 1)
    
    def _create_test_result_bytes(self, test_name: str, status: NoStdTestResultStatus, exec_time_ms: int) -> bytes:
        """Helper to create test result report bytes"""
        report_data = bytearray(64)
        report_data[0] = NoStdTestReportType.TEST_RESULT.value
        report_data[1] = 1  # Test ID
        report_data[2] = status.value
        
        # Test name
        name_bytes = test_name.encode('utf-8')[:32]
        report_data[4:4+len(name_bytes)] = name_bytes
        
        # Execution time
        report_data[60:64] = exec_time_ms.to_bytes(4, 'little')
        
        return bytes(report_data)
    
    def _create_suite_summary_bytes(self, suite_name: str, total: int, passed: int, 
                                   failed: int, skipped: int, exec_time_ms: int) -> bytes:
        """Helper to create suite summary report bytes"""
        report_data = bytearray(64)
        report_data[0] = NoStdTestReportType.SUITE_SUMMARY.value
        report_data[1] = 1  # Suite ID
        
        # Statistics
        report_data[4:6] = total.to_bytes(2, 'little')
        report_data[6:8] = passed.to_bytes(2, 'little')
        report_data[8:10] = failed.to_bytes(2, 'little')
        report_data[10:12] = skipped.to_bytes(2, 'little')
        report_data[12:16] = exec_time_ms.to_bytes(4, 'little')
        
        # Suite name
        name_bytes = suite_name.encode('utf-8')[:32]
        report_data[16:16+len(name_bytes)] = name_bytes
        
        return bytes(report_data)


class TestFirmwareFlasher(unittest.TestCase):
    """Test cases for firmware flasher"""
    
    def setUp(self):
        """Set up test fixtures"""
        self.flasher = FirmwareFlasher()
    
    def test_get_supported_formats(self):
        """Test getting supported firmware formats"""
        formats = self.flasher.get_supported_formats()
        self.assertIn(".uf2", formats)
    
    @patch('subprocess.run')
    def test_is_tool_available(self, mock_run):
        """Test checking if flashing tool is available"""
        # Mock successful picotool version check
        mock_run.return_value = Mock(returncode=0)
        
        self.assertTrue(self.flasher.is_tool_available())
        mock_run.assert_called_with(
            ["picotool", "version"],
            capture_output=True,
            text=True,
            timeout=5.0
        )


def main():
    """Run the integration tests"""
    # Set up logging
    logging.basicConfig(
        level=logging.DEBUG,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )
    
    # Run tests
    unittest.main(verbosity=2)


if __name__ == "__main__":
    main()