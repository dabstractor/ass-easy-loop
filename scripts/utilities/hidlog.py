#!/usr/bin/env python3
"""
USB HID Log Receiver for RP2040 pEMF/Battery Monitoring Device

This utility receives and displays log messages from the RP2040 device
via USB HID interface. It provides real-time log monitoring with filtering
and formatting capabilities.
"""

import hid
import struct
import time
import argparse
import sys
import json
from datetime import datetime
from enum import IntEnum
from typing import Optional, List, Tuple, TextIO
from pathlib import Path


class LogLevel(IntEnum):
    """Log levels matching the device-side enum"""
    DEBUG = 0
    INFO = 1
    WARN = 2
    ERROR = 3

    def __str__(self):
        return self.name


class LogMessage:
    """Represents a parsed log message from the device"""
    
    def __init__(self, timestamp: int, level: LogLevel, module: str, message: str):
        self.timestamp = timestamp
        self.level = level
        self.module = module
        self.message = message
    
    def format_timestamp(self, show_relative: bool = True) -> str:
        """Format timestamp for display"""
        if show_relative:
            # Show milliseconds since boot
            seconds = self.timestamp / 1000.0
            return f"{seconds:07.3f}s"
        else:
            # Show absolute time (when message was received)
            return datetime.now().strftime("%H:%M:%S.%f")[:-3]
    
    def __str__(self) -> str:
        return f"[{self.format_timestamp()}] [{str(self.level):5}] [{self.module:8}] {self.message}"
    
    def to_json(self) -> dict:
        """Convert log message to JSON format"""
        return {
            'timestamp': self.timestamp,
            'timestamp_formatted': self.format_timestamp(),
            'level': str(self.level),
            'module': self.module,
            'message': self.message,
            'received_at': datetime.now().isoformat()
        }


class HidLogReceiver:
    """USB HID log message receiver and parser"""
    
    # Default VID/PID - should match device configuration
    DEFAULT_VID = 0x1234
    DEFAULT_PID = 0x5678
    
    def __init__(self, vid: int = DEFAULT_VID, pid: int = DEFAULT_PID, 
                 device_path: Optional[str] = None, serial_number: Optional[str] = None):
        self.vid = vid
        self.pid = pid
        self.device_path = device_path
        self.serial_number = serial_number
        self.device = None
        self.connected = False
        self.output_file = None

    def list_devices(self) -> List[dict]:
        """List all available HID devices matching our VID/PID"""
        devices = []
        for device_info in hid.enumerate(self.vid, self.pid):
            devices.append({
                'path': device_info['path'],
                'serial_number': device_info['serial_number'],
                'manufacturer_string': device_info['manufacturer_string'],
                'product_string': device_info['product_string'],
            })
        return devices
    
    def connect(self) -> bool:
        """Connect to the HID device"""
        try:
            self.device = hid.device()
            
            if self.device_path:
                # Connect to specific device by path
                self.device.open_path(self.device_path.encode('utf-8'))
            elif self.serial_number:
                # Connect by serial number
                devices = self.list_devices()
                target_device = None
                for device in devices:
                    if device['serial_number'] == self.serial_number:
                        target_device = device
                        break
                
                if not target_device:
                    print(f"Device with serial number '{self.serial_number}' not found")
                    return False
                
                self.device.open_path(target_device['path'].encode('utf-8'))
            else:
                # Connect to first device with matching VID/PID
                self.device.open(self.vid, self.pid)
            
            # Set non-blocking mode for read operations
            self.device.set_nonblocking(1)
            self.connected = True
            
            # Get device info
            manufacturer = self.device.get_manufacturer_string()
            product = self.device.get_product_string()
            serial = self.device.get_serial_number_string()
            
            print(f"Connected to device:")
            print(f"  Manufacturer: {manufacturer}")
            print(f"  Product: {product}")
            print(f"  Serial: {serial}")
            print(f"  VID:PID: {self.vid:04X}:{self.pid:04X}")
            print()
            
            return True
            
        except Exception as e:
            print(f"Failed to connect to device: {e}")
            self.connected = False
            return False
    
    def disconnect(self):
        """Disconnect from the HID device"""
        if self.device:
            self.device.close()
            self.device = None
        self.connected = False
    
    def parse_log_message(self, data: bytes) -> Optional[LogMessage]:
        """Parse a log message from HID report data"""
        if len(data) < 64:
            return None
        
        try:
            # Parse according to the serialization format from design doc:
            # Byte 0: Log level
            # Bytes 1-8: Module name (null-padded)
            # Bytes 9-56: Message content (null-terminated)
            # Bytes 57-60: Timestamp (little-endian u32)
            # Bytes 61-63: Reserved/padding
            
            level_byte = data[0]
            if level_byte > 3:
                return None
            
            level = LogLevel(level_byte)
            
            module_bytes = data[1:9]
            module = module_bytes.rstrip(b'\x00').decode('utf-8', errors='ignore')
            
            message_bytes = data[9:57]
            null_pos = message_bytes.find(b'\x00')
            if null_pos >= 0:
                message_bytes = message_bytes[:null_pos]
            message = message_bytes.decode('utf-8', errors='ignore')
            
            timestamp = struct.unpack('<I', data[57:61])[0]
            
            return LogMessage(timestamp, level, module, message)
            
        except Exception as e:
            print(f"Error parsing log message: {e}")
            return None
    
    def read_log_message(self, timeout_ms: int = 100) -> Optional[LogMessage]:
        """Read and parse a single log message from the device"""
        if not self.connected or not self.device:
            return None
        
        try:
            data = self.device.read(64, timeout_ms)
            if not data:
                return None
            
            return self.parse_log_message(bytes(data))
            
        except Exception as e:
            print(f"Error reading from device: {e}")
            self.connected = False
            return None
    
    def read_raw_hid_report(self, timeout_ms: int = 100) -> Optional[bytes]:
        """Read raw HID report data for debugging"""
        if not self.connected or not self.device:
            return None
        
        try:
            data = self.device.read(64, timeout_ms)
            if not data:
                return None
            
            return bytes(data)
            
        except Exception as e:
            print(f"Error reading raw HID data: {e}")
            self.connected = False
            return None
    
    def monitor_logs(self, min_level: LogLevel = LogLevel.DEBUG, 
                     module_filter: Optional[str] = None,
                     show_raw: bool = False, json_output: bool = False) -> None:
        """Monitor and display log messages in real-time"""
        if not self.connected:
            print("Device not connected")
            return
        
        if not json_output:
            print(f"Monitoring logs (min level: {min_level}, module filter: {module_filter or 'none'})")
            print("Press Ctrl+C to stop")
            print("-" * 80)
        
        try:
            while True:
                message = self.read_log_message(timeout_ms=1000)
                
                if message:
                    # Apply filters
                    if message.level < min_level:
                        continue
                    
                    if module_filter and module_filter.upper() not in message.module.upper():
                        continue
                    
                    # Format output based on options
                    if json_output:
                        log_entry = json.dumps(message.to_json())
                    elif show_raw:
                        log_entry = f"Raw: level={message.level}, module='{message.module}', timestamp={message.timestamp}, message='{message.message}'"
                    else:
                        log_entry = str(message)

                    print(log_entry)

                    # Write to log file if specified
                    if self.output_file:
                        # Add timestamp prefix for file output
                        file_entry = f"{datetime.now().isoformat()} {log_entry}"
                        self.output_file.write(file_entry + '\n')
                        self.output_file.flush()
                
                time.sleep(0.001)
                
        except KeyboardInterrupt:
            print("\nStopping log monitor...")
        except Exception as e:
            print(f"Error during monitoring: {e}")
    
    def inspect_hid_reports(self) -> None:
        """Inspect raw HID reports for debugging"""
        if not self.connected:
            print("Device not connected")
            return
        
        print("Inspecting raw HID reports")
        print("Press Ctrl+C to stop")
        print("-" * 80)
        
        try:
            while True:
                raw_data = self.read_raw_hid_report(timeout_ms=1000)
                
                if raw_data:
                    # Display raw bytes in hex format
                    hex_data = ' '.join(f'{b:02X}' for b in raw_data)
                    print(f"Raw HID Report ({len(raw_data)} bytes): {hex_data}")
                    
                    # Try to parse as log message for comparison
                    parsed = self.parse_log_message(raw_data)
                    if parsed:
                        print(f"  Parsed: {parsed}")
                    else:
                        print("  Failed to parse as log message")
                    
                    print()
                
                time.sleep(0.001)
                
        except KeyboardInterrupt:
            print("\nStopping HID inspection...")
        except Exception as e:
            print(f"Error during HID inspection: {e}")


def main():
    """Main entry point with command-line argument parsing"""
    parser = argparse.ArgumentParser(
        description="USB HID Log Receiver for RP2040 Device",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s                          # Connect to first device and show all logs
  %(prog)s --level INFO             # Show only INFO and above
  %(prog)s --module BATTERY         # Show only battery-related logs
  %(prog)s --list                   # List available devices
  %(prog)s --device /dev/hidraw0    # Connect to specific device
  %(prog)s --vid 0x1234 --pid 0x5678 # Use custom VID/PID
        """
    )
    
    parser.add_argument('--vid', type=lambda x: int(x, 0), default=HidLogReceiver.DEFAULT_VID,
                        help=f'USB Vendor ID (default: 0x{HidLogReceiver.DEFAULT_VID:04X})')
    parser.add_argument('--pid', type=lambda x: int(x, 0), default=HidLogReceiver.DEFAULT_PID,
                        help=f'USB Product ID (default: 0x{HidLogReceiver.DEFAULT_PID:04X})')
    parser.add_argument('--device', type=str,
                        help='Specific device path (e.g., /dev/hidraw0)')
    parser.add_argument('--serial', type=str,
                        help='Connect to device with specific serial number')
    parser.add_argument('--level', type=str, choices=['DEBUG', 'INFO', 'WARN', 'ERROR'],
                        default='DEBUG', help='Minimum log level to display')
    parser.add_argument('--module', type=str,
                        help='Filter logs by module name (case-insensitive)')
    parser.add_argument('--list', action='store_true',
                        help='List available devices and exit')
    parser.add_argument('--raw', action='store_true',
                        help='Show raw message data for debugging')
    parser.add_argument('--log-file', type=str,
                        help='Save logs to file with timestamps')
    parser.add_argument('--inspect-hid', action='store_true',
                        help='Show raw HID report data for debugging')
    parser.add_argument('--json-output', action='store_true',
                        help='Output logs in JSON format')

    args = parser.parse_args()
    
    # Create receiver instance
    receiver = HidLogReceiver(vid=args.vid, pid=args.pid, device_path=args.device, serial_number=args.serial)

    # Handle list devices command
    if args.list:
        print("Available devices:")
        devices = receiver.list_devices()
        if not devices:
            print("  No devices found")
        else:
            for i, device in enumerate(devices):
                print(f"  {i}: {device['path']}")
                print(f"     Manufacturer: {device['manufacturer_string']}")
                print(f"     Product: {device['product_string']}")
                print(f"     Serial: {device['serial_number']}")
                print()
        return

    # Set up log file output if requested
    if args.log_file:
        try:
            receiver.output_file = open(args.log_file, 'a')
            print(f"Logging to file: {args.log_file}")
        except IOError as e:
            print(f"Error opening log file: {e}")
            sys.exit(1)
    
    if not receiver.connect():
        print("Failed to connect to device. Use --list to see available devices.")
        sys.exit(1)
    
    try:
        # Convert log level string to enum
        min_level = LogLevel[args.level]
        
        # Handle HID inspection mode
        if args.inspect_hid:
            receiver.inspect_hid_reports()
        else:
            # Start normal log monitoring
            receiver.monitor_logs(
                min_level=min_level,
                module_filter=args.module,
                show_raw=args.raw,
                json_output=args.json_output
            )
        
    finally:
        if receiver.output_file:
            receiver.output_file.close()
        receiver.disconnect()


if __name__ == '__main__':
    main()
