# Usage Examples

This document provides practical usage examples for the RP2040 pEMF/Battery Monitoring Device automated testing framework.

## Basic Usage Examples

### 1. Device Discovery and Connection

```python
#!/usr/bin/env python3
"""Basic device discovery and connection example"""

import sys
import os
sys.path.insert(0, 'test_framework')

from device_manager import UsbHidDeviceManager

def basic_connection_example():
    # Initialize device manager
    device_manager = UsbHidDeviceManager()
    
    # Discover available devices
    print("Discovering devices...")
    devices = device_manager.discover_devices()
    
    if not devices:
        print("No devices found")
        return False
    
    # Print discovered devices
    for i, device in enumerate(devices):
        print(f"Device {i}: {device.serial_number} - {device.product_string}")
    
    # Connect to first device
    device = devices[0]
    print(f"Connecting to device {device.serial_number}...")
    
    success = device_manager.connect_device(device.serial_number)
    if success:
        print("Connection successful!")
        
        # Disconnect when done
        device_manager.disconnect_device(device.serial_number)
        print("Disconnected")
        return True
    else:
        print("Connection failed")
        return False

if __name__ == "__main__":
    basic_connection_example()
```

### 2. System Health Query

```python
#!/usr/bin/env python3
"""System health query example"""

import sys
import os
sys.path.insert(0, 'test_framework')

from device_manager import UsbHidDeviceManager
from command_handler import CommandHandler

def system_health_example():
    # Initialize components
    device_manager = UsbHidDeviceManager()
    command_handler = CommandHandler(device_manager)
    
    # Connect to device
    devices = device_manager.discover_devices()
    if not devices:
        print("No devices found")
        return
    
    device = devices[0]
    if not device_manager.connect_device(device.serial_number):
        print("Failed to connect to device")
        return
    
    try:
        # Create system health query
        command = command_handler.create_system_state_query('system_health')
        
        # Send command and wait for response
        print("Querying system health...")
        response = command_handler.send_command_and_wait(device.serial_number, command)
        
        if response:
            print("System health response received:")
            print(f"  Status: {response.get('status', 'Unknown')}")
            print(f"  Uptime: {response.get('uptime_ms', 0)} ms")
            print(f"  Battery: {response.get('battery_voltage', 0)} V")
        else:
            print("No response received")
            
    finally:
        device_manager.disconnect_device(device.serial_number)

if __name__ == "__main__":
    system_health_example()
```

### 3. Running a Simple Test

```python
#!/usr/bin/env python3
"""Simple test execution example"""

import sys
import os
sys.path.insert(0, 'test_framework')

from device_manager import UsbHidDeviceManager
from command_handler import CommandHandler
from test_sequencer import TestSequencer, TestConfiguration, TestStep, TestType

def simple_test_example():
    # Initialize components
    device_manager = UsbHidDeviceManager()
    command_handler = CommandHandler(device_manager)
    test_sequencer = TestSequencer(device_manager, command_handler)
    
    # Connect to device
    devices = device_manager.discover_devices()
    if not devices:
        print("No devices found")
        return
    
    device = devices[0]
    if not device_manager.connect_device(device.serial_number):
        print("Failed to connect to device")
        return
    
    try:
        # Create test configuration
        config = TestConfiguration(
            name="Basic Communication Test",
            description="Test basic USB HID communication",
            steps=[
                TestStep(
                    name="communication_test",
                    test_type=TestType.USB_COMMUNICATION_TEST,
                    parameters={"message_count": 5, "timeout_ms": 1000},
                    timeout=10.0,
                    required=True
                )
            ]
        )
        
        # Execute test
        print("Running communication test...")
        results = test_sequencer.execute_test_sequence(config, [device.serial_number])
        
        # Print results
        for device_serial, device_results in results.items():
            print(f"\nResults for device {device_serial}:")
            for result in device_results:
                status = "PASS" if result.success else "FAIL"
                print(f"  {result.test_name}: {status}")
                if result.error_message:
                    print(f"    Error: {result.error_message}")
                    
    finally:
        device_manager.disconnect_device(device.serial_number)

if __name__ == "__main__":
    simple_test_example()
```

## Advanced Usage Examples

### 4. Comprehensive Test Suite

```python
#!/usr/bin/env python3
"""Comprehensive test suite example"""

import sys
import os
import time
sys.path.insert(0, 'test_framework')

from device_manager import UsbHidDeviceManager
from command_handler import CommandHandler
from test_sequencer import TestSequencer, TestConfiguration, TestStep, TestType
from result_collector import ResultCollector

def comprehensive_test_example():
    # Initialize all components
    device_manager = UsbHidDeviceManager()
    command_handler = CommandHandler(device_manager)
    test_sequencer = TestSequencer(device_manager, command_handler)
    result_collector = ResultCollector()
    
    # Connect to device
    devices = device_manager.discover_devices()
    if not devices:
        print("No devices found")
        return
    
    device = devices[0]
    if not device_manager.connect_device(device.serial_number):
        print("Failed to connect to device")
        return
    
    try:
        # Create comprehensive test configuration
        config = TestConfiguration(
            name="Hardware Validation Suite",
            description="Complete hardware validation test suite",
            steps=[
                TestStep(
                    name="communication_test",
                    test_type=TestType.USB_COMMUNICATION_TEST,
                    parameters={"message_count": 10},
                    timeout=15.0,
                    required=True
                ),
                TestStep(
                    name="pemf_timing_test",
                    test_type=TestType.PEMF_TIMING_VALIDATION,
                    parameters={"duration_ms": 10000, "tolerance_percent": 1.0},
                    timeout=30.0,
                    required=True,
                    depends_on=["communication_test"]
                ),
                TestStep(
                    name="battery_test",
                    test_type=TestType.BATTERY_ADC_CALIBRATION,
                    parameters={"reference_voltage": 3.7},
                    timeout=20.0,
                    required=True
                ),
                TestStep(
                    name="led_test",
                    test_type=TestType.LED_FUNCTIONALITY,
                    parameters={"pattern": "all_patterns"},
                    timeout=25.0,
                    required=False
                ),
                TestStep(
                    name="stress_test",
                    test_type=TestType.SYSTEM_STRESS_TEST,
                    parameters={"duration_ms": 30000, "load_level": 5},
                    timeout=60.0,
                    required=False,
                    depends_on=["pemf_timing_test", "battery_test"]
                )
            ],
            parallel_execution=False,
            global_timeout=300.0
        )
        
        # Execute test suite
        print("Running comprehensive test suite...")
        start_time = time.time()
        
        results = test_sequencer.execute_test_sequence(config, [device.serial_number])
        
        end_time = time.time()
        
        # Collect and analyze results
        suite_result = result_collector.collect_results(
            config.name, config.description, results, start_time, end_time
        )
        
        # Generate reports
        print("\nGenerating reports...")
        
        # Summary report
        summary = result_collector.generate_summary_report(suite_result)
        print("\n" + "="*50)
        print("TEST SUITE SUMMARY")
        print("="*50)
        print(summary)
        
        # Detailed report
        detailed = result_collector.generate_detailed_report(suite_result)
        print("\n" + "="*50)
        print("DETAILED RESULTS")
        print("="*50)
        print(detailed)
        
        # Export to files
        json_report = result_collector.export_json(suite_result, detailed=True)
        with open('test_results.json', 'w') as f:
            f.write(json_report)
        print("\nDetailed results saved to test_results.json")
        
        junit_xml = result_collector.export_junit_xml(suite_result)
        with open('test_results.xml', 'w') as f:
            f.write(junit_xml)
        print("JUnit XML results saved to test_results.xml")
        
    finally:
        device_manager.disconnect_device(device.serial_number)

if __name__ == "__main__":
    comprehensive_test_example()
```

For more examples, see the test_framework/example_usage.py file.