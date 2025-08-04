#!/usr/bin/env python3
"""
Comprehensive Test Example

This script demonstrates how to use all the comprehensive test scenarios
including hardware validation, stress testing, regression testing,
performance benchmarking, and integration testing.
"""

import logging
import time
import sys
import os
from pathlib import Path

# Add the parent directory to the Python path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from test_framework import (
    UsbHidDeviceManager, CommandHandler, TestSequencer, ResultCollector
)
from test_framework.test_scenarios import TestScenarios, TestScenarioType, ScenarioParameters
from test_framework.comprehensive_test_runner import ComprehensiveTestRunner
from test_framework.config_loader import ConfigurationLoader


def setup_logging():
    """Configure logging for the example"""
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
        handlers=[
            logging.StreamHandler(sys.stdout),
            logging.FileHandler('comprehensive_test_example.log')
        ]
    )


def demonstrate_individual_scenarios():
    """Demonstrate running individual test scenarios"""
    logger = logging.getLogger(__name__)
    logger.info("=== Demonstrating Individual Test Scenarios ===")
    
    # Load configuration
    config_loader = ConfigurationLoader()
    config_loader.load_configuration()
    
    # Initialize test runner
    runner = ComprehensiveTestRunner("example_results")
    runner.setup_logging("INFO", "scenario_demo.log")
    
    try:
        # Discover and connect to devices
        connected_devices = runner.discover_and_connect_devices()
        if not connected_devices:
            logger.error("No devices available for testing")
            return False
        
        logger.info(f"Testing with devices: {connected_devices}")
        
        # Demonstrate each scenario type
        scenarios_to_demo = [
            (TestScenarioType.HARDWARE_VALIDATION, "Basic hardware functionality validation"),
            (TestScenarioType.REGRESSION_TESTING, "Firmware regression validation"),
            (TestScenarioType.PERFORMANCE_BENCHMARKING, "Performance measurement and analysis"),
            (TestScenarioType.INTEGRATION_TESTING, "End-to-end system integration"),
            (TestScenarioType.STRESS_TESTING, "System stress and stability testing")
        ]
        
        for scenario_type, description in scenarios_to_demo:
            logger.info(f"\n--- Running {scenario_type.value.replace('_', ' ').title()} ---")
            logger.info(f"Description: {description}")
            
            # Get scenario-specific parameters
            parameters = config_loader.get_scenario_parameters(scenario_type.value)
            
            try:
                result = runner.run_scenario(scenario_type, connected_devices, parameters)
                
                # Display results
                suite_result = result['suite_result']
                metrics = suite_result.aggregate_metrics
                
                logger.info(f"Scenario Results:")
                logger.info(f"  Duration: {suite_result.duration:.2f} seconds")
                logger.info(f"  Total tests: {metrics.total_tests}")
                logger.info(f"  Passed: {metrics.passed_tests}")
                logger.info(f"  Failed: {metrics.failed_tests}")
                logger.info(f"  Success rate: {metrics.success_rate:.1f}%")
                
                if metrics.failed_tests > 0:
                    logger.warning(f"Scenario {scenario_type.value} had failures")
                    
                    # Show failure details
                    for device_serial, device_result in suite_result.device_results.items():
                        failed_tests = [e for e in device_result.executions 
                                      if e.status.value == "failed"]
                        if failed_tests:
                            logger.warning(f"  Device {device_serial} failures:")
                            for execution in failed_tests:
                                logger.warning(f"    - {execution.step.name}: {execution.error_message}")
                
            except Exception as e:
                logger.error(f"Scenario {scenario_type.value} failed: {e}")
        
        return True
        
    except Exception as e:
        logger.error(f"Demo failed: {e}")
        return False
    finally:
        runner.cleanup()


def demonstrate_full_test_suite():
    """Demonstrate running the complete test suite"""
    logger = logging.getLogger(__name__)
    logger.info("\n=== Demonstrating Full Test Suite ===")
    
    # Initialize test runner
    runner = ComprehensiveTestRunner("full_suite_results")
    runner.setup_logging("INFO", "full_suite_demo.log")
    
    try:
        # Discover and connect to devices
        connected_devices = runner.discover_and_connect_devices()
        if not connected_devices:
            logger.error("No devices available for testing")
            return False
        
        logger.info(f"Running full test suite on devices: {connected_devices}")
        
        # Load configuration with custom parameters
        config_loader = ConfigurationLoader()
        config_loader.load_configuration()
        
        # Create custom parameters for demo (shorter durations)
        demo_parameters = ScenarioParameters(
            pemf_test_duration_ms=2000,  # Shorter for demo
            stress_test_duration_ms=5000,  # Shorter for demo
            benchmark_iterations=20  # Fewer iterations for demo
        )
        
        # Run full test suite
        start_time = time.time()
        result = runner.run_full_test_suite(
            connected_devices, 
            demo_parameters,
            skip_scenarios=[]  # Run all scenarios
        )
        end_time = time.time()
        
        # Display comprehensive results
        logger.info(f"\nFull Test Suite Results:")
        logger.info(f"  Overall Success: {'PASS' if result['overall_success'] else 'FAIL'}")
        logger.info(f"  Total Duration: {result['total_duration']:.2f} seconds")
        logger.info(f"  Devices Tested: {len(result['devices_tested'])}")
        
        # Show results for each scenario
        for scenario_name, scenario_result in result['scenario_results'].items():
            if 'error' in scenario_result:
                logger.error(f"  {scenario_name}: ERROR - {scenario_result['error']}")
            else:
                suite_result = scenario_result['suite_result']
                metrics = suite_result.aggregate_metrics
                status = "PASS" if metrics.failed_tests == 0 else "FAIL"
                logger.info(f"  {scenario_name}: {status} "
                           f"({metrics.passed_tests}/{metrics.total_tests} passed, "
                           f"{suite_result.duration:.1f}s)")
        
        return result['overall_success']
        
    except Exception as e:
        logger.error(f"Full suite demo failed: {e}")
        return False
    finally:
        runner.cleanup()


def demonstrate_custom_scenario():
    """Demonstrate creating and running a custom test scenario"""
    logger = logging.getLogger(__name__)
    logger.info("\n=== Demonstrating Custom Test Scenario ===")
    
    # Initialize test runner
    runner = ComprehensiveTestRunner("custom_scenario_results")
    runner.setup_logging("INFO", "custom_scenario_demo.log")
    
    try:
        # Discover and connect to devices
        connected_devices = runner.discover_and_connect_devices()
        if not connected_devices:
            logger.error("No devices available for testing")
            return False
        
        # Define custom test scenario
        custom_test_steps = [
            {
                'name': 'quick_communication_check',
                'test_type': 5,  # USB_COMMUNICATION_TEST
                'parameters': {'message_count': 3, 'timeout_ms': 500},
                'timeout': 5.0,
                'required': True
            },
            {
                'name': 'basic_pemf_test',
                'test_type': 1,  # PEMF_TIMING_VALIDATION
                'parameters': {'duration_ms': 1000, 'tolerance_percent': 2.0},
                'timeout': 10.0,
                'required': True,
                'depends_on': ['quick_communication_check']
            },
            {
                'name': 'led_pattern_test',
                'test_type': 3,  # LED_FUNCTIONALITY
                'parameters': {'pattern': 'flash', 'duration_ms': 500},
                'timeout': 5.0,
                'required': False,
                'depends_on': ['quick_communication_check']
            }
        ]
        
        logger.info("Running custom scenario: 'Quick Device Validation'")
        
        result = runner.run_custom_scenario(
            "Quick Device Validation",
            custom_test_steps,
            connected_devices
        )
        
        # Display results
        suite_result = result['suite_result']
        metrics = suite_result.aggregate_metrics
        
        logger.info(f"Custom Scenario Results:")
        logger.info(f"  Duration: {suite_result.duration:.2f} seconds")
        logger.info(f"  Total tests: {metrics.total_tests}")
        logger.info(f"  Passed: {metrics.passed_tests}")
        logger.info(f"  Failed: {metrics.failed_tests}")
        logger.info(f"  Success rate: {metrics.success_rate:.1f}%")
        
        # Show detailed test results
        for device_serial, device_result in suite_result.device_results.items():
            logger.info(f"\n  Device {device_serial} Details:")
            for execution in device_result.executions:
                status_symbol = "✓" if execution.status.value == "completed" else "✗"
                duration_str = f"{execution.duration:.2f}s" if execution.duration else "N/A"
                logger.info(f"    {status_symbol} {execution.step.name}: "
                           f"{execution.status.value} ({duration_str})")
                
                if execution.error_message:
                    logger.info(f"      Error: {execution.error_message}")
        
        return metrics.failed_tests == 0
        
    except Exception as e:
        logger.error(f"Custom scenario demo failed: {e}")
        return False
    finally:
        runner.cleanup()


def demonstrate_configuration_management():
    """Demonstrate configuration loading and management"""
    logger = logging.getLogger(__name__)
    logger.info("\n=== Demonstrating Configuration Management ===")
    
    try:
        # Load default configuration
        config_loader = ConfigurationLoader()
        config = config_loader.load_configuration()
        
        logger.info("Loaded configuration successfully")
        
        # Display configuration sections
        logger.info("Configuration sections:")
        logger.info(f"  Timeout configurations: {config.timeout_configurations}")
        logger.info(f"  Execution settings: {config.execution_settings}")
        logger.info(f"  Reporting settings: {config.reporting_settings}")
        logger.info(f"  Device settings: {config.device_settings}")
        
        # Get scenario-specific parameters
        for scenario_type in TestScenarioType:
            params = config_loader.get_scenario_parameters(scenario_type.value)
            logger.info(f"  {scenario_type.value} parameters: "
                       f"pemf_duration={params.pemf_test_duration_ms}ms, "
                       f"stress_duration={params.stress_test_duration_ms}ms")
        
        # Validate configuration
        errors = config_loader.validate_configuration()
        if errors:
            logger.warning(f"Configuration validation errors: {errors}")
        else:
            logger.info("Configuration validation passed")
        
        # Demonstrate configuration updates
        logger.info("Updating scenario parameters...")
        config_loader.update_scenario_parameters('hardware_validation', {
            'pemf_test_duration_ms': 3000,
            'custom_parameter': 'demo_value'
        })
        
        updated_params = config_loader.get_scenario_parameters('hardware_validation')
        logger.info(f"Updated hardware_validation parameters: "
                   f"pemf_duration={updated_params.pemf_test_duration_ms}ms")
        
        # Save updated configuration
        output_path = "demo_config.json"
        config_loader.save_configuration(output_path)
        logger.info(f"Saved updated configuration to: {output_path}")
        
        return True
        
    except Exception as e:
        logger.error(f"Configuration demo failed: {e}")
        return False


def main():
    """Main demonstration function"""
    setup_logging()
    logger = logging.getLogger(__name__)
    
    logger.info("Starting Comprehensive Test Framework Demonstration")
    logger.info("=" * 60)
    
    success_count = 0
    total_demos = 4
    
    # Run all demonstrations
    demos = [
        ("Configuration Management", demonstrate_configuration_management),
        ("Individual Scenarios", demonstrate_individual_scenarios),
        ("Custom Scenario", demonstrate_custom_scenario),
        ("Full Test Suite", demonstrate_full_test_suite)
    ]
    
    for demo_name, demo_func in demos:
        logger.info(f"\nStarting {demo_name} demonstration...")
        try:
            if demo_func():
                logger.info(f"{demo_name} demonstration completed successfully")
                success_count += 1
            else:
                logger.warning(f"{demo_name} demonstration completed with issues")
        except Exception as e:
            logger.error(f"{demo_name} demonstration failed: {e}")
    
    # Summary
    logger.info("\n" + "=" * 60)
    logger.info(f"Demonstration Summary: {success_count}/{total_demos} successful")
    
    if success_count == total_demos:
        logger.info("All demonstrations completed successfully!")
        logger.info("The comprehensive test framework is ready for use.")
    else:
        logger.warning("Some demonstrations had issues. Check logs for details.")
    
    logger.info("Demonstration complete.")
    return 0 if success_count == total_demos else 1


if __name__ == "__main__":
    sys.exit(main())