#!/usr/bin/env python3
"""
Comprehensive USB device enumeration script
"""

try:
    import hid
    print("hidapi imported successfully")
    
    # Test enumeration
    devices = hid.enumerate()
    print(f"Found {len(devices)} HID devices")
    
    # Show all devices
    for i, device in enumerate(devices):
        print(f"\nDevice {i+1}:")
        print(f"  VID: 0x{device['vendor_id']:04x}")
        print(f"  PID: 0x{device['product_id']:04x}")
        print(f"  Manufacturer: {device.get('manufacturer_string', 'Unknown')}")
        print(f"  Product: {device.get('product_string', 'Unknown')}")
        print(f"  Serial: {device.get('serial_number', 'Unknown')}")
        
    # Look for our target device
    TARGET_VID = 0xfade
    TARGET_PID = 0x1212
    
    target_devices = [d for d in devices if d['vendor_id'] == TARGET_VID and d['product_id'] == TARGET_PID]
    print(f"\nFound {len(target_devices)} target devices (VID: 0x{TARGET_VID:04x}, PID: 0x{TARGET_PID:04x})")
    
    if target_devices:
        print("Target device found!")
        device_info = target_devices[0]
        print(f"  Manufacturer: {device_info.get('manufacturer_string', 'Unknown')}")
        print(f"  Product: {device_info.get('product_string', 'Unknown')}")
    else:
        print("Target device not found - please connect the device and ensure it's powered on")
        
except ImportError as e:
    print(f"Import error: {e}")
    print("Please install hidapi: pip install hidapi")
except Exception as e:
    print(f"Error: {e}")
    import traceback
    traceback.print_exc()