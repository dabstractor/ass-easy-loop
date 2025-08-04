#!/usr/bin/env python3
"""
Test script for bootloader fix

This script tests the fixed bootloader functionality and provides
a complete flash cycle test.
"""

import hid
import time
import struct
import subprocess
import os
import threading
from datetime import datetime

class BootloaderFixTester:
    def __init__(self):
        self.device = None
        self.logging = False
        self.log_messages = []
        self.log_thread = None
        
    def test_bootloader_command(self):
        """Test the fixed bootloader command"""
        print("=== TESTING FIXED BOOTLOADER COMMAND ===")
        
        try:
            device = hid.Device(0x1234, 0x5678)
            print("✓ Connected to device")
            
            # Create bootloader command
            command = bytearray(64)
            command[0] = 0x80  # ENTER_BOOTLOADER command type
            command[1] = 0x01  # Command ID
            
            # Short timeout for quick response
            timeout_ms = 1000
            payload = struct.pack('<I', timeout_ms)
            
            command[2] = len(payload)
            
            # Calculate checksum
            checksum = command[0] ^ command[1] ^ command[2]
            for byte in payload:
                checksum ^= byte
            command[3] = checksum
            
            # Add payload
            command[4:4+len(payload)] = payload
            
            print(f"Sending bootloader command...")
            
            # Send command
            bytes_sent = device.write(bytes(command))
            print(f"✓ Sent {bytes_sent} bytes")
            
            # Try to read response briefly
            try:
                for i in range(3):
                    data = device.read(64, timeout=500)
                    if data:
                        try:
                            text = bytes(data).decode('utf-8').rstrip('\x00')
                            if text and 'DIRECT BOOTLOADER' in text:
                                print(f"✓ Got bootloader response: {text}")
                                break
                        except:
                            pass
            except Exception as e:
                print(f"Device disconnected (expected): {e}")
            
            device.close()
            return True
            
        except Exception as e:
            print(f"✗ Command failed: {e}")
            return False
    
    def wait_for_bootloader_mode(self, timeout=15):
        """Wait for device to enter bootloader mode"""
        print("\nWaiting for bootloader mode...")
        
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            # Check via lsusb
            try:
                result = subprocess.run(['lsusb'], capture_output=True, text=True, timeout=2)
                if "2e8a:0003" in result.stdout:
                    print("✓ Device detected in bootloader mode via lsusb")
                    
                    # Also check for mount point
                    time.sleep(2)  # Give it time to mount
                    mount_points = [
                        "/run/media/dustin/RPI-RP2/",
                        "/media/RPI-RP2/", 
                        "/mnt/RPI-RP2/"
                    ]
                    
                    for mount_point in mount_points:
                        if os.path.exists(mount_point) and os.path.ismount(mount_point):
                            print(f"✓ Bootloader mounted at: {mount_point}")
                            return mount_point
                    
                    # If no mount point found, try to find it dynamically
                    try:
                        result = subprocess.run(['mount'], capture_output=True, text=True)
                        for line in result.stdout.split('\n'):
                            if 'RPI-RP2' in line:
                                parts = line.split()
                                if len(parts) >= 3:
                                    mount_point = parts[2]
                                    print(f"✓ Found RPI-RP2 mounted at: {mount_point}")
                                    return mount_point
                    except:
                        pass
                    
                    print("⚠ Bootloader detected but no mount point found yet")
                    
            except:
                pass
            
            print(".", end="", flush=True)
            time.sleep(1)
        
        print(f"\n✗ Bootloader mode not detected within {timeout} seconds")
        return None
    
    def build_and_flash_firmware(self, mount_point):
        """Build firmware and flash it"""
        print("\n=== BUILDING AND FLASHING FIRMWARE ===")
        
        # Build firmware
        print("Building firmware...")
        try:
            result = subprocess.run(
                ['cargo', 'build', '--release'], 
                capture_output=True, 
                text=True, 
                timeout=120
            )
            
            if result.returncode != 0:
                print(f"✗ Build failed: {result.stderr}")
                return False
                
            print("✓ Firmware build successful")
            
            # Convert to UF2
            print("Converting to UF2...")
            elf_path = 'target/thumbv6m-none-eabi/release/ass-easy-loop'
            result = subprocess.run([
                'elf2uf2-rs', 
                elf_path,
                'firmware.uf2'
            ], capture_output=True, text=True, timeout=30)
            
            if result.returncode != 0:
                print(f"✗ UF2 conversion failed: {result.stderr}")
                return False
                
            print("✓ UF2 conversion successful")
            
        except subprocess.TimeoutExpired:
            print("✗ Build timed out")
            return False
        except Exception as e:
            print(f"✗ Build failed: {e}")
            return False
        
        # Flash firmware
        print("Flashing firmware...")
        try:
            firmware_path = os.path.abspath('firmware.uf2')
            target_path = os.path.join(mount_point, 'firmware.uf2')
            
            result = subprocess.run(['cp', firmware_path, target_path], 
                                  capture_output=True, text=True)
            
            if result.returncode != 0:
                print(f"✗ Flash failed: {result.stderr}")
                return False
                
            print("✓ Firmware flashed successfully")
            
            # Wait for device to reboot
            print("Waiting for device to reboot...")
            time.sleep(5)
            
            return True
            
        except Exception as e:
            print(f"✗ Flash failed: {e}")
            return False
    
    def wait_for_device_reconnection(self, timeout=30):
        """Wait for device to reconnect after flashing"""
        print("\nWaiting for device reconnection...")
        
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            try:
                devices = hid.enumerate(0x1234, 0x5678)
                if devices:
                    print("✓ Device reconnected")
                    return True
            except:
                pass
                
            print(".", end="", flush=True)
            time.sleep(1)
        
        print(f"\n✗ Device did not reconnect within {timeout} seconds")
        return False
    
    def verify_device_functionality(self):
        """Verify device is working after flash"""
        print("\n=== VERIFYING DEVICE FUNCTIONALITY ===")
        
        try:
            device = hid.Device(0x1234, 0x5678)
            print("✓ Connected to device")
            
            # Try to read some data
            print("Reading device data for 5 seconds...")
            messages_received = 0
            
            for i in range(50):  # 5 seconds at 100ms intervals
                try:
                    data = device.read(64, timeout=100)
                    if data:
                        messages_received += 1
                        try:
                            text = bytes(data).decode('utf-8').rstrip('\x00')
                            if text and i < 3:  # Show first few messages
                                print(f"  Message: {text}")
                        except:
                            pass
                except:
                    pass
            
            device.close()
            
            if messages_received > 0:
                print(f"✓ Device is functional ({messages_received} messages received)")
                return True
            else:
                print("⚠ Device connected but no data received")
                return False
                
        except Exception as e:
            print(f"✗ Device verification failed: {e}")
            return False
    
    def run_complete_test(self):
        """Run complete bootloader fix test"""
        print("=== BOOTLOADER FIX COMPLETE TEST ===")
        print("Testing the fixed bootloader functionality")
        print()
        
        # Step 1: Test bootloader command
        if not self.test_bootloader_command():
            print("❌ Bootloader command test failed")
            return False
        
        # Step 2: Wait for bootloader mode
        mount_point = self.wait_for_bootloader_mode()
        if not mount_point:
            print("❌ Device did not enter bootloader mode")
            return False
        
        # Step 3: Build and flash firmware
        if not self.build_and_flash_firmware(mount_point):
            print("❌ Firmware build/flash failed")
            return False
        
        # Step 4: Wait for reconnection
        if not self.wait_for_device_reconnection():
            print("❌ Device did not reconnect")
            return False
        
        # Step 5: Verify functionality
        if not self.verify_device_functionality():
            print("❌ Device functionality verification failed")
            return False
        
        return True

def main():
    tester = BootloaderFixTester()
    
    try:
        success = tester.run_complete_test()
        
        if success:
            print("\n🎉 BOOTLOADER FIX TEST SUCCESS!")
            print("✓ Bootloader command works")
            print("✓ Device enters bootloader mode")
            print("✓ Firmware builds and flashes")
            print("✓ Device reconnects and functions")
            print("\nThe bootloader fix is working! You can now flash firmware autonomously.")
            return 0
        else:
            print("\n❌ BOOTLOADER FIX TEST FAILED")
            print("The fix needs more work")
            return 1
            
    except KeyboardInterrupt:
        print("\n\nTest interrupted by user")
        return 1
    except Exception as e:
        print(f"\n❌ TEST ERROR: {e}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    exit(main())