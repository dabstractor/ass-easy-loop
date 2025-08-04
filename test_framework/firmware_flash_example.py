#!/usr/bin/env python3
"""
Firmware Flashing Example

Demonstrates automated firmware flashing workflow including:
- Device discovery and connection
- Bootloader mode triggering
- Firmware flashing using external tools
- Device reconnection detection
- Multi-device parallel flashing

This example shows how to integrate firmware flashing into automated testing workflows.
"""

import sys
import time
import logging
import argparse
from pathlib import Path

from test_framework import (
    UsbHidDeviceManager, CommandHandler, FirmwareFlasher,
    FlashResult
)


def setup_logging(verbose: bool = False):
    """Set up logging configuration"""
    level = logging.DEBUG if verbose else logging.INFO
    logging.basicConfig(
        level=level,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
        handlers=[
            logging.StreamHandler(sys.stdout),
            logging.FileHandler('firmware_flash.log')
        ]
    )


def discover_and_connect_devices(device_manager: UsbHidDeviceManager) -> list:
    """Discover and connect to available test devices"""
    print("Discovering devices...")
    devices = device_manager.discover_devices()
    
    if not devices:
        print("No devices found!")
        return []
    
    print(f"Found {len(devices)} devices:")
    connected_devices = []
    
    for device in devices:
        print(f"  - {device.serial_number}: {device.product} ({device.status.value})")
        
        if device.status.value == "connected":
            if device_manager.connect_device(device.serial_number):
                connected_devices.append(device.serial_number)
                print(f"    Connected successfully")
            else:
                print(f"    Failed to connect")
    
    return connected_devices


def flash_single_device(flasher: FirmwareFlasher, device_serial: str, firmware_path: str):
    """Flash firmware to a single device with detailed progress reporting"""
    print(f"\n=== Flashing firmware to {device_serial} ===")
    print(f"Firmware: {firmware_path}")
    
    # Verify firmware file exists
    if not Path(firmware_path).exists():
        print(f"ERROR: Firmware file not found: {firmware_path}")
        return False
    
    # Start flash operation
    start_time = time.time()
    operation = flasher.flash_firmware(device_serial, firmware_path)
    
    # Report results
    print(f"\nFlash operation completed in {operation.total_duration:.2f}s")
    print(f"Result: {operation.result.value}")
    
    if operation.result == FlashResult.SUCCESS:
        print("✓ Firmware flashing successful!")
        print(f"  - Bootloader entry: {operation.bootloader_entry_time:.2f}s")
        print(f"  - Flash duration: {operation.flash_duration:.2f}s")
        print(f"  - Reconnection time: {operation.reconnection_time:.2f}s")
        return True
    else:
        print(f"✗ Firmware flashing failed: {operation.error_message}")
        return False


def flash_multiple_devices(flasher: FirmwareFlasher, device_serials: list, 
                          firmware_path: str, parallel: bool = True):
    """Flash firmware to multiple devices"""
    print(f"\n=== Flashing firmware to {len(device_serials)} devices ===")
    print(f"Firmware: {firmware_path}")
    print(f"Mode: {'Parallel' if parallel else 'Sequential'}")
    
    # Create device-firmware mapping
    device_firmware_map = {serial: firmware_path for serial in device_serials}
    
    # Execute flash operations
    start_time = time.time()
    results = flasher.flash_multiple_devices(
        device_firmware_map, 
        parallel=parallel, 
        max_parallel=4
    )
    total_time = time.time() - start_time
    
    # Report results
    print(f"\nMulti-device flash completed in {total_time:.2f}s")
    
    success_count = 0
    for device_serial, operation in results.items():
        status_icon = "✓" if operation.result == FlashResult.SUCCESS else "✗"
        print(f"  {status_icon} {device_serial}: {operation.result.value}")
        
        if operation.result == FlashResult.SUCCESS:
            success_count += 1
        else:
            print(f"    Error: {operation.error_message}")
    
    print(f"\nSummary: {success_count}/{len(device_serials)} devices flashed successfully")
    return success_count == len(device_serials)


def test_bootloader_triggering(flasher: FirmwareFlasher, device_serial: str):
    """Test bootloader mode triggering without flashing"""
    print(f"\n=== Testing bootloader mode triggering for {device_serial} ===")
    
    # Trigger bootloader mode
    print("Triggering bootloader mode...")
    success = flasher.trigger_bootloader_mode(device_serial, timeout_ms=5000)
    
    if success:
        print("✓ Device successfully entered bootloader mode")
        
        # Wait a moment then check if device is back
        print("Waiting for device to return to normal mode...")
        time.sleep(10)  # RP2040 bootloader timeout
        
        # Re-discover devices
        flasher.device_manager.discover_devices()
        device_info = flasher.device_manager.get_device_info(device_serial)
        
        if device_info and device_info.status.value == "connected":
            print("✓ Device returned to normal operation mode")
            return True
        else:
            print("⚠ Device did not return to normal mode (may need manual reset)")
            return False
    else:
        print("✗ Failed to trigger bootloader mode")
        return False


def verify_setup(flasher: FirmwareFlasher):
    """Verify that the flashing setup is correct"""
    print("=== Verifying firmware flashing setup ===")
    
    # Check flash tool availability
    if flasher.verify_flash_tool_availability():
        print(f"✓ Flash tool available: {flasher.flash_tool_path}")
    else:
        print("✗ Flash tool not available or not working")
        return False
    
    # Show supported formats
    formats = flasher.get_supported_firmware_formats()
    print(f"✓ Supported firmware formats: {', '.join(formats)}")
    
    return True


def main():
    """Main firmware flashing example"""
    parser = argparse.ArgumentParser(description="Firmware Flashing Example")
    parser.add_argument('--firmware', '-f', type=str, 
                       help='Path to firmware file to flash')
    parser.add_argument('--device', '-d', type=str,
                       help='Specific device serial number to flash')
    parser.add_argument('--parallel', '-p', action='store_true',
                       help='Flash multiple devices in parallel')
    parser.add_argument('--test-bootloader', '-t', action='store_true',
                       help='Test bootloader triggering without flashing')
    parser.add_argument('--verbose', '-v', action='store_true',
                       help='Enable verbose logging')
    parser.add_argument('--flash-tool', type=str,
                       help='Path to firmware flashing tool')
    
    args = parser.parse_args()
    
    # Set up logging
    setup_logging(args.verbose)
    
    # Initialize framework components
    print("Initializing test framework...")
    device_manager = UsbHidDeviceManager()
    command_handler = CommandHandler(device_manager)
    flasher = FirmwareFlasher(
        device_manager=device_manager,
        command_handler=command_handler,
        flash_tool_path=args.flash_tool
    )
    
    try:
        # Verify setup
        if not verify_setup(flasher):
            print("Setup verification failed. Please check your configuration.")
            return 1
        
        # Discover and connect to devices
        connected_devices = discover_and_connect_devices(device_manager)
        
        if not connected_devices:
            print("No devices available for testing.")
            return 1
        
        # Filter to specific device if requested
        if args.device:
            if args.device in connected_devices:
                connected_devices = [args.device]
            else:
                print(f"Requested device {args.device} not found in connected devices.")
                return 1
        
        # Test bootloader triggering only
        if args.test_bootloader:
            success = True
            for device_serial in connected_devices:
                if not test_bootloader_triggering(flasher, device_serial):
                    success = False
            return 0 if success else 1
        
        # Firmware flashing
        if not args.firmware:
            print("No firmware file specified. Use --firmware to specify firmware to flash.")
            print("Use --test-bootloader to test bootloader triggering without flashing.")
            return 1
        
        # Flash firmware
        if len(connected_devices) == 1:
            # Single device flashing
            success = flash_single_device(flasher, connected_devices[0], args.firmware)
        else:
            # Multi-device flashing
            success = flash_multiple_devices(
                flasher, connected_devices, args.firmware, args.parallel
            )
        
        return 0 if success else 1
        
    except KeyboardInterrupt:
        print("\nOperation cancelled by user.")
        return 1
    except Exception as e:
        print(f"Unexpected error: {e}")
        logging.exception("Unexpected error occurred")
        return 1
    finally:
        # Clean up connections
        device_manager.disconnect_all()
        print("Disconnected from all devices.")


if __name__ == '__main__':
    sys.exit(main())