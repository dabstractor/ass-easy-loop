#!/usr/bin/env python3
"""
Fixed bootloader flashing implementation

This script fixes the identified issues in the flashing process:
1. Uses manual BOOTSEL button method (since software bootloader command isn't working)
2. Handles elf2uf2-rs correctly
3. Provides robust error handling and recovery
4. Includes explicit user interaction for BOOTSEL button
"""

import os
import sys
import time
import subprocess
import hid
import threading
from datetime import datetime
from typing import Optional

class FixedBootloaderFlasher:
    def __init__(self):
        self.device = None
        self.logging = False
        self.log_messages = []
        self.log_thread = None
        
    def build_firmware(self) -> bool:
        """Build firmware with proper error handling"""
        print("=== STEP 1: Building Firmware ===")
        
        try:
            # Clean build first
            print("Cleaning previous build...")
            result = subprocess.run(['cargo', 'clean'], capture_output=True, text=True, timeout=30)
            if result.returncode != 0:
                print(f"Warning: cargo clean failed: {result.stderr}")
            
            # Build firmware with timeout
            print("Building firmware (this may take a while)...")
            result = subprocess.run(
                ['cargo', 'build', '--release'], 
                capture_output=True, 
                text=True, 
                timeout=120  # 2 minute timeout
            )
            
            if result.returncode != 0:
                print(f"✗ Build failed:")
                print(f"STDOUT: {result.stdout}")
                print(f"STDERR: {result.stderr}")
                return False
                
            print("✓ Firmware build successful")
            
            # Check if ELF file exists
            elf_path = 'target/thumbv6m-none-eabi/release/ass-easy-loop'
            if not os.path.exists(elf_path):
                print(f"✗ ELF file not found at: {elf_path}")
                return False
            
            print(f"✓ ELF file found: {elf_path}")
            
            # Convert to UF2 using correct elf2uf2-rs syntax
            print("Converting to UF2 format...")
            result = subprocess.run([
                'elf2uf2-rs', 
                elf_path,
                'firmware.uf2'
            ], capture_output=True, text=True, timeout=30)
            
            if result.returncode != 0:
                print(f"✗ UF2 conversion failed:")
                print(f"STDOUT: {result.stdout}")
                print(f"STDERR: {result.stderr}")
                return False
                
            # Verify UF2 file exists and has reasonable size
            if not os.path.exists('firmware.uf2'):
                print("✗ firmware.uf2 not found after conversion")
                return False
                
            file_size = os.path.getsize('firmware.uf2')
            if file_size < 1000:  # Should be at least 1KB
                print(f"✗ firmware.uf2 seems too small ({file_size} bytes)")
                return False
                
            print(f"✓ UF2 conversion successful - firmware.uf2 ({file_size} bytes)")
            return True
            
        except subprocess.TimeoutExpired:
            print("✗ Build process timed out")
            return False
        except Exception as e:
            print(f"✗ Build process failed: {e}")
            return False
    
    def wait_for_bootloader_mode_with_user_help(self, timeout: int = 60) -> Optional[str]:
        """Wait for user to put device in bootloader mode with clear instructions"""
        print("\n=== STEP 2: Enter Bootloader Mode (Manual) ===")
        print("The software bootloader command isn't working reliably, so we'll use manual method.")
        print()
        print("Please put the device in bootloader mode:")
        print("1. Disconnect the RP2040 device from USB")
        print("2. Hold down the BOOTSEL button (small button on the board)")
        print("3. While holding BOOTSEL, reconnect the USB cable")
        print("4. Release the BOOTSEL button")
        print("5. The device should appear as a USB mass storage device named 'RPI-RP2'")
        print()
        
        input("Press Enter when you have completed these steps...")
        
        # Give the system a moment to detect the device
        print("Waiting for bootloader device to appear...")
        time.sleep(3)
        
        # Check multiple possible mount points
        possible_mounts = [
            "/run/media/dustin/RPI-RP2/",
            "/media/RPI-RP2/",
            "/mnt/RPI-RP2/",
            "/media/pi/RPI-RP2/",
            "/Volumes/RPI-RP2/"  # macOS
        ]
        
        start_time = time.time()
        while time.time() - start_time < timeout:
            # Check each possible mount point
            for mount_point in possible_mounts:
                if os.path.exists(mount_point) and os.path.ismount(mount_point):
                    print(f"✓ Bootloader mount point found: {mount_point}")
                    
                    # Verify it's writable
                    try:
                        test_file = os.path.join(mount_point, "test_write.tmp")
                        with open(test_file, 'w') as f:
                            f.write("test")
                        os.remove(test_file)
                        print("✓ Mount point is writable")
                        return mount_point
                    except Exception as e:
                        print(f"⚠ Mount point found but not writable: {e}")
                        continue
            
            # Also check via lsusb
            try:
                result = subprocess.run(['lsusb'], capture_output=True, text=True)
                if "2e8a:0003" in result.stdout:
                    print("✓ RP2040 bootloader detected via lsusb")
                    
                    # Try to find mount point dynamically
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
            except:
                pass
            
            print(".", end="", flush=True)
            time.sleep(1)
        
        print(f"\n✗ Bootloader mount point not found within {timeout} seconds")
        print("\nTroubleshooting:")
        print("- Make sure you held the BOOTSEL button while connecting USB")
        print("- Try a different USB cable or port")
        print("- Check if the device appears in file manager as 'RPI-RP2'")
        print("- On some systems, you may need to manually mount the device")
        
        return None
    
    def flash_firmware_to_bootloader(self, mount_point: str) -> bool:
        """Flash firmware to bootloader mount point"""
        print("\n=== STEP 3: Flashing Firmware ===")
        
        try:
            firmware_path = os.path.abspath('firmware.uf2')
            target_path = os.path.join(mount_point, 'firmware.uf2')
            
            print(f"Copying firmware from: {firmware_path}")
            print(f"                   to: {target_path}")
            
            # Verify source file exists
            if not os.path.exists(firmware_path):
                print(f"✗ Source firmware file not found: {firmware_path}")
                return False
            
            # Copy firmware file
            result = subprocess.run(['cp', firmware_path, target_path], capture_output=True, text=True)
            
            if result.returncode != 0:
                print(f"✗ Copy failed: {result.stderr}")
                return False
                
            print("✓ Firmware copied successfully")
            
            # Verify the copy
            if os.path.exists(target_path):
                source_size = os.path.getsize(firmware_path)
                target_size = os.path.getsize(target_path)
                if source_size == target_size:
                    print(f"✓ Copy verified ({target_size} bytes)")
                else:
                    print(f"⚠ Size mismatch: source={source_size}, target={target_size}")
            
            # Wait for device to reboot (mount point should disappear)
            print("Waiting for device to reboot...")
            start_time = time.time()
            
            while time.time() - start_time < 15:  # Wait up to 15 seconds
                if not os.path.exists(mount_point):
                    print("✓ Device rebooted (mount point disappeared)")
                    return True
                time.sleep(0.5)
                print(".", end="", flush=True)
            
            print("\n⚠ Mount point still exists - device may not have rebooted automatically")
            print("This can be normal - the device might still be flashing")
            return True  # Continue anyway
            
        except Exception as e:
            print(f"✗ Flash failed: {e}")
            return False
    
    def wait_for_device_reconnection(self, timeout: int = 30) -> bool:
        """Wait for device to reconnect after flashing"""
        print("\n=== STEP 4: Waiting for Device Reconnection ===")
        
        # Give the device some time to boot
        print("Giving device time to boot new firmware...")
        time.sleep(5)
        
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            try:
                # Try to find our HID device
                devices = hid.enumerate(0x1234, 0x5678)
                if devices:
                    print("✓ Device reconnected as HID device")
                    print(f"  Found {len(devices)} device(s)")
                    for i, device in enumerate(devices):
                        print(f"    Device {i+1}: {device['path']}")
                    return True
                    
            except Exception as e:
                print(f"⚠ HID enumeration error: {e}")
                
            time.sleep(1)
            print(".", end="", flush=True)
        
        print(f"\n✗ Device did not reconnect within {timeout} seconds")
        
        # Provide troubleshooting info
        print("\nTroubleshooting:")
        print("- Check if device LED is blinking (indicates it's running)")
        print("- Try disconnecting and reconnecting USB")
        print("- Check if device appears in lsusb output")
        
        try:
            result = subprocess.run(['lsusb'], capture_output=True, text=True)
            if "1234:5678" in result.stdout:
                print("✓ Device found in lsusb with correct VID:PID")
            elif "2e8a" in result.stdout:
                print("! Device still appears as RP2040 (may need manual reset)")
            else:
                print("✗ Device not found in lsusb")
        except:
            pass
        
        return False
    
    def start_hid_logging_verification(self) -> bool:
        """Start HID logging to verify device functionality"""
        print("\n=== STEP 5: Starting HID Logging Verification ===")
        
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
        """HID logging loop with better error handling"""
        message_count = 0
        consecutive_errors = 0
        
        while self.logging:
            try:
                data = self.device.read(64, timeout=100)
                
                if data:
                    message_count += 1
                    consecutive_errors = 0  # Reset error count on successful read
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
                else:
                    # No data is normal, don't count as error
                    pass
                        
            except Exception as e:
                if self.logging:
                    consecutive_errors += 1
                    if consecutive_errors < 5:  # Only log first few errors
                        print(f"[DEBUG] HID read error: {e}")
                    elif consecutive_errors == 5:
                        print("[DEBUG] Suppressing further HID read errors...")
                    
                    # If too many consecutive errors, something is wrong
                    if consecutive_errors > 100:
                        print("[ERROR] Too many consecutive HID errors, stopping logging")
                        break
    
    def stop_hid_logging(self):
        """Stop HID logging"""
        if self.logging:
            self.logging = False
            if self.log_thread:
                self.log_thread.join(timeout=2)
            
            if self.device:
                try:
                    self.device.close()
                except:
                    pass
                self.device = None
                
            print("✓ HID logging stopped")
    
    def analyze_device_functionality(self) -> bool:
        """Analyze collected log messages to verify device functionality"""
        print("\n=== STEP 6: Analyzing Device Functionality ===")
        
        print(f"✓ Collected {len(self.log_messages)} log messages")
        
        if not self.log_messages:
            print("⚠ No log messages received")
            print("This could mean:")
            print("- Device is not logging (check firmware configuration)")
            print("- USB HID communication issue")
            print("- Device is running but not sending log messages")
            return False
        
        # Show first few messages
        print("\nFirst few log messages:")
        for i, message in enumerate(self.log_messages[:5]):
            print(f"  {message}")
        
        if len(self.log_messages) > 5:
            print(f"  ... and {len(self.log_messages) - 5} more messages")
        
        # Look for system health indicators
        indicators = {
            'system_startup': False,
            'task_activity': False,
            'usb_activity': False,
            'hardware_activity': False,
            'error_free': True
        }
        
        for message in self.log_messages:
            msg_lower = message.lower()
            
            if any(word in msg_lower for word in ['startup', 'init', 'boot', 'start', 'ready']):
                indicators['system_startup'] = True
            
            if any(word in msg_lower for word in ['task', 'pemf', 'battery', 'adc']):
                indicators['task_activity'] = True
                
            if any(word in msg_lower for word in ['usb', 'hid', 'command']):
                indicators['usb_activity'] = True
                
            if any(word in msg_lower for word in ['led', 'gpio', 'mosfet', 'pulse']):
                indicators['hardware_activity'] = True
                
            if any(word in msg_lower for word in ['error', 'fail', 'panic', 'crash']):
                indicators['error_free'] = False
        
        # Report findings
        print("\nSystem Health Analysis:")
        for indicator, found in indicators.items():
            status = "✓" if found else "✗"
            print(f"  {status} {indicator.replace('_', ' ').title()}: {found}")
        
        # Overall assessment
        healthy_indicators = sum(1 for found in indicators.values() if found)
        total_indicators = len(indicators)
        
        if healthy_indicators >= 3:  # At least 3 out of 5 indicators positive
            print("✓ Device appears to be functioning correctly")
            return True
        else:
            print("⚠ Device functionality unclear - may need investigation")
            print(f"  Found {healthy_indicators}/{total_indicators} positive indicators")
            return False
    
    def run_complete_fixed_cycle(self) -> bool:
        """Run the complete fixed flash and test cycle"""
        print("=== FIXED FIRMWARE FLASH AND TEST CYCLE ===")
        print("This version uses manual BOOTSEL button method and fixed build process")
        print()
        
        try:
            # Step 1: Build firmware
            if not self.build_firmware():
                print("\n❌ Build failed - cannot continue")
                return False
            
            # Step 2: Manual bootloader mode entry
            mount_point = self.wait_for_bootloader_mode_with_user_help()
            if not mount_point:
                print("\n❌ Could not enter bootloader mode - cannot continue")
                return False
            
            # Step 3: Flash firmware
            if not self.flash_firmware_to_bootloader(mount_point):
                print("\n❌ Firmware flashing failed - cannot continue")
                return False
            
            # Step 4: Wait for reconnection
            if not self.wait_for_device_reconnection():
                print("\n❌ Device did not reconnect - flash may have failed")
                return False
            
            # Step 5: Start logging and verify
            if not self.start_hid_logging_verification():
                print("\n❌ Could not start HID logging - device may not be working")
                return False
            
            # Let it log for a while
            print("Collecting log messages for 15 seconds...")
            time.sleep(15)
            
            # Step 6: Analyze functionality
            success = self.analyze_device_functionality()
            
            return success
            
        finally:
            self.stop_hid_logging()

def main():
    flasher = FixedBootloaderFlasher()
    
    try:
        success = flasher.run_complete_fixed_cycle()
        
        if success:
            print("\n🎉 FIXED FLASH CYCLE SUCCESS!")
            print("✓ Firmware built successfully")
            print("✓ Manual bootloader entry worked")
            print("✓ Firmware flashed successfully")
            print("✓ Device reconnected")
            print("✓ HID logging verified")
            print("✓ Device functionality confirmed")
            print("\nThe basic flash cycle is now working! You can iterate autonomously.")
            return 0
        else:
            print("\n❌ FIXED FLASH CYCLE FAILED")
            print("Check the error messages above for details")
            print("The manual BOOTSEL method should work - please try again")
            return 1
            
    except KeyboardInterrupt:
        print("\n\nCycle interrupted by user")
        return 1
    except Exception as e:
        print(f"\n❌ UNEXPECTED ERROR: {e}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    sys.exit(main())