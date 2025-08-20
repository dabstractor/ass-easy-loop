#!/bin/bash

# Lightweight flash script for RP2040 in bootloader mode
# This script is called by cargo run as the custom runner

# The first argument is the path to the ELF file
ELF_FILE="$1"

# Extract the binary name from the ELF file path
BINARY_NAME=$(basename "$ELF_FILE")
BINARY_NAME=${BINARY_NAME%.*}

echo "📦 Flashing $BINARY_NAME to RP2040..."

# Check if RP2040 is in bootloader mode (mounted drive)
MOUNT_POINT=""
if [ -d "/run/media/dustin/RPI-RP2" ]; then
    MOUNT_POINT="/run/media/dustin/RPI-RP2"
elif [ -d "/media/dustin/RPI-RP2" ]; then
    MOUNT_POINT="/media/dustin/RPI-RP2"
else
    echo -e "\033[31m❌ ERROR: RP2040 not found in bootloader mode\033[0m"
    echo "💡 Please put your RP2040 in bootloader mode:"
    echo "   1. Hold the BOOTSEL button"
    echo "   2. Plug in the RP2040 or press the reset button"
    echo "   3. Release BOOTSEL when the drive appears"
    echo ""
    echo "Then run 'cargo run' again."
    exit 1
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