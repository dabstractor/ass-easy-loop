#!/usr/bin/env python3
"""
Direct bootloader test with immediate reset command

This test sends a bootloader command and immediately checks if the firmware
can perform the basic reset operation to enter bootloader mode.
"""

import hid
import time
import struct
import subprocess
import threading

def send_bootloader_command_and_monitor():
    """Send bootloader command and monitor device disconnection"""
    print("=== DIRECT BOOTLOADER RESET TEST ===")
    
    try:
        # Connect to device
        device = hid.Device(0x1234, 0x5678)
        print("✓ Connected to device")
        
        # Create bootloader command with very short timeout
        command = bytearray(64)
        command[0] = 0x80  # ENTER_BOOTLOADER command type
        command[1] = 0x01  # Command ID
        
        # Use minimal timeout (100ms) to force immediate action
        timeout_ms = 100
        payload = struct.pack('<I', timeout_ms)
        
        command[2] = len(payload)
        
        # Calculate checksum
        checksum = command[0] ^ command[1] ^ command[2]
        for byte in payload:
            checksum ^= byte
        command[3] = checksum
        
        # Add payload
        command[4:4+len(payload)] = payload
        
        print(f"Sending bootloader command with {timeout_ms}ms timeout...")
        print(f"Command: {command[:12].hex()}")
        
        # Send command
        bytes_sent = device.write(bytes(command))
        print(f"✓ Sent {bytes_sent} bytes")
        
        # Try to read response immediately
        print("Attempting to read immediate response...")
        try:
            for i in range(5):
                data = device.read(64, timeout=200)  # 200ms timeout
                if data:
                    try:
                        text = bytes(data).decode('utf-8').rstrip('\x00')
                        if text:
                            print(f"Response {i+1}: {text}")
                        else:
                            hex_data = ' '.join(f'{b:02x}' for b in data[:16])
                            print(f"Response {i+1} (hex): {hex_data}")
                    except:
                        hex_data = ' '.join(f'{b:02x}' for b in data[:16])
                        print(f"Response {i+1} (hex): {hex_data}")
                else:
                    print(f"Response {i+1}: No data")
        except Exception as e:
            print(f"Read error (device may have reset): {e}")
        
        # Close device
        device.close()
        print("✓ Closed device connection")
        
        return True
        
    except Exception as e:
        print(f"✗ Command failed: {e}")
        return False

def check_device_reset_sequence():
    """Check if device goes through reset sequence"""
    print("\nMonitoring device reset sequence...")
    
    # Monitor for 10 seconds
    for i in range(10):
        print(f"  {i+1}s: ", end="", flush=True)
        
        # Check HID device
        try:
            devices = hid.enumerate(0x1234, 0x5678)
            if devices:
                print("HID-PRESENT ", end="")
            else:
                print("HID-ABSENT  ", end="")
        except:
            print("HID-ERROR   ", end="")
        
        # Check bootloader
        try:
            result = subprocess.run(['lsusb'], capture_output=True, text=True, timeout=1)
            if "2e8a:0003" in result.stdout:
                print("BOOTLOADER-PRESENT")
            else:
                print("BOOTLOADER-ABSENT")
        except:
            print("BOOTLOADER-ERROR")
        
        time.sleep(1)

def test_alternative_bootloader_approach():
    """Test if we can trigger bootloader through different means"""
    print("\n=== TESTING ALTERNATIVE APPROACHES ===")
    
    # Try sending multiple commands rapidly
    print("Trying rapid command sequence...")
    try:
        device = hid.Device(0x1234, 0x5678)
        
        for i in range(3):
            command = bytearray(64)
            command[0] = 0x80  # ENTER_BOOTLOADER
            command[1] = i + 1  # Different command IDs
            
            # Very short timeout
            timeout_ms = 50
            payload = struct.pack('<I', timeout_ms)
            command[2] = len(payload)
            
            # Calculate checksum
            checksum = command[0] ^ command[1] ^ command[2]
            for byte in payload:
                checksum ^= byte
            command[3] = checksum
            command[4:4+len(payload)] = payload
            
            device.write(bytes(command))
            print(f"  Sent command {i+1}")
            time.sleep(0.1)
        
        device.close()
        print("✓ Rapid sequence sent")
        
    except Exception as e:
        print(f"✗ Rapid sequence failed: {e}")
    
    # Wait and check result
    print("Waiting 5 seconds for effect...")
    time.sleep(5)
    
    # Check final state
    try:
        result = subprocess.run(['lsusb'], capture_output=True, text=True)
        if "2e8a:0003" in result.stdout:
            print("✓ Device is now in bootloader mode!")
            return True
        else:
            print("✗ Device still not in bootloader mode")
            return False
    except:
        print("✗ Could not check device state")
        return False

def main():
    print("=== DIRECT BOOTLOADER RESET TEST ===")
    print("Testing if firmware can perform basic bootloader reset")
    print()
    
    # Step 1: Send command and monitor immediate response
    if not send_bootloader_command_and_monitor():
        return 1
    
    # Step 2: Monitor device reset sequence
    check_device_reset_sequence()
    
    # Step 3: Try alternative approach
    if test_alternative_bootloader_approach():
        print("\n🎉 SUCCESS: Alternative approach worked!")
        print("Device entered bootloader mode")
        return 0
    else:
        print("\n❌ FAILURE: No approach worked")
        print("The firmware bootloader entry is not functioning")
        print("\nPossible issues:")
        print("1. Bootloader entry task is not running")
        print("2. Hardware validation is failing")
        print("3. Task shutdown is not completing")
        print("4. Reset mechanism is not working")
        print("5. Magic value is not being written correctly")
        return 1

if __name__ == "__main__":
    exit(main())