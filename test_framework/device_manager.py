"""
USB HID Device Discovery and Connection Management

Handles device discovery, connection management, and multi-device support
for the automated testing framework.
"""

import hid
import time
import logging
from typing import List, Optional, Dict, Any
from dataclasses import dataclass
from enum import Enum


class DeviceStatus(Enum):
    """Device connection status"""
    DISCONNECTED = "disconnected"
    CONNECTED = "connected"
    BOOTLOADER = "bootloader"
    ERROR = "error"


@dataclass
class DeviceInfo:
    """Information about a connected device"""
    vendor_id: int
    product_id: int
    serial_number: str
    manufacturer: str
    product: str
    path: bytes
    status: DeviceStatus
    last_seen: float


class UsbHidDeviceManager:
    """
    Manages USB HID device discovery and connection for automated testing.
    
    Supports multi-device testing with device identification and status tracking.
    """
    
    # Device identification constants
    VENDOR_ID = 0x2E8A  # Raspberry Pi Foundation
    PRODUCT_ID = 0x000A  # RP2040 USB HID
    BOOTLOADER_PRODUCT_ID = 0x0003  # RP2040 Bootloader
    
    def __init__(self, connection_timeout: float = 5.0, discovery_interval: float = 1.0):
        """
        Initialize the device manager.
        
        Args:
            connection_timeout: Timeout for device connection attempts
            discovery_interval: Interval between device discovery scans
        """
        self.connection_timeout = connection_timeout
        self.discovery_interval = discovery_interval
        self.devices: Dict[str, DeviceInfo] = {}
        self.connections: Dict[str, hid.Device] = {}
        self.logger = logging.getLogger(__name__)
        
    def discover_devices(self) -> List[DeviceInfo]:
        """
        Discover all connected test devices.
        
        Returns:
            List of discovered device information
        """
        discovered_devices = []
        current_time = time.time()
        
        try:
            # Scan for normal operation devices
            hid_devices = hid.enumerate(self.VENDOR_ID, self.PRODUCT_ID)
            for device_dict in hid_devices:
                device_info = self._create_device_info(device_dict, DeviceStatus.CONNECTED)
                discovered_devices.append(device_info)
                
            # Scan for bootloader mode devices
            bootloader_devices = hid.enumerate(self.VENDOR_ID, self.BOOTLOADER_PRODUCT_ID)
            for device_dict in bootloader_devices:
                device_info = self._create_device_info(device_dict, DeviceStatus.BOOTLOADER)
                discovered_devices.append(device_info)
                
            # Update device registry
            for device in discovered_devices:
                device.last_seen = current_time
                self.devices[device.serial_number] = device
                
            # Mark missing devices as disconnected
            for serial, device in self.devices.items():
                if device.last_seen < current_time - self.discovery_interval * 2:
                    device.status = DeviceStatus.DISCONNECTED
                    
        except Exception as e:
            self.logger.error(f"Device discovery failed: {e}")
            
        return discovered_devices
    
    def _create_device_info(self, device_dict: Dict[str, Any], status: DeviceStatus) -> DeviceInfo:
        """Create DeviceInfo from HID device dictionary"""
        return DeviceInfo(
            vendor_id=device_dict.get('vendor_id', 0),
            product_id=device_dict.get('product_id', 0),
            serial_number=device_dict.get('serial_number', ''),
            manufacturer=device_dict.get('manufacturer_string', ''),
            product=device_dict.get('product_string', ''),
            path=device_dict.get('path', b''),
            status=status,
            last_seen=time.time()
        )
    
    def connect_device(self, serial_number: str) -> bool:
        """
        Connect to a specific device by serial number.
        
        Args:
            serial_number: Device serial number to connect to
            
        Returns:
            True if connection successful, False otherwise
        """
        if serial_number in self.connections:
            return True
            
        device_info = self.devices.get(serial_number)
        if not device_info or device_info.status == DeviceStatus.DISCONNECTED:
            self.logger.error(f"Device {serial_number} not found or disconnected")
            return False
            
        try:
            device = hid.Device()
            device.open_path(device_info.path)
            device.set_nonblocking(True)
            
            self.connections[serial_number] = device
            self.logger.info(f"Connected to device {serial_number}")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to connect to device {serial_number}: {e}")
            return False
    
    def disconnect_device(self, serial_number: str) -> bool:
        """
        Disconnect from a specific device.
        
        Args:
            serial_number: Device serial number to disconnect from
            
        Returns:
            True if disconnection successful, False otherwise
        """
        if serial_number not in self.connections:
            return True
            
        try:
            self.connections[serial_number].close()
            del self.connections[serial_number]
            self.logger.info(f"Disconnected from device {serial_number}")
            return True
            
        except Exception as e:
            self.logger.error(f"Failed to disconnect from device {serial_number}: {e}")
            return False
    
    def disconnect_all(self) -> None:
        """Disconnect from all connected devices"""
        for serial_number in list(self.connections.keys()):
            self.disconnect_device(serial_number)
    
    def get_connected_devices(self) -> List[str]:
        """
        Get list of currently connected device serial numbers.
        
        Returns:
            List of connected device serial numbers
        """
        return list(self.connections.keys())
    
    def get_device_info(self, serial_number: str) -> Optional[DeviceInfo]:
        """
        Get device information for a specific device.
        
        Args:
            serial_number: Device serial number
            
        Returns:
            Device information or None if not found
        """
        return self.devices.get(serial_number)
    
    def is_device_connected(self, serial_number: str) -> bool:
        """
        Check if a device is currently connected.
        
        Args:
            serial_number: Device serial number to check
            
        Returns:
            True if device is connected, False otherwise
        """
        return serial_number in self.connections
    
    def wait_for_device(self, serial_number: str, timeout: float = None, 
                       expected_status: DeviceStatus = None) -> bool:
        """
        Wait for a specific device to become available.
        
        Args:
            serial_number: Device serial number to wait for
            timeout: Maximum time to wait (uses connection_timeout if None)
            expected_status: Expected device status (any connected status if None)
            
        Returns:
            True if device becomes available, False if timeout
        """
        if timeout is None:
            timeout = self.connection_timeout
            
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            self.discover_devices()
            device_info = self.get_device_info(serial_number)
            
            if device_info:
                if expected_status:
                    if device_info.status == expected_status:
                        return True
                else:
                    if device_info.status in [DeviceStatus.CONNECTED, DeviceStatus.BOOTLOADER]:
                        return True
                
            time.sleep(self.discovery_interval)
            
        return False
    
    def wait_for_device_reconnection(self, serial_number: str, timeout: float = 30.0) -> bool:
        """
        Wait for a device to reconnect after disconnection (e.g., after firmware flash).
        
        Args:
            serial_number: Device serial number to wait for
            timeout: Maximum time to wait for reconnection
            
        Returns:
            True if device reconnected successfully, False if timeout
        """
        self.logger.info(f"Waiting for device {serial_number} to reconnect...")
        
        # First wait for device to appear as connected (not bootloader)
        if not self.wait_for_device(serial_number, timeout, DeviceStatus.CONNECTED):
            return False
            
        # Then try to establish actual connection
        return self.connect_device(serial_number)
    
    def wait_for_bootloader_mode(self, serial_number: str, timeout: float = 10.0) -> bool:
        """
        Wait for a device to enter bootloader mode.
        
        Args:
            serial_number: Device serial number to wait for
            timeout: Maximum time to wait for bootloader mode
            
        Returns:
            True if device entered bootloader mode, False if timeout
        """
        return self.wait_for_device(serial_number, timeout, DeviceStatus.BOOTLOADER)
    
    def get_device_handle(self, serial_number: str) -> Optional[hid.Device]:
        """
        Get the HID device handle for a connected device.
        
        Args:
            serial_number: Device serial number
            
        Returns:
            HID device handle or None if not connected
        """
        return self.connections.get(serial_number)