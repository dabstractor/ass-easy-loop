#!/usr/bin/env python3
"""
Simple script to test bootloader device detection
"""

import sys
import time
import hid
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..'))
from test_framework.device_manager import UsbHidDeviceManager, DeviceStatus

def detect_bootloader_device():
    """Check if RP2040 bootloader device is present"""
    VENDOR_ID = 0x2E8A  # Raspberry Pi Foundation
    BOOTLOADER_PRODUCT_ID = 0x0003  # RP2040 Bootloader
    
    devices = hid.enumerate(VENDOR_ID, BOOTLOADER_PRODUCT_ID)
    return len(devices) > 0

def detect_normal_device():
    """Check if normal RP2040 device is present"""
    VENDOR_ID = 0x2E8A  # Raspberry Pi Foundation  
    PRODUCT_ID = 0x000A  # RP2040 USB HID
    
    devices = hid.enumerate(VENDOR_ID, PRODUCT_ID)
    return len(devices) > 0

def main():
    print("=== RP2040 Device Detection Test ===")
    print()
    
    # Check current device status
    if detect_bootloader_device():
        print("✓ Device is currently in BOOTLOADER mode")
        return
    elif detect_normal_device():
        print("✓ Device is currently in NORMAL mode")
        print("Need to put device in bootloader mode for flashing")
    else:
        print("✗ No RP2040 device detected")
        print("Please connect the device first")
    
    print()
    print("=== Instructions to Enter Bootloader Mode ===")
    print("1. Disconnect the RP2040 device from USB")
    print("2. Hold down the BOOTSEL button on the device")
    print("3. While holding BOOTSEL, reconnect the USB cable")
    print("4. Release the BOOTSEL button after connecting")
    print("5. The device should appear as a USB mass storage device")
    print()
    
    # Wait for bootloader mode
    print("Waiting for device to enter bootloader mode...")
    print("(Press Ctrl+C to cancel)")
    
    try:
        timeout = 60  # 60 second timeout
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            if detect_bootloader_device():
                print("✓ SUCCESS: Device is now in bootloader mode!")
                return True
            
            time.sleep(1)
            print(".", end="", flush=True)
        
        print()
        print("✗ TIMEOUT: Device did not enter bootloader mode within 60 seconds")
        return False
        
    except KeyboardInterrupt:
        print()
        print("Detection cancelled by user")
        return False

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)