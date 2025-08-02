"""
Unit tests for Test Sequencer
"""

import unittest
from unittest.mock import Mock, patch, MagicMock
import time
from concurrent.futures import Future

from test_framework.test_sequencer import (
    TestSequencer, TestConfiguration, TestStep, TestExecution, TestStatus
)
from test_framework.command_handler import TestType, TestResponse, ResponseStatus
from test_framework.device_manager import UsbHidDeviceManager


class TestTestSequencer(unittest.TestCase):
    """Test cases for TestSequencer"""
    
    def setUp(self):
        """Set up test fixtures"""
        self.mock_device_manager = Mock(spec=UsbHidDeviceManager)
        self.mock_command_handler = Mock()
        
        self.test_sequencer = TestSequencer(
            device_manager=self.mock_device_manager,
            command_handler=self.mock_command_handler
        )
        
        # Mock connected devices
        self.mock_device_manager.get_connected_devices.return_value = ['TEST123', 'TEST456']
        
        # Create test configuration
        self.test_config = TestConfiguration(
            name="Test Suite",
            description="Test description",
            steps=[
                TestStep(
                    name="test1",
                    test_type=TestType.USB_COMMUNICATION_TEST,
                    parameters={"message_count": 10},
                    timeout=5.0
                ),
                TestStep(
                    name="test2",
                    test_type=TestType.PEMF_TIMING_VALIDATION,
                    parameters={"duration_ms": 1000},
                    timeout=10.0,
                    depends_on=["test1"]
                )
            ]
        )
    
    def test_execute_test_sequence_sequential(self):
        """Test sequential test execution"""
        # Mock successful responses
        success_response = Mock(spec=TestResponse)
        success_response.status = ResponseStatus.SUCCESS
        self.mock_command_handler.send_command_and_wait.return_value = success_response
        
        results = self.test_sequencer.execute_test_sequence(
            self.test_config, 
            target_devices=['TEST123']
        )
        
        self.assertIn('TEST123', results)
        device_results = results['TEST123']
        self.assertEqual(len(device_results), 2)
        
        # Verify both tests completed successfully
        self.assertEqual(device_results[0].status, TestStatus.COMPLETED)
        self.assertEqual(device_results[1].status, TestStatus.COMPLETED)
    
    def test_execute_test_sequence_parallel(self):
        """Test parallel test execution"""
        self.test_config.parallel_execution = True
        
        # Mock successful responses
        success_response = Mock(spec=TestResponse)
        success_response.status = ResponseStatus.SUCCESS
        self.mock_command_handler.send_command_and_wait.return_value = success_response
        
        with patch('concurrent.futures.ThreadPoolExecutor') as mock_executor_class:
            # Mock executor behavior
            mock_future = Mock(spec=Future)
            mock_future.result.return_value = [
                TestExecution(
                    step=self.test_config.steps[0],
                    device_serial='TEST123',
                    status=TestStatus.COMPLETED
                )
            ]
            
            mock_executor_instance = Mock()
            mock_executor_instance.submit.return_value = mock_future
            
            # Mock the context manager behavior
            mock_executor_class.return_value.__enter__ = Mock(return_value=mock_executor_instance)
            mock_executor_class.return_value.__exit__ = Mock(return_value=None)
            
            results = self.test_sequencer.execute_test_sequence(
                self.test_config,
                target_devices=['TEST123']
            )
        
        self.assertIn('TEST123', results)
    
    def test_execute_single_test_success(self):
        """Test successful single test execution"""
        execution = TestExecution(
            step=self.test_config.steps[0],
            device_serial='TEST123',
            status=TestStatus.PENDING
        )
        
        # Mock successful response
        success_response = Mock(spec=TestResponse)
        success_response.status = ResponseStatus.SUCCESS
        self.mock_command_handler.send_command_and_wait.return_value = success_response
        
        self.test_sequencer._execute_single_test(execution)
        
        self.assertEqual(execution.status, TestStatus.COMPLETED)
        self.assertIsNotNone(execution.response)
        self.assertIsNotNone(execution.start_time)
        self.assertIsNotNone(execution.end_time)
    
    def test_execute_single_test_failure(self):
        """Test single test execution failure"""
        execution = TestExecution(
            step=self.test_config.steps[0],
            device_serial='TEST123',
            status=TestStatus.PENDING
        )
        
        # Mock error response
        error_response = Mock(spec=TestResponse)
        error_response.status = ResponseStatus.ERROR_HARDWARE_FAULT
        self.mock_command_handler.send_command_and_wait.return_value = error_response
        
        self.test_sequencer._execute_single_test(execution)
        
        self.assertEqual(execution.status, TestStatus.FAILED)
        self.assertIsNotNone(execution.error_message)
    
    def test_execute_single_test_timeout(self):
        """Test single test execution timeout"""
        execution = TestExecution(
            step=self.test_config.steps[0],
            device_serial='TEST123',
            status=TestStatus.PENDING
        )
        
        # Mock timeout (no response)
        self.mock_command_handler.send_command_and_wait.return_value = None
        
        self.test_sequencer._execute_single_test(execution)
        
        self.assertEqual(execution.status, TestStatus.TIMEOUT)
        self.assertIsNotNone(execution.error_message)
    
    def test_execute_single_test_with_retry(self):
        """Test single test execution with retry logic"""
        test_step = TestStep(
            name="retry_test",
            test_type=TestType.USB_COMMUNICATION_TEST,
            parameters={},
            retry_count=2
        )
        
        execution = TestExecution(
            step=test_step,
            device_serial='TEST123',
            status=TestStatus.PENDING
        )
        
        # Mock first call fails, second succeeds
        success_response = Mock(spec=TestResponse)
        success_response.status = ResponseStatus.SUCCESS
        
        self.mock_command_handler.send_command_and_wait.side_effect = [
            None,  # First attempt times out
            success_response  # Second attempt succeeds
        ]
        
        with patch('time.sleep'):  # Speed up test
            self.test_sequencer._execute_single_test(execution)
        
        self.assertEqual(execution.status, TestStatus.COMPLETED)
        self.assertEqual(execution.retry_attempt, 1)  # Second attempt (0-indexed)
    
    def test_execute_single_test_exception(self):
        """Test single test execution handles exceptions"""
        execution = TestExecution(
            step=self.test_config.steps[0],
            device_serial='TEST123',
            status=TestStatus.PENDING
        )
        
        # Mock exception during command sending
        self.mock_command_handler.send_command_and_wait.side_effect = Exception("USB error")
        
        self.test_sequencer._execute_single_test(execution)
        
        self.assertEqual(execution.status, TestStatus.FAILED)
        self.assertIn("USB error", execution.error_message)
    
    def test_should_execute_step_no_dependencies(self):
        """Test step execution check with no dependencies"""
        execution = TestExecution(
            step=self.test_config.steps[0],  # No dependencies
            device_serial='TEST123',
            status=TestStatus.PENDING
        )
        
        should_execute = self.test_sequencer._should_execute_step(execution, [execution])
        
        self.assertTrue(should_execute)
    
    def test_should_execute_step_dependencies_met(self):
        """Test step execution check with satisfied dependencies"""
        # Create executions for both steps
        execution1 = TestExecution(
            step=self.test_config.steps[0],
            device_serial='TEST123',
            status=TestStatus.COMPLETED
        )
        
        execution2 = TestExecution(
            step=self.test_config.steps[1],  # Depends on test1
            device_serial='TEST123',
            status=TestStatus.PENDING
        )
        
        should_execute = self.test_sequencer._should_execute_step(
            execution2, 
            [execution1, execution2]
        )
        
        self.assertTrue(should_execute)
    
    def test_should_execute_step_dependencies_not_met(self):
        """Test step execution check with unsatisfied dependencies"""
        # Create executions where dependency failed
        execution1 = TestExecution(
            step=self.test_config.steps[0],
            device_serial='TEST123',
            status=TestStatus.FAILED
        )
        
        execution2 = TestExecution(
            step=self.test_config.steps[1],  # Depends on test1
            device_serial='TEST123',
            status=TestStatus.PENDING
        )
        
        should_execute = self.test_sequencer._should_execute_step(
            execution2,
            [execution1, execution2]
        )
        
        self.assertFalse(should_execute)
    
    def test_execute_setup_commands(self):
        """Test setup command execution"""
        setup_command = Mock()
        self.test_config.setup_commands = [setup_command]
        
        self.test_sequencer._execute_setup_commands(
            self.test_config,
            ['TEST123']
        )
        
        self.mock_command_handler.send_command.assert_called_with('TEST123', setup_command)
    
    def test_execute_teardown_commands(self):
        """Test teardown command execution"""
        teardown_command = Mock()
        self.test_config.teardown_commands = [teardown_command]
        
        self.test_sequencer._execute_teardown_commands(
            self.test_config,
            ['TEST123']
        )
        
        self.mock_command_handler.send_command.assert_called_with('TEST123', teardown_command)
    
    def test_get_execution_status(self):
        """Test getting execution status"""
        execution = TestExecution(
            step=self.test_config.steps[0],
            device_serial='TEST123',
            status=TestStatus.RUNNING
        )
        
        self.test_sequencer.active_executions['TEST123'] = [execution]
        
        status = self.test_sequencer.get_execution_status('TEST123')
        
        self.assertEqual(len(status), 1)
        self.assertEqual(status[0].status, TestStatus.RUNNING)
    
    def test_cancel_execution(self):
        """Test canceling execution"""
        execution = TestExecution(
            step=self.test_config.steps[0],
            device_serial='TEST123',
            status=TestStatus.RUNNING
        )
        
        self.test_sequencer.active_executions['TEST123'] = [execution]
        
        result = self.test_sequencer.cancel_execution('TEST123')
        
        self.assertTrue(result)
        self.assertEqual(execution.status, TestStatus.FAILED)
        self.assertIn("cancelled", execution.error_message)
    
    def test_create_basic_validation_config(self):
        """Test creating basic validation configuration"""
        config = self.test_sequencer.create_basic_validation_config()
        
        self.assertEqual(config.name, "Basic Device Validation")
        self.assertGreater(len(config.steps), 0)
        self.assertFalse(config.parallel_execution)
        
        # Verify dependency structure
        health_check_step = next(s for s in config.steps if s.name == "system_health_check")
        self.assertEqual(len(health_check_step.depends_on), 0)
        
        dependent_steps = [s for s in config.steps if "system_health_check" in s.depends_on]
        self.assertGreater(len(dependent_steps), 0)


if __name__ == '__main__':
    unittest.main()