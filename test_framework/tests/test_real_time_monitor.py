"""
Integration Tests for Real-time Monitoring and Debugging

Tests the real-time monitoring system, verbose logging modes, failure capture,
and communication debugging capabilities.
"""

import unittest
import time
import threading
from unittest.mock import Mock, MagicMock, patch
import json
import tempfile
import os

from test_framework.real_time_monitor import (
    RealTimeMonitor, LogLevel, MonitoringEvent, MonitoringEventData,
    SystemStateSnapshot, CommunicationLog, ProgressStatus
)
from test_framework.command_handler import TestCommand, TestResponse, CommandType, ResponseStatus
from test_framework.test_sequencer import TestExecution, TestStep, TestStatus, TestType


class TestRealTimeMonitor(unittest.TestCase):
    """Test cases for real-time monitoring functionality"""
    
    def setUp(self):
        """Set up test fixtures"""
        self.monitor = RealTimeMonitor(
            log_level=LogLevel.DEBUG,
            max_history_size=100,
            enable_snapshots=True
        )
        self.test_device_serial = "TEST_DEVICE_001"
        
    def tearDown(self):
        """Clean up after tests"""
        if self.monitor.monitoring_active:
            self.monitor.stop_monitoring()
    
    def test_monitor_initialization(self):
        """Test monitor initialization with different configurations"""
        # Test default initialization
        monitor_default = RealTimeMonitor()
        self.assertEqual(monitor_default.log_level, LogLevel.NORMAL)
        self.assertEqual(monitor_default.max_history_size, 1000)
        self.assertTrue(monitor_default.enable_snapshots)
        
        # Test custom initialization
        monitor_custom = RealTimeMonitor(
            log_level=LogLevel.VERBOSE,
            max_history_size=500,
            enable_snapshots=False
        )
        self.assertEqual(monitor_custom.log_level, LogLevel.VERBOSE)
        self.assertEqual(monitor_custom.max_history_size, 500)
        self.assertFalse(monitor_custom.enable_snapshots)
    
    def test_monitoring_lifecycle(self):
        """Test starting and stopping monitoring"""
        # Initially not active
        self.assertFalse(self.monitor.monitoring_active)
        self.assertIsNone(self.monitor.monitor_thread)
        
        # Start monitoring
        self.monitor.start_monitoring()
        self.assertTrue(self.monitor.monitoring_active)
        self.assertIsNotNone(self.monitor.monitor_thread)
        self.assertTrue(self.monitor.monitor_thread.is_alive())
        
        # Stop monitoring
        self.monitor.stop_monitoring()
        self.assertFalse(self.monitor.monitoring_active)
        
        # Thread should finish
        time.sleep(0.2)
        self.assertFalse(self.monitor.monitor_thread.is_alive())
    
    def test_test_lifecycle_logging(self):
        """Test logging of test lifecycle events"""
        self.monitor.start_monitoring()
        
        # Log test started
        self.monitor.log_test_started(self.test_device_serial, "test_1", total_tests=3)
        
        # Verify progress tracking
        progress = self.monitor.get_device_progress(self.test_device_serial)
        self.assertIsNotNone(progress)
        self.assertEqual(progress.current_test, "test_1")
        self.assertEqual(progress.total_tests, 3)
        self.assertEqual(progress.completed_tests, 0)
        
        # Create mock execution for completion
        test_step = TestStep(
            name="test_1",
            test_type=TestType.USB_COMMUNICATION_TEST,
            parameters={"message_count": 10}
        )
        execution = TestExecution(
            step=test_step,
            device_serial=self.test_device_serial,
            status=TestStatus.COMPLETED,
            start_time=time.time() - 2.0,
            end_time=time.time()
        )
        
        # Log test completed
        self.monitor.log_test_completed(self.test_device_serial, "test_1", execution)
        
        # Wait for event processing
        time.sleep(0.1)
        
        # Verify progress update
        progress = self.monitor.get_device_progress(self.test_device_serial)
        self.assertEqual(progress.completed_tests, 1)
        self.assertEqual(progress.success_count, 1)
        self.assertEqual(progress.failure_count, 0)
        self.assertIsNone(progress.current_test)
    
    def test_test_failure_logging_and_snapshots(self):
        """Test failure logging and system state snapshot capture"""
        self.monitor.start_monitoring()
        
        # Create mock failed execution
        test_step = TestStep(
            name="failing_test",
            test_type=TestType.PEMF_TIMING_VALIDATION,
            parameters={"duration_ms": 5000}
        )
        execution = TestExecution(
            step=test_step,
            device_serial=self.test_device_serial,
            status=TestStatus.FAILED,
            start_time=time.time() - 3.0,
            end_time=time.time(),
            error_message="Test timeout occurred"
        )
        
        # Log test failure
        self.monitor.log_test_failed(self.test_device_serial, "failing_test", execution)
        
        # Wait for event processing and snapshot capture
        time.sleep(0.2)
        
        # Verify failure was logged
        progress = self.monitor.get_device_progress(self.test_device_serial)
        self.assertEqual(progress.failure_count, 1)
        
        # Verify snapshot was captured
        snapshots = self.monitor.get_system_snapshots(self.test_device_serial)
        self.assertEqual(len(snapshots), 1)
        
        snapshot = snapshots[0]
        self.assertEqual(snapshot.device_serial, self.test_device_serial)
        self.assertEqual(snapshot.test_name, "failing_test")
        self.assertEqual(snapshot.error_context, "Test timeout occurred")
        self.assertIsNotNone(snapshot.system_state)
        self.assertIsInstance(snapshot.device_logs, list)
        self.assertIsInstance(snapshot.communication_history, list)
        self.assertIsInstance(snapshot.performance_metrics, dict)
    
    def test_communication_logging(self):
        """Test device communication logging with correlation"""
        self.monitor.start_monitoring()
        
        # Create mock command
        command = TestCommand(
            command_type=CommandType.EXECUTE_TEST,
            command_id=1,
            payload={"test_type": "usb_test", "parameters": {"count": 5}}
        )
        
        # Log command sent
        correlation_id = self.monitor.log_command_sent(self.test_device_serial, command)
        self.assertIsNotNone(correlation_id)
        self.assertTrue(correlation_id.startswith("cmd_"))
        
        # Create mock response
        response = TestResponse(
            command_id=1,
            status=ResponseStatus.SUCCESS,
            response_type="test_result",
            data={"result": "passed", "duration": 2.5},
            timestamp=time.time()
        )
        
        # Log response received
        self.monitor.log_response_received(self.test_device_serial, response, correlation_id)
        
        # Wait for processing
        time.sleep(0.1)
        
        # Verify communication logs
        comm_logs = self.monitor.get_communication_logs(self.test_device_serial)
        self.assertEqual(len(comm_logs), 2)  # Command sent + response received
        
        sent_log = next(log for log in comm_logs if log.direction == 'sent')
        received_log = next(log for log in comm_logs if log.direction == 'received')
        
        self.assertEqual(sent_log.correlation_id, correlation_id)
        self.assertEqual(received_log.correlation_id, correlation_id)
        self.assertEqual(sent_log.message_type, "EXECUTE_TEST")
        self.assertEqual(received_log.message_type, "test_result")
    
    def test_device_communication_raw_logging(self):
        """Test raw device communication logging"""
        self.monitor.start_monitoring()
        
        # Log raw device messages
        self.monitor.log_device_communication(self.test_device_serial, "LOG: System initialized", "received")
        self.monitor.log_device_communication(self.test_device_serial, "LOG: Test started", "received")
        self.monitor.log_device_communication(self.test_device_serial, "ERROR: Timeout occurred", "received")
        
        # Wait for processing
        time.sleep(0.1)
        
        # Verify events were logged
        events = self.monitor.get_event_history(
            device_serial=self.test_device_serial,
            event_types=[MonitoringEvent.DEVICE_COMMUNICATION]
        )
        self.assertEqual(len(events), 3)
        
        # Check event data
        for event in events:
            self.assertEqual(event.device_serial, self.test_device_serial)
            self.assertEqual(event.event_type, MonitoringEvent.DEVICE_COMMUNICATION)
            self.assertIn('direction', event.data)
            self.assertIn('message', event.data)
    
    def test_progress_estimation(self):
        """Test progress estimation and completion time calculation"""
        self.monitor.start_monitoring()
        
        # Start a test sequence
        self.monitor.log_test_started(self.test_device_serial, "test_1", total_tests=4)
        
        # Simulate test completions with delays
        test_step = TestStep(name="test", test_type=TestType.USB_COMMUNICATION_TEST, parameters={})
        
        for i in range(3):
            execution = TestExecution(
                step=test_step,
                device_serial=self.test_device_serial,
                status=TestStatus.COMPLETED,
                start_time=time.time() - 1.0,
                end_time=time.time()
            )
            self.monitor.log_test_completed(self.test_device_serial, f"test_{i+1}", execution)
            time.sleep(0.1)  # Small delay between tests
        
        # Wait for progress estimation update
        time.sleep(0.2)
        
        # Check progress and estimation
        progress = self.monitor.get_device_progress(self.test_device_serial)
        self.assertEqual(progress.completed_tests, 3)
        self.assertEqual(progress.total_tests, 4)
        self.assertIsNotNone(progress.estimated_completion)
        
        # Estimated completion should be in the future
        self.assertGreater(progress.estimated_completion, time.time())
    
    def test_event_callbacks(self):
        """Test event callback registration and execution"""
        self.monitor.start_monitoring()
        
        # Setup callback tracking
        callback_calls = []
        
        def test_callback(event):
            callback_calls.append(event)
        
        # Register callback
        self.monitor.register_event_callback(MonitoringEvent.TEST_STARTED, test_callback)
        
        # Trigger event
        self.monitor.log_test_started(self.test_device_serial, "callback_test")
        
        # Wait for processing
        time.sleep(0.1)
        
        # Verify callback was called
        self.assertEqual(len(callback_calls), 1)
        self.assertEqual(callback_calls[0].event_type, MonitoringEvent.TEST_STARTED)
        self.assertEqual(callback_calls[0].device_serial, self.test_device_serial)
        self.assertEqual(callback_calls[0].test_name, "callback_test")
    
    def test_verbose_logging_levels(self):
        """Test different logging verbosity levels"""
        # Test each logging level
        for log_level in [LogLevel.MINIMAL, LogLevel.NORMAL, LogLevel.VERBOSE, LogLevel.DEBUG]:
            with self.subTest(log_level=log_level):
                monitor = RealTimeMonitor(log_level=log_level)
                monitor.start_monitoring()
                
                # Log various events
                monitor.log_test_started("device_001", "test_verbose")
                monitor.log_device_communication("device_001", "LOG: Test message")
                
                # Wait for processing
                time.sleep(0.1)
                
                # All events should be recorded regardless of log level
                events = monitor.get_event_history("device_001")
                self.assertGreaterEqual(len(events), 2)
                
                monitor.stop_monitoring()
    
    def test_monitoring_data_export(self):
        """Test exporting monitoring data to file"""
        self.monitor.start_monitoring()
        
        # Generate some test data
        self.monitor.log_test_started(self.test_device_serial, "export_test", total_tests=2)
        
        command = TestCommand(CommandType.SYSTEM_STATE_QUERY, 1, {"query": "health"})
        correlation_id = self.monitor.log_command_sent(self.test_device_serial, command)
        
        response = TestResponse(1, ResponseStatus.SUCCESS, "health_data", {"status": "ok"}, time.time())
        self.monitor.log_response_received(self.test_device_serial, response, correlation_id)
        
        # Wait for processing
        time.sleep(0.1)
        
        # Export to temporary file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as tmp_file:
            export_path = tmp_file.name
        
        try:
            self.monitor.export_monitoring_data(export_path)
            
            # Verify file was created and contains expected data
            self.assertTrue(os.path.exists(export_path))
            
            with open(export_path, 'r') as f:
                export_data = json.load(f)
            
            # Check structure
            self.assertIn('metadata', export_data)
            self.assertIn('device_progress', export_data)
            self.assertIn('event_history', export_data)
            self.assertIn('communication_logs', export_data)
            self.assertIn('system_snapshots', export_data)
            
            # Check content
            self.assertIn(self.test_device_serial, export_data['device_progress'])
            self.assertGreater(len(export_data['event_history']), 0)
            self.assertGreater(len(export_data['communication_logs']), 0)
            
        finally:
            # Clean up
            if os.path.exists(export_path):
                os.unlink(export_path)
    
    def test_event_history_filtering(self):
        """Test event history filtering capabilities"""
        self.monitor.start_monitoring()
        
        # Generate mixed events for multiple devices
        devices = ["device_001", "device_002"]
        
        for device in devices:
            self.monitor.log_test_started(device, f"test_{device}")
            self.monitor.log_device_communication(device, f"Message from {device}")
        
        # Wait for processing
        time.sleep(0.1)
        
        # Test device filtering
        device_001_events = self.monitor.get_event_history(device_serial="device_001")
        self.assertTrue(all(event.device_serial == "device_001" for event in device_001_events))
        
        # Test event type filtering
        comm_events = self.monitor.get_event_history(
            event_types=[MonitoringEvent.DEVICE_COMMUNICATION]
        )
        self.assertTrue(all(event.event_type == MonitoringEvent.DEVICE_COMMUNICATION for event in comm_events))
        
        # Test combined filtering
        device_001_comm = self.monitor.get_event_history(
            device_serial="device_001",
            event_types=[MonitoringEvent.DEVICE_COMMUNICATION]
        )
        self.assertTrue(all(
            event.device_serial == "device_001" and 
            event.event_type == MonitoringEvent.DEVICE_COMMUNICATION 
            for event in device_001_comm
        ))
    
    def test_system_state_snapshot_collection(self):
        """Test comprehensive system state snapshot collection"""
        self.monitor.start_monitoring()
        
        # Setup some system state
        self.monitor.log_test_started(self.test_device_serial, "snapshot_test", total_tests=1)
        
        # Add some communication history
        command = TestCommand(CommandType.EXECUTE_TEST, 1, {"test": "snapshot"})
        correlation_id = self.monitor.log_command_sent(self.test_device_serial, command)
        
        # Trigger failure to capture snapshot
        test_step = TestStep("snapshot_test", TestType.SYSTEM_STRESS_TEST, {})
        execution = TestExecution(
            step=test_step,
            device_serial=self.test_device_serial,
            status=TestStatus.FAILED,
            error_message="Snapshot test failure"
        )
        
        self.monitor.log_test_failed(self.test_device_serial, "snapshot_test", execution)
        
        # Wait for snapshot capture
        time.sleep(0.2)
        
        # Verify snapshot
        snapshots = self.monitor.get_system_snapshots(self.test_device_serial)
        self.assertEqual(len(snapshots), 1)
        
        snapshot = snapshots[0]
        
        # Verify snapshot completeness
        self.assertIsNotNone(snapshot.system_state)
        self.assertIsInstance(snapshot.device_logs, list)
        self.assertIsInstance(snapshot.communication_history, list)
        self.assertIsInstance(snapshot.performance_metrics, dict)
        
        # Check performance metrics
        self.assertIn('elapsed_time', snapshot.performance_metrics)
        self.assertIn('tests_per_second', snapshot.performance_metrics)
        self.assertIn('success_rate', snapshot.performance_metrics)
        self.assertIn('completion_percentage', snapshot.performance_metrics)
    
    def test_concurrent_monitoring(self):
        """Test monitoring with concurrent operations"""
        self.monitor.start_monitoring()
        
        # Function to simulate concurrent test operations
        def simulate_device_tests(device_serial, test_count):
            for i in range(test_count):
                self.monitor.log_test_started(device_serial, f"concurrent_test_{i}")
                time.sleep(0.01)  # Small delay
                
                test_step = TestStep(f"concurrent_test_{i}", TestType.USB_COMMUNICATION_TEST, {})
                execution = TestExecution(
                    step=test_step,
                    device_serial=device_serial,
                    status=TestStatus.COMPLETED
                )
                self.monitor.log_test_completed(device_serial, f"concurrent_test_{i}", execution)
        
        # Run concurrent operations
        threads = []
        devices = ["device_001", "device_002", "device_003"]
        
        for device in devices:
            thread = threading.Thread(target=simulate_device_tests, args=(device, 5))
            threads.append(thread)
            thread.start()
        
        # Wait for all threads to complete
        for thread in threads:
            thread.join()
        
        # Wait for event processing
        time.sleep(0.2)
        
        # Verify all events were processed
        for device in devices:
            progress = self.monitor.get_device_progress(device)
            self.assertIsNotNone(progress)
            self.assertEqual(progress.completed_tests, 5)
            self.assertEqual(progress.success_count, 5)


class TestMonitoringIntegration(unittest.TestCase):
    """Integration tests for monitoring with other components"""
    
    def setUp(self):
        """Set up integration test fixtures"""
        self.monitor = RealTimeMonitor(log_level=LogLevel.DEBUG)
        
    def tearDown(self):
        """Clean up after integration tests"""
        if self.monitor.monitoring_active:
            self.monitor.stop_monitoring()
    
    @patch('test_framework.device_manager.hid')
    def test_command_handler_integration(self, mock_hid):
        """Test integration with command handler"""
        from test_framework.device_manager import UsbHidDeviceManager
        from test_framework.command_handler import CommandHandler
        
        # Setup mocks
        mock_device = Mock()
        mock_device.write.return_value = 64
        mock_device.read.return_value = []
        mock_hid.Device.return_value = mock_device
        mock_hid.enumerate.return_value = [{
            'vendor_id': 0x2E8A,
            'product_id': 0x000A,
            'serial_number': 'TEST_DEVICE',
            'manufacturer_string': 'Test',
            'product_string': 'Test Device',
            'path': b'/test/path'
        }]
        
        # Create integrated components
        device_manager = UsbHidDeviceManager()
        command_handler = CommandHandler(device_manager, monitor=self.monitor)
        
        self.monitor.start_monitoring()
        
        # Connect to device
        device_manager.discover_devices()
        device_manager.connect_device('TEST_DEVICE')
        
        # Send command through handler
        command = command_handler.create_system_state_query("health")
        success = command_handler.send_command('TEST_DEVICE', command)
        
        self.assertTrue(success)
        
        # Wait for monitoring
        time.sleep(0.1)
        
        # Verify monitoring captured the communication
        comm_logs = self.monitor.get_communication_logs('TEST_DEVICE')
        self.assertGreater(len(comm_logs), 0)
        
        sent_log = next((log for log in comm_logs if log.direction == 'sent'), None)
        self.assertIsNotNone(sent_log)
        self.assertEqual(sent_log.message_type, 'SYSTEM_STATE_QUERY')
    
    def test_test_sequencer_integration(self):
        """Test integration with test sequencer"""
        from test_framework.test_sequencer import TestSequencer, TestConfiguration, TestStep
        from test_framework.device_manager import UsbHidDeviceManager
        from test_framework.command_handler import CommandHandler
        
        # Create mock components
        device_manager = Mock(spec=UsbHidDeviceManager)
        command_handler = Mock(spec=CommandHandler)
        
        # Setup mock responses
        device_manager.get_connected_devices.return_value = ['TEST_DEVICE']
        
        mock_response = TestResponse(
            command_id=1,
            status=ResponseStatus.SUCCESS,
            response_type="test_result",
            data={"result": "passed"},
            timestamp=time.time()
        )
        command_handler.send_command_and_wait.return_value = mock_response
        
        # Create sequencer with monitor
        sequencer = TestSequencer(device_manager, command_handler, monitor=self.monitor)
        
        self.monitor.start_monitoring()
        
        # Create simple test config
        config = TestConfiguration(
            name="Integration Test",
            description="Test monitoring integration",
            steps=[
                TestStep(
                    name="integration_test",
                    test_type=TestType.USB_COMMUNICATION_TEST,
                    parameters={"message_count": 1}
                )
            ]
        )
        
        # Execute test sequence
        results = sequencer.execute_test_sequence(config, ['TEST_DEVICE'])
        
        # Wait for monitoring
        time.sleep(0.1)
        
        # Verify monitoring captured test execution
        progress = self.monitor.get_device_progress('TEST_DEVICE')
        self.assertIsNotNone(progress)
        self.assertEqual(progress.completed_tests, 1)
        self.assertEqual(progress.success_count, 1)
        
        # Verify events were logged
        events = self.monitor.get_event_history('TEST_DEVICE')
        test_events = [e for e in events if e.event_type in [
            MonitoringEvent.TEST_STARTED, 
            MonitoringEvent.TEST_COMPLETED
        ]]
        self.assertGreaterEqual(len(test_events), 2)


if __name__ == '__main__':
    unittest.main()  
  
    def test_enhanced_failure_analysis(self):
        """Test enhanced failure analysis capabilities"""
        self.monitor.start_monitoring()
        
        # Create multiple failure scenarios
        devices = ["device_001", "device_002"]
        test_scenarios = [
            ("timeout_test", "Test timeout occurred"),
            ("communication_test", "USB communication failed"),
            ("hardware_test", "ADC reading invalid"),
            ("timeout_test", "Another timeout occurred")  # Repeat for pattern analysis
        ]
        
        for i, (test_name, error_msg) in enumerate(test_scenarios):
            device = devices[i % len(devices)]
            
            test_step = TestStep(test_name, TestType.USB_COMMUNICATION_TEST, {})
            execution = TestExecution(
                step=test_step,
                device_serial=device,
                status=TestStatus.FAILED,
                error_message=error_msg
            )
            
            self.monitor.log_test_failed(device, test_name, execution)
            time.sleep(0.01)  # Small delay between failures
        
        # Wait for processing
        time.sleep(0.2)
        
        # Get enhanced analysis
        analysis = self.monitor.get_enhanced_failure_analysis()
        
        # Verify enhanced analysis structure
        self.assertIn('failure_timeline', analysis)
        self.assertIn('failure_patterns', analysis)
        self.assertIn('communication_issues', analysis)
        self.assertIn('error_categories', analysis)
        self.assertIn('recovery_suggestions', analysis)
        
        # Check failure patterns
        self.assertEqual(analysis['failure_patterns']['timeout_test']['count'], 2)
        self.assertEqual(analysis['failure_patterns']['communication_test']['count'], 1)
        
        # Check error categorization
        timeout_pattern = analysis['failure_patterns']['timeout_test']
        self.assertIn('timeout', timeout_pattern['error_types'])
        
        # Check recovery suggestions
        self.assertIsInstance(analysis['recovery_suggestions'], list)
        self.assertGreater(len(analysis['recovery_suggestions']), 0)
    
    def test_real_time_debug_info(self):
        """Test real-time debugging information collection"""
        self.monitor.start_monitoring()
        
        # Setup test scenario
        self.monitor.log_test_started(self.test_device_serial, "debug_test", total_tests=2)
        
        # Add some communication
        command = TestCommand(CommandType.SYSTEM_STATE_QUERY, 1, {"query": "health"})
        correlation_id = self.monitor.log_command_sent(self.test_device_serial, command)
        
        response = TestResponse(1, ResponseStatus.SUCCESS, "health_data", {"status": "ok"}, time.time())
        self.monitor.log_response_received(self.test_device_serial, response, correlation_id)
        
        # Wait for processing
        time.sleep(0.1)
        
        # Get debug info
        debug_info = self.monitor.get_real_time_debug_info(self.test_device_serial)
        
        # Verify structure
        self.assertIn('monitoring_status', debug_info)
        self.assertIn('system_health', debug_info)
        self.assertIn('communication_stats', debug_info)
        self.assertIn('recent_activity', debug_info)
        
        # Check monitoring status
        self.assertTrue(debug_info['monitoring_status']['active'])
        self.assertEqual(debug_info['monitoring_status']['log_level'], LogLevel.DEBUG.value)
        
        # Check system health
        self.assertIn(self.test_device_serial, debug_info['system_health'])
        device_health = debug_info['system_health'][self.test_device_serial]
        self.assertEqual(device_health['current_test'], 'debug_test')
        self.assertIn('health_status', device_health)
        
        # Check communication stats
        self.assertIn(self.test_device_serial, debug_info['communication_stats'])
        comm_stats = debug_info['communication_stats'][self.test_device_serial]
        self.assertEqual(comm_stats['sent_commands'], 1)
        self.assertEqual(comm_stats['received_responses'], 1)
    
    def test_protocol_debugging_features(self):
        """Test enhanced protocol debugging features"""
        # Create monitor with debug level for protocol debugging
        debug_monitor = RealTimeMonitor(log_level=LogLevel.DEBUG)
        debug_monitor.start_monitoring()
        
        try:
            # Create command with protocol details
            command = TestCommand(CommandType.EXECUTE_TEST, 1, {"test_type": "protocol_test"})
            correlation_id = debug_monitor.log_command_sent(self.test_device_serial, command)
            
            # Verify protocol details were captured
            comm_logs = debug_monitor.get_communication_logs(self.test_device_serial)
            sent_log = next(log for log in comm_logs if log.direction == 'sent')
            
            self.assertIsNotNone(sent_log.protocol_details)
            self.assertIn('command_type_value', sent_log.protocol_details)
            self.assertIn('payload_json', sent_log.protocol_details)
            self.assertIn('checksum', sent_log.protocol_details)
            
            # Test response with latency calculation
            time.sleep(0.1)  # Simulate processing delay
            response = TestResponse(1, ResponseStatus.SUCCESS, "test_result", {"result": "passed"}, time.time())
            debug_monitor.log_response_received(self.test_device_serial, response, correlation_id)
            
            # Verify latency was calculated
            comm_logs = debug_monitor.get_communication_logs(self.test_device_serial)
            received_log = next(log for log in comm_logs if log.direction == 'received')
            
            self.assertIsNotNone(received_log.latency_ms)
            self.assertGreater(received_log.latency_ms, 50)  # Should be > 50ms due to sleep
            self.assertIn('latency_ms', received_log.protocol_details)
            
        finally:
            debug_monitor.stop_monitoring()
    
    def test_health_checks_and_periodic_reports(self):
        """Test health checking and periodic status reporting"""
        # Create monitor with short intervals for testing
        monitor = RealTimeMonitor(log_level=LogLevel.VERBOSE)
        monitor.health_check_interval = 0.5  # 500ms for testing
        monitor.periodic_status_interval = 0.3  # 300ms for testing
        monitor.start_monitoring()
        
        try:
            # Setup test scenario
            monitor.log_test_started(self.test_device_serial, "health_test", total_tests=5)
            
            # Wait for health checks and status reports
            time.sleep(1.0)
            
            # Verify health status was updated
            progress = monitor.get_device_progress(self.test_device_serial)
            self.assertIsNotNone(progress.health_status)
            self.assertIn(progress.health_status, ['healthy', 'warning', 'error'])
            
            # Simulate stalled test (no activity)
            progress.last_activity_time = time.time() - 70.0  # 70 seconds ago
            
            # Wait for health check
            time.sleep(0.6)
            
            # Health status should be warning or error
            updated_progress = monitor.get_device_progress(self.test_device_serial)
            self.assertIn(updated_progress.health_status, ['warning', 'error'])
            
        finally:
            monitor.stop_monitoring()
    
    def test_enhanced_device_communication_logging(self):
        """Test enhanced device communication logging with protocol analysis"""
        debug_monitor = RealTimeMonitor(log_level=LogLevel.DEBUG)
        debug_monitor.start_monitoring()
        
        try:
            # Test different message types
            test_messages = [
                ("LOG: System initialized", b"LOG: System initialized\x00"),
                ("ERROR: Test failed", b"ERROR: Test failed\x00"),
                ("TEST_RESPONSE:{\"status\":\"ok\"}", b"TEST_RESPONSE:{\"status\":\"ok\"}\x00"),
                ("DEBUG: Verbose info", b"DEBUG: Verbose info\x00")
            ]
            
            for message, raw_bytes in test_messages:
                debug_monitor.log_device_communication(
                    self.test_device_serial, message, 'received', raw_bytes
                )
            
            # Wait for processing
            time.sleep(0.1)
            
            # Verify enhanced logging
            events = debug_monitor.get_event_history(
                device_serial=self.test_device_serial,
                event_types=[MonitoringEvent.DEVICE_COMMUNICATION]
            )
            
            self.assertEqual(len(events), 4)
            
            # Check protocol analysis
            for event in events:
                self.assertIn('message_type_detected', event.data)
                self.assertIn('contains_json', event.data)
                self.assertIn('log_level_detected', event.data)
                self.assertIn('raw_bytes_hex', event.data)
            
            # Verify specific message type detection
            log_event = next(e for e in events if e.data['message'].startswith('LOG:'))
            self.assertEqual(log_event.data['message_type_detected'], 'log_message')
            self.assertEqual(log_event.data['log_level_detected'], 'info')
            
            error_event = next(e for e in events if e.data['message'].startswith('ERROR:'))
            self.assertEqual(error_event.data['message_type_detected'], 'error_message')
            self.assertEqual(error_event.data['log_level_detected'], 'error')
            
            json_event = next(e for e in events if e.data['message'].startswith('TEST_RESPONSE:'))
            self.assertEqual(json_event.data['message_type_detected'], 'test_response')
            self.assertTrue(json_event.data['contains_json'])
            
        finally:
            debug_monitor.stop_monitoring()


class TestEnhancedMonitoringIntegration(unittest.TestCase):
    """Integration tests for enhanced monitoring features"""
    
    def setUp(self):
        """Set up enhanced integration test fixtures"""
        self.monitor = RealTimeMonitor(log_level=LogLevel.DEBUG, enable_snapshots=True)
        
    def tearDown(self):
        """Clean up after enhanced integration tests"""
        if self.monitor.monitoring_active:
            self.monitor.stop_monitoring()
    
    def test_comprehensive_monitoring_workflow(self):
        """Test complete monitoring workflow with all enhanced features"""
        self.monitor.start_monitoring()
        
        # Simulate comprehensive test workflow
        device_serial = "COMPREHENSIVE_DEVICE"
        
        # Start test suite
        self.monitor.log_test_started(device_serial, "comprehensive_test_1", total_tests=3)
        
        # Simulate command/response cycle
        command = TestCommand(CommandType.EXECUTE_TEST, 1, {"test_type": "comprehensive"})
        correlation_id = self.monitor.log_command_sent(device_serial, command)
        
        # Add some device communication
        self.monitor.log_device_communication(device_serial, "LOG: Test started", 'received')
        
        # Simulate response
        time.sleep(0.05)  # Small delay for latency calculation
        response = TestResponse(1, ResponseStatus.SUCCESS, "test_result", {"result": "passed"}, time.time())
        self.monitor.log_response_received(device_serial, response, correlation_id)
        
        # Complete first test
        test_step = TestStep("comprehensive_test_1", TestType.USB_COMMUNICATION_TEST, {})
        execution = TestExecution(
            step=test_step,
            device_serial=device_serial,
            status=TestStatus.COMPLETED
        )
        self.monitor.log_test_completed(device_serial, "comprehensive_test_1", execution)
        
        # Start and fail second test
        self.monitor.log_test_started(device_serial, "comprehensive_test_2")
        
        failed_execution = TestExecution(
            step=TestStep("comprehensive_test_2", TestType.PEMF_TIMING_VALIDATION, {}),
            device_serial=device_serial,
            status=TestStatus.FAILED,
            error_message="Timing validation failed"
        )
        self.monitor.log_test_failed(device_serial, "comprehensive_test_2", failed_execution)
        
        # Wait for processing
        time.sleep(0.2)
        
        # Verify comprehensive monitoring data
        progress = self.monitor.get_device_progress(device_serial)
        self.assertEqual(progress.completed_tests, 2)
        self.assertEqual(progress.success_count, 1)
        self.assertEqual(progress.failure_count, 1)
        
        # Verify communication logs
        comm_logs = self.monitor.get_communication_logs(device_serial)
        self.assertGreater(len(comm_logs), 0)
        
        # Verify system snapshots
        snapshots = self.monitor.get_system_snapshots(device_serial)
        self.assertEqual(len(snapshots), 1)  # One failure snapshot
        
        # Verify enhanced analysis
        analysis = self.monitor.get_enhanced_failure_analysis(device_serial)
        self.assertEqual(analysis['total_failures'], 1)
        self.assertIn('comprehensive_test_2', analysis['failure_patterns'])
        
        # Verify debug info
        debug_info = self.monitor.get_real_time_debug_info(device_serial)
        self.assertIn(device_serial, debug_info['system_health'])
        self.assertIn(device_serial, debug_info['communication_stats'])


if __name__ == '__main__':
    unittest.main()