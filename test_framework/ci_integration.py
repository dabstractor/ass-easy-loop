#!/usr/bin/env python3
"""
Enhanced CI/CD Integration Module

Provides comprehensive CI/CD integration capabilities including:
- Headless operation with proper exit codes
- Parallel testing support for multiple devices
- Standard test result formats (JUnit XML, JSON, TAP, etc.)
- Automated device setup and cleanup
- Environment detection and configuration
- Pipeline integration utilities
"""

import sys
import os
import json
import time
import logging
import argparse
import threading
import concurrent.futures
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Any
from dataclasses import dataclass, asdict
from datetime import datetime
import xml.etree.ElementTree as ET

from test_framework.device_manager import UsbHidDeviceManager
from test_framework.command_handler import CommandHandler
from test_framework.test_sequencer import TestSequencer, TestConfiguration, TestStep, TestType
from test_framework.firmware_flasher import FirmwareFlasher, FlashResult
from test_framework.result_collector import ResultCollector, TestSuiteResult
from test_framework.report_generator import ReportGenerator


@dataclass
class CIEnvironmentInfo:
    """Information about the CI environment"""
    ci_system: str  # jenkins, github_actions, gitlab_ci, azure_devops, etc.
    build_number: Optional[str]
    branch_name: Optional[str]
    commit_hash: Optional[str]
    pull_request: Optional[str]
    workspace_path: str
    environment_variables: Dict[str, str]


@dataclass
class CITestConfiguration:
    """CI-specific test configuration"""
    test_config: TestConfiguration
    required_devices: int
    max_parallel_devices: int
    firmware_path: Optional[str]
    timeout_seconds: float
    retry_attempts: int
    fail_fast: bool
    generate_artifacts: bool
    artifact_retention_days: int


@dataclass
class CITestResult:
    """Comprehensive CI test result"""
    success: bool
    exit_code: int
    total_tests: int
    passed_tests: int
    failed_tests: int
    skipped_tests: int
    duration_seconds: float
    devices_tested: List[str]
    environment_info: CIEnvironmentInfo
    artifacts_generated: List[str]
    error_summary: Optional[str]


class CIIntegration:
    """
    Enhanced CI/CD integration with comprehensive automation support.
    
    Provides headless operation, parallel testing, standard reporting formats,
    and automated device management for CI/CD pipelines.
    """
    
    def __init__(self, output_dir: str = "ci_test_results", verbose: bool = False):
        """
        Initialize CI integration.
        
        Args:
            output_dir: Directory for test outputs and artifacts
            verbose: Enable verbose logging
        """
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(exist_ok=True)
        self.verbose = verbose
        
        # Setup logging for CI environment
        self.setup_ci_logging()
        self.logger = logging.getLogger(__name__)
        
        # Initialize framework components
        self.device_manager = UsbHidDeviceManager()
        self.command_handler = CommandHandler(self.device_manager)
        self.test_sequencer = TestSequencer(self.device_manager, self.command_handler)
        self.firmware_flasher = FirmwareFlasher(self.device_manager, self.command_handler)
        self.result_collector = ResultCollector(str(self.output_dir / "artifacts"))
        self.report_generator = ReportGenerator(str(self.output_dir))
        
        # CI environment detection
        self.environment_info = self.detect_ci_environment()
        
        # Test execution state
        self.test_results: Dict[str, Any] = {}
        self.flash_results: Dict[str, Any] = {}
        self.start_time: Optional[float] = None
        self.end_time: Optional[float] = None
        
    def setup_ci_logging(self):
        """Setup logging optimized for CI environments"""
        # Create logs directory
        log_dir = self.output_dir / "logs"
        log_dir.mkdir(exist_ok=True)
        
        # Configure logging level
        log_level = logging.DEBUG if self.verbose else logging.INFO
        
        # Create formatters
        detailed_formatter = logging.Formatter(
            '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        )
        simple_formatter = logging.Formatter(
            '%(levelname)s: %(message)s'
        )
        
        # Setup root logger
        root_logger = logging.getLogger()
        root_logger.setLevel(log_level)
        
        # Console handler (for CI output)
        console_handler = logging.StreamHandler(sys.stdout)
        console_handler.setLevel(log_level)
        console_handler.setFormatter(simple_formatter)
        root_logger.addHandler(console_handler)
        
        # File handler (for detailed logs)
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        log_file = log_dir / f"ci_test_{timestamp}.log"
        file_handler = logging.FileHandler(log_file)
        file_handler.setLevel(logging.DEBUG)
        file_handler.setFormatter(detailed_formatter)
        root_logger.addHandler(file_handler)
        
        self.logger = logging.getLogger(__name__)
        self.logger.info(f"CI logging initialized - log file: {log_file}")
    
    def detect_ci_environment(self) -> CIEnvironmentInfo:
        """Detect and gather CI environment information"""
        env_vars = dict(os.environ)
        
        # Detect CI system
        ci_system = "unknown"
        build_number = None
        branch_name = None
        commit_hash = None
        pull_request = None
        
        if "JENKINS_URL" in env_vars:
            ci_system = "jenkins"
            build_number = env_vars.get("BUILD_NUMBER")
            branch_name = env_vars.get("GIT_BRANCH")
            commit_hash = env_vars.get("GIT_COMMIT")
        elif "GITHUB_ACTIONS" in env_vars:
            ci_system = "github_actions"
            build_number = env_vars.get("GITHUB_RUN_NUMBER")
            branch_name = env_vars.get("GITHUB_REF_NAME")
            commit_hash = env_vars.get("GITHUB_SHA")
            pull_request = env_vars.get("GITHUB_EVENT_NUMBER")
        elif "GITLAB_CI" in env_vars:
            ci_system = "gitlab_ci"
            build_number = env_vars.get("CI_PIPELINE_ID")
            branch_name = env_vars.get("CI_COMMIT_REF_NAME")
            commit_hash = env_vars.get("CI_COMMIT_SHA")
            pull_request = env_vars.get("CI_MERGE_REQUEST_IID")
        elif "AZURE_HTTP_USER_AGENT" in env_vars:
            ci_system = "azure_devops"
            build_number = env_vars.get("BUILD_BUILDNUMBER")
            branch_name = env_vars.get("BUILD_SOURCEBRANCH")
            commit_hash = env_vars.get("BUILD_SOURCEVERSION")
        elif "CI" in env_vars:
            ci_system = "generic_ci"
        
        return CIEnvironmentInfo(
            ci_system=ci_system,
            build_number=build_number,
            branch_name=branch_name,
            commit_hash=commit_hash,
            pull_request=pull_request,
            workspace_path=os.getcwd(),
            environment_variables=env_vars
        )
    
    def load_ci_configuration(self, config_path: Optional[str] = None) -> CITestConfiguration:
        """Load CI test configuration from file or environment"""
        if config_path and Path(config_path).exists():
            with open(config_path, 'r') as f:
                config_data = json.load(f)
        else:
            # Use default configuration
            config_data = self.get_default_ci_configuration()
        
        # Parse test configuration
        test_config = self.parse_test_configuration(config_data.get('test_config', {}))
        
        return CITestConfiguration(
            test_config=test_config,
            required_devices=config_data.get('required_devices', 1),
            max_parallel_devices=config_data.get('max_parallel_devices', 4),
            firmware_path=config_data.get('firmware_path'),
            timeout_seconds=config_data.get('timeout_seconds', 300.0),
            retry_attempts=config_data.get('retry_attempts', 1),
            fail_fast=config_data.get('fail_fast', True),
            generate_artifacts=config_data.get('generate_artifacts', True),
            artifact_retention_days=config_data.get('artifact_retention_days', 30)
        )
    
    def get_default_ci_configuration(self) -> Dict[str, Any]:
        """Get default CI test configuration"""
        return {
            "test_config": {
                "name": "CI Validation Suite",
                "description": "Automated validation for CI/CD pipeline",
                "steps": [
                    {
                        "name": "device_communication_test",
                        "test_type": "USB_COMMUNICATION_TEST",
                        "parameters": {"message_count": 3, "timeout_ms": 1000},
                        "timeout": 10.0,
                        "required": True
                    },
                    {
                        "name": "pemf_timing_validation",
                        "test_type": "PEMF_TIMING_VALIDATION",
                        "parameters": {"duration_ms": 2000, "tolerance_percent": 1.0},
                        "timeout": 15.0,
                        "required": True,
                        "depends_on": ["device_communication_test"]
                    },
                    {
                        "name": "battery_monitoring_test",
                        "test_type": "BATTERY_ADC_CALIBRATION",
                        "parameters": {"reference_voltage": 3.3},
                        "timeout": 10.0,
                        "required": True,
                        "depends_on": ["device_communication_test"]
                    },
                    {
                        "name": "system_health_check",
                        "test_type": "SYSTEM_STRESS_TEST",
                        "parameters": {"duration_ms": 3000, "load_level": 2},
                        "timeout": 15.0,
                        "required": False,
                        "depends_on": ["pemf_timing_validation", "battery_monitoring_test"]
                    }
                ],
                "parallel_execution": True,
                "global_timeout": 120.0
            },
            "required_devices": 1,
            "max_parallel_devices": 4,
            "timeout_seconds": 300.0,
            "retry_attempts": 1,
            "fail_fast": True,
            "generate_artifacts": True,
            "artifact_retention_days": 30
        }
    
    def parse_test_configuration(self, config_data: Dict[str, Any]) -> TestConfiguration:
        """Parse test configuration from dictionary"""
        steps = []
        for step_data in config_data.get('steps', []):
            # Handle both string and enum values for test_type
            test_type_value = step_data['test_type']
            if isinstance(test_type_value, str):
                test_type = TestType[test_type_value]
            else:
                test_type = TestType(test_type_value)
            
            steps.append(TestStep(
                name=step_data['name'],
                test_type=test_type,
                parameters=step_data.get('parameters', {}),
                timeout=step_data.get('timeout', 30.0),
                retry_count=step_data.get('retry_count', 0),
                required=step_data.get('required', True),
                depends_on=step_data.get('depends_on', [])
            ))
        
        return TestConfiguration(
            name=config_data.get('name', 'CI Test Suite'),
            description=config_data.get('description', ''),
            steps=steps,
            parallel_execution=config_data.get('parallel_execution', True),
            max_parallel_devices=config_data.get('max_parallel_devices', 4),
            global_timeout=config_data.get('global_timeout', 300.0)
        )
    
    def discover_and_setup_devices(self, required_count: int, 
                                  max_attempts: int = 3) -> Tuple[List[str], bool]:
        """
        Discover and setup test devices with retry logic.
        
        Args:
            required_count: Minimum number of devices required
            max_attempts: Maximum discovery attempts
            
        Returns:
            Tuple of (connected_devices, success)
        """
        self.logger.info(f"Discovering devices (required: {required_count})...")
        
        connected_devices = []
        
        for attempt in range(max_attempts):
            self.logger.info(f"Discovery attempt {attempt + 1}/{max_attempts}")
            
            try:
                # Discover devices
                devices = self.device_manager.discover_devices()
                self.logger.info(f"Found {len(devices)} device(s)")
                
                # Attempt to connect to devices
                for device in devices:
                    if device.status.value == "connected":
                        if self.device_manager.connect_device(device.serial_number):
                            connected_devices.append(device.serial_number)
                            self.logger.info(f"Connected to device: {device.serial_number}")
                        else:
                            self.logger.warning(f"Failed to connect to device: {device.serial_number}")
                
                # Check if we have enough devices
                if len(connected_devices) >= required_count:
                    self.logger.info(f"Successfully connected to {len(connected_devices)} devices")
                    return connected_devices, True
                
                # If not enough devices and more attempts available, wait and retry
                if attempt < max_attempts - 1:
                    self.logger.warning(f"Only {len(connected_devices)} devices connected, retrying in 3 seconds...")
                    time.sleep(3.0)
                    # Disconnect current devices before retry
                    for device_serial in connected_devices:
                        self.device_manager.disconnect_device(device_serial)
                    connected_devices.clear()
                
            except Exception as e:
                self.logger.error(f"Device discovery attempt {attempt + 1} failed: {e}")
                if attempt < max_attempts - 1:
                    time.sleep(2.0)
        
        self.logger.error(f"Failed to connect to required devices: {len(connected_devices)}/{required_count}")
        return connected_devices, False
    
    def flash_firmware_parallel(self, devices: List[str], firmware_path: str,
                               max_parallel: int = 4) -> Tuple[Dict[str, Any], bool]:
        """
        Flash firmware to multiple devices in parallel.
        
        Args:
            devices: List of device serial numbers
            firmware_path: Path to firmware file
            max_parallel: Maximum parallel flash operations
            
        Returns:
            Tuple of (flash_results, overall_success)
        """
        if not Path(firmware_path).exists():
            self.logger.error(f"Firmware file not found: {firmware_path}")
            return {}, False
        
        self.logger.info(f"Flashing firmware to {len(devices)} devices (parallel: {max_parallel})...")
        
        # Create device-firmware mapping
        device_firmware_map = {device: firmware_path for device in devices}
        
        # Flash firmware in parallel
        flash_results = self.firmware_flasher.flash_multiple_devices(
            device_firmware_map,
            parallel=True,
            max_parallel=max_parallel
        )
        
        # Analyze results
        success_count = 0
        for device_serial, operation in flash_results.items():
            if operation.result == FlashResult.SUCCESS:
                success_count += 1
                self.logger.info(f"Firmware flash successful: {device_serial} "
                               f"({operation.total_duration:.1f}s)")
            else:
                self.logger.error(f"Firmware flash failed: {device_serial} - "
                                f"{operation.error_message}")
        
        overall_success = success_count == len(devices)
        self.logger.info(f"Firmware flash completed: {success_count}/{len(devices)} successful")
        
        return flash_results, overall_success
    
    def run_parallel_tests(self, config: CITestConfiguration, 
                          devices: List[str]) -> Tuple[TestSuiteResult, bool]:
        """
        Run tests on multiple devices in parallel.
        
        Args:
            config: CI test configuration
            devices: List of device serial numbers
            
        Returns:
            Tuple of (test_suite_result, overall_success)
        """
        self.logger.info(f"Running tests on {len(devices)} devices...")
        self.start_time = time.time()
        
        try:
            # Execute test sequence with parallel support
            execution_results = self.test_sequencer.execute_test_sequence(
                config.test_config,
                target_devices=devices,
                global_timeout=config.timeout_seconds
            )
            
            self.end_time = time.time()
            
            # Collect comprehensive results
            suite_result = self.result_collector.collect_results(
                config.test_config.name,
                config.test_config.description,
                execution_results,
                self.start_time,
                self.end_time
            )
            
            # Determine overall success
            overall_success = suite_result.aggregate_metrics.failed_tests == 0
            
            # Log summary
            self.log_test_summary(suite_result)
            
            return suite_result, overall_success
            
        except Exception as e:
            self.logger.error(f"Test execution failed: {e}")
            self.end_time = time.time()
            
            # Create minimal failure result
            from .result_collector import TestSuiteResult, TestMetrics
            failure_result = TestSuiteResult(
                suite_name=config.test_config.name,
                description=f"Test execution failed: {e}",
                start_time=self.start_time,
                end_time=self.end_time,
                duration=self.end_time - self.start_time,
                device_results={},
                aggregate_metrics=TestMetrics(
                    total_tests=0, passed_tests=0, failed_tests=1,
                    skipped_tests=0, success_rate=0.0
                ),
                performance_trends=[],
                artifacts=[],
                environment_info=asdict(self.environment_info)
            )
            
            return failure_result, False
    
    def log_test_summary(self, suite_result: TestSuiteResult):
        """Log comprehensive test summary for CI"""
        self.logger.info("=" * 60)
        self.logger.info(f"TEST SUITE COMPLETED: {suite_result.suite_name}")
        self.logger.info("=" * 60)
        
        metrics = suite_result.aggregate_metrics
        self.logger.info(f"Total Tests: {metrics.total_tests}")
        self.logger.info(f"Passed: {metrics.passed_tests}")
        self.logger.info(f"Failed: {metrics.failed_tests}")
        self.logger.info(f"Skipped: {metrics.skipped_tests}")
        self.logger.info(f"Success Rate: {metrics.success_rate:.1f}%")
        self.logger.info(f"Duration: {suite_result.duration:.1f} seconds")
        self.logger.info(f"Devices Tested: {len(suite_result.device_results)}")
        
        # Log device-specific results
        for device_serial, device_result in suite_result.device_results.items():
            device_metrics = device_result.metrics
            self.logger.info(f"  {device_serial}: {device_metrics.passed_tests}/"
                           f"{device_metrics.total_tests} passed "
                           f"({device_metrics.success_rate:.1f}%)")
        
        # Log failures if any
        if metrics.failed_tests > 0:
            self.logger.error("FAILED TESTS:")
            for device_serial, device_result in suite_result.device_results.items():
                for execution in device_result.executions:
                    if execution.status.value == 'failed':
                        self.logger.error(f"  {device_serial}: {execution.step.name} - "
                                        f"{execution.error_message}")
        
        self.logger.info("=" * 60)
    
    def generate_ci_reports(self, suite_result: TestSuiteResult, 
                           config: CITestConfiguration) -> List[str]:
        """
        Generate comprehensive CI reports in multiple formats.
        
        Args:
            suite_result: Test suite results
            config: CI configuration
            
        Returns:
            List of generated report file paths
        """
        self.logger.info("Generating CI reports...")
        
        # Determine report formats based on CI system and configuration
        report_formats = ['json', 'junit']  # Always generate these
        
        if config.generate_artifacts:
            report_formats.extend(['html', 'csv'])
        
        # Generate reports using the report generator
        try:
            report_files = self.report_generator.generate_comprehensive_report(
                suite_result, report_formats
            )
            
            generated_files = []
            for format_name, filepath in report_files.items():
                generated_files.append(filepath)
                self.logger.info(f"Generated {format_name.upper()} report: {filepath}")
            
            # Generate additional CI-specific formats
            if self.environment_info.ci_system in ['jenkins', 'azure_devops']:
                # Generate TAP format for Jenkins/Azure DevOps
                tap_file = self.generate_tap_report(suite_result)
                generated_files.append(tap_file)
            
            return generated_files
            
        except Exception as e:
            self.logger.error(f"Failed to generate some reports: {e}")
            return []
    
    def generate_tap_report(self, suite_result: TestSuiteResult) -> str:
        """Generate TAP (Test Anything Protocol) format report"""
        timestamp = time.strftime("%Y%m%d_%H%M%S")
        tap_file = self.output_dir / f"{suite_result.suite_name}_tap_{timestamp}.tap"
        
        with open(tap_file, 'w') as f:
            # TAP version
            f.write("TAP version 13\n")
            
            # Plan
            total_tests = suite_result.aggregate_metrics.total_tests
            f.write(f"1..{total_tests}\n")
            
            # Test results
            test_number = 1
            for device_serial, device_result in suite_result.device_results.items():
                for execution in device_result.executions:
                    test_name = f"{device_serial}.{execution.step.name}"
                    
                    if execution.status.value == 'completed':
                        f.write(f"ok {test_number} - {test_name}\n")
                    elif execution.status.value == 'skipped':
                        f.write(f"ok {test_number} - {test_name} # SKIP\n")
                    else:
                        f.write(f"not ok {test_number} - {test_name}\n")
                        if execution.error_message:
                            f.write(f"  ---\n")
                            f.write(f"  message: {execution.error_message}\n")
                            f.write(f"  severity: fail\n")
                            f.write(f"  ---\n")
                    
                    test_number += 1
        
        self.logger.info(f"Generated TAP report: {tap_file}")
        return str(tap_file)
    
    def cleanup_resources(self):
        """Clean up all resources and connections"""
        self.logger.info("Cleaning up CI resources...")
        
        try:
            # Disconnect all devices
            self.device_manager.disconnect_all()
            
            # Clean up old artifacts if configured
            self.cleanup_old_artifacts()
            
        except Exception as e:
            self.logger.error(f"Error during cleanup: {e}")
    
    def cleanup_old_artifacts(self, retention_days: int = 30):
        """Clean up old test artifacts"""
        try:
            import glob
            from datetime import datetime, timedelta
            
            cutoff_date = datetime.now() - timedelta(days=retention_days)
            
            # Find old files
            old_files = []
            for pattern in ['*.json', '*.xml', '*.html', '*.csv', '*.tap', '*.log']:
                for filepath in glob.glob(str(self.output_dir / pattern)):
                    file_path = Path(filepath)
                    if file_path.stat().st_mtime < cutoff_date.timestamp():
                        old_files.append(file_path)
            
            # Remove old files
            for file_path in old_files:
                try:
                    file_path.unlink()
                    self.logger.debug(f"Removed old artifact: {file_path}")
                except Exception as e:
                    self.logger.warning(f"Failed to remove old artifact {file_path}: {e}")
            
            if old_files:
                self.logger.info(f"Cleaned up {len(old_files)} old artifacts")
                
        except Exception as e:
            self.logger.warning(f"Failed to cleanup old artifacts: {e}")
    
    def run_ci_pipeline(self, config_path: Optional[str] = None) -> CITestResult:
        """
        Run complete CI pipeline with all steps.
        
        Args:
            config_path: Path to CI configuration file
            
        Returns:
            Comprehensive CI test result
        """
        pipeline_start = time.time()
        
        try:
            # Load configuration
            config = self.load_ci_configuration(config_path)
            self.logger.info(f"Loaded CI configuration: {config.test_config.name}")
            
            # Discover and setup devices
            devices, device_setup_success = self.discover_and_setup_devices(
                config.required_devices
            )
            
            if not device_setup_success:
                return CITestResult(
                    success=False,
                    exit_code=2,  # Device setup failure
                    total_tests=0,
                    passed_tests=0,
                    failed_tests=0,
                    skipped_tests=0,
                    duration_seconds=time.time() - pipeline_start,
                    devices_tested=[],
                    environment_info=self.environment_info,
                    artifacts_generated=[],
                    error_summary="Failed to discover and setup required devices"
                )
            
            # Flash firmware if specified
            flash_success = True
            if config.firmware_path:
                self.flash_results, flash_success = self.flash_firmware_parallel(
                    devices, config.firmware_path, config.max_parallel_devices
                )
                
                if not flash_success and config.fail_fast:
                    return CITestResult(
                        success=False,
                        exit_code=3,  # Firmware flash failure
                        total_tests=0,
                        passed_tests=0,
                        failed_tests=0,
                        skipped_tests=0,
                        duration_seconds=time.time() - pipeline_start,
                        devices_tested=devices,
                        environment_info=self.environment_info,
                        artifacts_generated=[],
                        error_summary="Firmware flashing failed"
                    )
            
            # Run tests
            suite_result, test_success = self.run_parallel_tests(config, devices)
            
            # Generate reports
            artifacts = []
            if config.generate_artifacts:
                artifacts = self.generate_ci_reports(suite_result, config)
            
            # Determine exit code
            exit_code = 0
            if not test_success:
                exit_code = 1  # Test failures
            elif not flash_success:
                exit_code = 3  # Firmware flash issues (non-fatal)
            
            return CITestResult(
                success=test_success and flash_success,
                exit_code=exit_code,
                total_tests=suite_result.aggregate_metrics.total_tests,
                passed_tests=suite_result.aggregate_metrics.passed_tests,
                failed_tests=suite_result.aggregate_metrics.failed_tests,
                skipped_tests=suite_result.aggregate_metrics.skipped_tests,
                duration_seconds=time.time() - pipeline_start,
                devices_tested=devices,
                environment_info=self.environment_info,
                artifacts_generated=artifacts,
                error_summary=None if test_success else "Test failures detected"
            )
            
        except Exception as e:
            self.logger.error(f"CI pipeline failed with exception: {e}")
            return CITestResult(
                success=False,
                exit_code=4,  # Unexpected error
                total_tests=0,
                passed_tests=0,
                failed_tests=0,
                skipped_tests=0,
                duration_seconds=time.time() - pipeline_start,
                devices_tested=[],
                environment_info=self.environment_info,
                artifacts_generated=[],
                error_summary=f"Unexpected error: {e}"
            )
        
        finally:
            # Always cleanup
            self.cleanup_resources()


def create_ci_integration(output_dir: str = "ci_test_results", 
                         verbose: bool = False) -> CIIntegration:
    """
    Factory function to create CI integration instance.
    
    Args:
        output_dir: Output directory for test results
        verbose: Enable verbose logging
        
    Returns:
        Configured CIIntegration instance
    """
    return CIIntegration(output_dir=output_dir, verbose=verbose)


def main():
    """Main entry point for CI integration"""
    parser = argparse.ArgumentParser(
        description="Enhanced CI/CD Integration for Automated Testing",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Run with default configuration
  python -m test_framework.ci_integration
  
  # Run with custom configuration
  python -m test_framework.ci_integration --config ci_config.json
  
  # Run with firmware flashing
  python -m test_framework.ci_integration --firmware firmware.uf2
  
  # Run with specific device count
  python -m test_framework.ci_integration --devices 2 --parallel 2
  
  # Verbose mode for debugging
  python -m test_framework.ci_integration --verbose
        """
    )
    
    parser.add_argument('--config', '-c', type=str,
                       help='CI configuration file (JSON)')
    parser.add_argument('--firmware', '-f', type=str,
                       help='Firmware file to flash before testing')
    parser.add_argument('--devices', '-d', type=int, default=1,
                       help='Minimum number of devices required')
    parser.add_argument('--parallel', '-p', type=int, default=4,
                       help='Maximum parallel operations')
    parser.add_argument('--timeout', '-t', type=float, default=300.0,
                       help='Global timeout in seconds')
    parser.add_argument('--output-dir', '-o', type=str, default='ci_test_results',
                       help='Output directory for results and artifacts')
    parser.add_argument('--verbose', '-v', action='store_true',
                       help='Enable verbose logging')
    parser.add_argument('--fail-fast', action='store_true',
                       help='Stop on first failure')
    parser.add_argument('--no-artifacts', action='store_true',
                       help='Skip artifact generation')
    
    args = parser.parse_args()
    
    # Create CI integration
    ci = CIIntegration(output_dir=args.output_dir, verbose=args.verbose)
    
    # Override configuration if command line arguments provided
    if any([args.firmware, args.devices != 1, args.parallel != 4, 
            args.timeout != 300.0, args.fail_fast, args.no_artifacts]):
        
        # Create custom configuration
        config_data = ci.get_default_ci_configuration()
        config_data['required_devices'] = args.devices
        config_data['max_parallel_devices'] = args.parallel
        config_data['timeout_seconds'] = args.timeout
        config_data['fail_fast'] = args.fail_fast
        config_data['generate_artifacts'] = not args.no_artifacts
        
        if args.firmware:
            config_data['firmware_path'] = args.firmware
        
        # Save temporary config
        temp_config = Path(args.output_dir) / "temp_ci_config.json"
        temp_config.parent.mkdir(exist_ok=True)
        with open(temp_config, 'w') as f:
            json.dump(config_data, f, indent=2)
        
        config_path = str(temp_config)
    else:
        config_path = args.config
    
    # Run CI pipeline
    result = ci.run_ci_pipeline(config_path)
    
    # Print final summary
    print(f"\n{'='*60}")
    print(f"CI PIPELINE COMPLETED")
    print(f"{'='*60}")
    print(f"Success: {result.success}")
    print(f"Exit Code: {result.exit_code}")
    print(f"Total Tests: {result.total_tests}")
    print(f"Passed: {result.passed_tests}")
    print(f"Failed: {result.failed_tests}")
    print(f"Duration: {result.duration_seconds:.1f}s")
    print(f"Devices: {len(result.devices_tested)}")
    print(f"Artifacts: {len(result.artifacts_generated)}")
    
    if result.error_summary:
        print(f"Error: {result.error_summary}")
    
    if result.artifacts_generated:
        print(f"\nGenerated Artifacts:")
        for artifact in result.artifacts_generated:
            print(f"  - {artifact}")
    
    print(f"{'='*60}")
    
    # Clean up temporary config if created
    if 'temp_config' in locals():
        try:
            temp_config.unlink()
        except:
            pass
    
    # Exit with appropriate code
    sys.exit(result.exit_code)


if __name__ == '__main__':
    main()