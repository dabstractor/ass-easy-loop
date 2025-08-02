"""
Test Sequencing and Orchestration

Manages test execution sequences, timing, and coordination across multiple devices.
"""

import time
import logging
from typing import List, Dict, Any, Optional, Callable
from dataclasses import dataclass, field
from enum import Enum
from concurrent.futures import ThreadPoolExecutor, Future
import threading

from .command_handler import CommandHandler, TestCommand, TestResponse, TestType
from .device_manager import UsbHidDeviceManager
from .real_time_monitor import RealTimeMonitor, LogLevel


class TestStatus(Enum):
    """Test execution status"""
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    TIMEOUT = "timeout"
    SKIPPED = "skipped"


@dataclass
class TestStep:
    """Individual test step configuration"""
    name: str
    test_type: TestType
    parameters: Dict[str, Any]
    timeout: float = 30.0
    retry_count: int = 0
    required: bool = True
    depends_on: List[str] = field(default_factory=list)
    
    
@dataclass
class TestConfiguration:
    """Complete test sequence configuration"""
    name: str
    description: str
    steps: List[TestStep]
    parallel_execution: bool = False
    max_parallel_devices: int = 4
    global_timeout: float = 300.0
    setup_commands: List[TestCommand] = field(default_factory=list)
    teardown_commands: List[TestCommand] = field(default_factory=list)


@dataclass
class TestExecution:
    """Test execution tracking"""
    step: TestStep
    device_serial: str
    status: TestStatus
    start_time: Optional[float] = None
    end_time: Optional[float] = None
    response: Optional[TestResponse] = None
    error_message: Optional[str] = None
    retry_attempt: int = 0
    
    @property
    def duration(self) -> Optional[float]:
        """Get test execution duration"""
        if self.start_time and self.end_time:
            return self.end_time - self.start_time
        return None


class TestSequencer:
    """
    Orchestrates test sequence execution across single or multiple devices.
    
    Supports both sequential and parallel test execution with dependency management,
    retry logic, and comprehensive result tracking.
    """
    
    def __init__(self, device_manager: UsbHidDeviceManager, command_handler: CommandHandler,
                 monitor: Optional[RealTimeMonitor] = None):
        """
        Initialize the test sequencer.
        
        Args:
            device_manager: Device manager instance
            command_handler: Command handler instance
            monitor: Optional real-time monitor for progress tracking
        """
        self.device_manager = device_manager
        self.command_handler = command_handler
        self.monitor = monitor
        self.logger = logging.getLogger(__name__)
        self.execution_lock = threading.Lock()
        self.active_executions: Dict[str, List[TestExecution]] = {}
        
    def execute_test_sequence(self, config: TestConfiguration, 
                            target_devices: List[str] = None,
                            global_timeout: float = None) -> Dict[str, List[TestExecution]]:
        """
        Execute a test sequence on specified devices.
        
        Args:
            config: Test configuration to execute
            target_devices: List of device serial numbers (None for all connected)
            global_timeout: Override global timeout for the entire sequence
            
        Returns:
            Dictionary mapping device serial numbers to test execution results
        """
        if target_devices is None:
            target_devices = self.device_manager.get_connected_devices()
            
        if not target_devices:
            self.logger.error("No target devices specified or connected")
            return {}
            
        # Use provided timeout or config timeout
        effective_timeout = global_timeout or config.global_timeout
        sequence_start_time = time.time()
            
        self.logger.info(f"Starting test sequence '{config.name}' on {len(target_devices)} devices "
                        f"(timeout: {effective_timeout}s)")
        
        # Initialize execution tracking
        with self.execution_lock:
            for device_serial in target_devices:
                self.active_executions[device_serial] = [
                    TestExecution(step=step, device_serial=device_serial, status=TestStatus.PENDING)
                    for step in config.steps
                ]
        
        # Execute setup commands
        self._execute_setup_commands(config, target_devices)
        
        try:
            if config.parallel_execution:
                results = self._execute_parallel_with_timeout(config, target_devices, effective_timeout, sequence_start_time)
            else:
                results = self._execute_sequential_with_timeout(config, target_devices, effective_timeout, sequence_start_time)
        finally:
            # Execute teardown commands
            self._execute_teardown_commands(config, target_devices)
            
        return results
    
    def _execute_sequential_with_timeout(self, config: TestConfiguration, 
                                       target_devices: List[str], 
                                       global_timeout: float,
                                       start_time: float) -> Dict[str, List[TestExecution]]:
        """Execute tests sequentially on all devices with global timeout"""
        results = {}
        
        for device_serial in target_devices:
            # Check global timeout
            elapsed_time = time.time() - start_time
            if elapsed_time >= global_timeout:
                self.logger.error(f"Global timeout reached, skipping remaining devices")
                # Mark remaining devices as timeout
                remaining_devices = target_devices[target_devices.index(device_serial):]
                for remaining_serial in remaining_devices:
                    with self.execution_lock:
                        timeout_executions = self.active_executions.get(remaining_serial, [])
                        for execution in timeout_executions:
                            execution.status = TestStatus.TIMEOUT
                            execution.error_message = "Global timeout reached"
                        results[remaining_serial] = timeout_executions
                break
                
            self.logger.info(f"Executing tests on device {device_serial}")
            remaining_timeout = global_timeout - elapsed_time
            device_results = self._execute_device_sequence_with_timeout(config, device_serial, remaining_timeout)
            results[device_serial] = device_results
            
        return results
    
    def _execute_parallel_with_timeout(self, config: TestConfiguration,
                                     target_devices: List[str],
                                     global_timeout: float,
                                     start_time: float) -> Dict[str, List[TestExecution]]:
        """Execute tests in parallel across multiple devices with global timeout"""
        results = {}
        max_workers = min(config.max_parallel_devices, len(target_devices))
        
        with ThreadPoolExecutor(max_workers=max_workers) as executor:
            # Submit device execution tasks
            future_to_device = {
                executor.submit(self._execute_device_sequence_with_timeout, config, device_serial, global_timeout): device_serial
                for device_serial in target_devices
            }
            
            # Calculate remaining timeout
            elapsed_time = time.time() - start_time
            remaining_timeout = max(0, global_timeout - elapsed_time)
            
            # Collect results as they complete
            for future in future_to_device:
                device_serial = future_to_device[future]
                try:
                    device_results = future.result(timeout=remaining_timeout)
                    results[device_serial] = device_results
                except Exception as e:
                    self.logger.error(f"Device {device_serial} execution failed: {e}")
                    # Create failed execution results
                    with self.execution_lock:
                        failed_executions = self.active_executions.get(device_serial, [])
                        for execution in failed_executions:
                            if execution.status == TestStatus.PENDING:
                                execution.status = TestStatus.TIMEOUT if "timeout" in str(e).lower() else TestStatus.FAILED
                                execution.error_message = str(e)
                        results[device_serial] = failed_executions
                        
        return results
    
    def _execute_device_sequence_with_timeout(self, config: TestConfiguration, 
                                            device_serial: str,
                                            timeout: float) -> List[TestExecution]:
        """Execute test sequence on a single device with timeout"""
        with self.execution_lock:
            executions = self.active_executions.get(device_serial, [])
            
        device_start_time = time.time()
        
        for execution in executions:
            # Check device-level timeout
            elapsed_time = time.time() - device_start_time
            if elapsed_time >= timeout:
                self.logger.error(f"Device timeout reached for {device_serial}, marking remaining tests as timeout")
                # Mark remaining tests as timeout
                remaining_executions = executions[executions.index(execution):]
                for remaining in remaining_executions:
                    if remaining.status == TestStatus.PENDING:
                        remaining.status = TestStatus.TIMEOUT
                        remaining.error_message = "Device execution timeout"
                break
                
            if not self._should_execute_step(execution, executions):
                execution.status = TestStatus.SKIPPED
                continue
                
            # Calculate remaining timeout for this test
            remaining_timeout = timeout - elapsed_time
            self._execute_single_test_with_timeout(execution, remaining_timeout)
            
            # Stop on critical failure
            if execution.status == TestStatus.FAILED and execution.step.required:
                self.logger.error(f"Critical test failed on {device_serial}: {execution.step.name}")
                # Mark remaining tests as skipped
                remaining_executions = executions[executions.index(execution) + 1:]
                for remaining in remaining_executions:
                    remaining.status = TestStatus.SKIPPED
                break
                
        return executions
    
    def _execute_device_sequence(self, config: TestConfiguration, 
                               device_serial: str) -> List[TestExecution]:
        """Execute test sequence on a single device (legacy method for compatibility)"""
        return self._execute_device_sequence_with_timeout(config, device_serial, config.global_timeout)
    
    def _execute_single_test_with_timeout(self, execution: TestExecution, max_timeout: float) -> None:
        """Execute a single test step with retry logic and timeout constraint"""
        max_attempts = execution.step.retry_count + 1
        test_timeout = min(execution.step.timeout, max_timeout)
        
        # Log test start to monitor
        if self.monitor:
            self.monitor.log_test_started(execution.device_serial, execution.step.name)
        
        for attempt in range(max_attempts):
            execution.retry_attempt = attempt
            execution.status = TestStatus.RUNNING
            execution.start_time = time.time()
            
            try:
                # Create and send test command
                command = self.command_handler.create_test_command(
                    execution.step.test_type,
                    execution.step.parameters
                )
                
                # Log command sending to monitor
                correlation_id = None
                if self.monitor:
                    correlation_id = self.monitor.log_command_sent(execution.device_serial, command)
                
                response = self.command_handler.send_command_and_wait(
                    execution.device_serial,
                    command,
                    timeout=test_timeout
                )
                
                execution.end_time = time.time()
                
                # Log response to monitor
                if self.monitor and response:
                    self.monitor.log_response_received(execution.device_serial, response, correlation_id)
                
                if response:
                    execution.response = response
                    if response.status.value == 0:  # SUCCESS
                        execution.status = TestStatus.COMPLETED
                        self.logger.info(f"Test '{execution.step.name}' completed on {execution.device_serial}")
                        
                        # Log success to monitor
                        if self.monitor:
                            self.monitor.log_test_completed(execution.device_serial, execution.step.name, execution)
                        return
                    else:
                        execution.error_message = f"Device returned error: {response.status.name}"
                        execution.status = TestStatus.FAILED
                else:
                    execution.error_message = "No response received (timeout)"
                    execution.status = TestStatus.TIMEOUT
                    
            except Exception as e:
                execution.end_time = time.time()
                execution.error_message = str(e)
                self.logger.error(f"Test execution error: {e}")
                
            # Retry logic - but don't retry if we already succeeded
            if execution.status == TestStatus.COMPLETED:
                return
                
            # Check if we have time for retry
            elapsed_time = time.time() - execution.start_time
            if attempt < max_attempts - 1 and elapsed_time < max_timeout - 2.0:  # Leave 2s buffer
                self.logger.warning(f"Retrying test '{execution.step.name}' on {execution.device_serial} "
                                  f"(attempt {attempt + 2}/{max_attempts})")
                time.sleep(1.0)  # Brief delay before retry
            else:
                # Keep the current status (FAILED or TIMEOUT) from the last attempt
                if execution.status not in [TestStatus.FAILED, TestStatus.TIMEOUT]:
                    execution.status = TestStatus.FAILED
                self.logger.error(f"Test '{execution.step.name}' failed on {execution.device_serial} "
                                f"after {max_attempts} attempts")
                
                # Log failure to monitor
                if self.monitor:
                    self.monitor.log_test_failed(execution.device_serial, execution.step.name, execution)
                break
    
    def _execute_single_test(self, execution: TestExecution) -> None:
        """Execute a single test step with retry logic (legacy method for compatibility)"""
        self._execute_single_test_with_timeout(execution, execution.step.timeout)
    
    def _should_execute_step(self, execution: TestExecution, 
                           all_executions: List[TestExecution]) -> bool:
        """Check if a test step should be executed based on dependencies"""
        if not execution.step.depends_on:
            return True
            
        # Check if all dependencies are satisfied
        for dependency in execution.step.depends_on:
            dependency_execution = next(
                (e for e in all_executions if e.step.name == dependency), None
            )
            
            if not dependency_execution or dependency_execution.status != TestStatus.COMPLETED:
                self.logger.warning(f"Skipping '{execution.step.name}' due to unmet dependency: {dependency}")
                return False
                
        return True
    
    def _execute_setup_commands(self, config: TestConfiguration, target_devices: List[str]) -> None:
        """Execute setup commands on all target devices"""
        if not config.setup_commands:
            return
            
        self.logger.info("Executing setup commands")
        for device_serial in target_devices:
            for command in config.setup_commands:
                try:
                    self.command_handler.send_command(device_serial, command)
                    time.sleep(0.1)  # Brief delay between commands
                except Exception as e:
                    self.logger.error(f"Setup command failed on {device_serial}: {e}")
    
    def _execute_teardown_commands(self, config: TestConfiguration, target_devices: List[str]) -> None:
        """Execute teardown commands on all target devices"""
        if not config.teardown_commands:
            return
            
        self.logger.info("Executing teardown commands")
        for device_serial in target_devices:
            for command in config.teardown_commands:
                try:
                    self.command_handler.send_command(device_serial, command)
                    time.sleep(0.1)  # Brief delay between commands
                except Exception as e:
                    self.logger.error(f"Teardown command failed on {device_serial}: {e}")
    
    def get_execution_status(self, device_serial: str) -> Optional[List[TestExecution]]:
        """Get current execution status for a device"""
        with self.execution_lock:
            return self.active_executions.get(device_serial)
    
    def cancel_execution(self, device_serial: str) -> bool:
        """Cancel ongoing test execution for a device"""
        with self.execution_lock:
            executions = self.active_executions.get(device_serial)
            if executions:
                for execution in executions:
                    if execution.status == TestStatus.RUNNING:
                        execution.status = TestStatus.FAILED
                        execution.error_message = "Execution cancelled"
                        execution.end_time = time.time()
                return True
        return False
    
    def create_basic_validation_config(self) -> TestConfiguration:
        """Create a basic device validation test configuration"""
        return TestConfiguration(
            name="Basic Device Validation",
            description="Basic functionality validation for RP2040 device",
            steps=[
                TestStep(
                    name="system_health_check",
                    test_type=TestType.USB_COMMUNICATION_TEST,
                    parameters={"message_count": 10, "timeout_ms": 1000},
                    timeout=10.0
                ),
                TestStep(
                    name="pemf_timing_validation",
                    test_type=TestType.PEMF_TIMING_VALIDATION,
                    parameters={"duration_ms": 5000, "tolerance_percent": 1.0},
                    timeout=15.0,
                    depends_on=["system_health_check"]
                ),
                TestStep(
                    name="battery_adc_test",
                    test_type=TestType.BATTERY_ADC_CALIBRATION,
                    parameters={"reference_voltage": 3.3},
                    timeout=10.0,
                    depends_on=["system_health_check"]
                ),
                TestStep(
                    name="led_functionality_test",
                    test_type=TestType.LED_FUNCTIONALITY,
                    parameters={"pattern": "all", "duration_ms": 2000},
                    timeout=10.0,
                    depends_on=["system_health_check"]
                )
            ],
            parallel_execution=False,
            global_timeout=120.0
        )