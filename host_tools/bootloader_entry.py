#!/usr/bin/env python3
"""
Bootloader Entry Tool for AsEasyLoop Device

This tool sends a USB HID command to trigger the RP2040 device to enter
bootloader mode automatically without manual BOOTSEL button intervention.
"""

import hid
import sys
import time

# USB VID/PID from src/config/usb.rs
VENDOR_ID = 0xfade
PRODUCT_ID = 0x1212

# HID command codes
ENTER_BOOTLOADER_CMD = 0x03

def find_device(vendor_id=VENDOR_ID, product_id=PRODUCT_ID):
    """Find the AsEasyLoop device by VID/PID"""
    devices = hid.enumerate(vendor_id, product_id)
    if not devices:
        return None
    return devices[0]

def trigger_bootloader_entry(vendor_id=VENDOR_ID, product_id=PRODUCT_ID):
    """Send EnterBootloader command via USB HID"""
    try:
        # Find and open device
        device_info = find_device(vendor_id, product_id)
        if not device_info:
            print(f"❌ Device not found (VID: 0x{vendor_id:04x}, PID: 0x{product_id:04x})")
            return False
            
        print(f"✅ Found device: {device_info['product_string']}")
        
        device = hid.Device(vendor_id, product_id)
        
        # Send bootloader entry command
        # 64-byte HID report with command in first byte
        report = [ENTER_BOOTLOADER_CMD] + [0x00] * 63
        
        print("📤 Sending bootloader entry command...")
        result = device.write(bytes(report))
        device.close()
        
        if result > 0:
            print("✅ Bootloader entry command sent successfully")
            print("⏳ Device should enter bootloader mode in ~100ms...")
            return True
        else:
            print("❌ Failed to send bootloader entry command")
            return False
            
    except Exception as e:
        print(f"❌ Error: {e}")
        return False

def wait_for_bootloader_mode(timeout_seconds=5):
    """Wait for device to appear in bootloader mode"""
    bootloader_vid = 0x2e8a  # Raspberry Pi Foundation
    bootloader_pid = 0x0003  # RP2 Boot (RPI-RP2)
    
    print(f"⏳ Waiting for bootloader mode (VID: 0x{bootloader_vid:04x}, PID: 0x{bootloader_pid:04x})...")
    
    start_time = time.time()
    while time.time() - start_time < timeout_seconds:
        devices = hid.enumerate(bootloader_vid, bootloader_pid)
        if devices:
            print("✅ Device entered bootloader mode successfully!")
            return True
        time.sleep(0.5)
    
    print("❌ Timeout waiting for bootloader mode")
    return False

def main():
    """Main function for command-line usage"""
    print("AsEasyLoop Bootloader Entry Tool")
    print("=" * 40)
    
    # Trigger bootloader entry
    if not trigger_bootloader_entry():
        sys.exit(1)
    
    # Wait for bootloader mode
    if not wait_for_bootloader_mode():
        sys.exit(1)
    
    print("\n🎉 Success! Device is now in bootloader mode.")
    print("💡 You can now flash firmware using './flash.sh' or picotool")

if __name__ == "__main__":
    main()