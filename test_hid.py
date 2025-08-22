#!/usr/bin/env python3
import hid
import time

# USB Vendor and Product IDs
VENDOR_ID = 0xfade
PRODUCT_ID = 0x1212

def main():
    print(f"Searching for device (VID: 0x{VENDOR_ID:04x}, PID: 0x{PRODUCT_ID:04x})...")
    
    # Try to find and open the device
    try:
        devices = hid.enumerate(VENDOR_ID, PRODUCT_ID)
        print(f"Found {len(devices)} device(s)")
        
        for device_info in devices:
            print(f"Device: {device_info}")
            
        if not devices:
            print("No devices found")
            return
            
        # Open the device
        device = hid.Device(VENDOR_ID, PRODUCT_ID)
        print("Device opened successfully")
        
        # Set non-blocking mode
        device.nonblocking = True
        
        print("Reading data... (Press Ctrl+C to stop)")
        try:
            while True:
                try:
                    data = device.read(64, timeout=1000)
                    if data:
                        print(f"Received {len(data)} bytes: {data}")
                        if len(data) == 64:
                            # Try to parse as log message
                            timestamp = int.from_bytes(data[0:4], byteorder='little')
                            level = data[4]
                            category = data[5]
                            content_len = data[6]
                            reserved = data[7]
                            content = data[8:8+min(content_len, 51)]
                            try:
                                content_str = content.decode('utf-8').rstrip('\x00')
                                print(f"  Parsed: [{timestamp}ms] Level={level} Category={category} Content='{content_str}'")
                            except:
                                print(f"  Raw content: {content}")
                    else:
                        print("No data received")
                except Exception as e:
                    print(f"Error reading: {e}")
                time.sleep(0.1)
        except KeyboardInterrupt:
            print("\nStopping...")
        finally:
            device.close()
            
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    main()