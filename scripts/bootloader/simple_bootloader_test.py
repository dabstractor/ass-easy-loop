#!/usr/bin/env python3
"""
Simple test of bootloader command functionality with correct VID/PID
"""

import hid
import time
import json
import struct

def find_device():
    """Find our device with the correct VID/PID"""
    devices = hid.enumerate(0x1234, 0x5678)
    if devices:
        print(f"✓ Found device: {devices[0]}")
        return True
    else:
        print("✗ Device not found")
        return False

def send_bootloader_command():
    """Send bootloader entry command to device"""
    try:
        device = hid.Device(0x1234, 0x5678)
        print("✓ Connected to device")
        
        # Create bootloader command as per the existing protocol
        command = bytearray(64)
        command[0] = 0x80  # ENTER_BOOTLOADER command type
        command[1] = 0x01  # Command ID
        
        # Payload: JSON with timeout_ms
        payload = json.dumps({"timeout_ms": 5000}).encode('utf-8')
        command[2] = len(payload)  # Payload length
        
        # Calculate XOR checksum (command_type ^ command_id ^ payload_length ^ all_payload_bytes)
        checksum = command[0] ^ command[1] ^ command[2]
        for byte in payload:
            checksum ^= byte
        command[3] = checksum
        
        # Add payload
        command[4:4+len(payload)] = payload
        
        print(f"Sending bootloader command: {command[:20].hex()}")
        
        # Send command
        bytes_sent = device.write(bytes(command))
        print(f"✓ Sent {bytes_sent} bytes")
        
        # Wait for response
        print("Waiting for response...")
        for i in range(10):  # Wait up to 10 seconds
            try:
                data = device.read(64, timeout=1000)
                if data:
                    try:
                        text = bytes(data).decode('utf-8').rstrip('\x00')
                        print(f"Response {i+1}: {text}")
                        
                        # Check if this looks like a bootloader response
                        if "bootloader" in text.lower() or "TEST_RESPONSE" in text:
                            print("✓ Got bootloader-related response!")
                            break
                    except:
                        hex_data = ' '.join(f'{b:02x}' for b in data[:16])
                        print(f"Response {i+1} (hex): {hex_data}")
                else:
                    print(f"Response {i+1}: No data")
            except Exception as e:
                print(f"Response {i+1}: Error - {e}")
        
        device.close()
        return True
        
    except Exception as e:
        print(f"✗ Failed to send bootloader command: {e}")
        return False

def check_for_bootloader_mode():
    """Check if device entered bootloader mode"""
    print("\nChecking for bootloader mode...")
    
    # Check USB devices
    import subprocess
    result = subprocess.run(['lsusb'], capture_output=True, text=True)
    
    if "2e8a:0003" in result.stdout:
        print("✓ Device is in bootloader mode (USB mass storage)")
        return True
    elif "1234:5678" in result.stdout:
        print("✓ Device is still in normal mode")
        return False
    else:
        print("? Device status unclear")
        return False

def main():
    print("=== Simple Bootloader Command Test ===")
    print()
    
    # Step 1: Find device
    if not find_device():
        print("Make sure device is connected and running firmware")
        return 1
    
    # Step 2: Send bootloader command
    print("\nSending bootloader entry command...")
    if not send_bootloader_command():
        return 1
    
    # Step 3: Wait and check for bootloader mode
    print("\nWaiting 5 seconds for bootloader entry...")
    time.sleep(5)
    
    if check_for_bootloader_mode():
        print("\n🎉 SUCCESS: Bootloader command worked!")
        print("Device entered bootloader mode via software command")
        return 0
    else:
        print("\n⚠ Bootloader command may not have worked")
        print("Device is still in normal mode")
        print("This means the bootloader command functionality needs debugging")
        return 1

if __name__ == "__main__":
    exit(main())