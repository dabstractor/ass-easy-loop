"""
Automated Test Framework for RP2040 pEMF/Battery Monitoring Device

This package provides a comprehensive testing framework for automated validation
of the RP2040 device firmware, including bootloader entry, test command execution,
system state validation, and comprehensive test scenarios.
"""

__version__ = "1.0.0"
__author__ = "Automated Testing Framework"

from .device_manager import UsbHidDeviceManager, DeviceInfo
from .command_handler import CommandHandler, TestCommand, TestResponse, TestType
from .test_sequencer import TestSequencer, TestConfiguration, TestStep
from .result_collector import ResultCollector
from .firmware_flasher import FirmwareFlasher, FlashResult, FlashOperation
from .test_scenarios import TestScenarios, TestScenarioType, ScenarioParameters
from .comprehensive_test_runner import ComprehensiveTestRunner
from .config_loader import ConfigurationLoader
from .ci_integration import CIIntegration, CIEnvironmentInfo, CITestConfiguration, CITestResult, create_ci_integration

__all__ = [
    # Core Framework Components
    'UsbHidDeviceManager',
    'DeviceInfo', 
    'CommandHandler',
    'TestCommand',
    'TestResponse',
    'TestType',
    'TestSequencer',
    'TestConfiguration',
    'TestStep',
    'ResultCollector',
    'FirmwareFlasher',
    'FlashResult',
    'FlashOperation',
    
    # Comprehensive Testing Components
    'TestScenarios',
    'TestScenarioType',
    'ScenarioParameters',
    'ComprehensiveTestRunner',
    'ConfigurationLoader',
    
    # CI/CD Integration Components
    'CIIntegration',
    'CIEnvironmentInfo',
    'CITestConfiguration',
    'CITestResult',
    'create_ci_integration',
]