#!/usr/bin/env python3
"""
Test HID logging functionality to verify device behavior
"""

import hid
import time
import sys
import threading
from datetime import datetime

class HIDLogger:
    def __init__(self, vid=0x1234, pid=0x5678):
        self.vid = vid
        self.pid = pid
        self.device = None
        self.running = False
        self.log_thread = None
        
    def connect(self):
        """Connect to the HID device"""
        try:
            self.device = hid.Device(self.vid, self.pid)
            print(f"✓ Connected to HID device VID={self.vid:04x} PID={self.pid:04x}")
            return True
        except Exception as e:
            print(f"✗ Failed to connect to HID device: {e}")
            return False
    
    def disconnect(self):
        """Disconnect from the HID device"""
        if self.device:
            self.device.close()
            self.device = None
            print("✓ Disconnected from HID device")
    
    def start_logging(self):
        """Start logging HID messages in a separate thread"""
        if not self.device:
            print("✗ No device connected")
            return False
            
        self.running = True
        self.log_thread = threading.Thread(target=self._log_loop, daemon=True)
        self.log_thread.start()
        print("✓ Started HID logging thread")
        return True
    
    def stop_logging(self):
        """Stop logging HID messages"""
        self.running = False
        if self.log_thread:
            self.log_thread.join(timeout=2)
        print("✓ Stopped HID logging")
    
    def _log_loop(self):
        """Main logging loop - runs in separate thread"""
        print("=== HID LOG MESSAGES ===")
        message_count = 0
        
        while self.running:
            try:
                # Read HID data with timeout
                data = self.device.read(64, timeout=100)  # 100ms timeout
                
                if data:
                    message_count += 1
                    timestamp = datetime.now().strftime("%H:%M:%S.%f")[:-3]
                    
                    # Try to decode as text
                    try:
                        text = bytes(data).decode('utf-8').rstrip('\x00')
                        if text:
                            print(f"[{timestamp}] #{message_count:04d}: {text}")
                        else:
                            # Show raw bytes if no text
                            hex_data = ' '.join(f'{b:02x}' for b in data[:16])
                            print(f"[{timestamp}] #{message_count:04d}: RAW: {hex_data}...")
                    except UnicodeDecodeError:
                        # Show raw bytes for non-text data
                        hex_data = ' '.join(f'{b:02x}' for b in data[:16])
                        print(f"[{timestamp}] #{message_count:04d}: RAW: {hex_data}...")
                        
            except Exception as e:
                if self.running:  # Only print error if we're still supposed to be running
                    print(f"[ERROR] HID read error: {e}")
                    time.sleep(0.1)
    
    def send_command(self, command_data):
        """Send a command to the device"""
        if not self.device:
            print("✗ No device connected")
            return False
            
        try:
            # Ensure command is 64 bytes
            if isinstance(command_data, str):
                command_data = command_data.encode('utf-8')
            
            command_bytes = bytearray(64)
            command_bytes[:len(command_data)] = command_data[:64]
            
            bytes_sent = self.device.write(bytes(command_bytes))
            print(f"✓ Sent {bytes_sent} bytes to device")
            return True
            
        except Exception as e:
            print(f"✗ Failed to send command: {e}")
            return False

def main():
    print("=== HID Logging Test ===")
    print("This will connect to the device and log all HID messages")
    print()
    
    logger = HIDLogger()
    
    # Connect to device
    if not logger.connect():
        print("Failed to connect to device. Make sure it's connected and running the correct firmware.")
        return 1
    
    try:
        # Start logging
        logger.start_logging()
        
        print()
        print("Logging started. The device should be sending log messages.")
        print("You should see system startup messages, task status, etc.")
        print()
        print("Commands you can try:")
        print("  'q' or 'quit' - Exit")
        print("  'test' - Send a test message")
        print("  'bootloader' - Try bootloader command (if implemented)")
        print()
        
        # Interactive command loop
        while True:
            try:
                user_input = input("Enter command (or 'q' to quit): ").strip().lower()
                
                if user_input in ['q', 'quit', 'exit']:
                    break
                elif user_input == 'test':
                    logger.send_command("TEST_COMMAND")
                elif user_input == 'bootloader':
                    # Try to send a bootloader command (this might not work yet)
                    bootloader_cmd = bytearray(64)
                    bootloader_cmd[0] = 0x80  # ENTER_BOOTLOADER command type
                    bootloader_cmd[1] = 0x01  # Command ID
                    bootloader_cmd[2] = 0x04  # Payload length
                    bootloader_cmd[3] = 0x85  # Simple checksum (0x80 + 0x01 + 0x04)
                    # Payload: timeout_ms = 5000 (little endian)
                    bootloader_cmd[4:8] = (5000).to_bytes(4, 'little')
                    
                    logger.send_command(bytes(bootloader_cmd))
                    print("Sent bootloader command - check logs for response")
                elif user_input:
                    # Send arbitrary command
                    logger.send_command(user_input)
                    
            except KeyboardInterrupt:
                break
                
    finally:
        logger.stop_logging()
        logger.disconnect()
    
    print("HID logging test completed")
    return 0

if __name__ == "__main__":
    sys.exit(main())