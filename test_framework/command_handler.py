"""
Command Transmission and Response Handling

Handles the communication protocol for sending test commands and receiving
responses from the RP2040 device via USB HID.
"""

import struct
import time
import logging
from typing import Optional, Dict, Any, List, Union
from dataclasses import dataclass
from enum import Enum
import json


class CommandType(Enum):
    """Test command types"""
    ENTER_BOOTLOADER = 0x80
    SYSTEM_STATE_QUERY = 0x81
    EXECUTE_TEST = 0x82
    CONFIGURATION_QUERY = 0x83
    PERFORMANCE_METRICS = 0x84


class ResponseStatus(Enum):
    """Command response status codes"""
    SUCCESS = 0x00
    ERROR_INVALID_COMMAND = 0x01
    ERROR_AUTHENTICATION_FAILED = 0x02
    ERROR_PARAMETER_INVALID = 0x03
    ERROR_SYSTEM_BUSY = 0x04
    ERROR_HARDWARE_FAULT = 0x05
    ERROR_TIMEOUT = 0x06


class TestType(Enum):
    """Available test types"""
    PEMF_TIMING_VALIDATION = 0x01
    BATTERY_ADC_CALIBRATION = 0x02
    LED_FUNCTIONALITY = 0x03
    SYSTEM_STRESS_TEST = 0x04
    USB_COMMUNICATION_TEST = 0x05


@dataclass
class TestCommand:
    """Test command structure"""
    command_type: CommandType
    command_id: int
    payload: Dict[str, Any]
    
    def to_bytes(self) -> bytes:
        """Convert command to 64-byte HID report format"""
        payload_json = json.dumps(self.payload).encode('utf-8')
        payload_length = min(len(payload_json), 61)
        
        # Calculate simple checksum for authentication
        checksum = (self.command_type.value + self.command_id + payload_length) & 0xFF
        
        # Build 64-byte report
        report = bytearray(64)
        report[0] = self.command_type.value
        report[1] = self.command_id
        report[2] = payload_length
        report[3] = checksum
        report[4:4+payload_length] = payload_json[:payload_length]
        
        return bytes(report)


@dataclass 
class TestResponse:
    """Test response structure"""
    command_id: int
    status: ResponseStatus
    response_type: str
    data: Dict[str, Any]
    timestamp: float
    
    @classmethod
    def from_log_message(cls, log_message: str) -> Optional['TestResponse']:
        """Parse response from device log message"""
        try:
            # Expected format: "TEST_RESPONSE:{json_data}"
            if not log_message.startswith("TEST_RESPONSE:"):
                return None
                
            json_data = log_message[14:]  # Remove "TEST_RESPONSE:" prefix
            response_dict = json.loads(json_data)
            
            return cls(
                command_id=response_dict.get('command_id', 0),
                status=ResponseStatus(response_dict.get('status', 0)),
                response_type=response_dict.get('type', 'unknown'),
                data=response_dict.get('data', {}),
                timestamp=time.time()
            )
            
        except (json.JSONDecodeError, ValueError, KeyError) as e:
            logging.getLogger(__name__).error(f"Failed to parse response: {e}")
            return None


class CommandHandler:
    """
    Handles command transmission and response processing for device communication.
    
    Manages the bidirectional communication protocol using USB HID output reports
    for commands and input reports (via logging) for responses.
    """
    
    def __init__(self, device_manager, response_timeout: float = 5.0, monitor=None):
        """
        Initialize the command handler.
        
        Args:
            device_manager: UsbHidDeviceManager instance
            response_timeout: Timeout for waiting for command responses
            monitor: Optional RealTimeMonitor for communication logging
        """
        self.device_manager = device_manager
        self.response_timeout = response_timeout
        self.monitor = monitor
        self.command_sequence = 0
        self.pending_commands: Dict[int, TestCommand] = {}
        self.response_buffer: List[str] = []
        self.logger = logging.getLogger(__name__)
        
    def send_command(self, serial_number: str, command: TestCommand) -> bool:
        """
        Send a test command to a specific device.
        
        Args:
            serial_number: Target device serial number
            command: Command to send
            
        Returns:
            True if command sent successfully, False otherwise
        """
        if not self.device_manager.is_device_connected(serial_number):
            self.logger.error(f"Device {serial_number} not connected")
            return False
            
        device_handle = self.device_manager.get_device_handle(serial_number)
        if not device_handle:
            self.logger.error(f"No device handle for {serial_number}")
            return False
            
        try:
            # Assign sequence number and track command
            command.command_id = self._get_next_command_id()
            self.pending_commands[command.command_id] = command
            
            # Send command as HID output report
            command_bytes = command.to_bytes()
            bytes_sent = device_handle.write(command_bytes)
            
            if bytes_sent == len(command_bytes):
                self.logger.debug(f"Sent command {command.command_type.name} to {serial_number}")
                
                # Log command to monitor if available
                if self.monitor:
                    correlation_id = self.monitor.log_command_sent(serial_number, command)
                    # Store correlation ID for response matching
                    command._correlation_id = correlation_id
                
                return True
            else:
                self.logger.error(f"Failed to send complete command to {serial_number}")
                return False
                
        except Exception as e:
            self.logger.error(f"Error sending command to {serial_number}: {e}")
            return False
    
    def read_responses(self, serial_number: str) -> List[TestResponse]:
        """
        Read and parse responses from a device.
        
        Args:
            serial_number: Device serial number to read from
            
        Returns:
            List of parsed responses
        """
        if not self.device_manager.is_device_connected(serial_number):
            return []
            
        device_handle = self.device_manager.get_device_handle(serial_number)
        if not device_handle:
            return []
            
        responses = []
        
        try:
            # Read available HID input reports (log messages)
            while True:
                data = device_handle.read(64, timeout_ms=10)
                if not data:
                    break
                    
                # Convert bytes to string (assuming UTF-8 log messages)
                try:
                    log_message = bytes(data).decode('utf-8').rstrip('\x00')
                    if log_message:
                        self.response_buffer.append(log_message)
                        
                        # Log raw device communication to monitor with raw bytes
                        if self.monitor:
                            self.monitor.log_device_communication(serial_number, log_message, 'received', bytes(data))
                except UnicodeDecodeError:
                    continue
            
            # Process buffered messages for test responses
            remaining_buffer = []
            for message in self.response_buffer:
                response = TestResponse.from_log_message(message)
                if response:
                    responses.append(response)
                    # Remove from pending commands if this is a response
                    if response.command_id in self.pending_commands:
                        del self.pending_commands[response.command_id]
                else:
                    # Keep non-response messages in buffer
                    remaining_buffer.append(message)
                    
            self.response_buffer = remaining_buffer
            
        except Exception as e:
            self.logger.error(f"Error reading responses from {serial_number}: {e}")
            
        return responses
    
    def wait_for_response(self, serial_number: str, command_id: int, 
                         timeout: float = None) -> Optional[TestResponse]:
        """
        Wait for a specific command response.
        
        Args:
            serial_number: Device serial number
            command_id: Command ID to wait for
            timeout: Timeout in seconds (uses default if None)
            
        Returns:
            Response if received, None if timeout
        """
        if timeout is None:
            timeout = self.response_timeout
            
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            responses = self.read_responses(serial_number)
            
            for response in responses:
                if response.command_id == command_id:
                    # Log response to monitor if available
                    if self.monitor:
                        # Try to find correlation ID from pending command
                        correlation_id = None
                        for cmd in self.pending_commands.values():
                            if cmd.command_id == command_id:
                                correlation_id = getattr(cmd, '_correlation_id', None)
                                break
                        self.monitor.log_response_received(serial_number, response, correlation_id)
                    
                    return response
                    
            time.sleep(0.1)  # Small delay to avoid busy waiting
            
        self.logger.warning(f"Timeout waiting for response to command {command_id}")
        return None
    
    def send_command_and_wait(self, serial_number: str, command: TestCommand,
                             timeout: float = None) -> Optional[TestResponse]:
        """
        Send a command and wait for its response.
        
        Args:
            serial_number: Target device serial number
            command: Command to send
            timeout: Response timeout in seconds
            
        Returns:
            Response if successful, None if failed or timeout
        """
        if not self.send_command(serial_number, command):
            return None
            
        return self.wait_for_response(serial_number, command.command_id, timeout)
    
    def create_bootloader_command(self, timeout_ms: int = 5000) -> TestCommand:
        """Create a bootloader entry command"""
        return TestCommand(
            command_type=CommandType.ENTER_BOOTLOADER,
            command_id=0,  # Will be assigned when sent
            payload={'timeout_ms': timeout_ms}
        )
    
    def create_system_state_query(self, query_type: str = 'system_health') -> TestCommand:
        """Create a system state query command"""
        return TestCommand(
            command_type=CommandType.SYSTEM_STATE_QUERY,
            command_id=0,  # Will be assigned when sent
            payload={'query_type': query_type}
        )
    
    def create_test_command(self, test_type: TestType, parameters: Dict[str, Any]) -> TestCommand:
        """Create a test execution command"""
        return TestCommand(
            command_type=CommandType.EXECUTE_TEST,
            command_id=0,  # Will be assigned when sent
            payload={
                'test_type': test_type.value,
                'parameters': parameters
            }
        )
    
    def get_pending_commands(self) -> Dict[int, TestCommand]:
        """Get dictionary of pending commands awaiting responses"""
        return self.pending_commands.copy()
    
    def clear_pending_commands(self) -> None:
        """Clear all pending commands (useful for cleanup)"""
        self.pending_commands.clear()
    
    def _get_next_command_id(self) -> int:
        """Get next command sequence number"""
        self.command_sequence = (self.command_sequence + 1) % 256
        return self.command_sequence