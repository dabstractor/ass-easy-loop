#!/usr/bin/env python3
"""
No-std Test Integration Example

This script demonstrates how to use the no_std test integration to execute
embedded tests and collect results.
"""

import sys
import logging
import argparse
from pathlib import Path

# Add the test framework to the path
sys.path.insert(0, str(Path(__file__).parent))

from device_manager import UsbHidDeviceManager
from command_handler import CommandHandler
from nostd_test_integration import NoStdTestIntegration
from firmware_flasher import FirmwareFlasher


def setup_logging(level: str = "INFO"):
    """Setup logging configuration"""
    logging.basicConfig(
        level=getattr(logging, level.upper()),
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )


def main():
    """Main example function"""
    parser = argparse.ArgumentParser(description="No-std Test Integration Example")
    parser.add_argument("--device", help="Target device serial number")
    parser.add_argument("--firmware", help="Path to test firmware")
    parser.add_argument("--suite", help="Test suite name to execute")
    parser.add_argument("--test", help="Specific test name to execute (requires --suite)")
    parser.add_argument("--list-tests", action="store_true", help="List available tests")
    parser.add_argument("--log-level", default="INFO", choices=["DEBUG", "INFO", "WARNING", "ERROR"])
    
    args = parser.parse_args()
    
    setup_logging(args.log_level)
    logger = logging.getLogger(__name__)
    
    # Initialize components
    device_manager = UsbHidDeviceManager()
    command_handler = CommandHandler(device_manager)
    firmware_flasher = FirmwareFlasher()
    nostd_integration = NoStdTestIntegration(device_manager, command_handler, firmware_flasher)
    
    try:
        # Discover devices
        logger.info("Discovering devices...")
        devices = device_manager.discover_devices()
        
        if not devices:
            logger.error("No devices found")
            return 1
        
        # Use specified device or first available
        if args.device:
            target_device = args.device
            if not any(d.serial_number == target_device for d in devices):
                logger.error(f"Device {target_device} not found")
                return 1
        else:
            target_device = devices[0].serial_number
            logger.info(f"Using device: {target_device}")
        
        # Connect to device
        if not device_manager.connect_device(target_device):
            logger.error(f"Failed to connect to device {target_device}")
            return 1
        
        # Flash firmware if provided
        if args.firmware:
            logger.info(f"Flashing test firmware: {args.firmware}")
            if not nostd_integration.flash_test_firmware(target_device, args.firmware):
                logger.error("Failed to flash test firmware")
                return 1
            logger.info("Firmware flashed successfully")
        
        # Execute requested operation
        if args.list_tests:
            logger.info("Listing available no_std tests...")
            test_list = nostd_integration.list_available_tests(target_device)
            
            if test_list:
                print(f"\nAvailable no_std tests on device {target_device}:")
                for suite_name, tests in test_list.items():
                    print(f"\nSuite: {suite_name}")
                    for test_name in tests:
                        print(f"  - {test_name}")
            else:
                print("No no_std tests found or device not responding")
                
        elif args.suite:
            if args.test:
                # Execute single test
                logger.info(f"Executing test '{args.test}' from suite '{args.suite}'")
                result = nostd_integration.execute_nostd_test(target_device, args.suite, args.test)
                
                if result:
                    print(f"\nTest Result:")
                    print(f"  Test: {result.test_name}")
                    print(f"  Status: {result.status.name}")
                    print(f"  Execution Time: {result.execution_time_ms}ms")
                    if result.error_message:
                        print(f"  Error: {result.error_message}")
                    
                    return 0 if result.status.name == 'PASS' else 1
                else:
                    logger.error("Test execution failed")
                    return 1
            else:
                # Execute entire suite
                logger.info(f"Executing test suite '{args.suite}'")
                suite_result = nostd_integration.execute_nostd_test_suite(target_device, args.suite)
                
                if suite_result:
                    metrics = suite_result.aggregate_metrics
                    print(f"\nTest Suite Results:")
                    print(f"  Suite: {suite_result.suite_name}")
                    print(f"  Total Tests: {metrics.total_tests}")
                    print(f"  Passed: {metrics.passed_tests}")
                    print(f"  Failed: {metrics.failed_tests}")
                    print(f"  Skipped: {metrics.skipped_tests}")
                    print(f"  Success Rate: {metrics.success_rate:.1f}%")
                    print(f"  Duration: {suite_result.duration:.2f}s")
                    
                    # Show individual test results
                    device_result = suite_result.device_results[target_device]
                    print(f"\nIndividual Test Results:")
                    for execution in device_result.executions:
                        status = execution.status.value.upper()
                        duration = f"{(execution.duration or 0) * 1000:.0f}ms"
                        print(f"  {execution.step.name}: {status} ({duration})")
                        if execution.error_message:
                            print(f"    Error: {execution.error_message}")
                    
                    return 0 if metrics.failed_tests == 0 else 1
                else:
                    logger.error("Test suite execution failed")
                    return 1
        else:
            logger.error("No operation specified. Use --list-tests, --suite, or --suite with --test")
            return 1
            
    except KeyboardInterrupt:
        logger.info("Operation interrupted by user")
        return 1
    except Exception as e:
        logger.error(f"Unexpected error: {e}")
        return 1
    finally:
        # Cleanup
        device_manager.disconnect_all()


if __name__ == "__main__":
    sys.exit(main())