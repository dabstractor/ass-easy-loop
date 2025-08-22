#!/usr/bin/env python3
"""
Simple test script to verify the logging system works
"""

try:
    import hid
    print("hidapi imported successfully")
    
    # Test enumeration
    devices = hid.enumerate()
    print(f"Found {len(devices)} HID devices")
    
    # Look for our logging device
    LOGGING_VID = 0xfade
    LOGGING_PID = 0x1212
    
    logging_devices = [d for d in devices if d['vendor_id'] == LOGGING_VID and d['product_id'] == LOGGING_PID]
    print(f"Found {len(logging_devices)} logging devices")
    
    if logging_devices:
        print("Logging device found:")
        device_info = logging_devices[0]
        print(f"  VID: 0x{device_info['vendor_id']:04x}")
        print(f"  PID: 0x{device_info['product_id']:04x}")
        print(f"  Manufacturer: {device_info.get('manufacturer_string', 'Unknown')}")
        print(f"  Product: {device_info.get('product_string', 'Unknown')}")
    else:
        print("No logging device found - this is expected if the device is not connected")
        
except ImportError as e:
    print(f"Import error: {e}")
    print("Please install hidapi: pip install hidapi")
except Exception as e:
    print(f"Error: {e}")