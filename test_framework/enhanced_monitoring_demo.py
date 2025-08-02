#!/usr/bin/env python3
"""
Enhanced Real-time Monitoring and Debugging Demo

This script demonstrates the enhanced real-time monitoring and debugging capabilities
implemented for the automated testing framework.
"""

import time
import json
from pathlib import Path

from test_framework.real_time_monitor import RealTimeMonitor, LogLevel
from test_framework.command_handler import TestCommand, TestResponse, CommandType, ResponseStatus
from test_framework.test_sequencer import TestExecution, TestStep, TestStatus, TestType


def demonstrate_enhanced_monitoring():
    """Demonstrate enhanced monitoring capabilities"""
    print("=== Enhanced Real-time Monitoring and Debugging Demo ===\n")
    
    # Create monitor with debug level for full feature demonstration
    monitor = RealTimeMonitor(
        log_level=LogLevel.DEBUG,
        enable_snapshots=True,
        max_history_size=100
    )
    
    print("1. Starting enhanced monitoring system...")
    monitor.start_monitoring()
    
    # Demonstrate real-time progress tracking
    print("\n2. Demonstrating real-time progress tracking...")
    device_serial = "DEMO_DEVICE_001"
    
    monitor.log_test_started(device_serial, "demo_test_1", total_tests=3)
    time.sleep(0.5)
    
    # Show progress
    progress = monitor.get_device_progress(device_serial)
    print(f"   Current test: {progress.current_test}")
    print(f"   Progress: {progress.completed_tests}/{progress.total_tests}")
    print(f"   Health status: {progress.health_status}")
    
    # Demonstrate enhanced communication logging with protocol debugging
    print("\n3. Demonstrating enhanced communication logging...")
    
    command = TestCommand(
        CommandType.EXECUTE_TEST, 
        1, 
        {"test_type": "demo", "parameters": {"duration_ms": 1000}}
    )
    correlation_id = monitor.log_command_sent(device_serial, command)
    print(f"   Sent command with correlation ID: {correlation_id}")
    
    # Simulate processing delay for latency measurement
    time.sleep(0.1)
    
    response = TestResponse(
        1, 
        ResponseStatus.SUCCESS, 
        "test_result", 
        {"result": "passed", "duration": 95.5}, 
        time.time()
    )
    monitor.log_response_received(device_serial, response, correlation_id)
    print("   Received response with latency measurement")
    
    # Show communication logs
    comm_logs = monitor.get_communication_logs(device_serial)
    for log in comm_logs:
        print(f"   {log.direction.upper()}: {log.message_type} "
              f"(latency: {log.latency_ms:.1f}ms)" if log.latency_ms else f"   {log.direction.upper()}: {log.message_type}")
    
    # Demonstrate device communication logging with protocol analysis
    print("\n4. Demonstrating device communication with protocol analysis...")
    
    test_messages = [
        "LOG: Test execution started",
        "DEBUG: ADC reading: 3.25V",
        "TEST_RESPONSE:{\"status\":\"running\",\"progress\":50}",
        "ERROR: Timeout occurred during test"
    ]
    
    for message in test_messages:
        monitor.log_device_communication(device_serial, message, 'received')
    
    print("   Logged various device messages with protocol analysis")
    
    # Complete the test successfully
    test_step = TestStep("demo_test_1", TestType.USB_COMMUNICATION_TEST, {})
    execution = TestExecution(
        step=test_step,
        device_serial=device_serial,
        status=TestStatus.COMPLETED,
        start_time=time.time() - 2.0,
        end_time=time.time()
    )
    monitor.log_test_completed(device_serial, "demo_test_1", execution)
    
    # Demonstrate failure capture and system state snapshots
    print("\n5. Demonstrating failure capture and system state snapshots...")
    
    monitor.log_test_started(device_serial, "demo_test_2")
    
    # Simulate a test failure
    failed_execution = TestExecution(
        step=TestStep("demo_test_2", TestType.PEMF_TIMING_VALIDATION, {}),
        device_serial=device_serial,
        status=TestStatus.FAILED,
        error_message="Timing validation failed - pulse accuracy outside tolerance",
        start_time=time.time() - 1.5,
        end_time=time.time()
    )
    monitor.log_test_failed(device_serial, "demo_test_2", failed_execution)
    
    # Wait for snapshot capture
    time.sleep(0.2)
    
    snapshots = monitor.get_system_snapshots(device_serial)
    print(f"   Captured {len(snapshots)} system state snapshot(s)")
    
    if snapshots:
        snapshot = snapshots[0]
        print(f"   Snapshot details:")
        print(f"     - Test: {snapshot.test_name}")
        print(f"     - Error: {snapshot.error_context}")
        print(f"     - Device logs: {len(snapshot.device_logs)} entries")
        print(f"     - Communication history: {len(snapshot.communication_history)} entries")
        print(f"     - Performance metrics: {len(snapshot.performance_metrics)} metrics")
    
    # Demonstrate enhanced failure analysis
    print("\n6. Demonstrating enhanced failure analysis...")
    
    analysis = monitor.get_enhanced_failure_analysis(device_serial)
    print(f"   Total failures: {analysis['total_failures']}")
    print(f"   Failure patterns: {list(analysis['failure_patterns'].keys())}")
    print(f"   Recovery suggestions: {len(analysis['recovery_suggestions'])}")
    
    for suggestion in analysis['recovery_suggestions']:
        print(f"     - {suggestion}")
    
    # Demonstrate real-time debug information
    print("\n7. Demonstrating real-time debug information...")
    
    debug_info = monitor.get_real_time_debug_info(device_serial)
    print(f"   Monitoring status: {debug_info['monitoring_status']['active']}")
    print(f"   Log level: {debug_info['monitoring_status']['log_level']}")
    print(f"   Protocol debugging: {debug_info['monitoring_status']['protocol_debug_enabled']}")
    
    if device_serial in debug_info['system_health']:
        health = debug_info['system_health'][device_serial]
        print(f"   Device health: {health['health_status']}")
        print(f"   Success rate: {health['success_rate']:.1%}")
        print(f"   Completion: {health['completion_percentage']:.1f}%")
    
    if device_serial in debug_info['communication_stats']:
        comm_stats = debug_info['communication_stats'][device_serial]
        print(f"   Communication stats:")
        print(f"     - Total messages: {comm_stats['total_messages']}")
        print(f"     - Sent commands: {comm_stats['sent_commands']}")
        print(f"     - Received responses: {comm_stats['received_responses']}")
        if comm_stats['avg_latency_ms']:
            print(f"     - Average latency: {comm_stats['avg_latency_ms']:.1f}ms")
    
    # Demonstrate periodic status reporting
    print("\n8. Demonstrating periodic status reporting...")
    print("   (Waiting for periodic status report...)")
    
    # Wait for a status report
    time.sleep(1.0)
    
    # Demonstrate monitoring data export
    print("\n9. Demonstrating monitoring data export...")
    
    export_file = "demo_monitoring_data.json"
    monitor.export_monitoring_data(export_file)
    print(f"   Exported monitoring data to: {export_file}")
    
    # Show export file size and structure
    if Path(export_file).exists():
        file_size = Path(export_file).stat().st_size
        print(f"   Export file size: {file_size} bytes")
        
        with open(export_file, 'r') as f:
            export_data = json.load(f)
        
        print(f"   Export contains:")
        print(f"     - Device progress: {len(export_data['device_progress'])} devices")
        print(f"     - Event history: {len(export_data['event_history'])} events")
        print(f"     - Communication logs: {len(export_data['communication_logs'])} logs")
        print(f"     - System snapshots: {len(export_data['system_snapshots'])} snapshots")
        print(f"     - Enhanced analysis: included")
        print(f"     - Real-time debug info: included")
    
    # Clean up
    print("\n10. Stopping monitoring system...")
    monitor.stop_monitoring()
    
    print("\n=== Demo Complete ===")
    print("\nKey Enhanced Features Demonstrated:")
    print("✓ Real-time progress tracking with health monitoring")
    print("✓ Enhanced communication logging with latency measurement")
    print("✓ Protocol debugging with detailed message analysis")
    print("✓ Automatic failure capture with system state snapshots")
    print("✓ Enhanced failure analysis with recovery suggestions")
    print("✓ Real-time debug information collection")
    print("✓ Periodic status reporting for long-running tests")
    print("✓ Comprehensive monitoring data export")
    
    print(f"\nMonitoring data has been exported to: {export_file}")
    print("You can examine this file to see the detailed monitoring information collected.")


if __name__ == "__main__":
    demonstrate_enhanced_monitoring()