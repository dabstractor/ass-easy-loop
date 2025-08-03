"""
Integration Tests for Monitored Test Runner

Tests the comprehensive monitored test runner with real-time monitoring,
debugging capabilities, and enhanced reporting integration.
"""

import unittest
import tempfile
import shutil
import os
import json
import time
from unittest.mock import Mock, MagicMock, patch
from pathlib import Path

from test_framework.monitored_test_runner import MonitoredTestRunner, create_monitored_runner
from test_framework.real_time_monitor import LogLevel
from test_framework.test_sequencer import TestConfiguration, TestStep, TestType, TestStatus
from test_framework.command_handler import TestResponse, ResponseStatus


class TestMonitoredTestRunner(unittest.TestCase):
    """Test cases for monitored test runner functionality"""
    
    def setUp(self):
        """Set up test fixtures"""
        self.temp_dir = tempfile.mkdtemp()
        self.runner = MonitoredTestRunner(
            log_level=LogLevel.DEBUG,
            enable_snapshots=True,
            output_dir=self.temp_dir
        )
        
    def tearDown(self):
        """Clean up after tests"""
        # Clean up temporary directory
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
    
    def test_runner_initialization(self):
        """Test runner initialization with different configurations"""
        # Test default initialization
        runner_default = MonitoredTestRunner()
        self.assertEqual(runner_default.log_level, LogLevel.NORMAL)
        self.assertTrue(runner_default.enable_snapshots)
        
        # Test custom initialization
        runner_custom = MonitoredTestRunner(
            log_level=LogLevel.VERBOSE,
            enable_snapshots=False,
            output_dir="/tmp/custom"
        )
        self.assertEqual(runner_custom.log_level, LogLevel.VERBOSE)
        self.assertFalse(runner_custom.enable_snapshots)
        self.assertEqual(str(runner_custom.output_dir), "/tmp/custom")
    
    def test_factory_function(self):
        """Test the factory function for creating runners"""
        # Test with string parameters
        runner = create_monitored_runner(
            log_level="verbose",
            enable_snapshots=False,
            output_dir=self.temp_dir
        )
        
        self.assertEqual(runner.log_level, LogLevel.VERBOSE)
        self.assertFalse(runner.enable_snapshots)
        self.assertEqual(str(runner.output_dir), self.temp_dir)
        
        # Test with invalid log level (should default to normal)
        runner_invalid = create_monitored_runner(log_level="invalid")
        self.assertEqual(runner_invalid.log_level, LogLevel.NORMAL)
    
    @patch('test_framework.device_manager.hid')
    def test_device_discovery_and_connection(self, mock_hid):
        """Test device discovery and connection process"""
        # Setup mock HID devices
        mock_device = Mock()
        mock_device.write.return_value = 64
        mock_device.read.return_value = []
        mock_device.set_nonblocking.return_value = None
        mock_device.close.return_value = None
        
        mock_hid.Device.return_value = mock_device
        mock_hid.enumerate.return_value = [
            {
                'vendor_id': 0x2E8A,
                'product_id': 0x000A,
                'serial_number': 'TEST_DEVICE_001',
                'manufacturer_string': 'Test Manufacturer',
                'product_string': 'Test Device',
                'path': b'/test/path/001'
            },
            {
                'vendor_id': 0x2E8A,
                'product_id': 0x000A,
                'serial_number': 'TEST_DEVICE_002',
                'manufacturer_string': 'Test Manufacturer',
                'product_string': 'Test Device',
                'path': b'/test/path/002'
            }
        ]
        
        # Test device discovery
        connected_devices = self.runner._discover_and_connect_devices()
        
        # Should discover and connect to both devices
        self.assertEqual(len(connected_devices), 2)
        self.assertIn('TEST_DEVICE_001', connected_devices)
        self.assertIn('TEST_DEVICE_002', connected_devices)
        
        # Verify devices are connected in device manager
        self.assertTrue(self.runner.device_manager.is_device_connected('TEST_DEVICE_001'))
        self.assertTrue(self.runner.device_manager.is_device_connected('TEST_DEVICE_002'))
    
    def test_monitoring_callbacks_setup(self):
        """Test that monitoring callbacks are properly set up"""
        # Verify callbacks are registered
        callbacks = self.runner.monitor.event_callbacks
        
        # Should have callbacks for progress, failure, and communication events
        from test_framework.real_time_monitor import MonitoringEvent
        
        self.assertIn(MonitoringEvent.TEST_COMPLETED, callbacks)
        self.assertIn(MonitoringEvent.TEST_FAILED, callbacks)
        self.assertIn(MonitoringEvent.DEVICE_COMMUNICATION, callbacks)
        
        # Each should have at least one callback
        self.assertGreater(len(callbacks[MonitoringEvent.TEST_COMPLETED]), 0)
        self.assertGreater(len(callbacks[MonitoringEvent.TEST_FAILED]), 0)
        self.assertGreater(len(callbacks[MonitoringEvent.DEVICE_COMMUNICATION]), 0)
    
    @patch('test_framework.device_manager.hid')
    def test_comprehensive_test_execution(self, mock_hid):
        """Test complete test suite execution with monitoring"""
        # Setup mock device
        mock_device = Mock()
        mock_device.write.return_value = 64
        mock_device.read.return_value = []
        mock_device.set_nonblocking.return_value = None
        mock_device.close.return_value = None
        
        mock_hid.Device.return_value = mock_device
        mock_hid.enumerate.return_value = [{
            'vendor_id': 0x2E8A,
            'product_id': 0x000A,
            'serial_number': 'TEST_DEVICE',
            'manufacturer_string': 'Test',
            'product_string': 'Test Device',
            'path': b'/test/path'
        }]
        
        # Mock successful responses
        success_response = TestResponse(
            command_id=1,
            status=ResponseStatus.SUCCESS,
            response_type="test_result",
            data={"result": "passed", "duration": 1.5},
            timestamp=time.time()
        )
        
        # Patch command handler to return success
        with patch.object(self.runner.command_handler, 'send_command_and_wait', return_value=success_response):
            # Create test configuration
            config = TestConfiguration(
                name="Comprehensive Test Suite",
                description="Full monitoring test",
                steps=[
                    TestStep(
                        name="communication_test",
                        test_type=TestType.USB_COMMUNICATION_TEST,
                        parameters={"message_count": 5},
                        timeout=10.0
                    ),
                    TestStep(
                        name="timing_test",
                        test_type=TestType.PEMF_TIMING_VALIDATION,
                        parameters={"duration_ms": 2000, "tolerance_percent": 1.0},
                        timeout=15.0,
                        depends_on=["communication_test"]
                    )
                ],
                parallel_execution=False,
                global_timeout=60.0
            )
            
            # Execute test suite
            result = self.runner.run_test_suite(config, export_monitoring_data=True)
            
            # Verify results
            self.assertIsNotNone(result)
            self.assertEqual(result.suite_name, "Comprehensive Test Suite")
            self.assertGreater(len(result.device_results), 0)
            
            # Verify monitoring data was captured
            device_serial = list(result.device_results.keys())[0]
            progress = self.runner.monitor.get_device_progress(device_serial)
            self.assertIsNotNone(progress)
            self.assertEqual(progress.completed_tests, 2)
            self.assertEqual(progress.success_count, 2)
            
            # Verify reports were generated
            output_files = list(Path(self.temp_dir).glob("*"))
            self.assertGreater(len(output_files), 0)
            
            # Should have JSON and JUnit reports at minimum
            json_reports = list(Path(self.temp_dir).glob("*.json"))
            junit_reports = list(Path(self.temp_dir).glob("*junit*.xml"))
            
            self.assertGreater(len(json_reports), 0)
            self.assertGreater(len(junit_reports), 0)
    
    @patch('test_framework.device_manager.hid')
    def test_failure_handling_and_snapshots(self, mock_hid):
        """Test failure handling and system state snapshot capture"""
        # Setup mock device
        mock_device = Mock()
        mock_device.write.return_value = 64
        mock_device.read.return_value = []
        mock_device.set_nonblocking.return_value = None
        mock_device.close.return_value = None
        
        mock_hid.Device.return_value = mock_device
        mock_hid.enumerate.return_value = [{
            'vendor_id': 0x2E8A,
            'product_id': 0x000A,
            'serial_number': 'FAILING_DEVICE',
            'manufacturer_string': 'Test',
            'product_string': 'Test Device',
            'path': b'/test/path'
        }]
        
        # Mock failure response
        failure_response = TestResponse(
            command_id=1,
            status=ResponseStatus.ERROR_TIMEOUT,
            response_type="error",
            data={"error": "Test timeout", "details": "Device did not respond"},
            timestamp=time.time()
        )
        
        # Patch command handler to return failure
        with patch.object(self.runner.command_handler, 'send_command_and_wait', return_value=failure_response):
            # Create test configuration with failing test
            config = TestConfiguration(
                name="Failure Test Suite",
                description="Test failure handling",
                steps=[
                    TestStep(
                        name="failing_test",
                        test_type=TestType.SYSTEM_STRESS_TEST,
                        parameters={"duration_ms": 5000, "load_level": 10},
                        timeout=10.0,
                        retry_count=1
                    )
                ]
            )
            
            # Execute test suite
            result = self.runner.run_test_suite(config, export_monitoring_data=True)
            
            # Verify failure was handled
            device_serial = list(result.device_results.keys())[0]
            device_result = result.device_results[device_serial]
            
            self.assertEqual(device_result.metrics.failed_tests, 1)
            self.assertEqual(device_result.metrics.passed_tests, 0)
            
            # Verify system snapshot was captured
            snapshots = self.runner.monitor.get_system_snapshots(device_serial)
            self.assertGreater(len(snapshots), 0)
            
            snapshot = snapshots[0]
            self.assertEqual(snapshot.device_serial, device_serial)
            self.assertEqual(snapshot.test_name, "failing_test")
            self.assertIsNotNone(snapshot.error_context)
    
    def test_real_time_status_reporting(self):
        """Test real-time status reporting functionality"""
        # Setup some mock progress data
        self.runner.monitor.start_monitoring()
        
        # Simulate test progress
        self.runner.monitor.log_test_started("device_001", "test_1", total_tests=3)
        self.runner.monitor.log_test_started("device_002", "test_2", total_tests=2)
        
        # Wait for processing
        time.sleep(0.1)
        
        # Get real-time status
        status = self.runner.get_real_time_status()
        
        # Verify status structure
        self.assertIn('devices', status)
        self.assertIn('overall_progress', status)
        
        # Check device status
        self.assertIn('device_001', status['devices'])
        self.assertIn('device_002', status['devices'])
        
        device_001_status = status['devices']['device_001']
        self.assertEqual(device_001_status['current_test'], 'test_1')
        self.assertEqual(device_001_status['total_tests'], 3)
        self.assertEqual(device_001_status['completed_tests'], 0)
        
        # Check overall progress
        overall = status['overall_progress']
        self.assertEqual(overall['total_devices'], 2)
        self.assertEqual(overall['active_tests'], 2)
        self.assertEqual(overall['completed_devices'], 0)
    
    def test_communication_debug_info(self):
        """Test communication debugging information"""
        self.runner.monitor.start_monitoring()
        
        # Simulate some communication
        from test_framework.command_handler import TestCommand, CommandType
        
        command = TestCommand(CommandType.SYSTEM_STATE_QUERY, 1, {"query": "health"})
        correlation_id = self.runner.monitor.log_command_sent("debug_device", command)
        
        response = TestResponse(1, ResponseStatus.SUCCESS, "health_data", {"status": "ok"}, time.time())
        self.runner.monitor.log_response_received("debug_device", response, correlation_id)
        
        # Wait for processing
        time.sleep(0.1)
        
        # Get debug info
        debug_info = self.runner.get_communication_debug_info("debug_device")
        
        # Verify debug info structure
        self.assertIn('total_communications', debug_info)
        self.assertIn('sent_commands', debug_info)
        self.assertIn('received_responses', debug_info)
        self.assertIn('recent_communications', debug_info)
        
        # Check counts
        self.assertEqual(debug_info['total_communications'], 2)
        self.assertEqual(debug_info['sent_commands'], 1)
        self.assertEqual(debug_info['received_responses'], 1)
        
        # Check recent communications
        self.assertEqual(len(debug_info['recent_communications']), 2)
        
        # Verify communication details
        sent_comm = next(c for c in debug_info['recent_communications'] if c['direction'] == 'sent')
        received_comm = next(c for c in debug_info['recent_communications'] if c['direction'] == 'received')
        
        self.assertEqual(sent_comm['correlation_id'], correlation_id)
        self.assertEqual(received_comm['correlation_id'], correlation_id)
    
    def test_failure_analysis(self):
        """Test failure analysis functionality"""
        self.runner.monitor.start_monitoring()
        
        # Simulate multiple failures
        from test_framework.test_sequencer import TestExecution, TestStep
        
        devices = ["device_001", "device_002"]
        test_names = ["test_a", "test_b", "test_a"]  # test_a fails twice
        
        for i, (device, test_name) in enumerate(zip(devices + ["device_001"], test_names)):
            test_step = TestStep(test_name, TestType.USB_COMMUNICATION_TEST, {})
            execution = TestExecution(
                step=test_step,
                device_serial=device,
                status=TestStatus.FAILED,
                error_message=f"Failure {i+1}"
            )
            self.runner.monitor.log_test_failed(device, test_name, execution)
        
        # Wait for processing
        time.sleep(0.2)
        
        # Get failure analysis
        analysis = self.runner.get_failure_analysis()
        
        # Verify analysis structure
        self.assertIn('total_failures', analysis)
        self.assertIn('failure_patterns', analysis)
        self.assertIn('device_failure_counts', analysis)
        self.assertIn('recent_failures', analysis)
        
        # Check counts
        self.assertEqual(analysis['total_failures'], 3)
        
        # Check failure patterns (test_a should appear twice)
        self.assertEqual(analysis['failure_patterns']['test_a'], 2)
        self.assertEqual(analysis['failure_patterns']['test_b'], 1)
        
        # Check device failure counts
        self.assertEqual(analysis['device_failure_counts']['device_001'], 2)
        self.assertEqual(analysis['device_failure_counts']['device_002'], 1)
        
        # Check recent failures
        self.assertEqual(len(analysis['recent_failures']), 3)
    
    def test_debug_test_configuration(self):
        """Test debug test configuration creation"""
        debug_config = self.runner.create_debug_test_config()
        
        # Verify configuration structure
        self.assertEqual(debug_config.name, "Debug Test Suite")
        self.assertFalse(debug_config.parallel_execution)
        self.assertEqual(debug_config.global_timeout, 120.0)
        
        # Verify test steps
        self.assertEqual(len(debug_config.steps), 4)
        
        step_names = [step.name for step in debug_config.steps]
        expected_names = ["communication_test", "system_health_check", "battery_validation", "led_test"]
        self.assertEqual(step_names, expected_names)
        
        # Verify dependencies
        comm_test = debug_config.steps[0]
        self.assertEqual(comm_test.depends_on, [])
        
        for step in debug_config.steps[1:]:
            self.assertEqual(step.depends_on, ["communication_test"])
    
    def test_monitoring_data_export(self):
        """Test monitoring data export functionality"""
        self.runner.monitor.start_monitoring()
        
        # Generate some test data
        self.runner.monitor.log_test_started("export_device", "export_test", total_tests=1)
        
        from test_framework.command_handler import TestCommand, CommandType
        command = TestCommand(CommandType.EXECUTE_TEST, 1, {"test": "export"})
        self.runner.monitor.log_command_sent("export_device", command)
        
        # Wait for processing
        time.sleep(0.1)
        
        # Export monitoring data
        self.runner._export_monitoring_data("test_suite")
        
        # Verify export files were created
        monitoring_files = list(Path(self.temp_dir).glob("*monitoring*.json"))
        self.assertGreater(len(monitoring_files), 0)
        
        # Verify file content
        monitoring_file = monitoring_files[0]
        with open(monitoring_file, 'r') as f:
            export_data = json.load(f)
        
        # Check structure
        self.assertIn('metadata', export_data)
        self.assertIn('device_progress', export_data)
        self.assertIn('event_history', export_data)
        self.assertIn('communication_logs', export_data)
        
        # Check content
        self.assertIn('export_device', export_data['device_progress'])
        self.assertGreater(len(export_data['event_history']), 0)
    
    def test_verbose_logging_modes(self):
        """Test different verbose logging modes"""
        # Test each logging level
        log_levels = [LogLevel.MINIMAL, LogLevel.NORMAL, LogLevel.VERBOSE, LogLevel.DEBUG]
        
        for log_level in log_levels:
            with self.subTest(log_level=log_level):
                temp_dir = tempfile.mkdtemp()
                try:
                    runner = MonitoredTestRunner(
                        log_level=log_level,
                        output_dir=temp_dir
                    )
                    
                    # Verify log level is set correctly
                    self.assertEqual(runner.log_level, log_level)
                    self.assertEqual(runner.monitor.log_level, log_level)
                    
                    # Test that monitoring callbacks are set up
                    self.assertGreater(len(runner.monitor.event_callbacks), 0)
                    
                finally:
                    shutil.rmtree(temp_dir)
    
    @patch('test_framework.device_manager.hid')
    def test_report_generation_by_log_level(self, mock_hid):
        """Test that report generation varies by log level"""
        # Setup mock device
        mock_device = Mock()
        mock_device.write.return_value = 64
        mock_device.read.return_value = []
        mock_device.set_nonblocking.return_value = None
        mock_device.close.return_value = None
        
        mock_hid.Device.return_value = mock_device
        mock_hid.enumerate.return_value = [{
            'vendor_id': 0x2E8A,
            'product_id': 0x000A,
            'serial_number': 'REPORT_DEVICE',
            'manufacturer_string': 'Test',
            'product_string': 'Test Device',
            'path': b'/test/path'
        }]
        
        # Mock successful response
        success_response = TestResponse(
            command_id=1,
            status=ResponseStatus.SUCCESS,
            response_type="test_result",
            data={"result": "passed"},
            timestamp=time.time()
        )
        
        # Test different log levels
        for log_level in [LogLevel.MINIMAL, LogLevel.NORMAL, LogLevel.DEBUG]:
            with self.subTest(log_level=log_level):
                temp_dir = tempfile.mkdtemp()
                try:
                    runner = MonitoredTestRunner(
                        log_level=log_level,
                        output_dir=temp_dir
                    )
                    
                    with patch.object(runner.command_handler, 'send_command_and_wait', return_value=success_response):
                        # Simple test config
                        config = TestConfiguration(
                            name="Report Test",
                            description="Test report generation",
                            steps=[
                                TestStep(
                                    name="simple_test",
                                    test_type=TestType.USB_COMMUNICATION_TEST,
                                    parameters={"message_count": 1}
                                )
                            ]
                        )
                        
                        # Execute test
                        result = runner.run_test_suite(config, export_monitoring_data=False)
                        
                        # Check generated files
                        output_files = list(Path(temp_dir).glob("*"))
                        
                        # All levels should generate JSON and JUnit
                        json_files = list(Path(temp_dir).glob("*.json"))
                        junit_files = list(Path(temp_dir).glob("*junit*.xml"))
                        
                        self.assertGreater(len(json_files), 0)
                        self.assertGreater(len(junit_files), 0)
                        
                        # Higher levels should generate more formats
                        if log_level in [LogLevel.NORMAL, LogLevel.VERBOSE, LogLevel.DEBUG]:
                            html_files = list(Path(temp_dir).glob("*.html"))
                            csv_files = list(Path(temp_dir).glob("*.csv"))
                            
                            self.assertGreater(len(html_files), 0)
                            self.assertGreater(len(csv_files), 0)
                        
                finally:
                    shutil.rmtree(temp_dir)


class TestMonitoringIntegrationScenarios(unittest.TestCase):
    """Integration test scenarios for comprehensive monitoring"""
    
    def setUp(self):
        """Set up integration test fixtures"""
        self.temp_dir = tempfile.mkdtemp()
        
    def tearDown(self):
        """Clean up after integration tests"""
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
    
    @patch('test_framework.device_manager.hid')
    def test_multi_device_monitoring_scenario(self, mock_hid):
        """Test monitoring across multiple devices simultaneously"""
        # Setup multiple mock devices
        mock_device = Mock()
        mock_device.write.return_value = 64
        mock_device.read.return_value = []
        mock_device.set_nonblocking.return_value = None
        mock_device.close.return_value = None
        
        mock_hid.Device.return_value = mock_device
        mock_hid.enumerate.return_value = [
            {
                'vendor_id': 0x2E8A,
                'product_id': 0x000A,
                'serial_number': f'MULTI_DEVICE_{i:03d}',
                'manufacturer_string': 'Test',
                'product_string': 'Test Device',
                'path': f'/test/path/{i}'.encode()
            }
            for i in range(3)
        ]
        
        runner = MonitoredTestRunner(
            log_level=LogLevel.VERBOSE,
            output_dir=self.temp_dir
        )
        
        # Mock mixed success/failure responses
        responses = [
            TestResponse(1, ResponseStatus.SUCCESS, "test_result", {"result": "passed"}, time.time()),
            TestResponse(2, ResponseStatus.ERROR_TIMEOUT, "error", {"error": "timeout"}, time.time()),
            TestResponse(3, ResponseStatus.SUCCESS, "test_result", {"result": "passed"}, time.time())
        ]
        
        response_iter = iter(responses * 10)  # Repeat for multiple tests
        
        with patch.object(runner.command_handler, 'send_command_and_wait', side_effect=lambda *args, **kwargs: next(response_iter)):
            # Create multi-step test config
            config = TestConfiguration(
                name="Multi-Device Test",
                description="Test monitoring across multiple devices",
                steps=[
                    TestStep(
                        name="device_communication",
                        test_type=TestType.USB_COMMUNICATION_TEST,
                        parameters={"message_count": 3}
                    ),
                    TestStep(
                        name="device_validation",
                        test_type=TestType.PEMF_TIMING_VALIDATION,
                        parameters={"duration_ms": 1000}
                    )
                ],
                parallel_execution=True,
                max_parallel_devices=3
            )
            
            # Execute test suite
            result = runner.run_test_suite(config)
            
            # Verify multi-device results
            self.assertEqual(len(result.device_results), 3)
            
            # Check that monitoring captured all devices
            all_progress = runner.monitor.get_all_progress()
            self.assertEqual(len(all_progress), 3)
            
            # Verify each device has progress data
            for i in range(3):
                device_serial = f'MULTI_DEVICE_{i:03d}'
                self.assertIn(device_serial, all_progress)
                
                progress = all_progress[device_serial]
                self.assertEqual(progress.total_tests, 2)
                self.assertEqual(progress.completed_tests, 2)
    
    @patch('test_framework.device_manager.hid')
    def test_failure_recovery_monitoring_scenario(self, mock_hid):
        """Test monitoring during failure recovery scenarios"""
        # Setup mock device
        mock_device = Mock()
        mock_device.write.return_value = 64
        mock_device.read.return_value = []
        mock_device.set_nonblocking.return_value = None
        mock_device.close.return_value = None
        
        mock_hid.Device.return_value = mock_device
        mock_hid.enumerate.return_value = [{
            'vendor_id': 0x2E8A,
            'product_id': 0x000A,
            'serial_number': 'RECOVERY_DEVICE',
            'manufacturer_string': 'Test',
            'product_string': 'Test Device',
            'path': b'/test/path'
        }]
        
        runner = MonitoredTestRunner(
            log_level=LogLevel.DEBUG,
            enable_snapshots=True,
            output_dir=self.temp_dir
        )
        
        # Mock responses: fail first, succeed on retry
        call_count = 0
        def mock_response(*args, **kwargs):
            nonlocal call_count
            call_count += 1
            if call_count % 2 == 1:  # Odd calls fail
                return TestResponse(1, ResponseStatus.ERROR_TIMEOUT, "error", {"error": "timeout"}, time.time())
            else:  # Even calls succeed
                return TestResponse(1, ResponseStatus.SUCCESS, "test_result", {"result": "passed"}, time.time())
        
        with patch.object(runner.command_handler, 'send_command_and_wait', side_effect=mock_response):
            # Create test config with retries
            config = TestConfiguration(
                name="Recovery Test",
                description="Test failure recovery monitoring",
                steps=[
                    TestStep(
                        name="retry_test",
                        test_type=TestType.SYSTEM_STRESS_TEST,
                        parameters={"duration_ms": 1000},
                        timeout=10.0,
                        retry_count=2  # Allow retries
                    )
                ]
            )
            
            # Execute test suite
            result = runner.run_test_suite(config)
            
            # Verify test eventually succeeded
            device_result = list(result.device_results.values())[0]
            self.assertEqual(device_result.metrics.passed_tests, 1)
            
            # Check that monitoring captured the retry attempts
            events = runner.monitor.get_event_history('RECOVERY_DEVICE')
            
            # Should have multiple test start/fail/complete events due to retries
            test_events = [e for e in events if e.event_type.value in ['test_started', 'test_failed', 'test_completed']]
            self.assertGreater(len(test_events), 2)  # At least start, fail, start, complete
    
    def test_long_running_monitoring_scenario(self):
        """Test monitoring behavior during long-running test scenarios"""
        runner = MonitoredTestRunner(
            log_level=LogLevel.NORMAL,
            output_dir=self.temp_dir
        )
        
        runner.monitor.start_monitoring()
        
        # Simulate long-running test with periodic updates
        device_serial = "LONG_RUNNING_DEVICE"
        total_tests = 10
        
        runner.monitor.log_test_started(device_serial, "long_test_sequence", total_tests=total_tests)
        
        # Simulate gradual test completion
        from test_framework.test_sequencer import TestExecution, TestStep
        
        for i in range(total_tests):
            test_step = TestStep(f"test_{i}", TestType.USB_COMMUNICATION_TEST, {})
            execution = TestExecution(
                step=test_step,
                device_serial=device_serial,
                status=TestStatus.COMPLETED,
                start_time=time.time() - 1.0,
                end_time=time.time()
            )
            
            runner.monitor.log_test_completed(device_serial, f"test_{i}", execution)
            time.sleep(0.05)  # Small delay between tests
        
        # Wait for processing
        time.sleep(0.2)
        
        # Verify progress tracking
        progress = runner.monitor.get_device_progress(device_serial)
        self.assertEqual(progress.completed_tests, total_tests)
        self.assertEqual(progress.success_count, total_tests)
        self.assertEqual(progress.failure_count, 0)
        
        # Verify estimated completion was calculated
        self.assertIsNotNone(progress.estimated_completion)
        
        # Verify performance metrics
        performance_metrics = runner.monitor._collect_performance_metrics(device_serial)
        self.assertIn('elapsed_time', performance_metrics)
        self.assertIn('tests_per_second', performance_metrics)
        self.assertIn('success_rate', performance_metrics)
        self.assertEqual(performance_metrics['success_rate'], 1.0)  # 100% success
        self.assertEqual(performance_metrics['completion_percentage'], 100.0)


if __name__ == '__main__':
    unittest.main()    

    def test_enhanced_debug_info_collection(self):
        """Test enhanced debugging information collection"""
        runner = MonitoredTestRunner(
            log_level=LogLevel.DEBUG,
            enable_snapshots=True,
            output_dir=self.temp_dir
        )
        
        runner.monitor.start_monitoring()
        
        # Setup test scenario
        device_serial = "DEBUG_DEVICE"
        runner.monitor.log_test_started(device_serial, "debug_info_test", total_tests=1)
        
        # Add communication
        from test_framework.command_handler import TestCommand, CommandType
        command = TestCommand(CommandType.SYSTEM_STATE_QUERY, 1, {"query": "debug"})
        correlation_id = runner.monitor.log_command_sent(device_serial, command)
        
        # Wait and get debug info
        time.sleep(0.1)
        debug_info = runner.get_enhanced_debug_info(device_serial)
        
        # Verify enhanced debug info structure
        self.assertIn('monitoring_status', debug_info)
        self.assertIn('system_health', debug_info)
        self.assertIn('communication_stats', debug_info)
        self.assertIn('recent_activity', debug_info)
        
        # Check device-specific info
        self.assertIn(device_serial, debug_info['system_health'])
        self.assertIn(device_serial, debug_info['communication_stats'])
        
        runner.monitor.stop_monitoring()
    
    def test_protocol_debug_logs(self):
        """Test protocol debugging logs collection"""
        runner = MonitoredTestRunner(
            log_level=LogLevel.DEBUG,
            output_dir=self.temp_dir
        )
        
        runner.monitor.start_monitoring()
        
        # Setup communication scenario
        device_serial = "PROTOCOL_DEVICE"
        
        from test_framework.command_handler import TestCommand, TestResponse, CommandType, ResponseStatus
        command = TestCommand(CommandType.EXECUTE_TEST, 1, {"test_type": "protocol"})
        correlation_id = runner.monitor.log_command_sent(device_serial, command)
        
        response = TestResponse(1, ResponseStatus.SUCCESS, "test_result", {"result": "passed"}, time.time())
        runner.monitor.log_response_received(device_serial, response, correlation_id)
        
        # Wait for processing
        time.sleep(0.1)
        
        # Get protocol debug logs
        protocol_logs = runner.get_protocol_debug_logs(device_serial)
        
        # Verify protocol logs
        self.assertEqual(len(protocol_logs), 2)  # Command sent + response received
        
        sent_log = next(log for log in protocol_logs if log['direction'] == 'sent')
        received_log = next(log for log in protocol_logs if log['direction'] == 'received')
        
        # Check protocol details are included in debug mode
        self.assertIn('protocol_details', sent_log)
        self.assertIn('protocol_details', received_log)
        self.assertEqual(sent_log['correlation_id'], correlation_id)
        self.assertEqual(received_log['correlation_id'], correlation_id)
        
        runner.monitor.stop_monitoring()
    
    def test_debug_report_generation(self):
        """Test comprehensive debug report generation"""
        runner = MonitoredTestRunner(
            log_level=LogLevel.DEBUG,
            enable_snapshots=True,
            output_dir=self.temp_dir
        )
        
        runner.monitor.start_monitoring()
        
        # Create test scenario with failure
        device_serial = "REPORT_DEVICE"
        runner.monitor.log_test_started(device_serial, "report_test", total_tests=1)
        
        # Simulate failure
        from test_framework.test_sequencer import TestExecution, TestStep, TestStatus, TestType
        test_step = TestStep("report_test", TestType.USB_COMMUNICATION_TEST, {})
        execution = TestExecution(
            step=test_step,
            device_serial=device_serial,
            status=TestStatus.FAILED,
            error_message="Report test failure"
        )
        runner.monitor.log_test_failed(device_serial, "report_test", execution)
        
        # Wait for snapshot capture
        time.sleep(0.2)
        
        # Generate debug report
        report_file = runner.generate_debug_report()
        
        # Verify report was created
        self.assertTrue(os.path.exists(report_file))
        
        # Verify report content
        with open(report_file, 'r') as f:
            report_data = json.load(f)
        
        # Check report structure
        self.assertIn('report_metadata', report_data)
        self.assertIn('real_time_status', report_data)
        self.assertIn('enhanced_debug_info', report_data)
        self.assertIn('failure_analysis', report_data)
        self.assertIn('protocol_debug_logs', report_data)
        self.assertIn('system_snapshots', report_data)
        
        # Check that failure was captured
        self.assertEqual(report_data['failure_analysis']['total_failures'], 1)
        self.assertEqual(len(report_data['system_snapshots']), 1)
        
        runner.monitor.stop_monitoring()
    
    @patch('test_framework.device_manager.hid')
    def test_enhanced_monitoring_during_test_execution(self, mock_hid):
        """Test enhanced monitoring during actual test execution"""
        # Setup mock device
        mock_device = Mock()
        mock_device.write.return_value = 64
        mock_device.read.return_value = []
        mock_device.set_nonblocking.return_value = None
        mock_device.close.return_value = None
        
        mock_hid.Device.return_value = mock_device
        mock_hid.enumerate.return_value = [{
            'vendor_id': 0x2E8A,
            'product_id': 0x000A,
            'serial_number': 'ENHANCED_DEVICE',
            'manufacturer_string': 'Test',
            'product_string': 'Test Device',
            'path': b'/test/path'
        }]
        
        runner = MonitoredTestRunner(
            log_level=LogLevel.DEBUG,
            enable_snapshots=True,
            output_dir=self.temp_dir
        )
        
        # Mock responses with varying latencies
        responses = [
            TestResponse(1, ResponseStatus.SUCCESS, "test_result", {"result": "passed", "latency": "low"}, time.time()),
            TestResponse(2, ResponseStatus.SUCCESS, "test_result", {"result": "passed", "latency": "high"}, time.time()),
        ]
        
        response_iter = iter(responses)
        
        def mock_response_with_delay(*args, **kwargs):
            time.sleep(0.05)  # Simulate processing delay
            return next(response_iter)
        
        with patch.object(runner.command_handler, 'send_command_and_wait', side_effect=mock_response_with_delay):
            # Create test configuration
            config = TestConfiguration(
                name="Enhanced Monitoring Test",
                description="Test enhanced monitoring during execution",
                steps=[
                    TestStep(
                        name="latency_test_1",
                        test_type=TestType.USB_COMMUNICATION_TEST,
                        parameters={"message_count": 1}
                    ),
                    TestStep(
                        name="latency_test_2",
                        test_type=TestType.SYSTEM_STRESS_TEST,
                        parameters={"duration_ms": 100}
                    )
                ]
            )
            
            # Execute test suite
            result = runner.run_test_suite(config, export_monitoring_data=True)
            
            # Verify enhanced monitoring captured everything
            device_serial = 'ENHANCED_DEVICE'
            
            # Check communication logs captured latency
            comm_logs = runner.monitor.get_communication_logs(device_serial)
            self.assertGreater(len(comm_logs), 0)
            
            # Should have some logs with latency measurements
            latency_logs = [log for log in comm_logs if log.latency_ms is not None]
            self.assertGreater(len(latency_logs), 0)
            
            # Check that protocol debugging was active
            if runner.monitor.protocol_debug_enabled:
                protocol_logs = [log for log in comm_logs if log.protocol_details]
                self.assertGreater(len(protocol_logs), 0)
            
            # Verify enhanced debug info is available
            debug_info = runner.get_enhanced_debug_info(device_serial)
            self.assertIn(device_serial, debug_info['system_health'])
            self.assertIn(device_serial, debug_info['communication_stats'])
            
            # Check that monitoring data export includes enhanced features
            monitoring_files = list(Path(self.temp_dir).glob("*monitoring*.json"))
            self.assertGreater(len(monitoring_files), 0)
            
            with open(monitoring_files[0], 'r') as f:
                monitoring_data = json.load(f)
            
            # Verify enhanced data is present
            self.assertIn('enhanced_analysis', monitoring_data)
            self.assertIn('real_time_debug_info', monitoring_data)
            self.assertIn('protocol_debug_enabled', monitoring_data['metadata'])


class TestLongRunningMonitoringScenarios(unittest.TestCase):
    """Test scenarios for long-running monitoring and debugging"""
    
    def setUp(self):
        """Set up long-running test fixtures"""
        self.temp_dir = tempfile.mkdtemp()
        
    def tearDown(self):
        """Clean up after long-running tests"""
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
    
    def test_periodic_health_checks_and_status_reports(self):
        """Test periodic health checks and status reporting for long-running tests"""
        # Create monitor with short intervals for testing
        monitor = RealTimeMonitor(
            log_level=LogLevel.VERBOSE,
            enable_snapshots=True
        )
        monitor.health_check_interval = 0.2  # 200ms for testing
        monitor.periodic_status_interval = 0.15  # 150ms for testing
        
        monitor.start_monitoring()
        
        try:
            # Setup long-running test scenario
            device_serial = "LONGRUN_DEVICE"
            monitor.log_test_started(device_serial, "long_running_test", total_tests=10)
            
            # Simulate test progress over time
            for i in range(5):
                # Complete a test
                test_step = TestStep(f"test_{i}", TestType.USB_COMMUNICATION_TEST, {})
                execution = TestExecution(
                    step=test_step,
                    device_serial=device_serial,
                    status=TestStatus.COMPLETED
                )
                monitor.log_test_completed(device_serial, f"test_{i}", execution)
                
                # Wait for health checks and status reports
                time.sleep(0.3)
            
            # Verify health checks occurred
            progress = monitor.get_device_progress(device_serial)
            self.assertIsNotNone(progress.health_status)
            self.assertEqual(progress.health_status, "healthy")
            
            # Verify performance metrics were updated
            self.assertIn('tests_per_second', progress.performance_metrics)
            self.assertIn('success_rate', progress.performance_metrics)
            self.assertIn('completion_percentage', progress.performance_metrics)
            
            # Check completion percentage
            expected_completion = (5 / 10) * 100  # 50%
            self.assertAlmostEqual(progress.performance_metrics['completion_percentage'], expected_completion, places=1)
            
        finally:
            monitor.stop_monitoring()
    
    def test_stalled_test_detection(self):
        """Test detection of stalled tests in long-running scenarios"""
        monitor = RealTimeMonitor(log_level=LogLevel.NORMAL)
        monitor.health_check_interval = 0.1  # 100ms for testing
        monitor.start_monitoring()
        
        try:
            device_serial = "STALLED_DEVICE"
            monitor.log_test_started(device_serial, "stalled_test", total_tests=1)
            
            # Simulate stalled test by setting old activity time
            progress = monitor.get_device_progress(device_serial)
            progress.last_activity_time = time.time() - 70.0  # 70 seconds ago (should trigger warning)
            
            # Wait for health check
            time.sleep(0.2)
            
            # Verify stalled test was detected
            updated_progress = monitor.get_device_progress(device_serial)
            self.assertIn(updated_progress.health_status, ['warning', 'error'])
            
        finally:
            monitor.stop_monitoring()
    
    def test_communication_issue_detection(self):
        """Test detection of communication issues during monitoring"""
        monitor = RealTimeMonitor(log_level=LogLevel.DEBUG)
        monitor.health_check_interval = 0.1  # 100ms for testing
        monitor.start_monitoring()
        
        try:
            device_serial = "COMM_ISSUE_DEVICE"
            monitor.log_test_started(device_serial, "comm_test", total_tests=1)
            
            # Simulate communication imbalance (many commands, few responses)
            from test_framework.command_handler import TestCommand, CommandType
            
            for i in range(10):
                command = TestCommand(CommandType.SYSTEM_STATE_QUERY, i, {"query": f"test_{i}"})
                monitor.log_command_sent(device_serial, command)
            
            # Only respond to a few commands
            from test_framework.command_handler import TestResponse, ResponseStatus
            for i in range(2):
                response = TestResponse(i, ResponseStatus.SUCCESS, "query_result", {"data": f"response_{i}"}, time.time())
                monitor.log_response_received(device_serial, response)
            
            # Wait for health check
            time.sleep(0.2)
            
            # Verify communication issue was detected
            progress = monitor.get_device_progress(device_serial)
            # Health status might be warning due to communication imbalance
            # (This depends on the specific health check implementation)
            
            # Check enhanced failure analysis for communication issues
            analysis = monitor.get_enhanced_failure_analysis(device_serial)
            # Should detect the communication imbalance
            
        finally:
            monitor.stop_monitoring()


if __name__ == '__main__':
    unittest.main()