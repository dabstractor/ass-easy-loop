# Documentation Index

This document provides a comprehensive index of all documentation files for the RP2040 pEMF/Battery monitoring device with USB HID logging functionality.

## Quick Start Guide

For new users, follow this recommended reading order:

1. **[README.md](README.md)** - Project overview and basic information
2. **[docs/setup/ARCH_LINUX_SETUP.md](docs/setup/ARCH_LINUX_SETUP.md)** - Development environment setup
3. **[docs/setup/SOFTWARE_SETUP.md](docs/setup/SOFTWARE_SETUP.md)** - Rust toolchain and build setup
4. **[docs/BOOTLOADER_FLASHING_GUIDE.md](docs/BOOTLOADER_FLASHING_GUIDE.md)** - Firmware flashing process
5. **[docs/setup/USB_HID_LOGGING_SETUP_GUIDE.md](docs/setup/USB_HID_LOGGING_SETUP_GUIDE.md)** - Complete USB HID setup
6. **[docs/api/USB_HID_USAGE_EXAMPLES.md](docs/api/USB_HID_USAGE_EXAMPLES.md)** - Usage examples and monitoring

## Complete Documentation Library

### Core Project Documentation

| Document | Description | Target Audience |
|----------|-------------|-----------------|
| **[README.md](README.md)** | Main project overview, specifications, and safety information | All users |
| **[WIRING_GUIDE.md](docs/hardware/WIRING_GUIDE.md)** | Hardware assembly and wiring instructions | Hardware builders |
| **[SOFTWARE_SETUP.md](docs/setup/SOFTWARE_SETUP.md)** | Rust development environment and build setup | Developers |

### USB HID Logging Documentation

| Document | Description | Target Audience |
|----------|-------------|-----------------|
| **[USB_HID_LOGGING_SETUP_GUIDE.md](docs/setup/USB_HID_LOGGING_SETUP_GUIDE.md)** | Comprehensive setup guide for USB HID logging system | All users |
| **[USB_HID_USAGE_EXAMPLES.md](docs/api/USB_HID_USAGE_EXAMPLES.md)** | Detailed usage examples and monitoring scenarios | Users, developers |
| **[USB_HID_TROUBLESHOOTING_GUIDE.md](docs/troubleshooting/USB_HID_TROUBLESHOOTING_GUIDE.md)** | Troubleshooting common USB HID issues | All users |
| **[HIDLOG_USAGE.md](docs/api/HIDLOG_USAGE.md)** | Basic hidlog.py utility usage | Users |

### Setup and Configuration Guides

| Document | Description | Target Audience |
|----------|-------------|-----------------|
| **[ARCH_LINUX_SETUP.md](docs/setup/ARCH_LINUX_SETUP.md)** | Complete Arch Linux development environment setup | Arch Linux users |
| **[docs/BOOTLOADER_FLASHING_GUIDE.md](docs/BOOTLOADER_FLASHING_GUIDE.md)** | Detailed firmware flashing instructions and troubleshooting | All users |

### Implementation and Technical Documentation

| Document | Description | Target Audience |
|----------|-------------|-----------------|
| **[.kiro/specs/usb-hid-logging/requirements.md](.kiro/specs/usb-hid-logging/requirements.md)** | Detailed requirements specification | Developers |
| **[.kiro/specs/usb-hid-logging/design.md](.kiro/specs/usb-hid-logging/design.md)** | System architecture and design document | Developers |
| **[.kiro/specs/usb-hid-logging/tasks.md](.kiro/specs/usb-hid-logging/tasks.md)** | Implementation task list and progress | Developers |

### Implementation Status Documents

| Document | Description | Target Audience |
|----------|-------------|-----------------|
| **[BATTERY_LOGGING_IMPLEMENTATION.md](docs/development/BATTERY_LOGGING_IMPLEMENTATION.md)** | Battery monitoring logging implementation | Developers |
| **[PANIC_HANDLER_IMPLEMENTATION.md](docs/development/PANIC_HANDLER_IMPLEMENTATION.md)** | Panic handler with USB logging | Developers |
| **[HARDWARE_VALIDATION_IMPLEMENTATION.md](docs/hardware/HARDWARE_VALIDATION_IMPLEMENTATION.md)** | Hardware validation testing | Developers, testers |

## Documentation by Use Case

### For First-Time Users

**Getting Started Sequence:**
1. Read [README.md](README.md) for project overview
2. Follow [ARCH_LINUX_SETUP.md](docs/setup/ARCH_LINUX_SETUP.md) for environment setup
3. Use [docs/BOOTLOADER_FLASHING_GUIDE.md](docs/BOOTLOADER_FLASHING_GUIDE.md) to flash firmware
4. Follow [USB_HID_LOGGING_SETUP_GUIDE.md](docs/setup/USB_HID_LOGGING_SETUP_GUIDE.md) for logging setup
5. Try examples from [USB_HID_USAGE_EXAMPLES.md](docs/api/USB_HID_USAGE_EXAMPLES.md)

### For Hardware Builders

**Hardware Assembly:**
1. [README.md](README.md) - Component specifications and pin assignments
2. [WIRING_GUIDE.md](docs/hardware/WIRING_GUIDE.md) - Detailed wiring instructions
3. [HARDWARE_VALIDATION_IMPLEMENTATION.md](docs/hardware/HARDWARE_VALIDATION_IMPLEMENTATION.md) - Testing procedures

### For Software Developers

**Development Setup:**
1. [SOFTWARE_SETUP.md](docs/setup/SOFTWARE_SETUP.md) - Rust toolchain setup
2. [ARCH_LINUX_SETUP.md](docs/setup/ARCH_LINUX_SETUP.md) - Development environment
3. [.kiro/specs/usb-hid-logging/design.md](.kiro/specs/usb-hid-logging/design.md) - Architecture overview
4. [.kiro/specs/usb-hid-logging/requirements.md](.kiro/specs/usb-hid-logging/requirements.md) - Requirements specification

### For Troubleshooting

**Problem Resolution:**
1. [USB_HID_TROUBLESHOOTING_GUIDE.md](docs/troubleshooting/USB_HID_TROUBLESHOOTING_GUIDE.md) - Comprehensive troubleshooting
2. [docs/BOOTLOADER_FLASHING_GUIDE.md](docs/BOOTLOADER_FLASHING_GUIDE.md) - Flashing issues
3. [ARCH_LINUX_SETUP.md](docs/setup/ARCH_LINUX_SETUP.md) - Environment issues
4. [USB_HID_LOGGING_SETUP_GUIDE.md](docs/setup/USB_HID_LOGGING_SETUP_GUIDE.md) - Setup problems

### For System Monitoring

**Monitoring and Analysis:**
1. [USB_HID_USAGE_EXAMPLES.md](docs/api/USB_HID_USAGE_EXAMPLES.md) - Comprehensive usage examples
2. [HIDLOG_USAGE.md](docs/api/HIDLOG_USAGE.md) - Basic utility usage
3. [BATTERY_LOGGING_IMPLEMENTATION.md](docs/development/BATTERY_LOGGING_IMPLEMENTATION.md) - Battery monitoring details

## Documentation Features by Category

### Setup and Installation
- **Complete Environment Setup**: Step-by-step Arch Linux development environment
- **Dependency Management**: All required packages and libraries
- **Permission Configuration**: USB device access and udev rules
- **Verification Steps**: Testing installation success

### Hardware and Firmware
- **Component Specifications**: Detailed hardware requirements
- **Wiring Instructions**: Pin assignments and connections
- **Firmware Flashing**: Multiple flashing methods with troubleshooting
- **Hardware Validation**: Testing procedures for real hardware

### USB HID Logging System
- **Comprehensive Setup**: Complete logging system configuration
- **Usage Examples**: Real-world monitoring scenarios
- **Troubleshooting**: Common issues and solutions
- **Performance Analysis**: System performance monitoring

### Development and Testing
- **Build System**: Rust embedded development setup
- **Testing Framework**: Automated testing procedures
- **Integration Examples**: Development workflow integration
- **Performance Monitoring**: Regression testing and optimization

## Document Maintenance

### Version Information
- All documentation is current as of the latest firmware version
- Setup guides are tested on current Arch Linux
- Examples are verified with actual hardware

### Update Policy
- Documentation is updated with each firmware release
- Setup guides are verified quarterly
- Troubleshooting guides are updated based on user feedback
- Examples are tested with each major release

### Contributing to Documentation
- Documentation improvements are welcome
- Please test all procedures before submitting changes
- Include version information and test environment details
- Follow existing documentation style and format

## Quick Reference

### Essential Commands
```bash
# List available devices
python3 scripts/utilities/hidlog.py --list

# Basic monitoring
python3 scripts/utilities/hidlog.py

# Monitor specific module
python3 scripts/utilities/hidlog.py --module BATTERY --level INFO

# Save logs to file
python3 scripts/utilities/hidlog.py --log-file device.log

# Run validation scripts
python3 scripts/validation/run_hardware_validation.py

# Test bootloader functionality
python3 scripts/bootloader/simple_bootloader_entry_test.py

# Enter bootloader mode
# Hold BOOTSEL while connecting USB

# Flash firmware
cargo run --release
```

### Key File Locations
```
Project Structure:
├── README.md                                      # Main project documentation
├── docs/                                          # All documentation
│   ├── setup/                                     # Setup and installation guides
│   │   ├── USB_HID_LOGGING_SETUP_GUIDE.md       # Complete setup guide
│   │   ├── ARCH_LINUX_SETUP.md                  # Environment setup
│   │   └── SOFTWARE_SETUP.md                    # Development setup
│   ├── api/                                       # API documentation
│   │   ├── USB_HID_USAGE_EXAMPLES.md            # Usage examples
│   │   └── HIDLOG_USAGE.md                      # Basic utility usage
│   ├── troubleshooting/                          # Troubleshooting guides
│   │   └── USB_HID_TROUBLESHOOTING_GUIDE.md     # Troubleshooting guide
│   ├── hardware/                                 # Hardware documentation
│   │   └── WIRING_GUIDE.md                      # Hardware assembly
│   ├── development/                              # Development documentation
│   ├── BOOTLOADER_FLASHING_GUIDE.md             # Firmware flashing guide
│   └── USB_HID_INTEGRATION_TESTS.md             # USB HID integration tests
├── scripts/                                      # Executable scripts
│   ├── utilities/                                # General utilities
│   │   └── hidlog.py                            # Log monitoring utility
│   ├── validation/                               # Validation scripts (includes validate_* executables)
│   ├── bootloader/                               # Bootloader scripts
│   └── testing/                                  # Test scripts
├── artifacts/                                    # Generated files
│   ├── test_results/                             # Test outputs
│   ├── firmware/                                 # Generated firmware
│   ├── logs/                                     # Log files
│   ├── bootloader_debugging_summary.md          # Bootloader debugging info
│   └── bootloader_fix.rs                        # Bootloader fixes
├── src/                                          # Firmware source code
├── tests/                                        # Test files
└── .kiro/specs/                                  # Technical specifications
```

### Support Resources
- **Technical Issues**: Check troubleshooting guides first
- **Setup Problems**: Follow setup guides step-by-step
- **Hardware Issues**: Refer to hardware validation documentation
- **Development Questions**: Review design and requirements documents

---

This documentation index provides a comprehensive guide to all available documentation for the RP2040 pEMF/Battery monitoring device with USB HID logging functionality. Choose the appropriate documents based on your role and current needs.
