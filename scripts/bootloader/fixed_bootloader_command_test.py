#!/usr/bin/env python3
"""
Fixed bootloader command test with correct payload format

The issue was that the firmware expects the timeout as raw little-endian bytes,
but we were sending it as JSON. This version sends the timeout correctly.
"""

import hid
import time
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

def send_fixed_bootloader_command():
    """Send bootloader entry command with correct payload format"""
    try:
        device = hid.Device(0x1234, 0x5678)
        print("✓ Connected to device")
        
        # Create bootloader command with correct format
        command = bytearray(64)
        command[0] = 0x80  # ENTER_BOOTLOADER command type
        command[1] = 0x01  # Command ID
        
        # Payload: timeout as 4-byte little-endian integer (not JSON!)
        timeout_ms = 5000
        payload = struct.pack('<I', timeout_ms)  # Little-endian 32-bit unsigned int
        
        command[2] = len(payload)  # Payload length (4 bytes)
        
        # Calculate XOR checksum (command_type ^ command_id ^ payload_length ^ all_payload_bytes)
        checksum = command[0] ^ command[1] ^ command[2]
        for byte in payload:
            checksum ^= byte
        command[3] = checksum
        
        # Add payload
        command[4:4+len(payload)] = payload
        
        print(f"Sending FIXED bootloader command:")
        print(f"  Command type: 0x{command[0]:02X}")
        print(f"  Command ID: {command[1]}")
        print(f"  Payload length: {command[2]}")
        print(f"  Checksum: 0x{command[3]:02X}")
        print(f"  Timeout (little-endian): {payload.hex()}")
        print(f"  Full command: {command[:20].hex()}")
        
        # Send command
        bytes_sent = device.write(bytes(command))
        print(f"✓ Sent {bytes_sent} bytes")
        
        # Wait for response and device disconnection
        print("Waiting for response and device disconnection...")
        device_disconnected = False
        responses_received = 0
        
        for i in range(30):  # Wait up to 30 seconds
            try:
                data = device.read(64, timeout=1000)
                if data:
                    responses_received += 1
                    try:
                        # Try to decode as text
                        text = bytes(data).decode('utf-8').rstrip('\x00')
                        if text:
                            print(f"Response {responses_received}: {text}")
                            # Look for bootloader-related messages
                            if any(word in text.lower() for word in ['bootloader', 'entering', 'ready', 'shutdown']):
                                print("✓ Got bootloader-related response!")
                        else:
                            # Show as hex if empty text
                            hex_data = ' '.join(f'{b:02x}' for b in data[:16])
                            print(f"Response {responses_received} (hex): {hex_data}")
                    except UnicodeDecodeError:
                        # Show as hex if not text
                        hex_data = ' '.join(f'{b:02x}' for b in data[:16])
                        print(f"Response {responses_received} (hex): {hex_data}")
                else:
                    print(f"Attempt {i+1}: No data (timeout)")
            except Exception as e:
                print(f"Attempt {i+1}: Read error - {e}")
                # Check if this indicates device disconnection
                if "No such device" in str(e) or "Device not found" in str(e):
                    print("✓ Device disconnected - likely entering bootloader mode!")
                    device_disconnected = True
                    break
                elif "Success" in str(e):
                    print("  (This 'Success' error might indicate device is rebooting)")
        
        device.close()
        
        # Give device time to enter bootloader mode
        if device_disconnected:
            print("\nWaiting 3 seconds for bootloader mode...")
            time.sleep(3)
        
        return device_disconnected or responses_received > 0
        
    except Exception as e:
        print(f"✗ Failed to send bootloader command: {e}")
        return False

def check_for_bootloader_mode():
    """Check if device entered bootloader mode"""
    print("\nChecking for bootloader mode...")
    
    # Method 1: Check USB devices
    import subprocess
    try:
        result = subprocess.run(['lsusb'], capture_output=True, text=True)
        
        if "2e8a:0003" in result.stdout:
            print("✓ Device is in bootloader mode (USB mass storage)")
            return True
        elif "1234:5678" in result.stdout:
            print("✓ Device is still in normal mode")
            return False
        else:
            print("? Device status unclear from lsusb")
    except:
        print("⚠ Could not run lsusb")
    
    # Method 2: Check for mount points
    import os
    mount_points = [
        "/run/media/dustin/RPI-RP2/",
        "/media/RPI-RP2/",
        "/mnt/RPI-RP2/"
    ]
    
    for mount_point in mount_points:
        if os.path.exists(mount_point) and os.path.ismount(mount_point):
            print(f"✓ Bootloader mounted at: {mount_point}")
            return True
    
    print("✗ No bootloader mount points found")
    return False

def test_device_reconnection():
    """Test if device reconnects in normal mode"""
    print("\nTesting device reconnection...")
    
    for i in range(10):  # Wait up to 10 seconds
        try:
            devices = hid.enumerate(0x1234, 0x5678)
            if devices:
                print(f"✓ Device reconnected in normal mode after {i+1} seconds")
                return True
        except:
            pass
        
        time.sleep(1)
        print(".", end="", flush=True)
    
    print("\n✗ Device did not reconnect in normal mode")
    return False

def main():
    print("=== FIXED Bootloader Command Test ===")
    print("This version sends the timeout as raw bytes, not JSON")
    print()
    
    # Step 1: Find device
    if not find_device():
        print("Make sure device is connected and running firmware")
        return 1
    
    # Step 2: Send FIXED bootloader command
    print("\nSending FIXED bootloader entry command...")
    if not send_fixed_bootloader_command():
        print("❌ Failed to send command")
        return 1
    
    # Step 3: Check for bootloader mode
    print("\nWaiting 5 seconds for bootloader entry...")
    time.sleep(5)
    
    if check_for_bootloader_mode():
        print("\n🎉 SUCCESS: FIXED bootloader command worked!")
        print("Device entered bootloader mode via software command")
        
        # Wait a bit more and check if it comes back
        print("\nWaiting to see if device returns to normal mode...")
        time.sleep(10)
        
        if test_device_reconnection():
            print("✓ Device returned to normal mode automatically")
        else:
            print("⚠ Device stayed in bootloader mode (this might be expected)")
        
        return 0
    else:
        print("\n⚠ Bootloader command may not have worked")
        print("Device is still in normal mode")
        print("The payload format fix may not be complete")
        return 1

if __name__ == "__main__":
    exit(main())