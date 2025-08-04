"""
Unit tests for Command Handler
"""

import unittest
from unittest.mock import Mock, patch, MagicMock
import json
import time

from test_framework.command_handler import (
    CommandHandler, TestCommand, TestResponse, CommandType, 
    ResponseStatus, TestType
)
from test_framework.device_manager import UsbHidDeviceManager


class TestCommandHandler(unittest.TestCase):
    """Test cases for CommandHandler"""
    
    def setUp(self):
        """Set up test fixtures"""
        self.mock_device_manager = Mock(spec=UsbHidDeviceManager)
        self.command_handler = CommandHandler(
            device_manager=self.mock_device_manager,
            response_timeout=2.0
        )
        
        # Mock device handle
        self.mock_device_handle = Mock()
        self.mock_device_manager.get_device_handle.return_value = self.mock_device_handle
        self.mock_device_manager.is_device_connected.return_value = True
    
    def test_create_bootloader_command(self):
        """Test creating bootloader entry command"""
        command = self.command_handler.create_bootloader_command(timeout_ms=5000)
        
        self.assertEqual(command.command_type, CommandType.ENTER_BOOTLOADER)
        self.assertEqual(command.payload['timeout_ms'], 5000)
    
    def test_create_system_state_query(self):
        """Test creating system state query command"""
        command = self.command_handler.create_system_state_query('system_health')
        
        self.assertEqual(command.command_type, CommandType.SYSTEM_STATE_QUERY)
        self.assertEqual(command.payload['query_type'], 'system_health')
    
    def test_create_test_command(self):
        """Test creating test execution command"""
        parameters = {'duration_ms': 1000, 'tolerance': 1.0}
        command = self.command_handler.create_test_command(
            TestType.PEMF_TIMING_VALIDATION, 
            parameters
        )
        
        self.assertEqual(command.command_type, CommandType.EXECUTE_TEST)
        self.assertEqual(command.payload['test_type'], TestType.PEMF_TIMING_VALIDATION.value)
        self.assertEqual(command.payload['parameters'], parameters)
    
    def test_command_to_bytes(self):
        """Test command serialization to bytes"""
        command = TestCommand(
            command_type=CommandType.ENTER_BOOTLOADER,
            command_id=42,
            payload={'timeout_ms': 5000}
        )
        
        command_bytes = command.to_bytes()
        
        self.assertEqual(len(command_bytes), 64)
        self.assertEqual(command_bytes[0], CommandType.ENTER_BOOTLOADER.value)
        self.assertEqual(command_bytes[1], 42)
        
        # Verify payload can be decoded
        payload_length = command_bytes[2]
        payload_json = command_bytes[4:4+payload_length].decode('utf-8')
        payload = json.loads(payload_json)
        self.assertEqual(payload['timeout_ms'], 5000)
    
    def test_send_command_success(self):
        """Test successful command transmission"""
        command = self.command_handler.create_bootloader_command()
        self.mock_device_handle.write.return_value = 64
        
        result = self.command_handler.send_command('TEST123', command)
        
        self.assertTrue(result)
        self.mock_device_handle.write.assert_called_once()
        self.assertIn(command.command_id, self.command_handler.pending_commands)
    
    def test_send_command_device_not_connected(self):
        """Test sending command to disconnected device"""
        self.mock_device_manager.is_device_connected.return_value = False
        command = self.command_handler.create_bootloader_command()
        
        result = self.command_handler.send_command('TEST123', command)
        
        self.assertFalse(result)
        self.mock_device_handle.write.assert_not_called()
    
    def test_send_command_write_failure(self):
        """Test command transmission failure"""
        command = self.command_handler.create_bootloader_command()
        self.mock_device_handle.write.return_value = 0  # Failed write
        
        result = self.command_handler.send_command('TEST123', command)
        
        self.assertFalse(result)
    
    def test_send_command_exception(self):
        """Test command transmission handles exceptions"""
        command = self.command_handler.create_bootloader_command()
        self.mock_device_handle.write.side_effect = Exception("USB error")
        
        result = self.command_handler.send_command('TEST123', command)
        
        self.assertFalse(result)
    
    def test_response_from_log_message_valid(self):
        """Test parsing valid response from log message"""
        log_message = 'TEST_RESPONSE:{"command_id": 42, "status": 0, "type": "ack", "data": {}}'
        
        response = TestResponse.from_log_message(log_message)
        
        self.assertIsNotNone(response)
        self.assertEqual(response.command_id, 42)
        self.assertEqual(response.status, ResponseStatus.SUCCESS)
        self.assertEqual(response.response_type, 'ack')
    
    def test_response_from_log_message_invalid(self):
        """Test parsing invalid log message"""
        log_message = 'REGULAR_LOG: This is not a test response'
        
        response = TestResponse.from_log_message(log_message)
        
        self.assertIsNone(response)
    
    def test_response_from_log_message_malformed_json(self):
        """Test parsing response with malformed JSON"""
        log_message = 'TEST_RESPONSE:{"invalid": json}'
        
        response = TestResponse.from_log_message(log_message)
        
        self.assertIsNone(response)
    
    def test_read_responses_success(self):
        """Test reading responses from device"""
        # Mock HID read to return test response
        response_json = '{"command_id": 42, "status": 0, "type": "ack", "data": {}}'
        log_message = f'TEST_RESPONSE:{response_json}'
        log_bytes = log_message.encode('utf-8').ljust(64, b'\x00')
        
        self.mock_device_handle.read.side_effect = [log_bytes, []]  # First call returns data, second returns empty
        
        responses = self.command_handler.read_responses('TEST123')
        
        self.assertEqual(len(responses), 1)
        self.assertEqual(responses[0].command_id, 42)
        self.assertEqual(responses[0].status, ResponseStatus.SUCCESS)
    
    def test_read_responses_mixed_messages(self):
        """Test reading responses mixed with regular log messages"""
        # Mock multiple messages
        response_msg = 'TEST_RESPONSE:{"command_id": 42, "status": 0, "type": "ack", "data": {}}'
        regular_msg = 'INFO: Regular log message'
        
        response_bytes = response_msg.encode('utf-8').ljust(64, b'\x00')
        regular_bytes = regular_msg.encode('utf-8').ljust(64, b'\x00')
        
        self.mock_device_handle.read.side_effect = [response_bytes, regular_bytes, []]
        
        responses = self.command_handler.read_responses('TEST123')
        
        self.assertEqual(len(responses), 1)
        self.assertEqual(len(self.command_handler.response_buffer), 1)  # Regular message buffered
        self.assertEqual(self.command_handler.response_buffer[0], regular_msg)
    
    def test_read_responses_device_not_connected(self):
        """Test reading responses from disconnected device"""
        self.mock_device_manager.is_device_connected.return_value = False
        
        responses = self.command_handler.read_responses('TEST123')
        
        self.assertEqual(len(responses), 0)
        self.mock_device_handle.read.assert_not_called()
    
    @patch('time.sleep')
    @patch('time.time')
    def test_wait_for_response_success(self, mock_time, mock_sleep):
        """Test waiting for specific response"""
        # Mock time progression
        mock_time.side_effect = [0, 0.5, 1.0, 1.5]  # Start, check, found, end
        
        # Mock response data
        response_json = '{"command_id": 42, "status": 0, "type": "ack", "data": {}}'
        log_message = f'TEST_RESPONSE:{response_json}'
        log_bytes = log_message.encode('utf-8').ljust(64, b'\x00')
        
        # First read returns empty, second returns response
        self.mock_device_handle.read.side_effect = [[], log_bytes, []]
        
        response = self.command_handler.wait_for_response('TEST123', 42, timeout=2.0)
        
        self.assertIsNotNone(response)
        self.assertEqual(response.command_id, 42)
    
    @patch('time.sleep')
    @patch('time.time')
    def test_wait_for_response_timeout(self, mock_time, mock_sleep):
        """Test waiting for response times out"""
        # Mock time progression to exceed timeout
        mock_time.side_effect = [0, 1.0, 2.5]  # Start, check, timeout
        
        self.mock_device_handle.read.return_value = []  # No data
        
        response = self.command_handler.wait_for_response('TEST123', 42, timeout=2.0)
        
        self.assertIsNone(response)
    
    def test_send_command_and_wait_success(self):
        """Test sending command and waiting for response"""
        command = self.command_handler.create_bootloader_command()
        self.mock_device_handle.write.return_value = 64
        
        # Mock successful send
        with patch.object(self.command_handler, 'send_command', return_value=True):
            with patch.object(self.command_handler, 'wait_for_response') as mock_wait:
                mock_response = Mock()
                mock_response.command_id = command.command_id
                mock_wait.return_value = mock_response
                
                response = self.command_handler.send_command_and_wait('TEST123', command)
        
        self.assertIsNotNone(response)
        self.assertEqual(response.command_id, command.command_id)
    
    def test_send_command_and_wait_send_failure(self):
        """Test send_command_and_wait when send fails"""
        command = self.command_handler.create_bootloader_command()
        self.mock_device_handle.write.return_value = 0  # Send failure
        
        response = self.command_handler.send_command_and_wait('TEST123', command)
        
        self.assertIsNone(response)
    
    def test_get_pending_commands(self):
        """Test getting pending commands"""
        command = self.command_handler.create_bootloader_command()
        command.command_id = 42
        self.command_handler.pending_commands[42] = command
        
        pending = self.command_handler.get_pending_commands()
        
        self.assertEqual(len(pending), 1)
        self.assertIn(42, pending)
        self.assertEqual(pending[42], command)
    
    def test_clear_pending_commands(self):
        """Test clearing pending commands"""
        command = self.command_handler.create_bootloader_command()
        command.command_id = 42
        self.command_handler.pending_commands[42] = command
        
        self.command_handler.clear_pending_commands()
        
        self.assertEqual(len(self.command_handler.pending_commands), 0)
    
    def test_command_sequence_increment(self):
        """Test command sequence number increments"""
        command1 = self.command_handler.create_bootloader_command()
        command2 = self.command_handler.create_bootloader_command()
        
        self.mock_device_handle.write.return_value = 64
        
        self.command_handler.send_command('TEST123', command1)
        self.command_handler.send_command('TEST123', command2)
        
        self.assertNotEqual(command1.command_id, command2.command_id)
        self.assertEqual(command2.command_id, command1.command_id + 1)


if __name__ == '__main__':
    unittest.main()