#!/usr/bin/env python3
"""
Debug bootloader entry with detailed logging
"""

import hid
import time
import json
import threading
from datetime import datetime

def send_bootloader_command_and_monitor():
    """Send bootloader command and monitor detailed responses"""
    
    # Connect to device
    device = hid.Device(0x1234, 0x5678)
    print("✓ Connected to device")
    
    # Start logging thread
    logging = True
    log_messages = []
    
    def log_loop():
        message_count = 0
        while logging:
            try:
                data = device.read(64, timeout=100)
                if data:
                    message_count += 1
                    timestamp = datetime.now().strftime("%H:%M:%S.%f")[:-3]
                    
                    try:
                        text = bytes(data).decode('utf-8').rstrip('\x00')
                        if text:
                            log_entry = f"[{timestamp}] #{message_count:04d}: {text}"
                            print(log_entry)
                            log_messages.append(log_entry)
                    except UnicodeDecodeError:
                        hex_data = ' '.join(f'{b:02x}' for b in data[:16])
                        log_entry = f"[{timestamp}] #{message_count:04d}: RAW: {hex_data}..."
                        print(log_entry)
                        log_messages.append(log_entry)
            except:
                pass
    
    log_thread = threading.Thread(target=log_loop, daemon=True)
    log_thread.start()
    
    print("=== Starting bootloader command test ===")
    print("Monitoring device logs...")
    
    # Wait a moment to see initial state
    time.sleep(2)
    
    print("\n=== Sending bootloader command ===")
    
    # Create bootloader command
    command = bytearray(64)
    command[0] = 0x80  # ENTER_BOOTLOADER command type
    command[1] = 0x01  # Command ID
    
    # Payload: JSON with timeout_ms
    payload = json.dumps({"timeout_ms": 10000}).encode('utf-8')  # 10 second timeout
    command[2] = len(payload)  # Payload length
    
    # Calculate XOR checksum
    checksum = command[0] ^ command[1] ^ command[2]
    for byte in payload:
        checksum ^= byte
    command[3] = checksum
    
    # Add payload
    command[4:4+len(payload)] = payload
    
    print(f"Sending command: {command[:20].hex()}")
    
    # Send command
    device.write(bytes(command))
    print("✓ Command sent")
    
    # Monitor for 30 seconds to see what happens
    print("\nMonitoring for 30 seconds...")
    start_time = time.time()
    
    while time.time() - start_time < 30:
        # Check if device disconnected (entered bootloader)
        try:
            # Try to enumerate the device
            devices = hid.enumerate(0x1234, 0x5678)
            if not devices:
                print("\n🎉 DEVICE DISCONNECTED - Likely entered bootloader mode!")
                break
        except:
            pass
        
        time.sleep(0.5)
    
    # Stop logging
    logging = False
    device.close()
    
    print(f"\n=== Collected {len(log_messages)} log messages ===")
    
    # Look for specific bootloader-related messages
    bootloader_keywords = [
        "bootloader", "reset", "shutdown", "task", "hardware", 
        "safe", "entering", "magic", "psm", "scb"
    ]
    
    relevant_messages = []
    for msg in log_messages:
        for keyword in bootloader_keywords:
            if keyword.lower() in msg.lower():
                relevant_messages.append(msg)
                break
    
    if relevant_messages:
        print("\nBootloader-related messages:")
        for msg in relevant_messages:
            print(f"  {msg}")
    else:
        print("\nNo obvious bootloader-related messages found")
    
    # Check final device state
    print("\n=== Final device state check ===")
    
    # Check for bootloader mode
    import subprocess
    result = subprocess.run(['lsusb'], capture_output=True, text=True)
    
    if "2e8a:0003" in result.stdout:
        print("✅ SUCCESS: Device is in bootloader mode!")
        return True
    elif "1234:5678" in result.stdout:
        print("❌ FAILED: Device is still in normal mode")
        return False
    else:
        print("❓ UNCLEAR: Device state unknown")
        return False

def main():
    print("=== Detailed Bootloader Entry Debug ===")
    print()
    
    try:
        success = send_bootloader_command_and_monitor()
        
        if success:
            print("\n🎉 Bootloader entry successful!")
            print("The software bootloader command is working correctly.")
        else:
            print("\n❌ Bootloader entry failed")
            print("Need to debug why the reset isn't happening.")
            
        return 0 if success else 1
        
    except Exception as e:
        print(f"\n❌ Error during test: {e}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    exit(main())