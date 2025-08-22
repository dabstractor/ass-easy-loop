#!/bin/bash

# Lightweight flash script for RP2040 in bootloader mode
# This script is called by cargo run as the custom runner

# The first argument is the path to the ELF file
ELF_FILE="$1"

# Extract the binary name from the ELF file path
BINARY_NAME=$(basename "$ELF_FILE")
BINARY_NAME=${BINARY_NAME%.*}

echo "📦 Flashing $BINARY_NAME to RP2040..."

# Function to check if device is in bootloader mode
check_bootloader_mode() {
    if [ -d "/run/media/dustin/RPI-RP2" ]; then
        echo "/run/media/dustin/RPI-RP2"
        return 0
    elif [ -d "/media/dustin/RPI-RP2" ]; then
        echo "/media/dustin/RPI-RP2"
        return 0
    else
        return 1
    fi
}

# Function to check if device is running our firmware
check_device_running() {
    # Check if our device (VID:PID = fade:1212) is connected and responding
    if command -v python3 &> /dev/null; then
        python3 -c "
import sys
try:
    import hid
    devices = hid.enumerate(0xfade, 0x1212)
    if devices:
        print('Device found running firmware')
        sys.exit(0)
    else:
        sys.exit(1)
except ImportError:
    sys.exit(1)  # hidapi not available
except Exception:
    sys.exit(1)  # Device not found or other error
" 2>/dev/null
        return $?
    else
        return 1  # Python not available
    fi
}

# Function to trigger bootloader mode
trigger_bootloader_mode() {
    echo "🔄 Attempting to trigger bootloader mode automatically..."
    
    SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
    BOOTLOADER_SCRIPT="$SCRIPT_DIR/host_tools/bootloader_entry.py"
    
    if [ ! -f "$BOOTLOADER_SCRIPT" ]; then
        echo "❌ ERROR: bootloader_entry.py not found at $BOOTLOADER_SCRIPT"
        return 1
    fi
    
    # Try to trigger bootloader mode using our automation tool
    echo "📤 Sending bootloader entry command..."
    python3 "$BOOTLOADER_SCRIPT" >/dev/null 2>&1
    
    # Always wait for bootloader mode regardless of script exit code
    # The script may fail to detect the device but still succeed in triggering bootloader mode
    echo "⏳ Waiting for device to enter bootloader mode..."
    
    # Wait up to 15 seconds for bootloader mode (increased from 10)
    for i in {1..15}; do
        sleep 1
        MOUNT_POINT=$(check_bootloader_mode)
        if [ $? -eq 0 ]; then
            echo "✅ Device successfully entered bootloader mode"
            return 0
        fi
        
        # Also check via lsusb for bootloader VID:PID
        if lsusb | grep -q "2e8a:0003"; then
            echo "✅ Device detected in bootloader mode via USB"
            # Wait a bit more for mount to appear
            sleep 2
            MOUNT_POINT=$(check_bootloader_mode)
            if [ $? -eq 0 ]; then
                echo "✅ Bootloader drive mounted"
                return 0
            else
                echo "⚠️  Device in bootloader mode but drive not mounted yet, waiting..."
                sleep 3
                MOUNT_POINT=$(check_bootloader_mode)
                if [ $? -eq 0 ]; then
                    echo "✅ Bootloader drive now mounted"
                    return 0
                fi
            fi
        fi
    done
    
    echo "❌ Device did not enter bootloader mode after command"
    return 1
}

# Check if RP2040 is in bootloader mode (mounted drive)
MOUNT_POINT=$(check_bootloader_mode)
if [ $? -ne 0 ]; then
    echo "📱 RP2040 not in bootloader mode, checking if device is running..."
    
    # Check if our device is running and can be triggered
    if check_device_running; then
        echo "✅ Found device running firmware, attempting automatic bootloader entry..."
        
        if trigger_bootloader_mode; then
            MOUNT_POINT=$(check_bootloader_mode)
        else
            echo -e "\033[31m❌ ERROR: Could not automatically enter bootloader mode\033[0m"
            echo "💡 Please manually put your RP2040 in bootloader mode:"
            echo "   1. Hold the BOOTSEL button"
            echo "   2. Plug in the RP2040 or press the reset button"
            echo "   3. Release BOOTSEL when the drive appears"
            echo ""
            echo "Then run 'cargo run' again."
            exit 1
        fi
    else
        echo -e "\033[31m❌ ERROR: RP2040 not found in bootloader mode or running firmware\033[0m"
        echo "💡 Please put your RP2040 in bootloader mode:"
        echo "   1. Hold the BOOTSEL button"
        echo "   2. Plug in the RP2040 or press the reset button"
        echo "   3. Release BOOTSEL when the drive appears"
        echo ""
        echo "Alternatively, ensure the device is running firmware with bootloader entry support."
        echo "Then run 'cargo run' again."
        exit 1
    fi
fi

echo "✅ RP2040 found at $MOUNT_POINT"

# Try to create UF2 file with picotool
echo "🔄 Converting ELF to UF2..."
UF2_FILE="/tmp/${BINARY_NAME}.uf2"

if [ -f "./picotool" ]; then
    ./picotool uf2 convert -t elf "$ELF_FILE" "$UF2_FILE" 2>/dev/null
    RESULT=$?
    if [ $RESULT -eq 0 ]; then
        echo "✅ UF2 created successfully with picotool"
    else
        echo "❌ ERROR: picotool failed to convert ELF to UF2"
        echo "💡 This is likely due to entry point issues in the ELF file"
        echo "💡 The current RTIC setup has problems that need to be fixed"
        exit 1
    fi
elif command -v elf2uf2-rs &> /dev/null; then
    elf2uf2-rs "$ELF_FILE" "$UF2_FILE" 2>/dev/null
    RESULT=$?
    if [ $RESULT -eq 0 ]; then
        echo "✅ UF2 created successfully with elf2uf2-rs"
    else
        echo "❌ ERROR: elf2uf2-rs failed to convert ELF to UF2"
        echo "💡 This is likely due to entry point issues in the ELF file"
        exit 1
    fi
else
    echo "❌ ERROR: No UF2 converter found"
    echo "💡 Please install elf2uf2-rs: cargo install elf2uf2-rs"
    exit 1
fi

# Verify the UF2 file was created and has content
if [ ! -f "$UF2_FILE" ] || [ ! -s "$UF2_FILE" ]; then
    echo "❌ ERROR: UF2 file was not created or is empty"
    exit 1
fi

# Copy UF2 to device
echo "💾 Copying to RP2040..."
cp "$UF2_FILE" "$MOUNT_POINT/"
COPY_RESULT=$?

# Clean up the temporary file
rm -f "$UF2_FILE" 2>/dev/null

if [ $COPY_RESULT -eq 0 ]; then
    echo "✅ Successfully copied to RP2040!"
    echo "🚀 Device should reset and run the application."
    echo ""
    echo "Note: If the device doesn't work as expected, check that:"
    echo "- Your RTIC app is properly configured"
    echo "- Entry point is set correctly"
    echo "- Memory layout is valid"
else
    echo "❌ ERROR: Failed to copy UF2 file to device!"
    exit 1
fi

echo -e "\033[32m✨ Flashing complete!\033[0m"
exit 0