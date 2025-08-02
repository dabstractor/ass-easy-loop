#!/usr/bin/env python3
"""
Comprehensive Test Runner for Hardware Validation

This script provides a comprehensive test runner that can execute all
test scenarios including hardware validation, stress testing, regression
testing, performance benchmarking, and integration testing.
"""

import argparse
import logging
import time
import sys
import os
from typing import List, Dict, Any, Optional
from pathlib import Path

from .device_manager import UsbHidDeviceManager
from .command_handler import CommandHandler
from .test_sequencer import TestSequencer
from .result_collector import ResultCollector
from .report_generator import ReportGenerator
from .test_scenarios import TestScenarios, TestScenarioType, ScenarioParameters


class ComprehensiveTestRunner:
    """
    Comprehensive test runner for executing all test scenarios.
    
    Supports individual scenario execution, full test suite execution,
    and configurable test parameters.
    """
    
    def __init__(self, output_dir: str = "test_results"):
        """
        Initialize the comprehensive test runner.
        
        Args:
            output_dir: Directory for test result outputs
        """
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(exist_ok=True)
        
        # Initialize framework components
        self.device_manager = UsbHidDeviceManager()
        self.command_handler = CommandHandler(self.device_manager)
        self.test_sequencer = TestSequencer(self.device_manager, self.command_handler)
        self.result_collector = ResultCollector(str(self.output_dir / "artifacts"))
        self.report_generator = ReportGenerator(str(self.output_dir))
        
        self.logger = logging.getLogger(__name__)
        
    def setup_logging(self, log_level: str = "INFO", log_file: Optional[str] = None):
        """
        Configure logging for the test runner.
        
        Args:
            log_level: Logging level (DEBUG, INFO, WARNING, ERROR)
            log_file: Optional log file path
        """
        log_format = '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        
        # Configure root logger
        logging.basicConfig(
            level=getattr(logging, log_level.upper()),
            format=log_format,
            handlers=[]
        )
        
        # Add console handler
        console_handler = logging.StreamHandler(sys.stdout)
        console_handler.setFormatter(logging.Formatter(log_format))
        logging.getLogger().addHandler(console_handler)
        
        # Add file handler if specified
        if log_file:
            file_handler = logging.FileHandler(log_file)
            file_handler.setFormatter(logging.Formatter(log_format))
            logging.getLogger().addHandler(file_handler)
    
    def discover_and_connect_devices(self, target_devices: Optional[List[str]] = None) -> List[str]:
        """
        Discover and connect to test devices.
        
        Args:
            target_devices: Specific device serial numbers to target
            
        Returns:
            List of connected device serial numbers
        """
        self.logger.info("Discovering devices...")
        devices = self.device_manager.discover_devices()
        
        if not devices:
            self.logger.error("No devices found. Please connect test devices.")
            return []
        
        self.logger.info(f"Found {len(devices)} device(s):")
        for device in devices:
            self.logger.info(f"  - {device.serial_number}: {device.product} ({device.status.value})")
        
        # Filter devices if specific targets specified
        if target_devices:
            devices = [d for d in devices if d.serial_number in target_devices]
            if not devices:
                self.logger.error("None of the specified target devices were found")
                return []
        
        # Connect to devices
        connected_devices = []
        for device in devices:
            if device.status.value == "connected":
                if self.device_manager.connect_device(device.serial_number):
                    connected_devices.append(device.serial_number)
                    self.logger.info(f"Connected to device: {device.serial_number}")
                else:
                    self.logger.error(f"Failed to connect to device: {device.serial_number}")
        
        return connected_devices
    
    def run_scenario(self, scenario_type: TestScenarioType, 
                    connected_devices: List[str],
                    parameters: Optional[ScenarioParameters] = None) -> Dict[str, Any]:
        """
        Run a specific test scenario.
        
        Args:
            scenario_type: Type of scenario to run
            connected_devices: List of connected device serial numbers
            parameters: Optional scenario parameters
            
        Returns:
            Test execution results
        """
        self.logger.info(f"Running {scenario_type.value} scenario...")
        
        # Create test scenarios with parameters
        test_scenarios = TestScenarios(parameters)
        test_config = test_scenarios.get_scenario_by_type(scenario_type)
        
        # Execute test sequence
        start_time = time.time()
        execution_results = self.test_sequencer.execute_test_sequence(
            test_config,
            target_devices=connected_devices
        )
        end_time = time.time()
        
        # Collect results
        suite_result = self.result_collector.collect_results(
            test_config.name,
            test_config.description,
            execution_results,
            start_time,
            end_time
        )
        
        # Generate comprehensive reports using the new report generator
        report_files = self.report_generator.generate_comprehensive_report(
            suite_result,
            formats=['html', 'json', 'junit', 'csv']
        )
        
        self.logger.info(f"Generated comprehensive reports:")
        for format_name, filepath in report_files.items():
            self.logger.info(f"  - {format_name}: {filepath}")
        
        return {
            'scenario_type': scenario_type.value,
            'suite_result': suite_result,
            'execution_results': execution_results
        }
    
    def run_full_test_suite(self, connected_devices: List[str],
                           parameters: Optional[ScenarioParameters] = None,
                           skip_scenarios: Optional[List[TestScenarioType]] = None) -> Dict[str, Any]:
        """
        Run the complete test suite with all scenarios.
        
        Args:
            connected_devices: List of connected device serial numbers
            parameters: Optional scenario parameters
            skip_scenarios: Scenarios to skip
            
        Returns:
            Complete test suite results
        """
        self.logger.info("Starting full test suite execution...")
        
        skip_scenarios = skip_scenarios or []
        all_scenarios = [
            TestScenarioType.HARDWARE_VALIDATION,
            TestScenarioType.REGRESSION_TESTING,
            TestScenarioType.PERFORMANCE_BENCHMARKING,
            TestScenarioType.INTEGRATION_TESTING,
            TestScenarioType.STRESS_TESTING  # Run stress testing last
        ]
        
        suite_start_time = time.time()
        scenario_results = {}
        overall_success = True
        
        for scenario_type in all_scenarios:
            if scenario_type in skip_scenarios:
                self.logger.info(f"Skipping {scenario_type.value} scenario")
                continue
            
            try:
                result = self.run_scenario(scenario_type, connected_devices, parameters)
                scenario_results[scenario_type.value] = result
                
                # Check if scenario passed
                suite_result = result['suite_result']
                if suite_result.aggregate_metrics.failed_tests > 0:
                    overall_success = False
                    self.logger.warning(f"Scenario {scenario_type.value} had failures")
                
            except Exception as e:
                self.logger.error(f"Scenario {scenario_type.value} failed with exception: {e}")
                overall_success = False
                scenario_results[scenario_type.value] = {
                    'error': str(e),
                    'scenario_type': scenario_type.value
                }
        
        suite_end_time = time.time()
        
        # Generate comprehensive report
        full_suite_result = {
            'overall_success': overall_success,
            'total_duration': suite_end_time - suite_start_time,
            'start_time': suite_start_time,
            'end_time': suite_end_time,
            'scenario_results': scenario_results,
            'device_count': len(connected_devices),
            'devices_tested': connected_devices
        }
        
        self._generate_full_suite_report(full_suite_result)
        
        return full_suite_result
    
    def run_custom_scenario(self, scenario_name: str, test_steps: List[Dict[str, Any]],
                           connected_devices: List[str]) -> Dict[str, Any]:
        """
        Run a custom test scenario defined by the user.
        
        Args:
            scenario_name: Name for the custom scenario
            test_steps: List of test step definitions
            connected_devices: List of connected device serial numbers
            
        Returns:
            Custom scenario execution results
        """
        from .test_sequencer import TestStep
        from .command_handler import TestType
        
        self.logger.info(f"Running custom scenario: {scenario_name}")
        
        # Convert test step definitions to TestStep objects
        steps = []
        for step_def in test_steps:
            step = TestStep(
                name=step_def['name'],
                test_type=TestType(step_def['test_type']),
                parameters=step_def.get('parameters', {}),
                timeout=step_def.get('timeout', 30.0),
                retry_count=step_def.get('retry_count', 0),
                required=step_def.get('required', True),
                depends_on=step_def.get('depends_on', [])
            )
            steps.append(step)
        
        # Create custom scenario
        test_scenarios = TestScenarios()
        test_config = test_scenarios.create_custom_scenario(
            scenario_name,
            f"Custom test scenario: {scenario_name}",
            steps
        )
        
        # Execute test sequence
        start_time = time.time()
        execution_results = self.test_sequencer.execute_test_sequence(
            test_config,
            target_devices=connected_devices
        )
        end_time = time.time()
        
        # Collect results
        suite_result = self.result_collector.collect_results(
            test_config.name,
            test_config.description,
            execution_results,
            start_time,
            end_time
        )
        
        # Generate reports
        self._generate_custom_scenario_report(scenario_name, suite_result)
        
        return {
            'scenario_name': scenario_name,
            'suite_result': suite_result,
            'execution_results': execution_results
        }
    
    def _generate_scenario_reports(self, scenario_type: TestScenarioType, suite_result):
        """Generate reports for a specific scenario"""
        scenario_name = scenario_type.value
        timestamp = time.strftime("%Y%m%d_%H%M%S")
        
        # JSON report
        json_report = self.result_collector.export_json(suite_result, detailed=True)
        json_file = self.output_dir / f"{scenario_name}_{timestamp}.json"
        with open(json_file, "w") as f:
            f.write(json_report)
        self.logger.info(f"JSON report saved: {json_file}")
        
        # JUnit XML report
        junit_xml = self.result_collector.export_junit_xml(suite_result)
        xml_file = self.output_dir / f"{scenario_name}_{timestamp}.xml"
        with open(xml_file, "w") as f:
            f.write(junit_xml)
        self.logger.info(f"JUnit XML report saved: {xml_file}")
        
        # Summary report
        summary = self.result_collector.generate_summary_report(suite_result)
        summary_file = self.output_dir / f"{scenario_name}_{timestamp}_summary.json"
        with open(summary_file, "w") as f:
            import json
            json.dump(summary, f, indent=2, default=str)
        self.logger.info(f"Summary report saved: {summary_file}")
    
    def _generate_custom_scenario_report(self, scenario_name: str, suite_result):
        """Generate reports for a custom scenario"""
        timestamp = time.strftime("%Y%m%d_%H%M%S")
        safe_name = scenario_name.replace(" ", "_").lower()
        
        # JSON report
        json_report = self.result_collector.export_json(suite_result, detailed=True)
        json_file = self.output_dir / f"custom_{safe_name}_{timestamp}.json"
        with open(json_file, "w") as f:
            f.write(json_report)
        self.logger.info(f"Custom scenario JSON report saved: {json_file}")
    
    def _generate_full_suite_report(self, full_suite_result: Dict[str, Any]):
        """Generate comprehensive report for full test suite"""
        timestamp = time.strftime("%Y%m%d_%H%M%S")
        
        # Full suite summary
        suite_file = self.output_dir / f"full_suite_{timestamp}.json"
        with open(suite_file, "w") as f:
            import json
            json.dump(full_suite_result, f, indent=2, default=str)
        self.logger.info(f"Full suite report saved: {suite_file}")
        
        # Generate HTML summary report
        self._generate_html_summary(full_suite_result, timestamp)
    
    def _generate_html_summary(self, full_suite_result: Dict[str, Any], timestamp: str):
        """Generate HTML summary report"""
        html_content = f"""
<!DOCTYPE html>
<html>
<head>
    <title>Comprehensive Test Suite Results</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .header {{ background-color: #f0f0f0; padding: 20px; border-radius: 5px; }}
        .scenario {{ margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }}
        .success {{ background-color: #d4edda; border-color: #c3e6cb; }}
        .failure {{ background-color: #f8d7da; border-color: #f5c6cb; }}
        .metrics {{ display: flex; gap: 20px; margin: 10px 0; }}
        .metric {{ padding: 10px; background-color: #f8f9fa; border-radius: 3px; }}
        table {{ width: 100%; border-collapse: collapse; margin: 10px 0; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Comprehensive Test Suite Results</h1>
        <p><strong>Execution Time:</strong> {time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(full_suite_result['start_time']))}</p>
        <p><strong>Total Duration:</strong> {full_suite_result['total_duration']:.2f} seconds</p>
        <p><strong>Overall Status:</strong> {'PASS' if full_suite_result['overall_success'] else 'FAIL'}</p>
        <p><strong>Devices Tested:</strong> {', '.join(full_suite_result['devices_tested'])}</p>
    </div>
"""
        
        # Add scenario results
        for scenario_name, result in full_suite_result['scenario_results'].items():
            if 'error' in result:
                html_content += f"""
    <div class="scenario failure">
        <h2>{scenario_name.replace('_', ' ').title()}</h2>
        <p><strong>Status:</strong> ERROR</p>
        <p><strong>Error:</strong> {result['error']}</p>
    </div>
"""
            else:
                suite_result = result['suite_result']
                metrics = suite_result.aggregate_metrics
                status_class = "success" if metrics.failed_tests == 0 else "failure"
                
                html_content += f"""
    <div class="scenario {status_class}">
        <h2>{scenario_name.replace('_', ' ').title()}</h2>
        <div class="metrics">
            <div class="metric">
                <strong>Total Tests:</strong> {metrics.total_tests}
            </div>
            <div class="metric">
                <strong>Passed:</strong> {metrics.passed_tests}
            </div>
            <div class="metric">
                <strong>Failed:</strong> {metrics.failed_tests}
            </div>
            <div class="metric">
                <strong>Success Rate:</strong> {metrics.success_rate:.1f}%
            </div>
            <div class="metric">
                <strong>Duration:</strong> {suite_result.duration:.2f}s
            </div>
        </div>
    </div>
"""
        
        html_content += """
</body>
</html>
"""
        
        html_file = self.output_dir / f"test_suite_summary_{timestamp}.html"
        with open(html_file, "w") as f:
            f.write(html_content)
        self.logger.info(f"HTML summary report saved: {html_file}")
    
    def cleanup(self):
        """Cleanup resources"""
        self.logger.info("Cleaning up...")
        self.device_manager.disconnect_all()


def main():
    """Main entry point for the comprehensive test runner"""
    parser = argparse.ArgumentParser(description="Comprehensive Test Runner for Hardware Validation")
    
    parser.add_argument("--scenario", choices=[t.value for t in TestScenarioType],
                       help="Run specific test scenario")
    parser.add_argument("--full-suite", action="store_true",
                       help="Run complete test suite")
    parser.add_argument("--devices", nargs="+",
                       help="Target specific device serial numbers")
    parser.add_argument("--output-dir", default="test_results",
                       help="Output directory for test results")
    parser.add_argument("--log-level", default="INFO",
                       choices=["DEBUG", "INFO", "WARNING", "ERROR"],
                       help="Logging level")
    parser.add_argument("--log-file", help="Log file path")
    parser.add_argument("--skip-scenarios", nargs="+",
                       choices=[t.value for t in TestScenarioType],
                       help="Scenarios to skip in full suite")
    
    # Scenario parameters
    parser.add_argument("--pemf-duration", type=int, default=5000,
                       help="pEMF test duration in milliseconds")
    parser.add_argument("--stress-duration", type=int, default=30000,
                       help="Stress test duration in milliseconds")
    parser.add_argument("--stress-load", type=int, default=80,
                       help="Stress test load level (0-100)")
    
    args = parser.parse_args()
    
    # Create test runner
    runner = ComprehensiveTestRunner(args.output_dir)
    runner.setup_logging(args.log_level, args.log_file)
    
    try:
        # Discover and connect to devices
        connected_devices = runner.discover_and_connect_devices(args.devices)
        if not connected_devices:
            return 1
        
        # Create scenario parameters
        parameters = ScenarioParameters(
            pemf_test_duration_ms=args.pemf_duration,
            stress_test_duration_ms=args.stress_duration,
            stress_load_level=args.stress_load
        )
        
        # Run tests based on arguments
        if args.scenario:
            scenario_type = TestScenarioType(args.scenario)
            result = runner.run_scenario(scenario_type, connected_devices, parameters)
            success = result['suite_result'].aggregate_metrics.failed_tests == 0
        elif args.full_suite:
            skip_scenarios = [TestScenarioType(s) for s in (args.skip_scenarios or [])]
            result = runner.run_full_test_suite(connected_devices, parameters, skip_scenarios)
            success = result['overall_success']
        else:
            # Default to hardware validation
            scenario_type = TestScenarioType.HARDWARE_VALIDATION
            result = runner.run_scenario(scenario_type, connected_devices, parameters)
            success = result['suite_result'].aggregate_metrics.failed_tests == 0
        
        return 0 if success else 1
        
    except KeyboardInterrupt:
        runner.logger.info("Test execution interrupted by user")
        return 1
    except Exception as e:
        runner.logger.error(f"Test execution failed: {e}")
        return 1
    finally:
        runner.cleanup()


if __name__ == "__main__":
    sys.exit(main())