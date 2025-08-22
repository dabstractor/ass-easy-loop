#!/usr/bin/env python3
"""
Simple test script to verify HID logging is working
"""

try:
    import hid
except ImportError:
    print("hidapi library not found. Please install it with: pip install hidapi")
    exit(1)

import time

# USB Vendor and Product IDs for the logging device
VENDOR_ID = 0xfade
PRODUCT_ID = 0x1212

def main():
    print(f"Searching for USB HID logging device (VID: 0x{VENDOR_ID:04x}, PID: 0x{PRODUCT_ID:04x})...")
    
    try:
        devices = hid.enumerate(VENDOR_ID, PRODUCT_ID)
        if not devices:
            print("Error: Logging device not found!")
            return
            
        print(f"Found {len(devices)} device(s)")
        for i, device in enumerate(devices):
            print(f"Device {i}:")
            for key, value in device.items():
                print(f"  {key}: {value}")
        
        # Try to open the device
        print("\nTrying to open device...")
        device = hid.Device(VENDOR_ID, PRODUCT_ID)
        print("Connected to logging device!")
        
        # Set non-blocking mode
        device.nonblocking = True
        
        print("Reading logs for 10 seconds... Press Ctrl+C to exit early")
        print("-" * 80)
        
        start_time = time.time()
        while time.time() - start_time < 10:
            try:
                # Read 64-byte input report
                data = device.read(64)
                
                if data and len(data) == 64:
                    # Parse basic log message
                    timestamp_ms = int.from_bytes(data[0:4], byteorder='little')
                    level = data[4]
                    category = data[5]
                    content_len = data[6]
                    
                    # Parse content (max 52 bytes)
                    content_bytes = data[8:8 + min(content_len, 51)]
                    content = content_bytes.decode('utf-8', errors='ignore').rstrip('\x00')
                    
                    # Check for truncation indicator
                    if content_len > 51:
                        if len(data) >= 62 and data[59:62] == b'...':
                            content += '...'
                    
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
                    
                    level_str = LOG_LEVELS.get(level, f"UNKNOWN({level})")
                    category_str = LOG_CATEGORIES.get(category, f"UNKNOWN({category})")
                    timestamp_sec = timestamp_ms / 1000.0
                    
                    print(f"[{timestamp_sec:.3f}s] [{level_str}] [{category_str}] {content}")
                            
                time.sleep(0.01)
                
            except KeyboardInterrupt:
                print("\nStopping...")
                break
            except Exception as e:
                print(f"Error reading from device: {e}")
                time.sleep(0.1)
                
    except Exception as e:
        print(f"Error: {e}")
    finally:
        print("Test completed.")

if __name__ == "__main__":
    main()