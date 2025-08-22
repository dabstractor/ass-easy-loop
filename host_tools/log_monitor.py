#!/usr/bin/env python3
"""
USB HID Log Monitor for Ass-Easy Loop Device

This script monitors USB HID log messages from the Ass-Easy Loop device
and displays them in a human-readable format.
"""

try:
    import hid
except ImportError:
    print("hidapi library not found. Please install it with: pip install hidapi")
    exit(1)
import argparse
import time
import struct
from typing import Optional, Dict, Any

# USB Vendor and Product IDs for the logging device
# Using the same VID/PID as the main device since logging is integrated
VENDOR_ID = 0xfade
PRODUCT_ID = 0x1212

# Log level mappings
LOG_LEVELS = {
    0: "DEBUG",
    1: "INFO",
    2: "WARN",
    3: "ERROR"
}

# Log category mappings
LOG_CATEGORIES = {
    0: "BATTERY",
    1: "PEMF",
    2: "SYSTEM",
    3: "USB"
}

def parse_log_message(data: bytes) -> Optional[Dict[str, Any]]:
    """
    Parse a log message from HID report data.
    
    Args:
        data: 64-byte HID report data
        
    Returns:
        Dictionary with parsed log message fields or None if invalid
    """
    if len(data) != 64:
        return None
    
    try:
        # Parse header fields
        timestamp_ms = struct.unpack('<I', data[0:4])[0]  # Little-endian 32-bit
        level = data[4]
        category = data[5]
        content_len = data[6]
        
        # Parse content (max 52 bytes)
        content_bytes = data[8:8 + min(content_len, 51)]
        content = content_bytes.decode('utf-8', errors='ignore').rstrip('\x00')
        
        # Check for truncation indicator
        if content_len > 51 and len(data) >= 62:
            if data[59:62] == b'...':
                content += '...'
        
        return {
            'timestamp_ms': timestamp_ms,
            'level': LOG_LEVELS.get(level, f"UNKNOWN({level})"),
            'category': LOG_CATEGORIES.get(category, f"UNKNOWN({category})"),
            'content': content,
            'content_len': content_len
        }
    except Exception as e:
        print(f"Error parsing log message: {e}")
        return None

def find_logging_device() -> Optional[hid.Device]:
    """
    Find and open the logging HID device.
    
    Returns:
        HID device object or None if not found
    """
    try:
        devices = hid.enumerate(VENDOR_ID, PRODUCT_ID)
        if not devices:
            return None
            
        device = hid.Device(VENDOR_ID, PRODUCT_ID)
        return device
    except Exception as e:
        print(f"Error opening device: {e}")
        return None

def send_logging_command(device: hid.Device, command_id: int, param1: int = 0, param2: int = 0, param3: int = 0) -> bool:
    """
    Send a logging command to the device.
    
    Args:
        device: HID device object
        command_id: Command ID (first byte)
        param1: First parameter byte
        param2: Second parameter byte
        param3: Third parameter byte
        
    Returns:
        True if successful, False otherwise
    """
    try:
        # Create 64-byte report
        report = [0] * 64
        report[0] = command_id
        report[1] = param1
        report[2] = param2
        report[3] = param3
        
        # Send output report
        device.write(bytes(report))
        return True
    except Exception as e:
        print(f"Error sending command: {e}")
        return False

def monitor_logs(save_to_file: Optional[str] = None, verbose: bool = False) -> None:
    """
    Monitor and display log messages from the device.
    
    Args:
        save_to_file: Optional file path to save logs
        verbose: Whether to show verbose output
    """
    # Set up file saving if requested
    log_file = None
    if save_to_file:
        try:
            log_file = open(save_to_file, 'w')
            print(f"Saving logs to: {save_to_file}")
        except Exception as e:
            print(f"Error opening log file: {e}")
            log_file = None
    
    device = None
    device_connected = False
    
    try:
        print(f"Searching for USB HID logging device (VID: 0x{VENDOR_ID:04x}, PID: 0x{PRODUCT_ID:04x})...")
        print("Press Ctrl+C to exit")
        print("-" * 80)
        
        while True:
            try:
                # Try to find and connect to device if not connected
                if not device_connected:
                    device = find_logging_device()
                    if device:
                        device.nonblocking = True
                        device_connected = True
                        print("Connected to logging device!")
                        print("Monitoring logs... Press Ctrl+C to exit")
                        print("-" * 80)
                    else:
                        # Device not found, wait before retrying
                        time.sleep(1)
                        continue
                
                # Read 64-byte input report
                data = device.read(64, timeout=1000)
                
                if data and len(data) == 64:
                    log_msg = parse_log_message(bytes(data))
                    if log_msg:
                        # Format timestamp
                        timestamp_sec = log_msg['timestamp_ms'] / 1000.0
                        timestamp_str = f"{timestamp_sec:.3f}s"
                        
                        # Format output
                        output = f"[{timestamp_str}] [{log_msg['level']}] [{log_msg['category']}] {log_msg['content']}"
                        
                        # Display to console
                        print(output)
                        
                        # Save to file if requested
                        if log_file:
                            log_file.write(output + '\n')
                            log_file.flush()
                        
                        # Verbose output
                        if verbose:
                            print(f"  Raw data: {data[:16]}...")
                            
                # Small delay to prevent excessive CPU usage
                time.sleep(0.01)
                
            except KeyboardInterrupt:
                print("\nStopping log monitor...")
                break
            except Exception as e:
                # Handle device disconnection or read errors
                if device_connected:
                    print("Device disconnected or communication error!")
                    if device:
                        try:
                            device.close()
                        except:
                            pass
                    device = None
                    device_connected = False
                    print("Searching for USB HID logging device...")
                elif verbose:
                    print(f"Error reading from device: {e}")
                time.sleep(1)
                
    finally:
        if device:
            try:
                device.close()
            except:
                pass
        if log_file:
            log_file.close()
            print(f"Log file closed: {save_to_file}")

def set_log_level(device: hid.Device, level: int) -> None:
    """Set the logging verbosity level."""
    if send_logging_command(device, 0x10, level):
        print(f"Log level set to {LOG_LEVELS.get(level, level)}")
    else:
        print("Failed to set log level")

def enable_category(device: hid.Device, category: int) -> None:
    """Enable a log category."""
    if send_logging_command(device, 0x11, category):
        print(f"Category {LOG_CATEGORIES.get(category, category)} enabled")
    else:
        print("Failed to enable category")

def disable_category(device: hid.Device, category: int) -> None:
    """Disable a log category."""
    if send_logging_command(device, 0x12, category):
        print(f"Category {LOG_CATEGORIES.get(category, category)} disabled")
    else:
        print("Failed to disable category")

def main() -> None:
    """Main function."""
    parser = argparse.ArgumentParser(description="USB HID Log Monitor for Ass-Easy Loop Device")
    parser.add_argument('-f', '--file', help='Save logs to file')
    parser.add_argument('-v', '--verbose', action='store_true', help='Verbose output')
    parser.add_argument('--set-level', type=int, choices=[0, 1, 2, 3], help='Set log level (0=DEBUG, 1=INFO, 2=WARN, 3=ERROR)')
    parser.add_argument('--enable-category', type=int, choices=[0, 1, 2, 3], help='Enable log category (0=BATTERY, 1=PEMF, 2=SYSTEM, 3=USB)')
    parser.add_argument('--disable-category', type=int, choices=[0, 1, 2, 3], help='Disable log category (0=BATTERY, 1=PEMF, 2=SYSTEM, 3=USB)')
    
    args = parser.parse_args()
    
    # Handle configuration commands
    if args.set_level is not None or args.enable_category is not None or args.disable_category is not None:
        device = find_logging_device()
        if not device:
            print("Error: Logging device not found!")
            return
            
        try:
            if args.set_level is not None:
                set_log_level(device, args.set_level)
            if args.enable_category is not None:
                enable_category(device, args.enable_category)
            if args.disable_category is not None:
                disable_category(device, args.disable_category)
        finally:
            device.close()
        return
    
    # Monitor logs by default
    monitor_logs(args.file, args.verbose)

if __name__ == "__main__":
    main()