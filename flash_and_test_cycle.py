#!/usr/bin/env python3
"""
Complete firmware flash and test cycle with HID logging verification
"""

import os
import sys
import time
import subprocess
import hid
import threading
from datetime import datetime

class FlashAndTestCycle:
    def __init__(self):
        self.device = None
        self.logging = False
        self.log_messages = []
        self.log_thread = None
        
    def build_firmware(self):
        """Build the firmware using cargo"""
        print("=== STEP 1: Building Firmware ===")
        
        try:
            # Clean build
            print("Cleaning previous build...")
            result = subprocess.run(['cargo', 'clean'], capture_output=True, text=True)
            if result.returncode != 0:
                print(f"Warning: cargo clean failed: {result.stderr}")
            
            # Build firmware
            print("Building firmware...")
            result = subprocess.run(['cargo', 'build', '--release'], capture_output=True, text=True)
            
            if result.returncode != 0:
                print(f"✗ Build failed: {result.stderr}")
                return False
                
            print("✓ Firmware build successful")
            
            # Convert to UF2
            print("Converting to UF2 format...")
            result = subprocess.run([
                'elf2uf2-rs', 
                'target/thumbv6m-none-eabi/release/ass-easy-loop', 
                'firmware.uf2'
            ], capture_output=True, text=True)
            
            if result.returncode != 0:
                print(f"✗ UF2 conversion failed: {result.stderr}")
                return False
                
            # Verify UF2 file exists
            if not os.path.exists('firmware.uf2'):
                print("✗ firmware.uf2 not found after conversion")
                return False
                
            file_size = os.path.getsize('firmware.uf2')
            print(f"✓ UF2 conversion successful - firmware.uf2 ({file_size} bytes)")
            return True
            
        except Exception as e:
            print(f"✗ Build process failed: {e}")
            return False
    
    def wait_for_bootloader_mode(self, timeout=60):
        """Wait for user to put device in bootloader mode"""
        print("\n=== STEP 2: Enter Bootloader Mode ===")
        print("Please put the device in bootloader mode:")
        print("1. Disconnect the RP2040 device from USB")
        print("2. Hold down the BOOTSEL button")
        print("3. Reconnect USB while holding BOOTSEL")
        print("4. Release BOOTSEL button")
        print("5. Device should appear as USB mass storage")
        print()
        
        input("Press Enter when you have completed these steps...")
        
        # Check for bootloader mount point
        mount_point = "/run/media/dustin/RPI-RP2/"
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            if os.path.exists(mount_point):
                print(f"✓ Bootloader mount point found: {mount_point}")
                return mount_point
                
            time.sleep(1)
            print(".", end="", flush=True)
        
        print(f"\n✗ Bootloader mount point not found within {timeout} seconds")
        return None
    
    def flash_firmware(self, mount_point):
        """Flash firmware to the device"""
        print("\n=== STEP 3: Flashing Firmware ===")
        
        try:
            firmware_path = os.path.abspath('firmware.uf2')
            target_path = os.path.join(mount_point, 'firmware.uf2')
            
            print(f"Copying {firmware_path} to {target_path}")
            
            # Copy firmware
            result = subprocess.run(['cp', firmware_path, target_path], capture_output=True, text=True)
            
            if result.returncode != 0:
                print(f"✗ Copy failed: {result.stderr}")
                return False
                
            print("✓ Firmware copied successfully")
            
            # Wait for device to reboot (mount point should disappear)
            print("Waiting for device to reboot...")
            start_time = time.time()
            
            while time.time() - start_time < 10:
                if not os.path.exists(mount_point):
                    print("✓ Device rebooted (mount point disappeared)")
                    return True
                time.sleep(0.5)
                print(".", end="", flush=True)
            
            print("\n⚠ Mount point still exists - device may not have rebooted")
            return True  # Continue anyway
            
        except Exception as e:
            print(f"✗ Flash failed: {e}")
            return False
    
    def wait_for_device_reconnection(self, timeout=30):
        """Wait for device to reconnect after flashing"""
        print("\n=== STEP 4: Waiting for Device Reconnection ===")
        
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            try:
                # Try to find HID device
                devices = hid.enumerate(0x1234, 0x5678)
                if devices:
                    print("✓ Device reconnected as HID device")
                    return True
                    
            except Exception as e:
                pass
                
            time.sleep(1)
            print(".", end="", flush=True)
        
        print(f"\n✗ Device did not reconnect within {timeout} seconds")
        return False
    
    def start_hid_logging(self):
        """Start logging HID messages"""
        print("\n=== STEP 5: Starting HID Logging ===")
        
        try:
            self.device = hid.Device(0x1234, 0x5678)
            print("✓ Connected to HID device")
            
            self.logging = True
            self.log_messages = []
            self.log_thread = threading.Thread(target=self._log_loop, daemon=True)
            self.log_thread.start()
            
            print("✓ HID logging started")
            return True
            
        except Exception as e:
            print(f"✗ Failed to start HID logging: {e}")
            return False
    
    def _log_loop(self):
        """HID logging loop"""
        message_count = 0
        
        while self.logging:
            try:
                data = self.device.read(64, timeout=100)
                
                if data:
                    message_count += 1
                    timestamp = datetime.now().strftime("%H:%M:%S.%f")[:-3]
                    
                    try:
                        text = bytes(data).decode('utf-8').rstrip('\x00')
                        if text:
                            log_entry = f"[{timestamp}] #{message_count:04d}: {text}"
                            print(log_entry)
                            self.log_messages.append(log_entry)
                    except UnicodeDecodeError:
                        hex_data = ' '.join(f'{b:02x}' for b in data[:16])
                        log_entry = f"[{timestamp}] #{message_count:04d}: RAW: {hex_data}..."
                        print(log_entry)
                        self.log_messages.append(log_entry)
                        
            except Exception as e:
                if self.logging:
                    print(f"[ERROR] HID read error: {e}")
    
    def stop_hid_logging(self):
        """Stop HID logging"""
        if self.logging:
            self.logging = False
            if self.log_thread:
                self.log_thread.join(timeout=2)
            
            if self.device:
                self.device.close()
                self.device = None
                
            print("✓ HID logging stopped")
    
    def test_device_functionality(self):
        """Test basic device functionality"""
        print("\n=== STEP 6: Testing Device Functionality ===")
        
        if not self.device:
            print("✗ No device connection for testing")
            return False
        
        # Let it log for a few seconds to see system messages
        print("Collecting log messages for 10 seconds...")
        time.sleep(10)
        
        # Analyze collected messages
        print(f"\n✓ Collected {len(self.log_messages)} log messages")
        
        # Look for specific indicators
        success_indicators = [
            "Success rate",
            "Task",
            "System",
            "USB",
            "pEMF",
            "Battery",
            "LED"
        ]
        
        found_indicators = []
        for indicator in success_indicators:
            for message in self.log_messages:
                if indicator.lower() in message.lower():
                    found_indicators.append(indicator)
                    break
        
        print(f"✓ Found system indicators: {', '.join(found_indicators)}")
        
        if found_indicators:
            print("✓ Device appears to be functioning correctly")
            return True
        else:
            print("⚠ No clear system indicators found in logs")
            return False
    
    def run_complete_cycle(self):
        """Run the complete flash and test cycle"""
        print("=== COMPLETE FIRMWARE FLASH AND TEST CYCLE ===")
        print()
        
        try:
            # Step 1: Build firmware
            if not self.build_firmware():
                return False
            
            # Step 2: Wait for bootloader mode
            mount_point = self.wait_for_bootloader_mode()
            if not mount_point:
                return False
            
            # Step 3: Flash firmware
            if not self.flash_firmware(mount_point):
                return False
            
            # Step 4: Wait for reconnection
            if not self.wait_for_device_reconnection():
                return False
            
            # Step 5: Start logging and test
            if not self.start_hid_logging():
                return False
            
            # Step 6: Test functionality
            success = self.test_device_functionality()
            
            return success
            
        finally:
            self.stop_hid_logging()

def main():
    cycle = FlashAndTestCycle()
    
    try:
        success = cycle.run_complete_cycle()
        
        if success:
            print("\n🎉 COMPLETE CYCLE SUCCESS!")
            print("✓ Firmware built successfully")
            print("✓ Device entered bootloader mode")
            print("✓ Firmware flashed successfully")
            print("✓ Device reconnected")
            print("✓ HID logging working")
            print("✓ Device functionality verified")
            return 0
        else:
            print("\n❌ CYCLE FAILED")
            print("Check the error messages above for details")
            return 1
            
    except KeyboardInterrupt:
        print("\n\nCycle interrupted by user")
        return 1
    except Exception as e:
        print(f"\n❌ UNEXPECTED ERROR: {e}")
        return 1

if __name__ == "__main__":
    sys.exit(main())