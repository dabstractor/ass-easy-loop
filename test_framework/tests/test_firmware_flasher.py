"""
Integration tests for firmware flashing and bootloader mode triggering.

Tests the complete workflow of bootloader entry, firmware flashing,
and device reconnection detection.
"""

import unittest
import time
import tempfile
import os
import subprocess
from unittest.mock import Mock, patch, MagicMock
import threading

from test_framework.firmware_flasher import FirmwareFlasher, FlashResult, FlashOperation
from test_framework.device_manager import UsbHidDeviceManager, DeviceStatus, DeviceInfo
from test_framework.command_handler import CommandHandler, TestResponse, ResponseStatus


class TestFirmwareFlasher(unittest.TestCase):
    """Test cases for firmware flashing functionality"""
    
    def setUp(self):
        """Set up test fixtures"""
        self.device_manager = Mock(spec=UsbHidDeviceManager)
        self.command_handler = Mock(spec=CommandHandler)
        self.flasher = FirmwareFlasher(
            device_manager=self.device_manager,
            command_handler=self.command_handler,
            bootloader_timeout=5.0,
            reconnection_timeout=10.0,
            flash_tool_path='/usr/bin/picotool'
        )
        
        # Create temporary firmware file for testing
        self.temp_firmware = tempfile.NamedTemporaryFile(suffix='.uf2', delete=False)
        self.temp_firmware.write(b'fake firmware data')
        self.temp_firmware.close()
        
    def tearDown(self):
        """Clean up test fixtures"""
        if os.path.exists(self.temp_firmware.name):
            os.unlink(self.temp_firmware.name)
    
    def test_detect_flash_tool(self):
        """Test automatic flash tool detection"""
        with patch('subprocess.run') as mock_run:
            # Mock successful tool detection
            mock_run.return_value.returncode = 0
            mock_run.return_value.stdout = '/usr/bin/picotool\n'
            
            flasher = FirmwareFlasher(self.device_manager, self.command_handler)
            self.assertEqual(flasher.flash_tool_path, '/usr/bin/picotool')
    
    def test_trigger_bootloader_mode_success(self):
        """Test successful bootloader mode triggering"""
        device_serial = "test_device_123"
        
        # Mock device connected
        self.device_manager.is_device_connected.return_value = True
        
        # Mock successful command response
        success_response = Mock()
        success_response.status.value = 0
        self.command_handler.send_command_and_wait.return_value = success_response
        
        # Mock device disconnection and bootloader mode detection
        self.device_manager.is_device_connected.side_effect = [True, False, False]
        
        bootloader_device = DeviceInfo(
            vendor_id=0x2E8A,
            product_id=0x0003,
            serial_number=device_serial,
            manufacturer="Test",
            product="Test Device",
            path=b"/dev/hidraw0",
            status=DeviceStatus.BOOTLOADER,
            last_seen=time.time()
        )
        self.device_manager.get_device_info.return_value = bootloader_device
        
        # Test bootloader triggering
        result = self.flasher.trigger_bootloader_mode(device_serial)
        
        self.assertTrue(result)
        self.command_handler.create_bootloader_command.assert_called_once_with(5000)
        self.command_handler.send_command_and_wait.assert_called_once()
        self.device_manager.discover_devices.assert_called()
    
    def test_trigger_bootloader_mode_device_not_connected(self):
        """Test bootloader triggering when device not connected"""
        device_serial = "test_device_123"
        
        # Mock device not connected
        self.device_manager.is_device_connected.return_value = False
        
        result = self.flasher.trigger_bootloader_mode(device_serial)
        
        self.assertFalse(result)
        self.command_handler.send_command_and_wait.assert_not_called()
    
    def test_trigger_bootloader_mode_command_failed(self):
        """Test bootloader triggering when command fails"""
        device_serial = "test_device_123"
        
        # Mock device connected
        self.device_manager.is_device_connected.return_value = True
        
        # Mock failed command response
        error_response = Mock()
        error_response.status.value = 1
        error_response.status.name = "ERROR_SYSTEM_BUSY"
        self.command_handler.send_command_and_wait.return_value = error_response
        
        result = self.flasher.trigger_bootloader_mode(device_serial)
        
        self.assertFalse(result)
    
    def test_trigger_bootloader_mode_timeout(self):
        """Test bootloader triggering timeout"""
        device_serial = "test_device_123"
        
        # Mock device connected
        self.device_manager.is_device_connected.return_value = True
        
        # Mock successful command response
        success_response = Mock()
        success_response.status.value = 0
        self.command_handler.send_command_and_wait.return_value = success_response
        
        # Mock device never disconnects (timeout scenario)
        self.device_manager.is_device_connected.return_value = True
        
        # Use shorter timeout for testing
        self.flasher.bootloader_timeout = 1.0
        
        result = self.flasher.trigger_bootloader_mode(device_serial)
        
        self.assertFalse(result)
    
    @patch('subprocess.run')
    def test_execute_firmware_flash_success(self, mock_run):
        """Test successful firmware flashing execution"""
        device_serial = "test_device_123"
        
        # Mock successful subprocess execution
        mock_run.return_value.returncode = 0
        mock_run.return_value.stderr = ""
        
        result = self.flasher._execute_firmware_flash(device_serial, self.temp_firmware.name)
        
        self.assertTrue(result)
        mock_run.assert_called_once()
        
        # Verify correct command was called
        called_args = mock_run.call_args[0][0]
        self.assertIn('picotool', called_args[0])
        self.assertIn('load', called_args)
        self.assertIn(self.temp_firmware.name, called_args)
    
    @patch('subprocess.run')
    def test_execute_firmware_flash_failure(self, mock_run):
        """Test firmware flashing execution failure"""
        device_serial = "test_device_123"
        
        # Mock failed subprocess execution
        mock_run.return_value.returncode = 1
        mock_run.return_value.stderr = "Flash failed: device not found"
        
        result = self.flasher._execute_firmware_flash(device_serial, self.temp_firmware.name)
        
        self.assertFalse(result)
    
    @patch('subprocess.run')
    def test_execute_firmware_flash_timeout(self, mock_run):
        """Test firmware flashing timeout"""
        device_serial = "test_device_123"
        
        # Mock subprocess timeout
        mock_run.side_effect = subprocess.TimeoutExpired('picotool', 60.0)
        
        result = self.flasher._execute_firmware_flash(device_serial, self.temp_firmware.name)
        
        self.assertFalse(result)
    
    def test_wait_for_reconnection_success(self):
        """Test successful device reconnection detection"""
        device_serial = "test_device_123"
        
        # Mock device reconnection
        connected_device = DeviceInfo(
            vendor_id=0x2E8A,
            product_id=0x000A,
            serial_number=device_serial,
            manufacturer="Test",
            product="Test Device",
            path=b"/dev/hidraw0",
            status=DeviceStatus.CONNECTED,
            last_seen=time.time()
        )
        
        self.device_manager.get_device_info.return_value = connected_device
        self.device_manager.connect_device.return_value = True
        
        result = self.flasher._wait_for_reconnection(device_serial)
        
        self.assertTrue(result)
        self.device_manager.discover_devices.assert_called()
        self.device_manager.connect_device.assert_called_with(device_serial)
    
    def test_wait_for_reconnection_timeout(self):
        """Test device reconnection timeout"""
        device_serial = "test_device_123"
        
        # Mock device never reconnects
        self.device_manager.get_device_info.return_value = None
        
        # Use shorter timeout for testing
        self.flasher.reconnection_timeout = 1.0
        
        result = self.flasher._wait_for_reconnection(device_serial)
        
        self.assertFalse(result)
    
    def test_flash_firmware_complete_workflow(self):
        """Test complete firmware flashing workflow"""
        device_serial = "test_device_123"
        
        # Mock successful bootloader triggering
        with patch.object(self.flasher, 'trigger_bootloader_mode', return_value=True):
            # Mock successful firmware flashing
            with patch.object(self.flasher, '_execute_firmware_flash', return_value=True):
                # Mock successful reconnection
                with patch.object(self.flasher, '_wait_for_reconnection', return_value=True):
                    
                    operation = self.flasher.flash_firmware(device_serial, self.temp_firmware.name)
                    
                    self.assertEqual(operation.result, FlashResult.SUCCESS)
                    self.assertEqual(operation.device_serial, device_serial)
                    self.assertEqual(operation.firmware_path, self.temp_firmware.name)
                    self.assertIsNotNone(operation.bootloader_entry_time)
                    self.assertIsNotNone(operation.flash_duration)
                    self.assertIsNotNone(operation.reconnection_time)
                    self.assertIsNotNone(operation.total_duration)
    
    def test_flash_firmware_bootloader_entry_failed(self):
        """Test firmware flashing with bootloader entry failure"""
        device_serial = "test_device_123"
        
        # Mock failed bootloader triggering
        with patch.object(self.flasher, 'trigger_bootloader_mode', return_value=False):
            
            operation = self.flasher.flash_firmware(device_serial, self.temp_firmware.name)
            
            self.assertEqual(operation.result, FlashResult.BOOTLOADER_ENTRY_FAILED)
            self.assertEqual(operation.error_message, "Failed to enter bootloader mode")
    
    def test_flash_firmware_flash_failed(self):
        """Test firmware flashing with flash operation failure"""
        device_serial = "test_device_123"
        
        # Mock successful bootloader triggering but failed flashing
        with patch.object(self.flasher, 'trigger_bootloader_mode', return_value=True):
            with patch.object(self.flasher, '_execute_firmware_flash', return_value=False):
                
                operation = self.flasher.flash_firmware(device_serial, self.temp_firmware.name)
                
                self.assertEqual(operation.result, FlashResult.FLASH_FAILED)
                self.assertEqual(operation.error_message, "Firmware flashing failed")
    
    def test_flash_firmware_reconnection_failed(self):
        """Test firmware flashing with reconnection failure"""
        device_serial = "test_device_123"
        
        # Mock successful bootloader and flash but failed reconnection
        with patch.object(self.flasher, 'trigger_bootloader_mode', return_value=True):
            with patch.object(self.flasher, '_execute_firmware_flash', return_value=True):
                with patch.object(self.flasher, '_wait_for_reconnection', return_value=False):
                    
                    operation = self.flasher.flash_firmware(device_serial, self.temp_firmware.name)
                    
                    self.assertEqual(operation.result, FlashResult.RECONNECTION_FAILED)
                    self.assertEqual(operation.error_message, "Device did not reconnect after flashing")
    
    def test_flash_multiple_devices_sequential(self):
        """Test sequential multi-device firmware flashing"""
        device_firmware_map = {
            "device_1": self.temp_firmware.name,
            "device_2": self.temp_firmware.name
        }
        
        # Mock successful flash operations
        with patch.object(self.flasher, 'flash_firmware') as mock_flash:
            mock_operation = Mock(spec=FlashOperation)
            mock_operation.result = FlashResult.SUCCESS
            mock_flash.return_value = mock_operation
            
            results = self.flasher.flash_multiple_devices(device_firmware_map, parallel=False)
            
            self.assertEqual(len(results), 2)
            self.assertEqual(mock_flash.call_count, 2)
            
            # Verify all devices were processed
            for device_serial in device_firmware_map:
                self.assertIn(device_serial, results)
                self.assertEqual(results[device_serial].result, FlashResult.SUCCESS)
    
    def test_flash_multiple_devices_parallel(self):
        """Test parallel multi-device firmware flashing"""
        device_firmware_map = {
            "device_1": self.temp_firmware.name,
            "device_2": self.temp_firmware.name,
            "device_3": self.temp_firmware.name
        }
        
        # Mock successful flash operations with delay to test parallelism
        def mock_flash_with_delay(device_serial, firmware_path):
            time.sleep(0.1)  # Small delay to simulate real operation
            operation = Mock(spec=FlashOperation)
            operation.result = FlashResult.SUCCESS
            operation.device_serial = device_serial
            operation.firmware_path = firmware_path
            return operation
        
        with patch.object(self.flasher, 'flash_firmware', side_effect=mock_flash_with_delay):
            start_time = time.time()
            results = self.flasher.flash_multiple_devices(device_firmware_map, parallel=True, max_parallel=2)
            duration = time.time() - start_time
            
            self.assertEqual(len(results), 3)
            
            # Parallel execution should be faster than sequential
            # (3 devices * 0.1s delay = 0.3s sequential, but parallel should be ~0.2s)
            self.assertLess(duration, 0.25)
            
            # Verify all devices were processed
            for device_serial in device_firmware_map:
                self.assertIn(device_serial, results)
                self.assertEqual(results[device_serial].result, FlashResult.SUCCESS)
    
    def test_verify_flash_tool_availability(self):
        """Test flash tool availability verification"""
        with patch('subprocess.run') as mock_run:
            # Mock successful tool verification
            mock_run.return_value.returncode = 0
            
            result = self.flasher.verify_flash_tool_availability()
            
            self.assertTrue(result)
            mock_run.assert_called_once()
    
    def test_verify_flash_tool_unavailable(self):
        """Test flash tool unavailability detection"""
        # Test with no flash tool path
        flasher = FirmwareFlasher(self.device_manager, self.command_handler, flash_tool_path=None)
        
        result = flasher.verify_flash_tool_availability()
        
        self.assertFalse(result)
    
    def test_get_supported_firmware_formats(self):
        """Test supported firmware format detection"""
        formats = self.flasher.get_supported_firmware_formats()
        
        # Should support common RP2040 formats for picotool
        expected_formats = ['.elf', '.uf2', '.bin']
        self.assertEqual(formats, expected_formats)
    
    def test_operation_tracking(self):
        """Test flash operation tracking and status"""
        device_serial = "test_device_123"
        
        # Start a flash operation in a separate thread
        def flash_operation():
            with patch.object(self.flasher, 'trigger_bootloader_mode', return_value=True):
                with patch.object(self.flasher, '_execute_firmware_flash', return_value=True):
                    with patch.object(self.flasher, '_wait_for_reconnection', return_value=True):
                        self.flasher.flash_firmware(device_serial, self.temp_firmware.name)
        
        thread = threading.Thread(target=flash_operation)
        thread.start()
        
        # Check operation status while running
        time.sleep(0.1)  # Give thread time to start
        operation = self.flasher.get_operation_status(device_serial)
        
        self.assertIsNotNone(operation)
        self.assertEqual(operation.device_serial, device_serial)
        
        thread.join()
        
        # Check final status
        final_operation = self.flasher.get_operation_status(device_serial)
        self.assertEqual(final_operation.result, FlashResult.SUCCESS)


if __name__ == '__main__':
    unittest.main()