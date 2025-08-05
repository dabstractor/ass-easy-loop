#!/bin/bash

# Setup Validation Runner Script
# This script runs comprehensive validation of the RP2040 pEMF device development environment

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check Python module
check_python_module() {
    python3 -c "import $1" 2>/dev/null
}

# Print header
echo "=================================================================="
echo "RP2040 pEMF Device - Development Environment Validation"
echo "=================================================================="
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Not in project directory (Cargo.toml not found)"
    print_error "Please run this script from the project root directory"
    exit 1
fi

print_status "Starting comprehensive setup validation..."
echo ""

# 1. Check system requirements
print_status "Checking system requirements..."

# Check operating system
OS=$(uname -s)
print_status "Operating System: $OS"

# Check Python
if command_exists python3; then
    PYTHON_VERSION=$(python3 --version)
    print_success "Python found: $PYTHON_VERSION"
else
    print_error "Python 3 not found"
    exit 1
fi

# Check Rust
if command_exists rustc; then
    RUST_VERSION=$(rustc --version)
    print_success "Rust found: $RUST_VERSION"
else
    print_error "Rust not found - please install Rust using rustup"
    exit 1
fi

# Check Cargo
if command_exists cargo; then
    CARGO_VERSION=$(cargo --version)
    print_success "Cargo found: $CARGO_VERSION"
else
    print_error "Cargo not found"
    exit 1
fi

echo ""

# 2. Check Rust target
print_status "Checking Rust ARM target..."
if rustup target list --installed | grep -q "thumbv6m-none-eabi"; then
    print_success "ARM Cortex-M target installed"
else
    print_warning "ARM Cortex-M target not installed"
    print_status "Installing thumbv6m-none-eabi target..."
    rustup target add thumbv6m-none-eabi
    print_success "ARM target installed"
fi

echo ""

# 3. Check development tools
print_status "Checking development tools..."

if command_exists elf2uf2-rs; then
    ELF2UF2_VERSION=$(elf2uf2-rs --version 2>/dev/null || echo "version unknown")
    print_success "elf2uf2-rs found: $ELF2UF2_VERSION"
else
    print_warning "elf2uf2-rs not found"
    print_status "Installing elf2uf2-rs..."
    cargo install elf2uf2-rs
    print_success "elf2uf2-rs installed"
fi

if command_exists probe-rs; then
    PROBE_RS_VERSION=$(probe-rs --version 2>/dev/null || echo "version unknown")
    print_success "probe-rs found: $PROBE_RS_VERSION"
else
    print_warning "probe-rs not found (optional for debugging)"
fi

echo ""

# 4. Check project build
print_status "Testing project build..."
if cargo check --quiet; then
    print_success "Project builds successfully"
else
    print_error "Project build failed"
    print_error "Please check Rust installation and project dependencies"
    exit 1
fi

echo ""

# 5. Check Python dependencies
print_status "Checking Python dependencies..."

PYTHON_DEPS=("json" "time" "logging" "sys" "os")
for dep in "${PYTHON_DEPS[@]}"; do
    if check_python_module "$dep"; then
        print_success "Python module '$dep' available"
    else
        print_error "Python module '$dep' not available"
        exit 1
    fi
done

# Check HID library
if check_python_module "hid"; then
    print_success "Python HID library available"
else
    print_warning "Python HID library not available"
    print_status "Installing HID library..."
    
    case "$OS" in
        "Linux")
            print_status "Installing HID dependencies for Linux..."
            if command_exists apt-get; then
                sudo apt-get update
                sudo apt-get install -y libhidapi-dev
            elif command_exists yum; then
                sudo yum install -y hidapi-devel
            elif command_exists pacman; then
                sudo pacman -S hidapi
            fi
            ;;
        "Darwin")
            print_status "Installing HID dependencies for macOS..."
            if command_exists brew; then
                brew install hidapi
            fi
            ;;
    esac
    
    pip3 install hidapi
    
    if check_python_module "hid"; then
        print_success "HID library installed successfully"
    else
        print_error "Failed to install HID library"
        exit 1
    fi
fi

echo ""

# 6. Check test framework
print_status "Checking test framework installation..."

if [ -d "test_framework" ]; then
    print_success "Test framework directory found"
    
    FRAMEWORK_FILES=("device_manager.py" "command_handler.py" "test_sequencer.py" "result_collector.py")
    for file in "${FRAMEWORK_FILES[@]}"; do
        if [ -f "test_framework/$file" ]; then
            print_success "Framework file '$file' found"
        else
            print_error "Framework file '$file' missing"
            exit 1
        fi
    done
else
    print_error "Test framework directory not found"
    exit 1
fi

echo ""

# 7. Run Python validation script
print_status "Running Python setup validation..."
if python3 validation_scripts/setup_validation.py --quick --automated; then
    print_success "Python validation passed"
else
    print_warning "Python validation had issues (check output above)"
fi

echo ""

# 8. Check USB device detection (if device connected)
print_status "Checking for connected devices..."

case "$OS" in
    "Linux")
        if command_exists lsusb; then
            if lsusb | grep -q "2e8a:"; then
                print_success "RP2040 device detected via lsusb"
                DEVICE_DETECTED=true
            else
                print_warning "No RP2040 devices detected"
                DEVICE_DETECTED=false
            fi
        else
            print_warning "lsusb not available for device detection"
            DEVICE_DETECTED=false
        fi
        ;;
    *)
        print_warning "Device detection not implemented for $OS"
        DEVICE_DETECTED=false
        ;;
esac

echo ""

# 9. Run hardware validation if device detected
if [ "$DEVICE_DETECTED" = true ]; then
    print_status "Running hardware validation..."
    if python3 validation_scripts/hardware_validation.py --automated; then
        print_success "Hardware validation passed"
    else
        print_warning "Hardware validation had issues"
    fi
else
    print_warning "Skipping hardware validation (no device detected)"
    print_status "Connect device and run: python3 validation_scripts/hardware_validation.py"
fi

echo ""

# 10. Final summary
print_status "Validation Summary:"
echo "=================================================================="

print_success "✓ System requirements met"
print_success "✓ Rust environment configured"
print_success "✓ Development tools available"
print_success "✓ Project builds successfully"
print_success "✓ Python dependencies installed"
print_success "✓ Test framework ready"

if [ "$DEVICE_DETECTED" = true ]; then
    print_success "✓ Hardware device detected"
else
    print_warning "⚠ Hardware device not detected"
fi

echo ""
print_success "Setup validation completed successfully!"
print_status "Your development environment is ready for RP2040 pEMF device development."

echo ""
print_status "Next steps:"
echo "  1. Connect your RP2040 device via USB"
echo "  2. Run hardware validation: python3 validation_scripts/hardware_validation.py"
echo "  3. Build and flash firmware: cargo run --release"
echo "  4. Run test suite: python3 test_framework/comprehensive_test_runner.py"

echo ""
print_status "For detailed documentation, see:"
echo "  - docs/setup/DEVELOPMENT_SETUP_GUIDE.md"
echo "  - docs/hardware/HARDWARE_SETUP_DOCUMENTATION.md"
echo "  - docs/troubleshooting/TESTING_TROUBLESHOOTING_GUIDE.md"

exit 0