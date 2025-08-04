#!/usr/bin/env python3
"""
Example usage of the Automated Test Framework

This script demonstrates how to use the test framework to discover devices,
execute test sequences, and collect results.
"""

import logging
import time
from test_framework import (
    UsbHidDeviceManager, CommandHandler, TestSequencer, ResultCollector
)
from test_framework.command_handler import TestType
from test_framework.test_sequencer import TestStep, TestConfiguration


def setup_logging():
    """Configure logging for the example"""
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )


def main():
    """Main example function"""
    setup_logging()
    logger = logging.getLogger(__name__)
    
    logger.info("Starting Automated Test Framework Example")
    
    # Initialize framework components
    device_manager = UsbHidDeviceManager()
    command_handler = CommandHandler(device_manager)
    test_sequencer = TestSequencer(device_manager, command_handler)
    result_collector = ResultCollector()
    
    try:
        # Discover available devices
        logger.info("Discovering devices...")
        devices = device_manager.discover_devices()
        
        if not devices:
            logger.error("No devices found. Please connect a test device.")
            return
        
        logger.info(f"Found {len(devices)} device(s):")
        for device in devices:
            logger.info(f"  - {device.serial_number}: {device.product} ({device.status.value})")
        
        # Connect to the first available device
        target_device = devices[0]
        logger.info(f"Connecting to device: {target_device.serial_number}")
        
        if not device_manager.connect_device(target_device.serial_number):
            logger.error("Failed to connect to device")
            return
        
        # Create a custom test configuration
        test_config = TestConfiguration(
            name="Example Test Suite",
            description="Demonstration of automated testing capabilities",
            steps=[
                TestStep(
                    name="communication_test",
                    test_type=TestType.USB_COMMUNICATION_TEST,
                    parameters={"message_count": 5, "timeout_ms": 1000},
                    timeout=10.0
                ),
                TestStep(
                    name="system_health_check",
                    test_type=TestType.PEMF_TIMING_VALIDATION,
                    parameters={"duration_ms": 2000, "tolerance_percent": 1.0},
                    timeout=15.0,
                    depends_on=["communication_test"]
                ),
                TestStep(
                    name="battery_validation",
                    test_type=TestType.BATTERY_ADC_CALIBRATION,
                    parameters={"reference_voltage": 3.3},
                    timeout=10.0,
                    depends_on=["communication_test"]
                )
            ],
            parallel_execution=False,
            global_timeout=60.0
        )
        
        # Execute test sequence
        logger.info("Starting test execution...")
        start_time = time.time()
        
        execution_results = test_sequencer.execute_test_sequence(
            test_config,
            target_devices=[target_device.serial_number]
        )
        
        end_time = time.time()
        
        # Collect and analyze results
        logger.info("Collecting results...")
        suite_result = result_collector.collect_results(
            test_config.name,
            test_config.description,
            execution_results,
            start_time,
            end_time
        )
        
        # Display results summary
        logger.info("Test Results Summary:")
        logger.info(f"  Suite: {suite_result.suite_name}")
        logger.info(f"  Duration: {suite_result.duration:.2f} seconds")
        logger.info(f"  Devices tested: {len(suite_result.device_results)}")
        
        metrics = suite_result.aggregate_metrics
        logger.info(f"  Total tests: {metrics.total_tests}")
        logger.info(f"  Passed: {metrics.passed_tests}")
        logger.info(f"  Failed: {metrics.failed_tests}")
        logger.info(f"  Skipped: {metrics.skipped_tests}")
        logger.info(f"  Success rate: {metrics.success_rate:.1f}%")
        
        # Display detailed results for each device
        for device_serial, device_result in suite_result.device_results.items():
            logger.info(f"\nDevice {device_serial} Results:")
            logger.info(f"  Overall status: {device_result.overall_status.value}")
            
            for execution in device_result.executions:
                status_symbol = "✓" if execution.status.value == "completed" else "✗"
                duration_str = f"{execution.duration:.2f}s" if execution.duration else "N/A"
                logger.info(f"    {status_symbol} {execution.step.name}: {execution.status.value} ({duration_str})")
                
                if execution.error_message:
                    logger.info(f"      Error: {execution.error_message}")
        
        # Generate reports
        logger.info("\nGenerating reports...")
        
        # JSON report
        json_report = result_collector.export_json(suite_result, detailed=True)
        with open("test_results.json", "w") as f:
            f.write(json_report)
        logger.info("  Detailed JSON report saved to: test_results.json")
        
        # JUnit XML report
        junit_xml = result_collector.export_junit_xml(suite_result)
        with open("test_results.xml", "w") as f:
            f.write(junit_xml)
        logger.info("  JUnit XML report saved to: test_results.xml")
        
        # Failure analysis
        if metrics.failed_tests > 0:
            logger.info("\nFailure Analysis:")
            failure_analysis = result_collector.get_failure_analysis(suite_result)
            
            for test_name, count in failure_analysis["failure_by_test"].items():
                logger.info(f"  {test_name}: {count} failure(s)")
            
            for recommendation in failure_analysis["recommendations"]:
                logger.info(f"  Recommendation: {recommendation}")
        
        # Performance analysis
        logger.info("\nPerformance Analysis:")
        performance_analysis = result_collector.get_performance_analysis(suite_result)
        
        for test_name, times in performance_analysis["execution_times"].items():
            logger.info(f"  {test_name}: avg={times['mean']:.2f}s, "
                       f"min={times['min']:.2f}s, max={times['max']:.2f}s")
        
    except KeyboardInterrupt:
        logger.info("Test execution interrupted by user")
    except Exception as e:
        logger.error(f"Test execution failed: {e}")
        raise
    finally:
        # Cleanup
        logger.info("Cleaning up...")
        device_manager.disconnect_all()
        logger.info("Example completed")


if __name__ == "__main__":
    main()