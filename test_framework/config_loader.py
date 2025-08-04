"""
Configuration Loader for Test Scenarios

Loads and manages configuration settings for test scenarios,
including parameters, timeouts, and execution settings.
"""

import json
import os
from typing import Dict, Any, Optional, List
from pathlib import Path
from dataclasses import dataclass, asdict

from .test_scenarios import ScenarioParameters


@dataclass
class TimeoutConfiguration:
    """Timeout configuration settings"""
    short_timeout: float = 10.0
    medium_timeout: float = 30.0
    long_timeout: float = 120.0
    global_timeout_multiplier: float = 1.5


@dataclass
class ExecutionSettings:
    """Test execution settings"""
    parallel_execution: bool = False
    max_parallel_devices: int = 4
    retry_failed_tests: bool = True
    max_retry_attempts: int = 2
    continue_on_failure: bool = True


@dataclass
class ReportingSettings:
    """Reporting configuration settings"""
    generate_html_reports: bool = True
    generate_junit_xml: bool = True
    generate_json_reports: bool = True
    detailed_logging: bool = True
    performance_analysis: bool = True
    failure_analysis: bool = True


@dataclass
class DeviceSettings:
    """Device management settings"""
    connection_timeout: float = 5.0
    discovery_interval: float = 1.0
    auto_reconnect: bool = True
    device_validation: bool = True


@dataclass
class TestConfiguration:
    """Complete test configuration"""
    scenario_parameters: Dict[str, Dict[str, Any]]
    timeout_configurations: TimeoutConfiguration
    execution_settings: ExecutionSettings
    reporting_settings: ReportingSettings
    device_settings: DeviceSettings


class ConfigurationLoader:
    """
    Loads and manages test scenario configurations.
    
    Supports loading from JSON files, environment variables,
    and programmatic configuration.
    """
    
    DEFAULT_CONFIG_FILE = "scenario_config.json"
    
    def __init__(self, config_file: Optional[str] = None):
        """
        Initialize the configuration loader.
        
        Args:
            config_file: Path to configuration file (uses default if None)
        """
        self.config_file = config_file or self.DEFAULT_CONFIG_FILE
        self.config: Optional[TestConfiguration] = None
        
    def load_configuration(self, config_path: Optional[str] = None) -> TestConfiguration:
        """
        Load configuration from file.
        
        Args:
            config_path: Path to configuration file (overrides instance setting)
            
        Returns:
            Loaded test configuration
        """
        config_file = config_path or self.config_file
        
        # Try to find config file in multiple locations
        search_paths = [
            config_file,
            os.path.join(os.path.dirname(__file__), config_file),
            os.path.join(os.getcwd(), config_file)
        ]
        
        config_data = None
        for path in search_paths:
            if os.path.exists(path):
                with open(path, 'r') as f:
                    config_data = json.load(f)
                break
        
        if config_data is None:
            # Use default configuration if no file found
            config_data = self._get_default_configuration()
        
        # Parse configuration sections
        scenario_params = config_data.get('scenario_parameters', {})
        
        timeout_config = TimeoutConfiguration(
            **config_data.get('timeout_configurations', {})
        )
        
        execution_settings = ExecutionSettings(
            **config_data.get('test_execution_settings', {})
        )
        
        reporting_settings = ReportingSettings(
            **config_data.get('reporting_settings', {})
        )
        
        device_settings = DeviceSettings(
            **config_data.get('device_settings', {})
        )
        
        self.config = TestConfiguration(
            scenario_parameters=scenario_params,
            timeout_configurations=timeout_config,
            execution_settings=execution_settings,
            reporting_settings=reporting_settings,
            device_settings=device_settings
        )
        
        return self.config
    
    def get_scenario_parameters(self, scenario_name: str) -> ScenarioParameters:
        """
        Get scenario parameters for a specific scenario.
        
        Args:
            scenario_name: Name of the scenario
            
        Returns:
            Scenario parameters object
        """
        if not self.config:
            self.load_configuration()
        
        scenario_config = self.config.scenario_parameters.get(scenario_name, {})
        
        # Merge with default parameters
        default_params = asdict(ScenarioParameters())
        merged_params = {**default_params, **scenario_config}
        
        # Apply timeout configurations
        timeout_config = self.config.timeout_configurations
        merged_params.update({
            'short_timeout': timeout_config.short_timeout,
            'medium_timeout': timeout_config.medium_timeout,
            'long_timeout': timeout_config.long_timeout
        })
        
        return ScenarioParameters(**merged_params)
    
    def get_execution_settings(self) -> ExecutionSettings:
        """Get test execution settings"""
        if not self.config:
            self.load_configuration()
        return self.config.execution_settings
    
    def get_reporting_settings(self) -> ReportingSettings:
        """Get reporting settings"""
        if not self.config:
            self.load_configuration()
        return self.config.reporting_settings
    
    def get_device_settings(self) -> DeviceSettings:
        """Get device management settings"""
        if not self.config:
            self.load_configuration()
        return self.config.device_settings
    
    def save_configuration(self, output_path: str):
        """
        Save current configuration to file.
        
        Args:
            output_path: Path to save configuration file
        """
        if not self.config:
            raise ValueError("No configuration loaded to save")
        
        config_dict = {
            'scenario_parameters': self.config.scenario_parameters,
            'timeout_configurations': asdict(self.config.timeout_configurations),
            'test_execution_settings': asdict(self.config.execution_settings),
            'reporting_settings': asdict(self.config.reporting_settings),
            'device_settings': asdict(self.config.device_settings)
        }
        
        with open(output_path, 'w') as f:
            json.dump(config_dict, f, indent=2)
    
    def update_scenario_parameters(self, scenario_name: str, parameters: Dict[str, Any]):
        """
        Update parameters for a specific scenario.
        
        Args:
            scenario_name: Name of the scenario to update
            parameters: Parameters to update
        """
        if not self.config:
            self.load_configuration()
        
        if scenario_name not in self.config.scenario_parameters:
            self.config.scenario_parameters[scenario_name] = {}
        
        self.config.scenario_parameters[scenario_name].update(parameters)
    
    def _get_default_configuration(self) -> Dict[str, Any]:
        """Get default configuration if no file is found"""
        return {
            "scenario_parameters": {
                "hardware_validation": {
                    "pemf_test_duration_ms": 5000,
                    "pemf_tolerance_percent": 1.0,
                    "battery_reference_voltage": 3.3,
                    "led_test_duration_ms": 2000,
                    "usb_message_count": 50,
                    "usb_timeout_ms": 1000
                },
                "stress_testing": {
                    "stress_test_duration_ms": 30000,
                    "stress_load_level": 80,
                    "memory_stress_iterations": 1000
                },
                "regression_testing": {
                    "baseline_comparison": True,
                    "performance_tolerance_percent": 5.0
                },
                "performance_benchmarking": {
                    "benchmark_iterations": 100,
                    "timing_precision_us": 10
                },
                "integration_testing": {
                    "end_to_end_validation": True,
                    "multi_subsystem_coordination": True
                }
            },
            "timeout_configurations": {
                "short_timeout": 10.0,
                "medium_timeout": 30.0,
                "long_timeout": 120.0,
                "global_timeout_multiplier": 1.5
            },
            "test_execution_settings": {
                "parallel_execution": False,
                "max_parallel_devices": 4,
                "retry_failed_tests": True,
                "max_retry_attempts": 2,
                "continue_on_failure": True
            },
            "reporting_settings": {
                "generate_html_reports": True,
                "generate_junit_xml": True,
                "generate_json_reports": True,
                "detailed_logging": True,
                "performance_analysis": True,
                "failure_analysis": True
            },
            "device_settings": {
                "connection_timeout": 5.0,
                "discovery_interval": 1.0,
                "auto_reconnect": True,
                "device_validation": True
            }
        }
    
    def validate_configuration(self) -> List[str]:
        """
        Validate the current configuration.
        
        Returns:
            List of validation errors (empty if valid)
        """
        if not self.config:
            return ["No configuration loaded"]
        
        errors = []
        
        # Validate timeout configurations
        timeouts = self.config.timeout_configurations
        if timeouts.short_timeout <= 0:
            errors.append("Short timeout must be positive")
        if timeouts.medium_timeout <= timeouts.short_timeout:
            errors.append("Medium timeout must be greater than short timeout")
        if timeouts.long_timeout <= timeouts.medium_timeout:
            errors.append("Long timeout must be greater than medium timeout")
        
        # Validate execution settings
        execution = self.config.execution_settings
        if execution.max_parallel_devices <= 0:
            errors.append("Max parallel devices must be positive")
        if execution.max_retry_attempts < 0:
            errors.append("Max retry attempts cannot be negative")
        
        # Validate device settings
        device = self.config.device_settings
        if device.connection_timeout <= 0:
            errors.append("Connection timeout must be positive")
        if device.discovery_interval <= 0:
            errors.append("Discovery interval must be positive")
        
        # Validate scenario parameters
        for scenario_name, params in self.config.scenario_parameters.items():
            if 'pemf_test_duration_ms' in params and params['pemf_test_duration_ms'] <= 0:
                errors.append(f"pEMF test duration must be positive in {scenario_name}")
            if 'stress_load_level' in params:
                load_level = params['stress_load_level']
                if not (0 <= load_level <= 100):
                    errors.append(f"Stress load level must be 0-100 in {scenario_name}")
        
        return errors


def load_configuration_from_env() -> Dict[str, Any]:
    """
    Load configuration overrides from environment variables.
    
    Returns:
        Dictionary of configuration overrides
    """
    env_config = {}
    
    # Check for common environment variable overrides
    env_mappings = {
        'TEST_PEMF_DURATION': ('scenario_parameters.hardware_validation.pemf_test_duration_ms', int),
        'TEST_STRESS_DURATION': ('scenario_parameters.stress_testing.stress_test_duration_ms', int),
        'TEST_STRESS_LOAD': ('scenario_parameters.stress_testing.stress_load_level', int),
        'TEST_SHORT_TIMEOUT': ('timeout_configurations.short_timeout', float),
        'TEST_MEDIUM_TIMEOUT': ('timeout_configurations.medium_timeout', float),
        'TEST_LONG_TIMEOUT': ('timeout_configurations.long_timeout', float),
        'TEST_PARALLEL_EXECUTION': ('test_execution_settings.parallel_execution', lambda x: x.lower() == 'true'),
        'TEST_MAX_DEVICES': ('test_execution_settings.max_parallel_devices', int),
        'TEST_RETRY_TESTS': ('test_execution_settings.retry_failed_tests', lambda x: x.lower() == 'true'),
        'TEST_GENERATE_HTML': ('reporting_settings.generate_html_reports', lambda x: x.lower() == 'true'),
        'TEST_DETAILED_LOGGING': ('reporting_settings.detailed_logging', lambda x: x.lower() == 'true')
    }
    
    for env_var, (config_path, converter) in env_mappings.items():
        value = os.environ.get(env_var)
        if value is not None:
            try:
                converted_value = converter(value)
                _set_nested_dict_value(env_config, config_path, converted_value)
            except (ValueError, TypeError):
                pass  # Ignore invalid environment variable values
    
    return env_config


def _set_nested_dict_value(dictionary: Dict[str, Any], path: str, value: Any):
    """Set a value in a nested dictionary using dot notation path"""
    keys = path.split('.')
    current = dictionary
    
    for key in keys[:-1]:
        if key not in current:
            current[key] = {}
        current = current[key]
    
    current[keys[-1]] = value