"""
Firmware Flashing and Bootloader Management

Handles bootloader mode triggering, firmware flashing using existing tools,
and device reconnection detection for automated testing workflows.
"""

import os
import subprocess
import time
import logging
from typing import Optional, Dict, Any, List
from dataclasses import dataclass
from enum import Enum
import threading

from .device_manager import UsbHidDeviceManager, DeviceStatus
from .command_handler import CommandHandler, CommandType, TestCommand


class FlashResult(Enum):
    """Firmware flashing result status"""
    SUCCESS = "success"
    BOOTLOADER_ENTRY_FAILED = "bootloader_entry_failed"
    FLASH_FAILED = "flash_failed"
    RECONNECTION_FAILED = "reconnection_failed"
    TIMEOUT = "timeout"
    ERROR = "error"


@dataclass
class FlashOperation:
    """Firmware flashing operation tracking"""
    device_serial: str
    firmware_path: str
    start_time: float
    end_time: Optional[float] = None
    result: Optional[FlashResult] = None
    error_message: Optional[str] = None
    bootloader_entry_time: Optional[float] = None
    flash_duration: Optional[float] = None
    reconnection_time: Optional[float] = None
    
    @property
    def total_duration(self) -> Optional[float]:
        """Get total operation duration"""
        if self.start_time and self.end_time:
            return self.end_time - self.start_time
        return None


class FirmwareFlasher:
    """
    Manages firmware flashing operations including bootloader mode triggering,
    firmware deployment, and device reconnection detection.
    
    Integrates with existing firmware flashing tools and provides automated
    workflow orchestration for CI/CD environments.
    """
    
    def __init__(self, device_manager: UsbHidDeviceManager, command_handler: CommandHandler,
                 bootloader_timeout: float = 10.0, reconnection_timeout: float = 30.0,
                 flash_tool_path: str = None):
        """
        Initialize the firmware flasher.
        
        Args:
            device_manager: Device manager instance
            command_handler: Command handler instance
            bootloader_timeout: Timeout for bootloader mode entry
            reconnection_timeout: Timeout for device reconnection after flash
            flash_tool_path: Path to firmware flashing tool (auto-detect if None)
        """
        self.device_manager = device_manager
        self.command_handler = command_handler
        self.bootloader_timeout = bootloader_timeout
        self.reconnection_timeout = reconnection_timeout
        self.logger = logging.getLogger(__name__)
        self.flash_tool_path = flash_tool_path or self._detect_flash_tool()
        self.active_operations: Dict[str, FlashOperation] = {}
        self.operation_lock = threading.Lock()
        
    def _detect_flash_tool(self) -> Optional[str]:
        """Auto-detect available firmware flashing tools"""
        # Check for common RP2040 flashing tools
        tools_to_check = [
            'picotool',  # Official Raspberry Pi tool
            'uf2conv.py',  # UF2 conversion tool
            'rp2040load',  # Alternative loader
        ]
        
        for tool in tools_to_check:
            try:
                result = subprocess.run(['which', tool], capture_output=True, text=True)
                if result.returncode == 0:
                    tool_path = result.stdout.strip()
                    self.logger.info(f"Detected flash tool: {tool_path}")
                    return tool_path
            except Exception:
                continue
                
        self.logger.warning("No firmware flashing tool detected. Manual path required.")
        return None
    
    def trigger_bootloader_mode(self, device_serial: str, timeout_ms: int = 5000) -> bool:
        """
        Trigger bootloader mode entry on a specific device.
        
        Args:
            device_serial: Target device serial number
            timeout_ms: Bootloader entry timeout in milliseconds
            
        Returns:
            True if bootloader mode triggered successfully, False otherwise
        """
        if not self.device_manager.is_device_connected(device_serial):
            self.logger.error(f"Device {device_serial} not connected")
            return False
            
        self.logger.info(f"Triggering bootloader mode on device {device_serial}")
        
        try:
            # Create bootloader entry command
            bootloader_command = self.command_handler.create_bootloader_command(timeout_ms)
            
            # Send command and wait for response
            response = self.command_handler.send_command_and_wait(
                device_serial, bootloader_command, timeout=self.bootloader_timeout
            )
            
            if response and response.status.value == 0:  # SUCCESS
                self.logger.info(f"Bootloader command acknowledged by {device_serial}")
                
                # Wait for device to disconnect (entering bootloader mode)
                start_time = time.time()
                while time.time() - start_time < self.bootloader_timeout:
                    if not self.device_manager.is_device_connected(device_serial):
                        self.logger.info(f"Device {device_serial} disconnected, checking for bootloader mode")
                        
                        # Wait a moment for bootloader to initialize
                        time.sleep(1.0)
                        
                        # Check if device reappeared in bootloader mode
                        self.device_manager.discover_devices()
                        device_info = self.device_manager.get_device_info(device_serial)
                        
                        if device_info and device_info.status == DeviceStatus.BOOTLOADER:
                            self.logger.info(f"Device {device_serial} successfully entered bootloader mode")
                            return True
                            
                    time.sleep(0.5)
                    
                self.logger.error(f"Device {device_serial} did not enter bootloader mode within timeout")
                return False
                
            else:
                error_msg = f"Bootloader command failed: {response.status.name if response else 'No response'}"
                self.logger.error(error_msg)
                return False
                
        except Exception as e:
            self.logger.error(f"Error triggering bootloader mode on {device_serial}: {e}")
            return False
    
    def flash_firmware(self, device_serial: str, firmware_path: str) -> FlashOperation:
        """
        Flash firmware to a device with full workflow automation.
        
        Args:
            device_serial: Target device serial number
            firmware_path: Path to firmware file to flash
            
        Returns:
            FlashOperation with complete operation results
        """
        operation = FlashOperation(
            device_serial=device_serial,
            firmware_path=firmware_path,
            start_time=time.time()
        )
        
        with self.operation_lock:
            self.active_operations[device_serial] = operation
            
        try:
            self.logger.info(f"Starting firmware flash operation for {device_serial}")
            
            # Step 1: Trigger bootloader mode
            bootloader_start = time.time()
            if not self.trigger_bootloader_mode(device_serial):
                operation.result = FlashResult.BOOTLOADER_ENTRY_FAILED
                operation.error_message = "Failed to enter bootloader mode"
                return operation
                
            operation.bootloader_entry_time = time.time() - bootloader_start
            
            # Step 2: Flash firmware using external tool
            flash_start = time.time()
            if not self._execute_firmware_flash(device_serial, firmware_path):
                operation.result = FlashResult.FLASH_FAILED
                operation.error_message = "Firmware flashing failed"
                return operation
                
            operation.flash_duration = time.time() - flash_start
            
            # Step 3: Wait for device reconnection
            reconnection_start = time.time()
            if not self._wait_for_reconnection(device_serial):
                operation.result = FlashResult.RECONNECTION_FAILED
                operation.error_message = "Device did not reconnect after flashing"
                return operation
                
            operation.reconnection_time = time.time() - reconnection_start
            operation.result = FlashResult.SUCCESS
            
            self.logger.info(f"Firmware flash completed successfully for {device_serial}")
            
        except Exception as e:
            operation.result = FlashResult.ERROR
            operation.error_message = str(e)
            self.logger.error(f"Firmware flash operation failed: {e}")
            
        finally:
            operation.end_time = time.time()
            
        return operation
    
    def _execute_firmware_flash(self, device_serial: str, firmware_path: str) -> bool:
        """Execute the actual firmware flashing using external tools"""
        if not self.flash_tool_path:
            self.logger.error("No firmware flashing tool available")
            return False
            
        if not os.path.exists(firmware_path):
            self.logger.error(f"Firmware file not found: {firmware_path}")
            return False
            
        try:
            # Determine flashing command based on tool type
            if 'picotool' in self.flash_tool_path:
                cmd = [self.flash_tool_path, 'load', firmware_path, '--force']
            elif 'uf2conv.py' in self.flash_tool_path:
                # For UF2 conversion, we need to find the bootloader mount point
                mount_point = self._find_bootloader_mount()
                if not mount_point:
                    self.logger.error("Could not find bootloader mount point")
                    return False
                cmd = ['cp', firmware_path, mount_point]
            else:
                # Generic approach - assume tool accepts firmware path as argument
                cmd = [self.flash_tool_path, firmware_path]
                
            self.logger.info(f"Executing flash command: {' '.join(cmd)}")
            
            # Execute flashing command with timeout
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=60.0  # 60 second timeout for flashing
            )
            
            if result.returncode == 0:
                self.logger.info("Firmware flashing completed successfully")
                return True
            else:
                self.logger.error(f"Flash command failed: {result.stderr}")
                return False
                
        except subprocess.TimeoutExpired:
            self.logger.error("Firmware flashing timed out")
            return False
        except Exception as e:
            self.logger.error(f"Error executing flash command: {e}")
            return False
    
    def _find_bootloader_mount(self) -> Optional[str]:
        """Find the RP2040 bootloader mount point"""
        # Common mount points for RP2040 bootloader
        possible_mounts = [
            '/media/RPI-RP2',
            '/mnt/RPI-RP2',
            '/Volumes/RPI-RP2',  # macOS
        ]
        
        for mount in possible_mounts:
            if os.path.exists(mount) and os.path.ismount(mount):
                return mount
                
        # Try to find dynamically mounted bootloader
        try:
            result = subprocess.run(['mount'], capture_output=True, text=True)
            for line in result.stdout.split('\n'):
                if 'RPI-RP2' in line:
                    parts = line.split()
                    if len(parts) >= 3:
                        return parts[2]  # Mount point is typically the 3rd field
        except Exception:
            pass
            
        return None
    
    def _wait_for_reconnection(self, device_serial: str) -> bool:
        """Wait for device to reconnect after firmware flashing"""
        self.logger.info(f"Waiting for device {device_serial} to reconnect...")
        
        start_time = time.time()
        while time.time() - start_time < self.reconnection_timeout:
            # Discover devices to update status
            self.device_manager.discover_devices()
            
            # Check if device is back in normal operation mode
            device_info = self.device_manager.get_device_info(device_serial)
            if device_info and device_info.status == DeviceStatus.CONNECTED:
                # Try to connect to verify it's working
                if self.device_manager.connect_device(device_serial):
                    self.logger.info(f"Device {device_serial} reconnected successfully")
                    return True
                    
            time.sleep(1.0)
            
        self.logger.error(f"Device {device_serial} did not reconnect within {self.reconnection_timeout}s")
        return False
    
    def flash_multiple_devices(self, device_firmware_map: Dict[str, str],
                             parallel: bool = True, max_parallel: int = 4) -> Dict[str, FlashOperation]:
        """
        Flash firmware to multiple devices.
        
        Args:
            device_firmware_map: Mapping of device serial numbers to firmware paths
            parallel: Whether to flash devices in parallel
            max_parallel: Maximum number of parallel flash operations
            
        Returns:
            Dictionary mapping device serial numbers to flash operation results
        """
        if not device_firmware_map:
            return {}
            
        self.logger.info(f"Starting firmware flash for {len(device_firmware_map)} devices")
        
        if parallel:
            return self._flash_parallel(device_firmware_map, max_parallel)
        else:
            return self._flash_sequential(device_firmware_map)
    
    def _flash_sequential(self, device_firmware_map: Dict[str, str]) -> Dict[str, FlashOperation]:
        """Flash devices sequentially"""
        results = {}
        
        for device_serial, firmware_path in device_firmware_map.items():
            self.logger.info(f"Flashing device {device_serial}")
            operation = self.flash_firmware(device_serial, firmware_path)
            results[device_serial] = operation
            
            if operation.result != FlashResult.SUCCESS:
                self.logger.error(f"Flash failed for {device_serial}: {operation.error_message}")
                
        return results
    
    def _flash_parallel(self, device_firmware_map: Dict[str, str], max_parallel: int) -> Dict[str, FlashOperation]:
        """Flash devices in parallel using threading"""
        from concurrent.futures import ThreadPoolExecutor, Future
        
        results = {}
        max_workers = min(max_parallel, len(device_firmware_map))
        
        with ThreadPoolExecutor(max_workers=max_workers) as executor:
            # Submit flash operations
            future_to_device = {
                executor.submit(self.flash_firmware, device_serial, firmware_path): device_serial
                for device_serial, firmware_path in device_firmware_map.items()
            }
            
            # Collect results as they complete
            for future in future_to_device:
                device_serial = future_to_device[future]
                try:
                    operation = future.result(timeout=300.0)  # 5 minute timeout per device
                    results[device_serial] = operation
                except Exception as e:
                    # Create failed operation result
                    operation = FlashOperation(
                        device_serial=device_serial,
                        firmware_path=device_firmware_map[device_serial],
                        start_time=time.time(),
                        end_time=time.time(),
                        result=FlashResult.ERROR,
                        error_message=str(e)
                    )
                    results[device_serial] = operation
                    self.logger.error(f"Flash operation failed for {device_serial}: {e}")
                    
        return results
    
    def get_operation_status(self, device_serial: str) -> Optional[FlashOperation]:
        """Get current flash operation status for a device"""
        with self.operation_lock:
            return self.active_operations.get(device_serial)
    
    def cancel_operation(self, device_serial: str) -> bool:
        """Cancel ongoing flash operation for a device"""
        with self.operation_lock:
            operation = self.active_operations.get(device_serial)
            if operation and not operation.end_time:
                operation.result = FlashResult.ERROR
                operation.error_message = "Operation cancelled"
                operation.end_time = time.time()
                return True
        return False
    
    def verify_flash_tool_availability(self) -> bool:
        """Verify that firmware flashing tools are available"""
        if not self.flash_tool_path:
            return False
            
        try:
            # Test tool availability
            result = subprocess.run([self.flash_tool_path, '--help'], 
                                  capture_output=True, timeout=5.0)
            return result.returncode == 0
        except Exception:
            return False
    
    def get_supported_firmware_formats(self) -> List[str]:
        """Get list of supported firmware file formats"""
        if not self.flash_tool_path:
            return []
            
        if 'picotool' in self.flash_tool_path:
            return ['.elf', '.uf2', '.bin']
        elif 'uf2conv.py' in self.flash_tool_path:
            return ['.uf2']
        else:
            return ['.uf2', '.bin']  # Common formats