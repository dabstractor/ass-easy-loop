#!/usr/bin/env python3
"""
CI/CD Integration Example

Demonstrates automated testing framework integration with CI/CD pipelines:
- Headless operation with proper exit codes
- Parallel testing with multiple devices
- Standard test result formats (JUnit XML, JSON)
- Automated device setup and cleanup
- Comprehensive logging and error reporting

This example shows how to integrate the testing framework into automated
build and deployment processes.
"""

import sys
import os
import json
import time
import logging
import argparse
from pathlib import Path
from typing import Dict, List, Optional
import xml.etree.ElementTree as ET
from datetime import datetime

from test_framework import (
    UsbHidDeviceManager, CommandHandler, TestSequencer, FirmwareFlasher,
    TestConfiguration, TestStep, TestType, FlashResult
)


class CITestRunner:
    """
    CI/CD integration test runner with headless operation support.
    
    Provides automated device discovery, test execution, result collection,
    and reporting in formats suitable for CI/CD systems.
    """
    
    def __init__(self, config_file: Optional[str] = None, verbose: bool = False):
        """
        Initialize CI test runner.
        
        Args:
            config_file: Path to test configuration file
            verbose: Enable verbose logging
        """
        self.setup_logging(verbose)
        self.logger = logging.getLogger(__name__)
        
        # Initialize framework components
        self.device_manager = UsbHidDeviceManager()
        self.command_handler = CommandHandler(self.device_manager)
        self.test_sequencer = TestSequencer(self.device_manager, self.command_handler)
        self.firmware_flasher = FirmwareFlasher(self.device_manager, self.command_handler)
        
        # Load test configuration
        self.test_config = self.load_test_configuration(config_file)
        
        # Results tracking
        self.test_results = {}
        self.flash_results = {}
        self.start_time = None
        self.end_time = None
        
    def setup_logging(self, verbose: bool):
        """Set up logging for CI environment"""
        level = logging.DEBUG if verbose else logging.INFO
        
        # Create logs directory
        os.makedirs('test_logs', exist_ok=True)
        
        # Configure logging
        logging.basicConfig(
            level=level,
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
            handlers=[
                logging.StreamHandler(sys.stdout),
                logging.FileHandler(f'test_logs/ci_test_{datetime.now().strftime("%Y%m%d_%H%M%S")}.log')
            ]
        )
        
    def load_test_configuration(self, config_file: Optional[str]) -> TestConfiguration:
        """Load test configuration from file or use default"""
        if config_file and Path(config_file).exists():
            try:
                with open(config_file, 'r') as f:
                    config_data = json.load(f)
                return self.parse_test_configuration(config_data)
            except Exception as e:
                self.logger.error(f"Failed to load config file {config_file}: {e}")
                self.logger.info("Using default test configuration")
        
        # Return default CI test configuration
        return self.create_ci_test_configuration()
    
    def create_ci_test_configuration(self) -> TestConfiguration:
        """Create default CI test configuration"""
        return TestConfiguration(
            name="CI Validation Suite",
            description="Comprehensive validation for CI/CD pipeline",
            steps=[
                TestStep(
                    name="device_communication_test",
                    test_type=TestType.USB_COMMUNICATION_TEST,
                    parameters={"message_count": 5, "timeout_ms": 1000},
                    timeout=10.0,
                    required=True
                ),
                TestStep(
                    name="pemf_timing_validation",
                    test_type=TestType.PEMF_TIMING_VALIDATION,
                    parameters={"duration_ms": 3000, "tolerance_percent": 1.0},
                    timeout=15.0,
                    required=True,
                    depends_on=["device_communication_test"]
                ),
                TestStep(
                    name="battery_monitoring_test",
                    test_type=TestType.BATTERY_ADC_CALIBRATION,
                    parameters={"reference_voltage": 3.3},
                    timeout=10.0,
                    required=True,
                    depends_on=["device_communication_test"]
                ),
                TestStep(
                    name="led_control_test",
                    test_type=TestType.LED_FUNCTIONALITY,
                    parameters={"pattern": "basic", "duration_ms": 1000},
                    timeout=8.0,
                    required=False,
                    depends_on=["device_communication_test"]
                ),
                TestStep(
                    name="system_stress_test",
                    test_type=TestType.SYSTEM_STRESS_TEST,
                    parameters={"duration_ms": 5000, "load_level": 3},
                    timeout=20.0,
                    required=False,
                    depends_on=["pemf_timing_validation", "battery_monitoring_test"]
                )
            ],
            parallel_execution=True,
            max_parallel_devices=8,
            global_timeout=120.0
        )
    
    def parse_test_configuration(self, config_data: dict) -> TestConfiguration:
        """Parse test configuration from JSON data"""
        steps = []
        for step_data in config_data.get('steps', []):
            steps.append(TestStep(
                name=step_data['name'],
                test_type=TestType(step_data['test_type']),
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
    
    def discover_and_setup_devices(self, required_devices: int = 1) -> List[str]:
        """Discover and set up test devices"""
        self.logger.info("Discovering test devices...")
        
        # Discover devices with retry
        max_attempts = 3
        connected_devices = []
        
        for attempt in range(max_attempts):
            devices = self.device_manager.discover_devices()
            
            # Filter for connected devices and attempt connection
            for device in devices:
                if device.status.value == "connected":
                    if self.device_manager.connect_device(device.serial_number):
                        connected_devices.append(device.serial_number)
                        self.logger.info(f"Connected to device: {device.serial_number}")
            
            if len(connected_devices) >= required_devices:
                break
                
            if attempt < max_attempts - 1:
                self.logger.warning(f"Only found {len(connected_devices)} devices, retrying...")
                time.sleep(2.0)
        
        if len(connected_devices) < required_devices:
            self.logger.error(f"Insufficient devices: found {len(connected_devices)}, required {required_devices}")
            return []
        
        self.logger.info(f"Successfully set up {len(connected_devices)} devices")
        return connected_devices
    
    def flash_firmware_if_needed(self, devices: List[str], firmware_path: Optional[str]) -> bool:
        """Flash firmware to devices if firmware path provided"""
        if not firmware_path:
            self.logger.info("No firmware specified, skipping firmware flash")
            return True
        
        if not Path(firmware_path).exists():
            self.logger.error(f"Firmware file not found: {firmware_path}")
            return False
        
        self.logger.info(f"Flashing firmware to {len(devices)} devices...")
        
        # Create device-firmware mapping
        device_firmware_map = {device: firmware_path for device in devices}
        
        # Flash firmware in parallel
        self.flash_results = self.firmware_flasher.flash_multiple_devices(
            device_firmware_map,
            parallel=True,
            max_parallel=4
        )
        
        # Check results
        success_count = 0
        for device, operation in self.flash_results.items():
            if operation.result == FlashResult.SUCCESS:
                success_count += 1
                self.logger.info(f"Firmware flash successful: {device}")
            else:
                self.logger.error(f"Firmware flash failed: {device} - {operation.error_message}")
        
        if success_count == len(devices):
            self.logger.info("All devices flashed successfully")
            return True
        else:
            self.logger.error(f"Firmware flash failed on {len(devices) - success_count} devices")
            return False
    
    def run_tests(self, devices: List[str], timeout: Optional[float] = None) -> bool:
        """Run test suite on devices"""
        self.logger.info(f"Running test suite on {len(devices)} devices...")
        self.start_time = time.time()
        
        try:
            # Execute test sequence
            self.test_results = self.test_sequencer.execute_test_sequence(
                self.test_config,
                target_devices=devices,
                global_timeout=timeout
            )
            
            self.end_time = time.time()
            
            # Analyze results
            return self.analyze_test_results()
            
        except Exception as e:
            self.logger.error(f"Test execution failed: {e}")
            self.end_time = time.time()
            return False
    
    def analyze_test_results(self) -> bool:
        """Analyze test results and determine overall success"""
        total_tests = 0
        passed_tests = 0
        failed_tests = 0
        timeout_tests = 0
        
        for device_serial, executions in self.test_results.items():
            self.logger.info(f"\nResults for device {device_serial}:")
            
            for execution in executions:
                total_tests += 1
                
                if execution.status.value == "completed":
                    passed_tests += 1
                    self.logger.info(f"  ✓ {execution.step.name}: PASSED ({execution.duration:.2f}s)")
                elif execution.status.value == "failed":
                    failed_tests += 1
                    self.logger.error(f"  ✗ {execution.step.name}: FAILED - {execution.error_message}")
                elif execution.status.value == "timeout":
                    timeout_tests += 1
                    self.logger.error(f"  ⏱ {execution.step.name}: TIMEOUT")
                else:
                    self.logger.warning(f"  - {execution.step.name}: {execution.status.value.upper()}")
        
        # Summary
        duration = self.end_time - self.start_time if self.start_time and self.end_time else 0
        self.logger.info(f"\n=== Test Summary ===")
        self.logger.info(f"Total tests: {total_tests}")
        self.logger.info(f"Passed: {passed_tests}")
        self.logger.info(f"Failed: {failed_tests}")
        self.logger.info(f"Timeout: {timeout_tests}")
        self.logger.info(f"Duration: {duration:.2f}s")
        
        # Determine overall success
        success = failed_tests == 0 and timeout_tests == 0
        self.logger.info(f"Overall result: {'PASSED' if success else 'FAILED'}")
        
        return success
    
    def generate_junit_xml(self, output_path: str):
        """Generate JUnit XML test report"""
        root = ET.Element("testsuites")
        root.set("name", self.test_config.name)
        root.set("tests", str(sum(len(executions) for executions in self.test_results.values())))
        
        total_failures = 0
        total_time = 0
        
        for device_serial, executions in self.test_results.items():
            testsuite = ET.SubElement(root, "testsuite")
            testsuite.set("name", f"Device_{device_serial}")
            testsuite.set("tests", str(len(executions)))
            
            suite_failures = 0
            suite_time = 0
            
            for execution in executions:
                testcase = ET.SubElement(testsuite, "testcase")
                testcase.set("classname", f"Device_{device_serial}")
                testcase.set("name", execution.step.name)
                
                if execution.duration:
                    testcase.set("time", f"{execution.duration:.3f}")
                    suite_time += execution.duration
                
                if execution.status.value == "failed":
                    failure = ET.SubElement(testcase, "failure")
                    failure.set("message", execution.error_message or "Test failed")
                    failure.text = execution.error_message or "Test failed"
                    suite_failures += 1
                elif execution.status.value == "timeout":
                    failure = ET.SubElement(testcase, "failure")
                    failure.set("message", "Test timeout")
                    failure.text = "Test execution timed out"
                    suite_failures += 1
                elif execution.status.value == "skipped":
                    ET.SubElement(testcase, "skipped")
            
            testsuite.set("failures", str(suite_failures))
            testsuite.set("time", f"{suite_time:.3f}")
            total_failures += suite_failures
            total_time += suite_time
        
        root.set("failures", str(total_failures))
        root.set("time", f"{total_time:.3f}")
        
        # Write XML file
        tree = ET.ElementTree(root)
        tree.write(output_path, encoding='utf-8', xml_declaration=True)
        self.logger.info(f"JUnit XML report written to: {output_path}")
    
    def generate_json_report(self, output_path: str):
        """Generate JSON test report"""
        report = {
            "test_suite": {
                "name": self.test_config.name,
                "description": self.test_config.description,
                "start_time": self.start_time,
                "end_time": self.end_time,
                "duration": self.end_time - self.start_time if self.start_time and self.end_time else 0
            },
            "devices": {},
            "summary": {
                "total_tests": 0,
                "passed": 0,
                "failed": 0,
                "timeout": 0,
                "skipped": 0
            }
        }
        
        for device_serial, executions in self.test_results.items():
            device_report = {
                "serial_number": device_serial,
                "tests": []
            }
            
            for execution in executions:
                test_report = {
                    "name": execution.step.name,
                    "status": execution.status.value,
                    "start_time": execution.start_time,
                    "end_time": execution.end_time,
                    "duration": execution.duration,
                    "error_message": execution.error_message,
                    "retry_attempt": execution.retry_attempt
                }
                
                if execution.response:
                    test_report["response"] = {
                        "status": execution.response.status.name,
                        "data": execution.response.data
                    }
                
                device_report["tests"].append(test_report)
                
                # Update summary
                report["summary"]["total_tests"] += 1
                if execution.status.value == "completed":
                    report["summary"]["passed"] += 1
                elif execution.status.value == "failed":
                    report["summary"]["failed"] += 1
                elif execution.status.value == "timeout":
                    report["summary"]["timeout"] += 1
                elif execution.status.value == "skipped":
                    report["summary"]["skipped"] += 1
            
            report["devices"][device_serial] = device_report
        
        # Add firmware flash results if available
        if self.flash_results:
            report["firmware_flash"] = {}
            for device_serial, operation in self.flash_results.items():
                report["firmware_flash"][device_serial] = {
                    "result": operation.result.value,
                    "error_message": operation.error_message,
                    "total_duration": operation.total_duration,
                    "bootloader_entry_time": operation.bootloader_entry_time,
                    "flash_duration": operation.flash_duration,
                    "reconnection_time": operation.reconnection_time
                }
        
        # Write JSON file
        with open(output_path, 'w') as f:
            json.dump(report, f, indent=2, default=str)
        
        self.logger.info(f"JSON report written to: {output_path}")
    
    def cleanup(self):
        """Clean up resources"""
        self.logger.info("Cleaning up...")
        self.device_manager.disconnect_all()


def main():
    """Main CI integration entry point"""
    parser = argparse.ArgumentParser(description="CI/CD Integration Test Runner")
    parser.add_argument('--config', '-c', type=str,
                       help='Test configuration file (JSON)')
    parser.add_argument('--firmware', '-f', type=str,
                       help='Firmware file to flash before testing')
    parser.add_argument('--devices', '-d', type=int, default=1,
                       help='Minimum number of devices required')
    parser.add_argument('--timeout', '-t', type=float,
                       help='Global test timeout in seconds')
    parser.add_argument('--output-dir', '-o', type=str, default='test_results',
                       help='Output directory for test reports')
    parser.add_argument('--junit-xml', action='store_true',
                       help='Generate JUnit XML report')
    parser.add_argument('--json-report', action='store_true',
                       help='Generate JSON report')
    parser.add_argument('--verbose', '-v', action='store_true',
                       help='Enable verbose logging')
    
    args = parser.parse_args()
    
    # Create output directory
    os.makedirs(args.output_dir, exist_ok=True)
    
    # Initialize test runner
    runner = CITestRunner(args.config, args.verbose)
    
    try:
        # Discover and set up devices
        devices = runner.discover_and_setup_devices(args.devices)
        if not devices:
            print("ERROR: Failed to discover required devices", file=sys.stderr)
            return 1
        
        # Flash firmware if provided
        if args.firmware:
            if not runner.flash_firmware_if_needed(devices, args.firmware):
                print("ERROR: Firmware flashing failed", file=sys.stderr)
                return 1
        
        # Run tests
        success = runner.run_tests(devices, args.timeout)
        
        # Generate reports
        if args.junit_xml:
            junit_path = os.path.join(args.output_dir, 'test_results.xml')
            runner.generate_junit_xml(junit_path)
        
        if args.json_report:
            json_path = os.path.join(args.output_dir, 'test_results.json')
            runner.generate_json_report(json_path)
        
        # Return appropriate exit code
        return 0 if success else 1
        
    except KeyboardInterrupt:
        print("Test execution cancelled by user", file=sys.stderr)
        return 130  # Standard exit code for SIGINT
    except Exception as e:
        print(f"Unexpected error: {e}", file=sys.stderr)
        logging.exception("Unexpected error occurred")
        return 1
    finally:
        runner.cleanup()


if __name__ == '__main__':
    sys.exit(main())