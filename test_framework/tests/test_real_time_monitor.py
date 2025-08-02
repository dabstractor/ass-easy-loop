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