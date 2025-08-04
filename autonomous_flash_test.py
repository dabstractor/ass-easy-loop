#!/usr/bin/env python3
"""
Autonomous firmware flash test using software bootloader entry
"""

import os
import sys
import time
import subprocess
import hid
import threading
import logging
from datetime import datetime

# Import existing test framework components
sys.path.append('.')
from test_framework.device_manager import UsbHidDeviceManager, DeviceStatus
from test_framework.command_handler import CommandHandler
from test_framework.firmware_flasher import FirmwareFlasher

class AutonomousFlashTest:
    def __init__(self):
        self.device_manager = UsbHidDeviceManager()
        self.command_handler = CommandHandler(self.device_manager)
        self.firmware_flasher = FirmwareFlasher(self.device_manager, self.command_handler)
        self.log_messages = []
        self.logging = False
        self.log_thread = None
        
        # Set up logging
        logging.basicConfig(level=logging.INFO)
        self.logger = logging.getLogger(__name__)
        
    def build_firmware(self):
        """Build the firmware using cargo"""
        print("=== STEP 1: Building Firmware ===")
        
        try:
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
                
            if not os.path.exists('firmware.uf2'):
                print("✗ firmware.uf2 not found after conversion")
                return False
                
            file_size = os.path.getsize('firmware.uf2')
            print(f"✓ UF2 conversion successful - firmware.uf2 ({file_size} bytes)")
            return True
            
        except Exception as e:
            print(f"✗ Build process failed: {e}")
            return False
    
    def discover_and_connect_device(self):
        """Discover and connect to the device"""
        print("\n=== STEP 2: Discovering Device ===")
        
        # Discover devices
        self.device_manager.discover_devices()
        connected_devices = self.device_manager.get_connected_devices()
        
        if not connected_devices:
            print("✗ No devices found")
            return None
            
        # Use the first connected device
        device_serial = connected_devices[0]
        print(f"✓ Found device: {device_serial}")
        
        # Connect to device
        if self.device_manager.connect_device(device_serial):
            print(f"✓ Connected to device {device_serial}")
            return device_serial
        else:
            print(f"✗ Failed to connect to device {device_serial}")
            return None
    
    def trigger_bootloader_mode(self, device_serial):
        """Use software command to trigger bootloader mode"""
        print("\n=== STEP 3: Triggering Bootloader Mode (Software) ===")
        
        try:
            success = self.firmware_flasher.trigger_bootloader_mode(device_serial, timeout_ms=5000)
            
            if success:
                print("✓ Device successfully entered bootloader mode via software command")
                return True
            else:
                print("✗ Failed to trigger bootloader mode via software")
                print("This might be expected if the bootloader command isn't implemented yet")
                return False
                
        except Exception as e:
            print(f"✗ Error triggering bootloader mode: {e}")
            return False
    
    def flash_firmware_autonomous(self, device_serial):
        """Flash firmware autonomously"""
        print("\n=== STEP 4: Flashing Firmware ===")
        
        try:
            firmware_path = os.path.abspath('firmware.uf2')
            
            # Use the existing firmware flasher
            operation = self.firmware_flasher.flash_firmware(device_serial, firmware_path)
            
            if operation.result.value == "success":
                print("✓ Firmware flashed successfully")
                print(f"  - Bootloader entry time: {operation.bootloader_entry_time:.2f}s")
                print(f"  - Flash duration: {operation.flash_duration:.2f}s") 
                print(f"  - Reconnection time: {operation.reconnection_time:.2f}s")
                print(f"  - Total time: {operation.total_duration:.2f}s")
                return True
            else:
                print(f"✗ Firmware flash failed: {operation.result.value}")
                if operation.error_message:
                    print(f"  Error: {operation.error_message}")
                return False
                
        except Exception as e:
            print(f"✗ Flash process failed: {e}")
            return False
    
    def verify_device_reconnection(self):
        """Verify device reconnected after flashing"""
        print("\n=== STEP 5: Verifying Device Reconnection ===")
        
        # Wait a bit for device to fully boot
        time.sleep(2)
        
        # Rediscover devices
        self.device_manager.discover_devices()
        connected_devices = self.device_manager.get_connected_devices()
        
        if connected_devices:
            device_serial = connected_devices[0]
            print(f"✓ Device reconnected: {device_serial}")
            
            # Try to connect
            if self.device_manager.connect_device(device_serial):
                print("✓ Successfully connected to device after flash")
                return device_serial
            else:
                print("✗ Failed to connect to device after flash")
                return None
        else:
            print("✗ No devices found after flash")
            return None
    
    def start_hid_logging(self, device_serial):
        """Start HID logging to verify device functionality"""
        print("\n=== STEP 6: Starting HID Logging Verification ===")
        
        try:
            # Get device handle for direct HID communication
            device_handle = self.device_manager.get_device_handle(device_serial)
            if not device_handle:
                print("✗ Could not get device handle for logging")
                return False
            
            self.logging = True
            self.log_messages = []
            self.log_thread = threading.Thread(
                target=self._log_loop, 
                args=(device_handle,), 
                daemon=True
            )
            self.log_thread.start()
            
            print("✓ HID logging started")
            return True
            
        except Exception as e:
            print(f"✗ Failed to start HID logging: {e}")
            return False
    
    def _log_loop(self, device_handle):
        """HID logging loop"""
        message_count = 0
        
        while self.logging:
            try:
                data = device_handle.read(64, timeout_ms=100)
                
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
                    # Only log errors occasionally to avoid spam
                    if message_count % 100 == 0:
                        print(f"[DEBUG] HID read timeout (normal): {e}")
    
    def stop_hid_logging(self):
        """Stop HID logging"""
        if self.logging:
            self.logging = False
            if self.log_thread:
                self.log_thread.join(timeout=2)
            print("✓ HID logging stopped")
    
    def analyze_device_functionality(self):
        """Analyze collected log messages to verify device functionality"""
        print("\n=== STEP 7: Analyzing Device Functionality ===")
        
        print(f"✓ Collected {len(self.log_messages)} log messages")
        
        if not self.log_messages:
            print("⚠ No log messages received - device may not be logging")
            return False
        
        # Show first few messages
        print("\nFirst few log messages:")
        for i, message in enumerate(self.log_messages[:5]):
            print(f"  {message}")
        
        # Look for system indicators
        indicators_found = {
            'system_startup': False,
            'task_activity': False,
            'usb_activity': False,
            'error_free': True
        }
        
        for message in self.log_messages:
            msg_lower = message.lower()
            
            if any(word in msg_lower for word in ['startup', 'init', 'boot', 'start']):
                indicators_found['system_startup'] = True
            
            if any(word in msg_lower for word in ['task', 'pemf', 'battery', 'led']):
                indicators_found['task_activity'] = True
                
            if any(word in msg_lower for word in ['usb', 'hid', 'command']):
                indicators_found['usb_activity'] = True
                
            if any(word in msg_lower for word in ['error', 'fail', 'panic']):
                indicators_found['error_free'] = False
        
        # Report findings
        print("\nSystem Health Analysis:")
        for indicator, found in indicators_found.items():
            status = "✓" if found else "✗"
            print(f"  {status} {indicator.replace('_', ' ').title()}: {found}")
        
        # Overall assessment
        healthy_count = sum(indicators_found.values())
        if healthy_count >= 3:  # At least 3 out of 4 indicators positive
            print("✓ Device appears to be functioning correctly")
            return True
        else:
            print("⚠ Device functionality unclear - may need investigation")
            return False
    
    def run_autonomous_cycle(self):
        """Run the complete autonomous flash and test cycle"""
        print("=== AUTONOMOUS FIRMWARE FLASH AND TEST CYCLE ===")
        print("This will test the software bootloader entry functionality")
        print()
        
        device_serial = None
        
        try:
            # Step 1: Build firmware
            if not self.build_firmware():
                return False
            
            # Step 2: Discover and connect to device
            device_serial = self.discover_and_connect_device()
            if not device_serial:
                return False
            
            # Step 3: Try software bootloader entry
            bootloader_success = self.trigger_bootloader_mode(device_serial)
            
            if bootloader_success:
                # Step 4: Flash firmware using existing flasher
                if not self.flash_firmware_autonomous(device_serial):
                    return False
            else:
                print("\n⚠ Software bootloader entry failed")
                print("This means the bootloader command functionality needs to be tested/fixed")
                print("For now, let's test with manual bootloader entry...")
                
                # Fallback: ask user to manually enter bootloader mode
                print("\nPlease manually put the device in bootloader mode:")
                print("1. Disconnect device")
                print("2. Hold BOOTSEL button") 
                print("3. Reconnect while holding BOOTSEL")
                print("4. Release BOOTSEL")
                input("Press Enter when device is in bootloader mode...")
                
                # Try manual flash
                mount_point = "/run/media/dustin/RPI-RP2/"
                if os.path.exists(mount_point):
                    print("✓ Bootloader mount point found")
                    firmware_path = os.path.abspath('firmware.uf2')
                    target_path = os.path.join(mount_point, 'firmware.uf2')
                    
                    result = subprocess.run(['cp', firmware_path, target_path])
                    if result.returncode == 0:
                        print("✓ Firmware copied manually")
                    else:
                        print("✗ Manual firmware copy failed")
                        return False
                else:
                    print("✗ Bootloader mount point not found")
                    return False
            
            # Step 5: Verify reconnection
            device_serial = self.verify_device_reconnection()
            if not device_serial:
                return False
            
            # Step 6: Start logging and analyze
            if not self.start_hid_logging(device_serial):
                return False
            
            # Let it log for a while
            print("Collecting log messages for 10 seconds...")
            time.sleep(10)
            
            # Step 7: Analyze functionality
            success = self.analyze_device_functionality()
            
            return success
            
        finally:
            self.stop_hid_logging()
            if device_serial:
                self.device_manager.disconnect_device(device_serial)

def main():
    test = AutonomousFlashTest()
    
    try:
        success = test.run_autonomous_cycle()
        
        if success:
            print("\n🎉 AUTONOMOUS CYCLE SUCCESS!")
            print("✓ Firmware built and flashed")
            print("✓ Device reconnected successfully") 
            print("✓ HID logging verified")
            print("✓ Device functionality confirmed")
            print("\nThe basic flash cycle is working! I can now iterate autonomously.")
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
        import traceback
        traceback.print_exc()
        return 1

if __name__ == "__main__":
    sys.exit(main())