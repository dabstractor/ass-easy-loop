#!/usr/bin/env python3
"""
Integration tests for comprehensive test scenarios

Tests the complete test scenario execution including hardware validation,
stress testing, regression testing, performance benchmarking, and integration testing.
"""

import unittest
import time
from unittest.mock import Mock, MagicMock, patch
import sys
import os

# Add parent directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from test_framework.test_scenarios import TestScenarios, TestScenarioType, ScenarioParameters
from test_framework.comprehensive_test_runner import ComprehensiveTestRunner
from test_framework.test_sequencer import TestStatus
from test_framework.command_handler import TestType, ResponseStatus


class TestComprehensiveScenarios(unittest.TestCase):
    """Test comprehensive test scenarios"""
    
    def setUp(self):
        """Set up test fixtures"""
        self.test_scenarios = TestScenarios()
        self.parameters = ScenarioParameters(
            pemf_test_duration_ms=1000,  # Shorter for testing
            stress_test_duration_ms=2000,
            benchmark_iterations=10
        )
        
    def test_hardware_validation_suite_creation(self):
        """Test hardware validation suite creation"""
        config = self.test_scenarios.create_hardware_validation_suite()
        
        self.assertEqual(config.name, "Hardware Validation Suite")
        self.assertGreater(len(config.steps), 5)
        
        # Check for required test types
        test_types = [step.test_type for step in config.steps]
        self.assertIn(TestType.PEMF_TIMING_VALIDATION, test_types)
        self.assertIn(TestType.BATTERY_ADC_CALIBRATION, test_types)
        self.assertIn(TestType.LED_FUNCTIONALITY, test_types)
        self.assertIn(TestType.USB_COMMUNICATION_TEST, test_types)
        
        # Verify dependencies
        dependent_steps = [step for step in config.steps if step.depends_on]
        self.assertGreater(len(dependent_steps), 0)
        
    def test_stress_testing_suite_creation(self):
        """Test stress testing suite creation"""
        config = self.test_scenarios.create_stress_testing_suite()
        
        self.assertEqual(config.name, "System Stress Testing Suite")
        self.assertGreater(len(config.steps), 4)
        
        # Check for stress test types
        test_types = [step.test_type for step in config.steps]
        self.assertIn(TestType.SYSTEM_STRESS_TEST, test_types)
        
        # Verify stress test parameters
        stress_steps = [step for step in config.steps if step.test_type == TestType.SYSTEM_STRESS_TEST]
        self.assertGreater(len(stress_steps), 0)
        
        for step in stress_steps:
            self.assertIn('load_level', step.parameters)
            self.assertIn('duration_ms', step.parameters)
            
    def test_regression_testing_suite_creation(self):
        """Test regression testing suite creation"""
        config = self.test_scenarios.create_regression_testing_suite()
        
        self.assertEqual(config.name, "Firmware Regression Testing Suite")
        self.assertGreater(len(config.steps), 5)
        
        # Check for regression-specific parameters
        regression_steps = [step for step in config.steps if 'regression' in step.name.lower()]
        self.assertGreater(len(regression_steps), 3)
        
    def test_performance_benchmarking_suite_creation(self):
        """Test performance benchmarking suite creation"""
        config = self.test_scenarios.create_performance_benchmarking_suite()
        
        self.assertEqual(config.name, "Performance Benchmarking Suite")
        self.assertGreater(len(config.steps), 5)
        
        # Check for benchmark-specific parameters
        benchmark_steps = [step for step in config.steps if 'benchmark' in step.name.lower()]
        self.assertGreater(len(benchmark_steps), 3)
        
    def test_integration_testing_suite_creation(self):
        """Test integration testing suite creation"""
        config = self.test_scenarios.create_integration_testing_suite()
        
        self.assertEqual(config.name, "System Integration Testing Suite")
        self.assertGreater(len(config.steps), 5)
        
        # Check for integration-specific tests
        integration_steps = [step for step in config.steps if 'integration' in step.name.lower()]
        self.assertGreater(len(integration_steps), 3)
        
    def test_scenario_parameters_customization(self):
        """Test scenario parameter customization"""
        custom_params = ScenarioParameters(
            pemf_test_duration_ms=10000,
            stress_test_duration_ms=60000,
            benchmark_iterations=200
        )
        
        scenarios = TestScenarios(custom_params)
        config = scenarios.create_hardware_validation_suite()
        
        # Find pEMF test step and verify custom parameters
        pemf_steps = [step for step in config.steps if step.test_type == TestType.PEMF_TIMING_VALIDATION]
        self.assertGreater(len(pemf_steps), 0)
        
        # Check that at least one step uses the base duration (some may use multiples)
        found_base_duration = False
        for step in pemf_steps:
            if 'duration_ms' in step.parameters:
                if step.parameters['duration_ms'] == 10000:
                    found_base_duration = True
                    break
        self.assertTrue(found_base_duration, "Should find at least one pEMF test with base duration")
                
    def test_get_scenario_by_type(self):
        """Test getting scenarios by type"""
        for scenario_type in TestScenarioType:
            config = self.test_scenarios.get_scenario_by_type(scenario_type)
            self.assertIsNotNone(config)
            self.assertIsInstance(config.name, str)
            self.assertGreater(len(config.steps), 0)
            
    def test_custom_scenario_creation(self):
        """Test custom scenario creation"""
        from test_framework.test_sequencer import TestStep
        
        custom_steps = [
            TestStep(
                name="custom_test_1",
                test_type=TestType.USB_COMMUNICATION_TEST,
                parameters={"message_count": 5},
                timeout=10.0
            ),
            TestStep(
                name="custom_test_2",
                test_type=TestType.LED_FUNCTIONALITY,
                parameters={"pattern": "solid"},
                timeout=5.0,
                depends_on=["custom_test_1"]
            )
        ]
        
        config = self.test_scenarios.create_custom_scenario(
            "Custom Test",
            "Custom test description",
            custom_steps
        )
        
        self.assertEqual(config.name, "Custom Test")
        self.assertEqual(len(config.steps), 2)
        self.assertEqual(config.steps[1].depends_on, ["custom_test_1"])


class TestComprehensiveTestRunner(unittest.TestCase):
    """Test comprehensive test runner"""
    
    def setUp(self):
        """Set up test fixtures"""
        self.runner = ComprehensiveTestRunner("test_output")
        
        # Mock the framework components
        self.runner.device_manager = Mock()
        self.runner.command_handler = Mock()
        self.runner.test_sequencer = Mock()
        self.runner.result_collector = Mock()
        
    def test_runner_initialization(self):
        """Test runner initialization"""
        runner = ComprehensiveTestRunner("custom_output")
        self.assertEqual(str(runner.output_dir), "custom_output")
        
    @patch('test_framework.comprehensive_test_runner.logging')
    def test_setup_logging(self, mock_logging):
        """Test logging setup"""
        self.runner.setup_logging("DEBUG", "test.log")
        mock_logging.basicConfig.assert_called_once()
        
    def test_discover_and_connect_devices(self):
        """Test device discovery and connection"""
        # Mock device discovery
        mock_device = Mock()
        mock_device.serial_number = "TEST001"
        mock_device.product = "Test Device"
        mock_device.status.value = "connected"
        
        self.runner.device_manager.discover_devices.return_value = [mock_device]
        self.runner.device_manager.connect_device.return_value = True
        
        connected = self.runner.discover_and_connect_devices()
        
        self.assertEqual(connected, ["TEST001"])
        self.runner.device_manager.discover_devices.assert_called_once()
        self.runner.device_manager.connect_device.assert_called_once_with("TEST001")
        
    def test_discover_and_connect_devices_no_devices(self):
        """Test device discovery with no devices"""
        self.runner.device_manager.discover_devices.return_value = []
        
        connected = self.runner.discover_and_connect_devices()
        
        self.assertEqual(connected, [])
        
    def test_discover_and_connect_devices_target_filter(self):
        """Test device discovery with target filtering"""
        mock_device1 = Mock()
        mock_device1.serial_number = "TEST001"
        mock_device1.status.value = "connected"
        
        mock_device2 = Mock()
        mock_device2.serial_number = "TEST002"
        mock_device2.status.value = "connected"
        
        self.runner.device_manager.discover_devices.return_value = [mock_device1, mock_device2]
        self.runner.device_manager.connect_device.return_value = True
        
        connected = self.runner.discover_and_connect_devices(["TEST001"])
        
        self.assertEqual(connected, ["TEST001"])
        self.runner.device_manager.connect_device.assert_called_once_with("TEST001")
        
    def test_run_scenario(self):
        """Test running a specific scenario"""
        # Mock test execution
        mock_suite_result = Mock()
        mock_suite_result.aggregate_metrics.failed_tests = 0
        
        self.runner.test_sequencer.execute_test_sequence.return_value = {"TEST001": []}
        self.runner.result_collector.collect_results.return_value = mock_suite_result
        
        with patch.object(self.runner, '_generate_scenario_reports'):
            result = self.runner.run_scenario(
                TestScenarioType.HARDWARE_VALIDATION,
                ["TEST001"]
            )
        
        self.assertEqual(result['scenario_type'], 'hardware_validation')
        self.assertEqual(result['suite_result'], mock_suite_result)
        
    def test_run_custom_scenario(self):
        """Test running a custom scenario"""
        test_steps = [
            {
                'name': 'custom_test',
                'test_type': TestType.USB_COMMUNICATION_TEST.value,
                'parameters': {'message_count': 5},
                'timeout': 10.0
            }
        ]
        
        mock_suite_result = Mock()
        self.runner.test_sequencer.execute_test_sequence.return_value = {"TEST001": []}
        self.runner.result_collector.collect_results.return_value = mock_suite_result
        
        with patch.object(self.runner, '_generate_custom_scenario_report'):
            result = self.runner.run_custom_scenario(
                "Custom Test",
                test_steps,
                ["TEST001"]
            )
        
        self.assertEqual(result['scenario_name'], "Custom Test")
        self.assertEqual(result['suite_result'], mock_suite_result)
        
    def test_cleanup(self):
        """Test cleanup functionality"""
        self.runner.cleanup()
        self.runner.device_manager.disconnect_all.assert_called_once()


class TestScenarioIntegration(unittest.TestCase):
    """Integration tests for complete scenario execution"""
    
    def setUp(self):
        """Set up integration test fixtures"""
        self.mock_device_manager = Mock()
        self.mock_command_handler = Mock()
        
        # Mock successful device discovery
        mock_device = Mock()
        mock_device.serial_number = "INTEGRATION_TEST"
        mock_device.product = "Test Device"
        mock_device.status.value = "connected"
        
        self.mock_device_manager.discover_devices.return_value = [mock_device]
        self.mock_device_manager.connect_device.return_value = True
        self.mock_device_manager.get_connected_devices.return_value = ["INTEGRATION_TEST"]
        
    @patch('test_framework.comprehensive_test_runner.TestSequencer')
    @patch('test_framework.comprehensive_test_runner.ResultCollector')
    def test_hardware_validation_integration(self, mock_result_collector_class, mock_sequencer_class):
        """Test hardware validation scenario integration"""
        # Mock test sequencer
        mock_sequencer = Mock()
        mock_sequencer_class.return_value = mock_sequencer
        
        # Mock successful test execution
        from test_framework.test_sequencer import TestExecution, TestStatus
        mock_execution = Mock()
        mock_execution.status = TestStatus.COMPLETED
        mock_execution.step.name = "test_step"
        mock_execution.step.required = True
        mock_execution.duration = 1.0
        mock_execution.start_time = time.time()
        mock_execution.end_time = time.time() + 1.0
        
        mock_sequencer.execute_test_sequence.return_value = {
            "INTEGRATION_TEST": [mock_execution]
        }
        
        # Mock result collector
        mock_collector = Mock()
        mock_result_collector_class.return_value = mock_collector
        
        mock_suite_result = Mock()
        mock_suite_result.aggregate_metrics.failed_tests = 0
        mock_suite_result.aggregate_metrics.passed_tests = 1
        mock_suite_result.aggregate_metrics.total_tests = 1
        mock_collector.collect_results.return_value = mock_suite_result
        
        # Create runner with mocked components
        runner = ComprehensiveTestRunner("test_output")
        runner.device_manager = self.mock_device_manager
        runner.command_handler = self.mock_command_handler
        
        # Run hardware validation scenario
        connected_devices = runner.discover_and_connect_devices()
        self.assertEqual(connected_devices, ["INTEGRATION_TEST"])
        
        with patch.object(runner, '_generate_scenario_reports'):
            result = runner.run_scenario(
                TestScenarioType.HARDWARE_VALIDATION,
                connected_devices
            )
        
        # Verify results
        self.assertEqual(result['scenario_type'], 'hardware_validation')
        self.assertIsNotNone(result['suite_result'])
        
        # Verify test sequencer was called with correct configuration
        mock_sequencer.execute_test_sequence.assert_called_once()
        call_args = mock_sequencer.execute_test_sequence.call_args
        test_config = call_args[0][0]
        self.assertEqual(test_config.name, "Hardware Validation Suite")
        
    @patch('test_framework.comprehensive_test_runner.TestSequencer')
    @patch('test_framework.comprehensive_test_runner.ResultCollector')
    def test_full_suite_integration(self, mock_result_collector_class, mock_sequencer_class):
        """Test full test suite integration"""
        # Mock test sequencer
        mock_sequencer = Mock()
        mock_sequencer_class.return_value = mock_sequencer
        
        # Mock successful test execution for all scenarios
        mock_execution = Mock()
        mock_execution.status = TestStatus.COMPLETED
        mock_execution.step.name = "test_step"
        mock_execution.step.required = True
        mock_execution.duration = 1.0
        mock_execution.start_time = time.time()
        mock_execution.end_time = time.time() + 1.0
        
        mock_sequencer.execute_test_sequence.return_value = {
            "INTEGRATION_TEST": [mock_execution]
        }
        
        # Mock result collector
        mock_collector = Mock()
        mock_result_collector_class.return_value = mock_collector
        
        mock_suite_result = Mock()
        mock_suite_result.aggregate_metrics.failed_tests = 0
        mock_suite_result.aggregate_metrics.passed_tests = 1
        mock_suite_result.aggregate_metrics.total_tests = 1
        mock_collector.collect_results.return_value = mock_suite_result
        
        # Create runner with mocked components
        runner = ComprehensiveTestRunner("test_output")
        runner.device_manager = self.mock_device_manager
        runner.command_handler = self.mock_command_handler
        
        # Run full test suite
        connected_devices = runner.discover_and_connect_devices()
        
        with patch.object(runner, '_generate_scenario_reports'), \
             patch.object(runner, '_generate_full_suite_report'):
            result = runner.run_full_test_suite(connected_devices)
        
        # Verify results
        self.assertTrue(result['overall_success'])
        self.assertGreater(len(result['scenario_results']), 3)  # At least 4 scenarios
        
        # Verify all scenarios were executed
        expected_scenarios = [
            'hardware_validation',
            'regression_testing', 
            'performance_benchmarking',
            'integration_testing',
            'stress_testing'
        ]
        
        for scenario in expected_scenarios:
            self.assertIn(scenario, result['scenario_results'])


if __name__ == '__main__':
    unittest.main()