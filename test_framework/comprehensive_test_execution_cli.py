#!/usr/bin/env python3
"""
Comprehensive Test Execution CLI

This module provides a command-line interface for executing comprehensive tests
on the pEMF device, including individual test suites, comprehensive test runs,
and validation of all converted tests.

Requirements: 4.2, 4.3, 4.4, 5.4, 6.5
"""

import argparse
import sys
import time
import json
from typing import Dict, List, Optional, Any
from dataclasses import dataclass
from enum import Enum

from comprehensive_test_runner import ComprehensiveTestRunner
from nostd_test_integration import NoStdTestIntegration
from firmware_flasher import FirmwareFlasher


class TestExecutionMode(Enum):
    """Test execution modes"""
    INDIVIDUAL_SUITE = "suite"
    INDIVIDUAL_TEST = "test"
    COMPREHENSIVE_ALL = "all"
    VALIDATION = "validation"
    PERFORMANCE = "performance"


@dataclass
class TestExecutionConfig:
    """Configuration for test execution"""
    mode: TestExecutionMode
    suite_name: Optional[str] = None
    test_name: Optional[str] = None
    timeout_seconds: int = 300  # 5 minutes default
    stop_on_failure: bool = False
    validate_resources: bool = True
    validate_timing: bool = True
    max_memory_usage: int = 32768  # 32KB
    max_cpu_usage: int = 80  # 80%
    output_format: str = "json"  # json, text, xml
    output_file: Optional[str] = None
    verbose: bool = False
    dry_run: bool = False


@dataclass
class TestExecutionResult:
    """Result of test execution"""
    success: bool
    total_suites: int
    total_tests: int
    tests_passed: int
    tests_failed: int
    tests_skipped: int
    execution_time_ms: int
    success_rate: float
    errors: List[str]
    warnings: List[str]
    resource_usage: Dict[str, Any]
    detailed_results: Dict[str, Any]


class ComprehensiveTestExecutionCLI:
    """Command-line interface for comprehensive test execution"""
    
    def __init__(self):
        self.test_runner = ComprehensiveTestRunner()
        self.nostd_integration = NoStdTestIntegration()
        self.firmware_flasher = FirmwareFlasher()
        self.config: Optional[TestExecutionConfig] = None
        
    def parse_arguments(self) -> TestExecutionConfig:
        """Parse command-line arguments"""
        parser = argparse.ArgumentParser(
            description="Comprehensive Test Execution CLI for pEMF Device",
            formatter_class=argparse.RawDescriptionHelpFormatter,
            epilog="""
Examples:
  # Run all test suites comprehensively
  %(prog)s --mode all --timeout 600
  
  # Run specific test suite
  %(prog)s --mode suite --suite-name system_state_unit_tests
  
  # Run specific test
  %(prog)s --mode test --suite-name core_functionality --test-name test_battery_state_machine
  
  # Run comprehensive validation
  %(prog)s --mode validation --stop-on-failure
  
  # Run performance benchmarks
  %(prog)s --mode performance --validate-resources --output-format json
            """
        )
        
        # Execution mode
        parser.add_argument(
            "--mode", "-m",
            type=str,
            choices=[mode.value for mode in TestExecutionMode],
            required=True,
            help="Test execution mode"
        )
        
        # Test selection
        parser.add_argument(
            "--suite-name", "-s",
            type=str,
            help="Name of specific test suite to run (for suite/test modes)"
        )
        
        parser.add_argument(
            "--test-name", "-t",
            type=str,
            help="Name of specific test to run (for test mode)"
        )
        
        # Execution options
        parser.add_argument(
            "--timeout",
            type=int,
            default=300,
            help="Timeout in seconds for test execution (default: 300)"
        )
        
        parser.add_argument(
            "--stop-on-failure",
            action="store_true",
            help="Stop execution on first test failure"
        )
        
        # Validation options
        parser.add_argument(
            "--validate-resources",
            action="store_true",
            default=True,
            help="Validate resource usage during execution"
        )
        
        parser.add_argument(
            "--validate-timing",
            action="store_true",
            default=True,
            help="Validate timing constraints during execution"
        )
        
        parser.add_argument(
            "--max-memory-usage",
            type=int,
            default=32768,
            help="Maximum allowed memory usage in bytes (default: 32768)"
        )
        
        parser.add_argument(
            "--max-cpu-usage",
            type=int,
            default=80,
            help="Maximum allowed CPU usage percentage (default: 80)"
        )
        
        # Output options
        parser.add_argument(
            "--output-format", "-f",
            type=str,
            choices=["json", "text", "xml"],
            default="json",
            help="Output format for results (default: json)"
        )
        
        parser.add_argument(
            "--output-file", "-o",
            type=str,
            help="Output file for results (default: stdout)"
        )
        
        parser.add_argument(
            "--verbose", "-v",
            action="store_true",
            help="Enable verbose output"
        )
        
        parser.add_argument(
            "--dry-run",
            action="store_true",
            help="Show what would be executed without running tests"
        )
        
        # Device options
        parser.add_argument(
            "--device-path",
            type=str,
            help="Path to USB HID device (auto-detect if not specified)"
        )
        
        parser.add_argument(
            "--flash-firmware",
            action="store_true",
            help="Flash firmware before running tests"
        )
        
        parser.add_argument(
            "--firmware-path",
            type=str,
            help="Path to firmware file for flashing"
        )
        
        args = parser.parse_args()
        
        # Validate arguments
        if args.mode in ["suite", "test"] and not args.suite_name:
            parser.error(f"--suite-name is required for mode '{args.mode}'")
            
        if args.mode == "test" and not args.test_name:
            parser.error("--test-name is required for mode 'test'")
            
        if args.flash_firmware and not args.firmware_path:
            parser.error("--firmware-path is required when --flash-firmware is specified")
        
        # Create configuration
        return TestExecutionConfig(
            mode=TestExecutionMode(args.mode),
            suite_name=args.suite_name,
            test_name=args.test_name,
            timeout_seconds=args.timeout,
            stop_on_failure=args.stop_on_failure,
            validate_resources=args.validate_resources,
            validate_timing=args.validate_timing,
            max_memory_usage=args.max_memory_usage,
            max_cpu_usage=args.max_cpu_usage,
            output_format=args.output_format,
            output_file=args.output_file,
            verbose=args.verbose,
            dry_run=args.dry_run
        )
    
    def execute_tests(self, config: TestExecutionConfig) -> TestExecutionResult:
        """Execute tests based on configuration"""
        self.config = config
        
        if config.verbose:
            print(f"Starting test execution in mode: {config.mode.value}")
            
        if config.dry_run:
            return self._dry_run_execution(config)
        
        start_time = time.time()
        
        try:
            # Initialize test runner with configuration
            self.test_runner.configure(
                timeout_seconds=config.timeout_seconds,
                stop_on_failure=config.stop_on_failure,
                validate_resources=config.validate_resources,
                validate_timing=config.validate_timing,
                max_memory_usage=config.max_memory_usage,
                max_cpu_usage=config.max_cpu_usage
            )
            
            # Execute based on mode
            if config.mode == TestExecutionMode.COMPREHENSIVE_ALL:
                result = self._execute_comprehensive_all()
            elif config.mode == TestExecutionMode.INDIVIDUAL_SUITE:
                result = self._execute_individual_suite(config.suite_name)
            elif config.mode == TestExecutionMode.INDIVIDUAL_TEST:
                result = self._execute_individual_test(config.suite_name, config.test_name)
            elif config.mode == TestExecutionMode.VALIDATION:
                result = self._execute_validation()
            elif config.mode == TestExecutionMode.PERFORMANCE:
                result = self._execute_performance_benchmark()
            else:
                raise ValueError(f"Unsupported execution mode: {config.mode}")
            
            # Calculate execution time
            execution_time_ms = int((time.time() - start_time) * 1000)
            result.execution_time_ms = execution_time_ms
            
            if config.verbose:
                print(f"Test execution completed in {execution_time_ms}ms")
                
            return result
            
        except Exception as e:
            return TestExecutionResult(
                success=False,
                total_suites=0,
                total_tests=0,
                tests_passed=0,
                tests_failed=0,
                tests_skipped=0,
                execution_time_ms=int((time.time() - start_time) * 1000),
                success_rate=0.0,
                errors=[f"Test execution failed: {str(e)}"],
                warnings=[],
                resource_usage={},
                detailed_results={}
            )
    
    def _execute_comprehensive_all(self) -> TestExecutionResult:
        """Execute comprehensive test run of all suites"""
        if self.config.verbose:
            print("Executing comprehensive test run of all suites...")
            
        # Get all available test suites
        available_suites = self.test_runner.get_available_test_suites()
        
        if not available_suites:
            return TestExecutionResult(
                success=False,
                total_suites=0,
                total_tests=0,
                tests_passed=0,
                tests_failed=0,
                tests_skipped=0,
                execution_time_ms=0,
                success_rate=0.0,
                errors=["No test suites available"],
                warnings=[],
                resource_usage={},
                detailed_results={}
            )
        
        # Execute all suites
        results = self.test_runner.run_comprehensive_test_suite()
        
        return self._convert_runner_results(results)
    
    def _execute_individual_suite(self, suite_name: str) -> TestExecutionResult:
        """Execute specific test suite"""
        if self.config.verbose:
            print(f"Executing test suite: {suite_name}")
            
        results = self.test_runner.run_test_suite(suite_name)
        return self._convert_runner_results(results)
    
    def _execute_individual_test(self, suite_name: str, test_name: str) -> TestExecutionResult:
        """Execute specific test"""
        if self.config.verbose:
            print(f"Executing test: {suite_name}::{test_name}")
            
        results = self.test_runner.run_individual_test(suite_name, test_name)
        return self._convert_runner_results(results)
    
    def _execute_validation(self) -> TestExecutionResult:
        """Execute comprehensive validation"""
        if self.config.verbose:
            print("Executing comprehensive validation...")
            
        # Run validation to ensure all converted tests pass
        validation_results = self.test_runner.run_comprehensive_validation()
        
        return TestExecutionResult(
            success=validation_results.get("overall_success", False),
            total_suites=validation_results.get("total_suites", 0),
            total_tests=validation_results.get("total_tests", 0),
            tests_passed=validation_results.get("tests_passed", 0),
            tests_failed=validation_results.get("tests_failed", 0),
            tests_skipped=validation_results.get("tests_skipped", 0),
            execution_time_ms=validation_results.get("execution_time_ms", 0),
            success_rate=validation_results.get("success_rate", 0.0),
            errors=validation_results.get("errors", []),
            warnings=validation_results.get("warnings", []),
            resource_usage=validation_results.get("resource_usage", {}),
            detailed_results=validation_results
        )
    
    def _execute_performance_benchmark(self) -> TestExecutionResult:
        """Execute performance benchmark tests"""
        if self.config.verbose:
            print("Executing performance benchmark tests...")
            
        # Run performance-focused test suite
        results = self.test_runner.run_performance_benchmark()
        return self._convert_runner_results(results)
    
    def _convert_runner_results(self, runner_results: Dict[str, Any]) -> TestExecutionResult:
        """Convert test runner results to CLI result format"""
        return TestExecutionResult(
            success=runner_results.get("success", False),
            total_suites=runner_results.get("total_suites", 0),
            total_tests=runner_results.get("total_tests", 0),
            tests_passed=runner_results.get("tests_passed", 0),
            tests_failed=runner_results.get("tests_failed", 0),
            tests_skipped=runner_results.get("tests_skipped", 0),
            execution_time_ms=runner_results.get("execution_time_ms", 0),
            success_rate=runner_results.get("success_rate", 0.0),
            errors=runner_results.get("errors", []),
            warnings=runner_results.get("warnings", []),
            resource_usage=runner_results.get("resource_usage", {}),
            detailed_results=runner_results
        )
    
    def _dry_run_execution(self, config: TestExecutionConfig) -> TestExecutionResult:
        """Perform dry run without executing tests"""
        print("DRY RUN - No tests will be executed")
        print(f"Mode: {config.mode.value}")
        
        if config.suite_name:
            print(f"Suite: {config.suite_name}")
        if config.test_name:
            print(f"Test: {config.test_name}")
            
        print(f"Timeout: {config.timeout_seconds}s")
        print(f"Stop on failure: {config.stop_on_failure}")
        print(f"Validate resources: {config.validate_resources}")
        print(f"Validate timing: {config.validate_timing}")
        print(f"Output format: {config.output_format}")
        
        if config.output_file:
            print(f"Output file: {config.output_file}")
        
        return TestExecutionResult(
            success=True,
            total_suites=0,
            total_tests=0,
            tests_passed=0,
            tests_failed=0,
            tests_skipped=0,
            execution_time_ms=0,
            success_rate=100.0,
            errors=[],
            warnings=["Dry run - no tests executed"],
            resource_usage={},
            detailed_results={"dry_run": True}
        )
    
    def format_output(self, result: TestExecutionResult) -> str:
        """Format test execution result for output"""
        if self.config.output_format == "json":
            return self._format_json_output(result)
        elif self.config.output_format == "text":
            return self._format_text_output(result)
        elif self.config.output_format == "xml":
            return self._format_xml_output(result)
        else:
            raise ValueError(f"Unsupported output format: {self.config.output_format}")
    
    def _format_json_output(self, result: TestExecutionResult) -> str:
        """Format result as JSON"""
        output = {
            "success": result.success,
            "summary": {
                "total_suites": result.total_suites,
                "total_tests": result.total_tests,
                "tests_passed": result.tests_passed,
                "tests_failed": result.tests_failed,
                "tests_skipped": result.tests_skipped,
                "execution_time_ms": result.execution_time_ms,
                "success_rate": result.success_rate
            },
            "errors": result.errors,
            "warnings": result.warnings,
            "resource_usage": result.resource_usage,
            "detailed_results": result.detailed_results
        }
        
        return json.dumps(output, indent=2)
    
    def _format_text_output(self, result: TestExecutionResult) -> str:
        """Format result as human-readable text"""
        lines = []
        lines.append("=" * 60)
        lines.append("COMPREHENSIVE TEST EXECUTION RESULTS")
        lines.append("=" * 60)
        lines.append("")
        
        # Summary
        lines.append("SUMMARY:")
        lines.append(f"  Overall Success: {'✓' if result.success else '✗'}")
        lines.append(f"  Total Suites: {result.total_suites}")
        lines.append(f"  Total Tests: {result.total_tests}")
        lines.append(f"  Tests Passed: {result.tests_passed}")
        lines.append(f"  Tests Failed: {result.tests_failed}")
        lines.append(f"  Tests Skipped: {result.tests_skipped}")
        lines.append(f"  Execution Time: {result.execution_time_ms}ms")
        lines.append(f"  Success Rate: {result.success_rate:.1f}%")
        lines.append("")
        
        # Errors
        if result.errors:
            lines.append("ERRORS:")
            for error in result.errors:
                lines.append(f"  ✗ {error}")
            lines.append("")
        
        # Warnings
        if result.warnings:
            lines.append("WARNINGS:")
            for warning in result.warnings:
                lines.append(f"  ⚠ {warning}")
            lines.append("")
        
        # Resource usage
        if result.resource_usage:
            lines.append("RESOURCE USAGE:")
            for key, value in result.resource_usage.items():
                lines.append(f"  {key}: {value}")
            lines.append("")
        
        lines.append("=" * 60)
        
        return "\n".join(lines)
    
    def _format_xml_output(self, result: TestExecutionResult) -> str:
        """Format result as XML"""
        lines = []
        lines.append('<?xml version="1.0" encoding="UTF-8"?>')
        lines.append('<test_execution_result>')
        lines.append(f'  <success>{str(result.success).lower()}</success>')
        lines.append('  <summary>')
        lines.append(f'    <total_suites>{result.total_suites}</total_suites>')
        lines.append(f'    <total_tests>{result.total_tests}</total_tests>')
        lines.append(f'    <tests_passed>{result.tests_passed}</tests_passed>')
        lines.append(f'    <tests_failed>{result.tests_failed}</tests_failed>')
        lines.append(f'    <tests_skipped>{result.tests_skipped}</tests_skipped>')
        lines.append(f'    <execution_time_ms>{result.execution_time_ms}</execution_time_ms>')
        lines.append(f'    <success_rate>{result.success_rate}</success_rate>')
        lines.append('  </summary>')
        
        if result.errors:
            lines.append('  <errors>')
            for error in result.errors:
                lines.append(f'    <error>{error}</error>')
            lines.append('  </errors>')
        
        if result.warnings:
            lines.append('  <warnings>')
            for warning in result.warnings:
                lines.append(f'    <warning>{warning}</warning>')
            lines.append('  </warnings>')
        
        lines.append('</test_execution_result>')
        
        return "\n".join(lines)
    
    def write_output(self, formatted_output: str) -> None:
        """Write formatted output to file or stdout"""
        if self.config.output_file:
            try:
                with open(self.config.output_file, 'w') as f:
                    f.write(formatted_output)
                if self.config.verbose:
                    print(f"Results written to: {self.config.output_file}")
            except Exception as e:
                print(f"Error writing to output file: {e}", file=sys.stderr)
                print(formatted_output)
        else:
            print(formatted_output)


def main():
    """Main entry point for CLI"""
    cli = ComprehensiveTestExecutionCLI()
    
    try:
        # Parse command-line arguments
        config = cli.parse_arguments()
        
        # Execute tests
        result = cli.execute_tests(config)
        
        # Format and output results
        formatted_output = cli.format_output(result)
        cli.write_output(formatted_output)
        
        # Exit with appropriate code
        sys.exit(0 if result.success else 1)
        
    except KeyboardInterrupt:
        print("\nTest execution interrupted by user", file=sys.stderr)
        sys.exit(130)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()