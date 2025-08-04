"""
Unit tests for USB HID Device Manager
"""

import unittest
from unittest.mock import Mock, patch, MagicMock
import time

from test_framework.device_manager import UsbHidDeviceManager, DeviceInfo, DeviceStatus


class TestUsbHidDeviceManager(unittest.TestCase):
    """Test cases for UsbHidDeviceManager"""
    
    def setUp(self):
        """Set up test fixtures"""
        self.device_manager = UsbHidDeviceManager(
            connection_timeout=2.0,
            discovery_interval=0.5
        )
        
        # Mock device data
        self.mock_device_dict = {
            'vendor_id': 0x2E8A,
            'product_id': 0x000A,
            'serial_number': 'TEST123',
            'manufacturer_string': 'Test Manufacturer',
            'product_string': 'Test Device',
            'path': b'/dev/hidraw0'
        }
    
    @patch('test_framework.device_manager.hid.enumerate')
    def test_discover_devices_normal_operation(self, mock_enumerate):
        """Test device discovery in normal operation mode"""
        def mock_enumerate_side_effect(vid, pid):
            if pid == 0x000A:
                return [self.mock_device_dict]
            elif pid == 0x0003:
                return []
            return []
        
        mock_enumerate.side_effect = mock_enumerate_side_effect
        
        devices = self.device_manager.discover_devices()
        
        self.assertEqual(len(devices), 1)
        device = devices[0]
        self.assertEqual(device.vendor_id, 0x2E8A)
        self.assertEqual(device.product_id, 0x000A)
        self.assertEqual(device.serial_number, 'TEST123')
        self.assertEqual(device.status, DeviceStatus.CONNECTED)
        
        # Verify device is stored in registry
        self.assertIn('TEST123', self.device_manager.devices)
    
    @patch('test_framework.device_manager.hid.enumerate')
    def test_discover_devices_bootloader_mode(self, mock_enumerate):
        """Test device discovery in bootloader mode"""
        bootloader_device = self.mock_device_dict.copy()
        bootloader_device['product_id'] = 0x0003
        
        def mock_enumerate_side_effect(vid, pid):
            if pid == 0x000A:
                return []
            elif pid == 0x0003:
                return [bootloader_device]
            return []
        
        mock_enumerate.side_effect = mock_enumerate_side_effect
        
        devices = self.device_manager.discover_devices()
        
        self.assertEqual(len(devices), 1)
        device = devices[0]
        self.assertEqual(device.status, DeviceStatus.BOOTLOADER)
    
    @patch('test_framework.device_manager.hid.enumerate')
    def test_discover_devices_handles_exceptions(self, mock_enumerate):
        """Test device discovery handles exceptions gracefully"""
        mock_enumerate.side_effect = Exception("USB error")
        
        devices = self.device_manager.discover_devices()
        
        self.assertEqual(len(devices), 0)
    
    @patch('test_framework.device_manager.hid.Device')
    def test_connect_device_success(self, mock_hid_device):
        """Test successful device connection"""
        # Set up device in registry
        device_info = DeviceInfo(
            vendor_id=0x2E8A,
            product_id=0x000A,
            serial_number='TEST123',
            manufacturer='Test',
            product='Test Device',
            path=b'/dev/hidraw0',
            status=DeviceStatus.CONNECTED,
            last_seen=time.time()
        )
        self.device_manager.devices['TEST123'] = device_info
        
        # Mock HID device
        mock_device_instance = Mock()
        mock_hid_device.return_value = mock_device_instance
        
        result = self.device_manager.connect_device('TEST123')
        
        self.assertTrue(result)
        mock_device_instance.open_path.assert_called_once_with(b'/dev/hidraw0')
        mock_device_instance.set_nonblocking.assert_called_once_with(True)
        self.assertIn('TEST123', self.device_manager.connections)
    
    def test_connect_device_not_found(self):
        """Test connection to non-existent device"""
        result = self.device_manager.connect_device('NONEXISTENT')
        
        self.assertFalse(result)
        self.assertNotIn('NONEXISTENT', self.device_manager.connections)
    
    @patch('test_framework.device_manager.hid.Device')
    def test_connect_device_exception(self, mock_hid_device):
        """Test device connection handles exceptions"""
        # Set up device in registry
        device_info = DeviceInfo(
            vendor_id=0x2E8A,
            product_id=0x000A,
            serial_number='TEST123',
            manufacturer='Test',
            product='Test Device',
            path=b'/dev/hidraw0',
            status=DeviceStatus.CONNECTED,
            last_seen=time.time()
        )
        self.device_manager.devices['TEST123'] = device_info
        
        # Mock HID device to raise exception
        mock_device_instance = Mock()
        mock_device_instance.open_path.side_effect = Exception("Connection failed")
        mock_hid_device.return_value = mock_device_instance
        
        result = self.device_manager.connect_device('TEST123')
        
        self.assertFalse(result)
        self.assertNotIn('TEST123', self.device_manager.connections)
    
    def test_disconnect_device_success(self):
        """Test successful device disconnection"""
        # Set up connected device
        mock_device = Mock()
        self.device_manager.connections['TEST123'] = mock_device
        
        result = self.device_manager.disconnect_device('TEST123')
        
        self.assertTrue(result)
        mock_device.close.assert_called_once()
        self.assertNotIn('TEST123', self.device_manager.connections)
    
    def test_disconnect_device_not_connected(self):
        """Test disconnection from non-connected device"""
        result = self.device_manager.disconnect_device('NONEXISTENT')
        
        self.assertTrue(result)  # Should return True for already disconnected
    
    def test_disconnect_all(self):
        """Test disconnecting all devices"""
        # Set up multiple connected devices
        mock_device1 = Mock()
        mock_device2 = Mock()
        self.device_manager.connections['TEST1'] = mock_device1
        self.device_manager.connections['TEST2'] = mock_device2
        
        self.device_manager.disconnect_all()
        
        mock_device1.close.assert_called_once()
        mock_device2.close.assert_called_once()
        self.assertEqual(len(self.device_manager.connections), 0)
    
    def test_get_connected_devices(self):
        """Test getting list of connected devices"""
        self.device_manager.connections['TEST1'] = Mock()
        self.device_manager.connections['TEST2'] = Mock()
        
        connected = self.device_manager.get_connected_devices()
        
        self.assertEqual(set(connected), {'TEST1', 'TEST2'})
    
    def test_is_device_connected(self):
        """Test checking device connection status"""
        self.device_manager.connections['TEST123'] = Mock()
        
        self.assertTrue(self.device_manager.is_device_connected('TEST123'))
        self.assertFalse(self.device_manager.is_device_connected('NONEXISTENT'))
    
    def test_get_device_handle(self):
        """Test getting device handle"""
        mock_device = Mock()
        self.device_manager.connections['TEST123'] = mock_device
        
        handle = self.device_manager.get_device_handle('TEST123')
        
        self.assertEqual(handle, mock_device)
        self.assertIsNone(self.device_manager.get_device_handle('NONEXISTENT'))
    
    @patch('test_framework.device_manager.hid.enumerate')
    @patch('time.sleep')
    def test_wait_for_device_success(self, mock_sleep, mock_enumerate):
        """Test waiting for device to become available"""
        # Mock enumerate to return empty first, then device
        def mock_enumerate_side_effect(vid, pid):
            if not hasattr(mock_enumerate_side_effect, 'call_count'):
                mock_enumerate_side_effect.call_count = 0
            mock_enumerate_side_effect.call_count += 1
            
            if pid == 0x000A:
                if mock_enumerate_side_effect.call_count <= 2:  # First discover call
                    return []
                else:  # Second discover call
                    return [self.mock_device_dict]
            elif pid == 0x0003:
                return []
            return []
        
        mock_enumerate.side_effect = mock_enumerate_side_effect
        
        result = self.device_manager.wait_for_device('TEST123', timeout=1.0)
        
        self.assertTrue(result)
        mock_sleep.assert_called()
    
    @patch('test_framework.device_manager.hid.enumerate')
    @patch('time.sleep')
    def test_wait_for_device_timeout(self, mock_sleep, mock_enumerate):
        """Test waiting for device times out"""
        mock_enumerate.return_value = []
        
        result = self.device_manager.wait_for_device('TEST123', timeout=0.1)
        
        self.assertFalse(result)


if __name__ == '__main__':
    unittest.main()