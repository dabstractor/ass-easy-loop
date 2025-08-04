#!/usr/bin/env python3
"""
Debug and fix bootloader flashing issues

This script identifies specific failure points in the existing flashing code
and provides fixes for device detection, tool paths, and communication issues.
"""

import os
import sys
import time
import subprocess
import hid
import json
import threading
import logging
from datetime import datetime
from typing import Optional, List, Dict, Any

# Set up logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

class BootloaderFlashingDebugger:
    def __init__(self):
        self.device = None
        self.issues_found = []
        self.fixes_applied = []
        
    def run_comprehensive_debug(self):
        """Run comprehensive debugging of the flashing process"""
        print("=== BOOTLOADER FLASHING DEBUGGER ===")
        print("Identifying and fixing issues in the flashing process...")
        print()
        
        # Step 1: Check build environment
        self.debug_build_environment()
        
        # Step 2: Check device detection
        self.debug_device_detection()
        
        # Step 3: Test bootloader command
        self.debug_bootloader_command()
        
        # Step 4: Check flashing tools
        self.debug_flashing_tools()
        
        # Step 5: Test manual bootloader mode
        self.debug_manual_bootloader_mode()
        
        # Step 6: Summary and recommendations
        self.print_debug_summary()
        
        return len(self.issues_found) == 0
    
    def debug_build_environment(self):
        """Debug firmware build environment"""
        print("=== DEBUGGING BUILD ENVIRONMENT ===")
        
        # Check Rust toolchain
        try:
            result = subprocess.run(['rustc', '--version'], capture_output=True, text=True)
            if result.returncode == 0:
                print(f"✓ Rust toolchain: {result.stdout.strip()}")
            else:
                self.issues_found.append("Rust toolchain not found")
                print("✗ Rust toolchain not found")
        except FileNotFoundError:
            self.issues_found.append("Rust not installed")
            print("✗ Rust not installed")
        
        # Check RP2040 target
        try:
            result = subprocess.run(['rustup', 'target', 'list', '--installed'], capture_output=True, text=True)
            if 'thumbv6m-none-eabi' in result.stdout:
                print("✓ RP2040 target (thumbv6m-none-eabi) installed")
            else:
                self.issues_found.append("RP2040 target not installed")
                print("✗ RP2040 target (thumbv6m-none-eabi) not installed")
                print("  Fix: rustup target add thumbv6m-none-eabi")
        except FileNotFoundError:
            self.issues_found.append("rustup not found")
            print("✗ rustup not found")
        
        # Check elf2uf2-rs
        try:
            result = subprocess.run(['elf2uf2-rs', '--version'], capture_output=True, text=True)
            if result.returncode == 0:
                print(f"✓ elf2uf2-rs: {result.stdout.strip()}")
            else:
                self.issues_found.append("elf2uf2-rs not working")
                print("✗ elf2uf2-rs not working properly")
        except FileNotFoundError:
            self.issues_found.append("elf2uf2-rs not installed")
            print("✗ elf2uf2-rs not installed")
            print("  Fix: cargo install elf2uf2-rs")
        
        # Test build
        print("\nTesting firmware build...")
        try:
            result = subprocess.run(['cargo', 'check'], capture_output=True, text=True, timeout=30)
            if result.returncode == 0:
                print("✓ Firmware builds successfully")
            else:
                self.issues_found.append(f"Build errors: {result.stderr}")
                print(f"✗ Build errors found:")
                print(f"  {result.stderr}")
        except subprocess.TimeoutExpired:
            self.issues_found.append("Build timeout")
            print("✗ Build timed out")
        except Exception as e:
            self.issues_found.append(f"Build test failed: {e}")
            print(f"✗ Build test failed: {e}")
        
        print()
    
    def debug_device_detection(self):
        """Debug USB device detection"""
        print("=== DEBUGGING DEVICE DETECTION ===")
        
        # Check for HID devices
        try:
            devices = hid.enumerate()
            print(f"Found {len(devices)} HID devices total")
            
            # Look for our device
            our_devices = [d for d in devices if d['vendor_id'] == 0x1234 and d['product_id'] == 0x5678]
            if our_devices:
                print(f"✓ Found {len(our_devices)} of our devices:")
                for i, device in enumerate(our_devices):
                    print(f"  Device {i+1}:")
                    print(f"    Path: {device['path']}")
                    print(f"    Serial: {device.get('serial_number', 'N/A')}")
                    print(f"    Manufacturer: {device.get('manufacturer_string', 'N/A')}")
                    print(f"    Product: {device.get('product_string', 'N/A')}")
            else:
                self.issues_found.append("Device not found with VID:PID 1234:5678")
                print("✗ Device not found with VID:PID 1234:5678")
                
                # Check for RP2040 bootloader
                bootloader_devices = [d for d in devices if d['vendor_id'] == 0x2E8A and d['product_id'] == 0x0003]
                if bootloader_devices:
                    print("! Found RP2040 in bootloader mode:")
                    for device in bootloader_devices:
                        print(f"    Path: {device['path']}")
                else:
                    print("! No RP2040 bootloader found either")
                
                # Show all devices for debugging
                print("\nAll HID devices:")
                for device in devices[:10]:  # Limit to first 10
                    print(f"  VID:PID {device['vendor_id']:04X}:{device['product_id']:04X} - {device.get('product_string', 'Unknown')}")
        
        except Exception as e:
            self.issues_found.append(f"HID enumeration failed: {e}")
            print(f"✗ HID enumeration failed: {e}")
        
        print()
    
    def debug_bootloader_command(self):
        """Debug bootloader command functionality"""
        print("=== DEBUGGING BOOTLOADER COMMAND ===")
        
        try:
            # Try to connect to device
            device = hid.Device(0x1234, 0x5678)
            print("✓ Connected to device")
            
            # Create bootloader command
            command = bytearray(64)
            command[0] = 0x80  # ENTER_BOOTLOADER command type
            command[1] = 0x01  # Command ID
            
            # Payload: JSON with timeout_ms
            payload = json.dumps({"timeout_ms": 5000}).encode('utf-8')
            command[2] = len(payload)  # Payload length
            
            # Calculate XOR checksum
            checksum = command[0] ^ command[1] ^ command[2]
            for byte in payload:
                checksum ^= byte
            command[3] = checksum
            
            # Add payload
            command[4:4+len(payload)] = payload
            
            print(f"Sending bootloader command: {command[:20].hex()}")
            
            # Send command
            bytes_sent = device.write(bytes(command))
            print(f"✓ Sent {bytes_sent} bytes")
            
            # Wait for response with better error handling
            print("Waiting for response...")
            responses_received = 0
            bootloader_response_found = False
            
            for i in range(20):  # Wait up to 20 seconds
                try:
                    data = device.read(64, timeout=1000)
                    if data:
                        responses_received += 1
                        try:
                            # Try to decode as text
                            text = bytes(data).decode('utf-8').rstrip('\x00')
                            if text:
                                print(f"Response {responses_received}: {text}")
                                if any(word in text.lower() for word in ['bootloader', 'entering', 'ready']):
                                    bootloader_response_found = True
                                    break
                        except UnicodeDecodeError:
                            # Show as hex if not text
                            hex_data = ' '.join(f'{b:02x}' for b in data[:16])
                            print(f"Response {responses_received} (hex): {hex_data}")
                    else:
                        print(f"Response {i+1}: No data (timeout)")
                except Exception as e:
                    print(f"Response {i+1}: Read error - {e}")
                    # This might be normal - device could be rebooting
                    if "Success" in str(e):
                        print("  (This 'Success' error might indicate device is rebooting)")
            
            device.close()
            
            if bootloader_response_found:
                print("✓ Bootloader command appears to be working")
            elif responses_received > 0:
                print("⚠ Device responded but no clear bootloader confirmation")
                self.issues_found.append("Bootloader command response unclear")
            else:
                print("✗ No response to bootloader command")
                self.issues_found.append("No response to bootloader command")
            
        except Exception as e:
            self.issues_found.append(f"Bootloader command test failed: {e}")
            print(f"✗ Bootloader command test failed: {e}")
        
        print()
    
    def debug_flashing_tools(self):
        """Debug firmware flashing tools"""
        print("=== DEBUGGING FLASHING TOOLS ===")
        
        # Check for picotool
        try:
            result = subprocess.run(['picotool', 'version'], capture_output=True, text=True)
            if result.returncode == 0:
                print(f"✓ picotool available: {result.stdout.strip()}")
            else:
                print("✗ picotool not working")
        except FileNotFoundError:
            print("✗ picotool not found")
            print("  Note: picotool is optional, we can use file copy method")
        
        # Check for common mount points
        mount_points = [
            "/run/media/dustin/RPI-RP2/",
            "/media/RPI-RP2/",
            "/mnt/RPI-RP2/",
            "/Volumes/RPI-RP2/"
        ]
        
        print("\nChecking for bootloader mount points:")
        found_mount = False
        for mount_point in mount_points:
            if os.path.exists(mount_point):
                print(f"✓ Found mount point: {mount_point}")
                found_mount = True
                
                # Check if it's actually mounted
                try:
                    result = subprocess.run(['mount'], capture_output=True, text=True)
                    if 'RPI-RP2' in result.stdout:
                        print("✓ RPI-RP2 is currently mounted")
                    else:
                        print("⚠ Mount point exists but may not be mounted")
                except:
                    print("⚠ Could not check mount status")
            else:
                print(f"✗ No mount at: {mount_point}")
        
        if not found_mount:
            print("! No bootloader mount points found")
            print("  This is normal if device is not in bootloader mode")
        
        print()
    
    def debug_manual_bootloader_mode(self):
        """Test manual bootloader mode entry"""
        print("=== TESTING MANUAL BOOTLOADER MODE ===")
        print("This will test the manual BOOTSEL button process")
        print()
        
        # Ask user to put device in bootloader mode
        print("Please put the device in bootloader mode:")
        print("1. Disconnect the RP2040 device from USB")
        print("2. Hold down the BOOTSEL button")
        print("3. Reconnect USB while holding BOOTSEL")
        print("4. Release BOOTSEL button")
        print("5. Device should appear as USB mass storage")
        print()
        
        input("Press Enter when you have completed these steps...")
        
        # Check for bootloader device
        print("Checking for bootloader device...")
        time.sleep(2)  # Give it a moment
        
        # Method 1: Check USB devices
        try:
            result = subprocess.run(['lsusb'], capture_output=True, text=True)
            if "2e8a:0003" in result.stdout:
                print("✓ RP2040 bootloader detected via lsusb")
            else:
                print("✗ RP2040 bootloader not found via lsusb")
                self.issues_found.append("Manual bootloader mode failed")
        except:
            print("⚠ Could not run lsusb")
        
        # Method 2: Check mount points
        mount_found = False
        for mount_point in ["/run/media/dustin/RPI-RP2/", "/media/RPI-RP2/", "/mnt/RPI-RP2/"]:
            if os.path.exists(mount_point) and os.path.ismount(mount_point):
                print(f"✓ Bootloader mounted at: {mount_point}")
                mount_found = True
                
                # Test file operations
                try:
                    test_file = os.path.join(mount_point, "test.txt")
                    with open(test_file, 'w') as f:
                        f.write("test")
                    os.remove(test_file)
                    print("✓ Mount point is writable")
                except Exception as e:
                    print(f"✗ Mount point not writable: {e}")
                    self.issues_found.append(f"Mount point not writable: {e}")
                break
        
        if not mount_found:
            print("✗ No bootloader mount point found")
            self.issues_found.append("No bootloader mount point found")
            
            # Try to find it manually
            try:
                result = subprocess.run(['mount'], capture_output=True, text=True)
                for line in result.stdout.split('\n'):
                    if 'RPI-RP2' in line:
                        parts = line.split()
                        if len(parts) >= 3:
                            mount_point = parts[2]
                            print(f"! Found RPI-RP2 mounted at: {mount_point}")
                            break
            except:
                pass
        
        print()
    
    def print_debug_summary(self):
        """Print summary of issues found and fixes applied"""
        print("=== DEBUG SUMMARY ===")
        
        if not self.issues_found:
            print("🎉 No issues found! The flashing process should work.")
        else:
            print(f"❌ Found {len(self.issues_found)} issues:")
            for i, issue in enumerate(self.issues_found, 1):
                print(f"  {i}. {issue}")
        
        if self.fixes_applied:
            print(f"\n✓ Applied {len(self.fixes_applied)} fixes:")
            for i, fix in enumerate(self.fixes_applied, 1):
                print(f"  {i}. {fix}")
        
        print("\n=== RECOMMENDATIONS ===")
        
        # Provide specific recommendations based on issues found
        if any("Rust" in issue for issue in self.issues_found):
            print("• Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh")
        
        if any("RP2040 target" in issue for issue in self.issues_found):
            print("• Install RP2040 target: rustup target add thumbv6m-none-eabi")
        
        if any("elf2uf2-rs" in issue for issue in self.issues_found):
            print("• Install elf2uf2-rs: cargo install elf2uf2-rs")
        
        if any("bootloader command" in issue.lower() for issue in self.issues_found):
            print("• The bootloader command functionality needs debugging in the firmware")
            print("• For now, use manual BOOTSEL button method for flashing")
        
        if any("mount" in issue.lower() for issue in self.issues_found):
            print("• Check USB connections and try different USB ports")
            print("• Ensure BOOTSEL button process is followed exactly")
            print("• Try manual mount: sudo mkdir -p /mnt/rpi-rp2 && sudo mount /dev/disk/by-label/RPI-RP2 /mnt/rpi-rp2")
        
        print("\n=== NEXT STEPS ===")
        if not self.issues_found:
            print("1. Try running the complete flash cycle")
            print("2. If it fails, run this debugger again to identify new issues")
        else:
            print("1. Fix the issues listed above")
            print("2. Run this debugger again to verify fixes")
            print("3. Once all issues are resolved, try the complete flash cycle")

def main():
    debugger = BootloaderFlashingDebugger()
    
    try:
        success = debugger.run_comprehensive_debug()
        
        if success:
            print("\n🎉 DEBUGGING COMPLETE - NO ISSUES FOUND")
            return 0
        else:
            print("\n❌ DEBUGGING COMPLETE - ISSUES FOUND")
            print("Please address the issues above and run again")
            return 1
            
    except KeyboardInterrupt:
        print("\n\nDebugging interrupted by user")
        return 1
    except Exception as e:
        print(f"\n❌ DEBUGGING FAILED: {e}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    sys.exit(main())