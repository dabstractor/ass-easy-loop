"""
No-std Test Integration Module

This module provides integration between the Python test framework and the
embedded no_std test framework running on the RP2040 device. It handles:
- No-std test execution commands
- Test result collection from USB HID reports
- Integration with existing test reporting pipeline
- Bootloader flashing workflow for test firmware
"""

import time
import json
import logging
from typing import List, Dict, Any, Optional, Tuple, Union
from dataclasses import dataclass, field
from enum import Enum
import struct

from .command_handler import CommandHandler, TestCommand, TestResponse, CommandType, ResponseStatus
from .device_manager import UsbHidDeviceManager
from .result_collector import TestSuiteResult, DeviceTestResult, TestMetrics, TestArtifact
from .test_sequencer import TestExecution, TestStatus, TestStep
from .firmware_flasher import FirmwareFlasher


class NoStdTestReportType(Enum):
    """Test result report types from embedded no_std tests"""
    TEST_RESULT = 0x92
    SUITE_SUMMARY = 0x93
    STATUS_UPDATE = 0x94
    BATCH_START = 0x95
    BATCH_END = 0x96


class NoStdTestResultStatus(Enum):
    """Test result status codes from embedded tests"""
    PASS = 0x00
    FAIL = 0x01
    SKIP = 0x02
    RUNNING = 0x03
    TIMEOUT = 0x04
    ERROR = 0x05


@dataclass
class NoStdTestResult:
    """Parsed no_std test result from USB HID report"""
    test_id: int
    test_name: str
    status: NoStdTestResultStatus
    error_message: str
    execution_time_ms: int
    timestamp: float = field(default_factory=time.time)


@dataclass
class NoStdSuiteSummary:
    """Parsed no_std test suite summary from USB HID report"""
    suite_id: int
    suite_name: str
    total_tests: int
    passed: int
    failed: int
    skipped: int
    execution_time_ms: int
    timestamp: float = field(default_factory=time.time)


@dataclass
class NoStdTestCommand:
    """Command to execute no_std tests on device"""
    command_type: str  # 'run_suite', 'run_test', 'list_tests', 'reset_framework'
    suite_name: Optional[str] = None
    test_name: Optional[str] = None
    parameters: Dict[str, Any] = field(default_factory=dict)


class NoStdTestIntegration:
    """
    Integration layer between Python test framework and embedded no_std tests.
    
    Handles test execution commands, result collection, and integration with
    the existing test reporting pipeline.
    """
    
    def __init__(self, device_manager: UsbHidDeviceManager, command_handler: CommandHandler,
                 firmware_flasher: Optional[FirmwareFlasher] = None):
        """
        Initialize the no_std test integration.
        
        Args:
            device_manager: Device manager instance
            command_handler: Command handler instance
            firmware_flasher: Optional firmware flasher for test firmware deployment
        """
        self.device_manager = device_manager
        self.command_handler = command_handler
        self.firmware_flasher = firmware_flasher
        self.logger = logging.getLogger(__name__)
        
        # Result collection
        self.collected_results: Dict[str, List[NoStdTestResult]] = {}
        self.collected_summaries: Dict[str, List[NoStdSuiteSummary]] = {}
        self.active_batches: Dict[str, List[NoStdTestResult]] = {}
        
        # Test execution tracking
        self.active_executions: Dict[str, Dict[str, Any]] = {}
        
    def flash_test_firmware(self, device_serial: str, firmware_path: str, 
                           timeout: float = 30.0) -> bool:
        """
        Flash test firmware to device using bootloader.
        
        Args:
            device_serial: Target device serial number
            firmware_path: Path to test firmware binary
            timeout: Timeout for flashing operation
            
        Returns:
            True if flashing successful, False otherwise
        """
        if not self.firmware_flasher:
            self.logger.error("No firmware flasher configured")
            return False
            
        self.logger.info(f"Flashing test firmware to device {device_serial}")
        
        try:
            # Enter bootloader mode
            bootloader_command = self.command_handler.create_bootloader_command(timeout_ms=5000)
            response = self.command_handler.send_command_and_wait(
                device_serial, bootloader_command, timeout=10.0
            )
            
            if not response or response.status != ResponseStatus.SUCCESS:
                self.logger.error(f"Failed to enter bootloader mode on {device_serial}")
                return False
                
            # Wait for device to enter bootloader mode
            if not self.device_manager.wait_for_bootloader_mode(device_serial, timeout=10.0):
                self.logger.error(f"Device {device_serial} did not enter bootloader mode")
                return False
                
            # Flash the firmware
            flash_success = self.firmware_flasher.flash_firmware(
                device_serial, firmware_path, timeout=timeout
            )
            
            if not flash_success:
                self.logger.error(f"Failed to flash firmware to {device_serial}")
                return False
                
            # Wait for device to reconnect
            if not self.device_manager.wait_for_device_reconnection(device_serial, timeout=30.0):
                self.logger.error(f"Device {device_serial} did not reconnect after flashing")
                return False
                
            self.logger.info(f"Successfully flashed test firmware to {device_serial}")
            return True
            
        except Exception as e:
            self.logger.error(f"Error flashing test firmware to {device_serial}: {e}")
            return False
    
    def execute_nostd_test_suite(self, device_serial: str, suite_name: str,
                                timeout: float = 60.0) -> Optional[TestSuiteResult]:
        """
        Execute a no_std test suite on the device.
        
        Args:
            device_serial: Target device serial number
            suite_name: Name of test suite to execute
            timeout: Timeout for test execution
            
        Returns:
            Test suite results or None if failed
        """
        self.logger.info(f"Executing no_std test suite '{suite_name}' on device {device_serial}")
        
        # Clear previous results for this device
        self.collected_results[device_serial] = []
        self.collected_summaries[device_serial] = []
        self.active_batches[device_serial] = []
        
        # Create test execution command
        test_command = self._create_nostd_test_command("run_suite", suite_name=suite_name)
        
        # Send command to device
        command = TestCommand(
            command_type=CommandType.EXECUTE_TEST,
            command_id=0,  # Will be assigned by command handler
            payload={
                'test_type': 0xFF,  # Special value for no_std tests
                'parameters': {
                    'command_type': test_command.command_type,
                    'suite_name': test_command.suite_name,
                    'parameters': test_command.parameters
                }
            }
        )
        
        # Track execution
        execution_start = time.time()
        self.active_executions[device_serial] = {
            'suite_name': suite_name,
            'start_time': execution_start,
            'timeout': timeout,
            'status': 'running'
        }
        
        try:
            # Send the command
            if not self.command_handler.send_command(device_serial, command):
                self.logger.error(f"Failed to send test command to {device_serial}")
                return None
                
            # Collect results with timeout
            suite_result = self._collect_test_results(device_serial, suite_name, timeout)
            
            # Update execution tracking
            self.active_executions[device_serial]['status'] = 'completed' if suite_result else 'failed'
            
            return suite_result
            
        except Exception as e:
            self.logger.error(f"Error executing no_std test suite on {device_serial}: {e}")
            self.active_executions[device_serial]['status'] = 'error'
            return None
    
    def execute_nostd_test(self, device_serial: str, suite_name: str, test_name: str,
                          timeout: float = 30.0) -> Optional[NoStdTestResult]:
        """
        Execute a single no_std test on the device.
        
        Args:
            device_serial: Target device serial number
            suite_name: Name of test suite containing the test
            test_name: Name of specific test to execute
            timeout: Timeout for test execution
            
        Returns:
            Test result or None if failed
        """
        self.logger.info(f"Executing no_std test '{test_name}' from suite '{suite_name}' on device {device_serial}")
        
        # Create test execution command
        test_command = self._create_nostd_test_command("run_test", suite_name=suite_name, test_name=test_name)
        
        # Send command to device
        command = TestCommand(
            command_type=CommandType.EXECUTE_TEST,
            command_id=0,
            payload={
                'test_type': 0xFF,  # Special value for no_std tests
                'parameters': {
                    'command_type': test_command.command_type,
                    'suite_name': test_command.suite_name,
                    'test_name': test_command.test_name,
                    'parameters': test_command.parameters
                }
            }
        )
        
        try:
            # Send the command and wait for response
            response = self.command_handler.send_command_and_wait(device_serial, command, timeout=timeout)
            
            if not response or response.status != ResponseStatus.SUCCESS:
                self.logger.error(f"Test execution failed on {device_serial}")
                return None
                
            # Collect single test result
            start_time = time.time()
            while time.time() - start_time < timeout:
                results = self._read_and_parse_test_reports(device_serial)
                
                for result in results:
                    if result.test_name == test_name:
                        return result
                        
                time.sleep(0.1)
                
            self.logger.warning(f"Timeout waiting for test result from {device_serial}")
            return None
            
        except Exception as e:
            self.logger.error(f"Error executing no_std test on {device_serial}: {e}")
            return None
    
    def list_available_tests(self, device_serial: str, timeout: float = 10.0) -> Optional[Dict[str, List[str]]]:
        """
        Get list of available test suites and tests from device.
        
        Args:
            device_serial: Target device serial number
            timeout: Timeout for command response
            
        Returns:
            Dictionary mapping suite names to test names, or None if failed
        """
        self.logger.info(f"Listing available no_std tests on device {device_serial}")
        
        # Create list tests command
        test_command = self._create_nostd_test_command("list_tests")
        
        command = TestCommand(
            command_type=CommandType.CONFIGURATION_QUERY,
            command_id=0,
            payload={
                'query_type': 'nostd_tests',
                'parameters': {
                    'command_type': test_command.command_type,
                    'parameters': test_command.parameters
                }
            }
        )
        
        try:
            response = self.command_handler.send_command_and_wait(device_serial, command, timeout=timeout)
            
            if response and response.status == ResponseStatus.SUCCESS:
                # Parse test list from response data
                if 'test_suites' in response.data:
                    return response.data['test_suites']
                    
            self.logger.error(f"Failed to get test list from {device_serial}")
            return None
            
        except Exception as e:
            self.logger.error(f"Error listing tests on {device_serial}: {e}")
            return None
    
    def reset_test_framework(self, device_serial: str, timeout: float = 5.0) -> bool:
        """
        Reset the no_std test framework on the device.
        
        Args:
            device_serial: Target device serial number
            timeout: Timeout for command response
            
        Returns:
            True if reset successful, False otherwise
        """
        self.logger.info(f"Resetting no_std test framework on device {device_serial}")
        
        test_command = self._create_nostd_test_command("reset_framework")
        
        command = TestCommand(
            command_type=CommandType.EXECUTE_TEST,
            command_id=0,
            payload={
                'test_type': 0xFF,
                'parameters': {
                    'command_type': test_command.command_type,
                    'parameters': test_command.parameters
                }
            }
        )
        
        try:
            response = self.command_handler.send_command_and_wait(device_serial, command, timeout=timeout)
            return response is not None and response.status == ResponseStatus.SUCCESS
            
        except Exception as e:
            self.logger.error(f"Error resetting test framework on {device_serial}: {e}")
            return False
    
    def _create_nostd_test_command(self, command_type: str, suite_name: Optional[str] = None,
                                  test_name: Optional[str] = None, 
                                  parameters: Optional[Dict[str, Any]] = None) -> NoStdTestCommand:
        """Create a no_std test command"""
        return NoStdTestCommand(
            command_type=command_type,
            suite_name=suite_name,
            test_name=test_name,
            parameters=parameters or {}
        )
    
    def _collect_test_results(self, device_serial: str, suite_name: str, 
                             timeout: float) -> Optional[TestSuiteResult]:
        """
        Collect test results from device over the specified timeout period.
        
        Args:
            device_serial: Device to collect results from
            suite_name: Expected suite name
            timeout: Maximum time to wait for results
            
        Returns:
            Collected test suite results or None if failed
        """
        start_time = time.time()
        suite_completed = False
        
        while time.time() - start_time < timeout and not suite_completed:
            # Read and parse test reports
            results = self._read_and_parse_test_reports(device_serial)
            
            # Process results
            for result in results:
                if device_serial not in self.collected_results:
                    self.collected_results[device_serial] = []
                self.collected_results[device_serial].append(result)
            
            # Check for suite completion
            if device_serial in self.collected_summaries:
                for summary in self.collected_summaries[device_serial]:
                    if summary.suite_name == suite_name:
                        suite_completed = True
                        break
            
            time.sleep(0.1)  # Small delay to avoid busy waiting
        
        if not suite_completed:
            self.logger.warning(f"Test suite '{suite_name}' did not complete within timeout on {device_serial}")
        
        # Convert collected results to TestSuiteResult format
        return self._convert_to_test_suite_result(device_serial, suite_name)
    
    def _read_and_parse_test_reports(self, device_serial: str) -> List[NoStdTestResult]:
        """
        Read USB HID reports from device and parse test results.
        
        Args:
            device_serial: Device to read from
            
        Returns:
            List of parsed test results
        """
        results = []
        
        try:
            device_handle = self.device_manager.get_device_handle(device_serial)
            if not device_handle:
                return results
            
            # Read available HID input reports
            while True:
                data = device_handle.read(64, timeout_ms=10)
                if not data:
                    break
                
                # Parse the report
                parsed_result = self._parse_test_report(bytes(data))
                if parsed_result:
                    results.append(parsed_result)
                    
        except Exception as e:
            self.logger.error(f"Error reading test reports from {device_serial}: {e}")
            
        return results
    
    def _parse_test_report(self, report_data: bytes) -> Optional[Union[NoStdTestResult, NoStdSuiteSummary]]:
        """
        Parse a USB HID test report into structured data.
        
        Args:
            report_data: 64-byte HID report data
            
        Returns:
            Parsed test result/summary or None if not a test report
        """
        if len(report_data) != 64:
            return None
            
        try:
            report_type = NoStdTestReportType(report_data[0])
            
            if report_type == NoStdTestReportType.TEST_RESULT:
                return self._parse_test_result_report(report_data)
            elif report_type == NoStdTestReportType.SUITE_SUMMARY:
                return self._parse_suite_summary_report(report_data)
            elif report_type == NoStdTestReportType.STATUS_UPDATE:
                return self._parse_status_update_report(report_data)
            elif report_type == NoStdTestReportType.BATCH_START:
                self._handle_batch_start(report_data)
            elif report_type == NoStdTestReportType.BATCH_END:
                self._handle_batch_end(report_data)
                
        except (ValueError, struct.error) as e:
            # Not a test report or malformed data
            self.logger.debug(f"Ignoring non-test report data: {e}")
            
        return None
    
    def _parse_test_result_report(self, report_data: bytes) -> NoStdTestResult:
        """Parse individual test result report"""
        test_id = report_data[1]
        status = NoStdTestResultStatus(report_data[2])
        
        # Extract test name (bytes 4-35, null-terminated)
        test_name_bytes = report_data[4:36]
        test_name = test_name_bytes.split(b'\x00')[0].decode('utf-8', errors='ignore')
        
        # Extract error message (bytes 36-59, null-terminated)
        error_bytes = report_data[36:60]
        error_message = error_bytes.split(b'\x00')[0].decode('utf-8', errors='ignore')
        
        # Extract execution time (bytes 60-63, little-endian)
        execution_time_ms = struct.unpack('<I', report_data[60:64])[0]
        
        return NoStdTestResult(
            test_id=test_id,
            test_name=test_name,
            status=status,
            error_message=error_message,
            execution_time_ms=execution_time_ms
        )
    
    def _parse_suite_summary_report(self, report_data: bytes) -> NoStdSuiteSummary:
        """Parse test suite summary report"""
        suite_id = report_data[1]
        
        # Extract statistics (bytes 4-15)
        total_tests = struct.unpack('<H', report_data[4:6])[0]
        passed = struct.unpack('<H', report_data[6:8])[0]
        failed = struct.unpack('<H', report_data[8:10])[0]
        skipped = struct.unpack('<H', report_data[10:12])[0]
        execution_time_ms = struct.unpack('<I', report_data[12:16])[0]
        
        # Extract suite name (bytes 16-47, null-terminated)
        suite_name_bytes = report_data[16:48]
        suite_name = suite_name_bytes.split(b'\x00')[0].decode('utf-8', errors='ignore')
        
        return NoStdSuiteSummary(
            suite_id=suite_id,
            suite_name=suite_name,
            total_tests=total_tests,
            passed=passed,
            failed=failed,
            skipped=skipped,
            execution_time_ms=execution_time_ms
        )
    
    def _parse_status_update_report(self, report_data: bytes) -> Optional[NoStdTestResult]:
        """Parse status update report"""
        # Status updates are treated as special test results
        test_id = report_data[1]
        status = NoStdTestResultStatus(report_data[2])
        
        # Extract status message (bytes 36-59)
        message_bytes = report_data[36:60]
        message = message_bytes.split(b'\x00')[0].decode('utf-8', errors='ignore')
        
        return NoStdTestResult(
            test_id=test_id,
            test_name="__status_update__",
            status=status,
            error_message=message,
            execution_time_ms=0
        )
    
    def _handle_batch_start(self, report_data: bytes) -> None:
        """Handle batch start marker"""
        batch_size = report_data[1]
        self.logger.debug(f"Starting test result batch (size: {batch_size})")
    
    def _handle_batch_end(self, report_data: bytes) -> None:
        """Handle batch end marker"""
        self.logger.debug("Test result batch completed")
    
    def _convert_to_test_suite_result(self, device_serial: str, suite_name: str) -> Optional[TestSuiteResult]:
        """
        Convert collected no_std test results to TestSuiteResult format.
        
        Args:
            device_serial: Device serial number
            suite_name: Test suite name
            
        Returns:
            Converted test suite result or None if no results
        """
        if device_serial not in self.collected_results:
            return None
            
        results = self.collected_results[device_serial]
        if not results:
            return None
            
        # Find suite summary
        suite_summary = None
        if device_serial in self.collected_summaries:
            for summary in self.collected_summaries[device_serial]:
                if summary.suite_name == suite_name:
                    suite_summary = summary
                    break
        
        # Convert results to TestExecution format
        executions = []
        for result in results:
            if result.test_name == "__status_update__":
                continue  # Skip status updates
                
            # Create TestStep for this result
            test_step = TestStep(
                name=result.test_name,
                test_type=None,  # No specific test type for no_std tests
                parameters={},
                timeout=30.0,
                required=True
            )
            
            # Determine TestStatus from NoStdTestResultStatus
            if result.status == NoStdTestResultStatus.PASS:
                status = TestStatus.COMPLETED
            elif result.status == NoStdTestResultStatus.FAIL:
                status = TestStatus.FAILED
            elif result.status == NoStdTestResultStatus.SKIP:
                status = TestStatus.SKIPPED
            elif result.status == NoStdTestResultStatus.TIMEOUT:
                status = TestStatus.TIMEOUT
            else:
                status = TestStatus.FAILED
            
            # Create TestExecution
            execution = TestExecution(
                step=test_step,
                device_serial=device_serial,
                status=status,
                start_time=result.timestamp,
                end_time=result.timestamp + (result.execution_time_ms / 1000.0),
                error_message=result.error_message if result.error_message else None
            )
            
            executions.append(execution)
        
        # Calculate metrics
        total_tests = len(executions)
        passed_tests = sum(1 for e in executions if e.status == TestStatus.COMPLETED)
        failed_tests = sum(1 for e in executions if e.status == TestStatus.FAILED)
        skipped_tests = sum(1 for e in executions if e.status == TestStatus.SKIPPED)
        timeout_tests = sum(1 for e in executions if e.status == TestStatus.TIMEOUT)
        
        total_duration = sum((e.end_time - e.start_time) for e in executions if e.start_time and e.end_time)
        average_duration = total_duration / total_tests if total_tests > 0 else 0.0
        success_rate = (passed_tests / total_tests * 100) if total_tests > 0 else 0.0
        
        metrics = TestMetrics(
            total_tests=total_tests,
            passed_tests=passed_tests,
            failed_tests=failed_tests,
            skipped_tests=skipped_tests,
            timeout_tests=timeout_tests,
            total_duration=total_duration,
            average_duration=average_duration,
            success_rate=success_rate
        )
        
        # Determine overall status
        if failed_tests > 0 or timeout_tests > 0:
            overall_status = TestStatus.FAILED
        elif passed_tests == total_tests:
            overall_status = TestStatus.COMPLETED
        else:
            overall_status = TestStatus.COMPLETED  # Some skipped but no failures
        
        # Create DeviceTestResult
        device_start_time = min((e.start_time for e in executions if e.start_time), default=time.time())
        device_end_time = max((e.end_time for e in executions if e.end_time), default=time.time())
        
        device_result = DeviceTestResult(
            device_serial=device_serial,
            executions=executions,
            metrics=metrics,
            start_time=device_start_time,
            end_time=device_end_time,
            overall_status=overall_status
        )
        
        # Create TestSuiteResult
        suite_result = TestSuiteResult(
            suite_name=suite_name,
            description=f"No-std test suite: {suite_name}",
            device_results={device_serial: device_result},
            aggregate_metrics=metrics,
            start_time=device_start_time,
            end_time=device_end_time,
            duration=device_end_time - device_start_time,
            artifacts=[],
            performance_trends=[],
            environment_info={'test_type': 'nostd_embedded'}
        )
        
        return suite_result
    
    def get_execution_status(self, device_serial: str) -> Optional[Dict[str, Any]]:
        """Get current execution status for a device"""
        return self.active_executions.get(device_serial)
    
    def clear_results(self, device_serial: str) -> None:
        """Clear collected results for a device"""
        if device_serial in self.collected_results:
            del self.collected_results[device_serial]
        if device_serial in self.collected_summaries:
            del self.collected_summaries[device_serial]
        if device_serial in self.active_batches:
            del self.active_batches[device_serial]
        if device_serial in self.active_executions:
            del self.active_executions[device_serial]