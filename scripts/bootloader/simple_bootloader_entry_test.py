#!/usr/bin/env python3
"""
Simple bootloader entry test with direct reset approach

This bypasses the complex task shutdown sequence and tries a more direct approach
to entering bootloader mode. If the firmware supports it, this should work.
"""

import hid
import time
import struct
import subprocess
import os

def send_simple_bootloader_command():
    """Send a simple bootloader command and monitor for device reset"""
    try:
        device = hid.Device(0x1234, 0x5678)
        print("✓ Connected to device")
        
        # Create bootloader command with shorter timeout
        command = bytearray(64)
        command[0] = 0x80  # ENTER_BOOTLOADER command type
        command[1] = 0x01  # Command ID
        
        # Use a very short timeout (500ms) to force quick action
        timeout_ms = 500
        payload = struct.pack('<I', timeout_ms)
        
        command[2] = len(payload)
        
        # Calculate checksum
        checksum = command[0] ^ command[1] ^ command[2]
        for byte in payload:
            checksum ^= byte
        command[3] = checksum
        
        # Add payload
        command[4:4+len(payload)] = payload
        
        print(f"Sending bootloader command with {timeout_ms}ms timeout...")
        
        # Send command
        bytes_sent = device.write(bytes(command))
        print(f"✓ Sent {bytes_sent} bytes")
        
        # Close device immediately to avoid interfering with reset
        device.close()
        print("✓ Closed device connection")
        
        return True
        
    except Exception as e:
        print(f"✗ Failed to send command: {e}")
        return False

def monitor_device_state_changes():
    """Monitor for device state changes over time"""
    print("\nMonitoring device state changes...")
    
    states = []
    
    for i in range(20):  # Monitor for 20 seconds
        state = {
            'time': i,
            'hid_device_present': False,
            'bootloader_present': False,
            'mount_point_present': False
        }
        
        # Check for HID device
        try:
            devices = hid.enumerate(0x1234, 0x5678)
            state['hid_device_present'] = len(devices) > 0
        except:
            pass
        
        # Check for bootloader via lsusb
        try:
            result = subprocess.run(['lsusb'], capture_output=True, text=True)
            state['bootloader_present'] = "2e8a:0003" in result.stdout
        except:
            pass
        
        # Check for mount point
        mount_points = ["/run/media/dustin/RPI-RP2/", "/media/RPI-RP2/", "/mnt/RPI-RP2/"]
        for mount_point in mount_points:
            if os.path.exists(mount_point) and os.path.ismount(mount_point):
                state['mount_point_present'] = True
                break
        
        states.append(state)
        
        # Print current state
        hid_status = "HID" if state['hid_device_present'] else "---"
        boot_status = "BOOT" if state['bootloader_present'] else "----"
        mount_status = "MOUNT" if state['mount_point_present'] else "-----"
        
        print(f"  {i:2d}s: {hid_status} | {boot_status} | {mount_status}")
        
        time.sleep(1)
    
    return states

def analyze_state_changes(states):
    """Analyze the device state changes to determine what happened"""
    print("\n=== STATE CHANGE ANALYSIS ===")
    
    # Look for transitions
    hid_disappeared = False
    bootloader_appeared = False
    mount_appeared = False
    hid_returned = False
    
    for i in range(1, len(states)):
        prev = states[i-1]
        curr = states[i]
        
        if prev['hid_device_present'] and not curr['hid_device_present']:
            hid_disappeared = True
            print(f"✓ HID device disappeared at {i}s (device reset?)")
        
        if not prev['bootloader_present'] and curr['bootloader_present']:
            bootloader_appeared = True
            print(f"✓ Bootloader appeared at {i}s")
        
        if not prev['mount_point_present'] and curr['mount_point_present']:
            mount_appeared = True
            print(f"✓ Mount point appeared at {i}s")
        
        if not prev['hid_device_present'] and curr['hid_device_present']:
            hid_returned = True
            print(f"✓ HID device returned at {i}s")
    
    # Determine outcome
    if bootloader_appeared and mount_appeared:
        print("\n🎉 SUCCESS: Device entered bootloader mode!")
        print("✓ Bootloader command worked")
        print("✓ Device is ready for firmware flashing")
        return True
    elif hid_disappeared and hid_returned:
        print("\n⚠ PARTIAL SUCCESS: Device reset but returned to normal mode")
        print("✓ Bootloader command caused device reset")
        print("✗ Device did not enter bootloader mode")
        print("This suggests the bootloader entry logic is working but not completing")
        return False
    elif hid_disappeared:
        print("\n⚠ DEVICE RESET: Device disappeared but didn't return")
        print("✓ Bootloader command caused device reset")
        print("? Device may be in bootloader mode but not detected")
        return False
    else:
        print("\n❌ NO CHANGE: Device state didn't change")
        print("✗ Bootloader command had no visible effect")
        print("This suggests the command is not being processed correctly")
        return False

def test_firmware_flash_if_bootloader():
    """If device is in bootloader mode, test firmware flashing"""
    print("\n=== TESTING FIRMWARE FLASH ===")
    
    # Check if we have a firmware file
    if not os.path.exists('../../artifacts/firmware/firmware.uf2'):
        print("✗ No artifacts/firmware/firmware.uf2 file found")
        print("Run 'cargo build --release && elf2uf2-rs target/thumbv6m-none-eabi/release/ass-easy-loop artifacts/firmware/firmware.uf2' first")
        return False
    
    # Find mount point
    mount_points = ["/run/media/dustin/RPI-RP2/", "/media/RPI-RP2/", "/mnt/RPI-RP2/"]
    mount_point = None
    
    for mp in mount_points:
        if os.path.exists(mp) and os.path.ismount(mp):
            mount_point = mp
            break
    
    if not mount_point:
        print("✗ No bootloader mount point found")
        return False
    
    print(f"✓ Found bootloader mount point: {mount_point}")
    
    # Copy firmware
    try:
        firmware_path = os.path.abspath('../../artifacts/firmware/firmware.uf2')
        target_path = os.path.join(mount_point, 'firmware.uf2')
        
        print(f"Copying firmware to bootloader...")
        result = subprocess.run(['cp', firmware_path, target_path], capture_output=True, text=True)
        
        if result.returncode == 0:
            print("✓ Firmware copied successfully")
            
            # Wait for device to reboot
            print("Waiting for device to reboot...")
            time.sleep(5)
            
            # Check if device returned
            try:
                devices = hid.enumerate(0x1234, 0x5678)
                if devices:
                    print("✓ Device returned to normal mode after flashing")
                    return True
                else:
                    print("⚠ Device not found after flashing")
                    return False
            except:
                print("⚠ Could not check device status after flashing")
                return False
        else:
            print(f"✗ Firmware copy failed: {result.stderr}")
            return False
            
    except Exception as e:
        print(f"✗ Firmware flash failed: {e}")
        return False

def main():
    print("=== SIMPLE BOOTLOADER ENTRY TEST ===")
    print("This test uses a direct approach with short timeout")
    print()
    
    # Step 1: Send bootloader command
    if not send_simple_bootloader_command():
        return 1
    
    # Step 2: Monitor device state changes
    states = monitor_device_state_changes()
    
    # Step 3: Analyze what happened
    bootloader_success = analyze_state_changes(states)
    
    # Step 4: If in bootloader mode, test flashing
    if bootloader_success:
        flash_success = test_firmware_flash_if_bootloader()
        if flash_success:
            print("\n🎉 COMPLETE SUCCESS!")
            print("✓ Bootloader command worked")
            print("✓ Device entered bootloader mode")
            print("✓ Firmware flashing worked")
            print("✓ Device returned to normal mode")
            print("\nThe bootloader entry functionality is now working!")
            return 0
        else:
            print("\n⚠ PARTIAL SUCCESS")
            print("✓ Bootloader command worked")
            print("✗ Firmware flashing had issues")
            return 1
    else:
        print("\n❌ BOOTLOADER ENTRY FAILED")
        print("The bootloader command is not working properly")
        print("Check the firmware bootloader entry implementation")
        return 1

if __name__ == "__main__":
    exit(main())