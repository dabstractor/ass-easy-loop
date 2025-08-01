# Implementation Plan

- [x] 1. Set up USB HID dependencies and basic project structure
  - Add required USB HID crates to Cargo.toml (usbd-hid, usb-device, heapless for host utilities)
  - Create logging module structure with basic types and interfaces
  - Define compile-time configuration constants for USB VID/PID and logging levels
  - _Requirements: 1.1, 1.2, 1.3, 8.1_

- [x] 2. Implement core logging data structures and message formatting
  - Create LogLevel enum and LogMessage struct with proper serialization
  - Implement message formatting functions that convert log data to fixed-size binary format
  - Create LogReport struct with HID report descriptor using gen_hid_descriptor macro
  - Write unit tests for message serialization and deserialization
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 10.1_

- [x] 3. Implement thread-safe message queue with circular buffer
  - Create LogQueue struct using atomic operations for lock-free access
  - Implement enqueue/dequeue operations with proper overflow handling (FIFO eviction)
  - Add queue statistics tracking (messages sent, dropped, queue utilization)
  - Write unit tests for queue operations including concurrent access scenarios
  - _Requirements: 7.4, 7.5, 10.1_

- [x] 4. Create logging interface macros and integration points
  - Implement log_debug!, log_info!, log_warn!, log_error! macros
  - Create core log_message function that formats and queues messages
  - Add module name tracking and timestamp generation using RTIC monotonic timer
  - Write unit tests for macro expansion and message formatting
  - _Requirements: 2.1, 2.2, 8.1, 8.3_

- [x] 5. Integrate USB HID device initialization into existing RTIC app
  - Modify init function to set up USB bus allocator and HID class device
  - Configure USB device descriptors with custom VID/PID and device strings
  - Add USB-related shared resources to RTIC app structure
  - Ensure USB initialization doesn't interfere with existing GPIO/ADC/timer setup
  - _Requirements: 1.1, 1.2, 1.3, 1.4_

- [x] 6. Implement low-priority USB polling task
  - Create usb_poll_task with priority 0 for USB device polling and enumeration
  - Handle USB device state changes and enumeration events
  - Implement proper error handling for USB connection/disconnection
  - Add USB connection status tracking for graceful degradation
  - _Requirements: 1.5, 7.1, 7.3_

- [x] 7. Implement USB HID transmission task
  - Create usb_hid_task with priority 1 for log message transmission
  - Implement message dequeuing and HID report generation
  - Add transmission error handling and retry logic
  - Ensure task doesn't block when USB is disconnected
  - _Requirements: 2.5, 7.1, 7.3, 7.4_

- [x] 8. Integrate logging calls into existing battery monitoring system
  - Add log messages for battery state changes with ADC readings and calculated voltages
  - Log periodic battery voltage readings at configurable intervals
  - Add error logging for ADC read failures with diagnostic information
  - Add battery threshold crossing warnings (low/normal/charging transitions)
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

- [x] 9. Integrate logging calls into existing pEMF pulse generation system
  - Add startup logging for pEMF pulse generation initialization
  - Implement timing validation logging to detect pulse timing deviations
  - Add error logging for pulse generation failures or timing conflicts
  - Log pulse timing statistics periodically for performance monitoring
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

- [x] 10. Add system status and diagnostic logging
  - Log system boot sequence and hardware initialization status
  - Add RTIC task timing monitoring with delay warnings
  - Implement memory usage tracking and resource warnings
  - Add comprehensive error logging with detailed diagnostic information
  - Log system uptime and task execution statistics
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 11. Implement enhanced panic handler with USB logging
  - Modify panic handler to attempt logging panic information via USB
  - Add best-effort USB message flushing before system halt
  - Ensure panic handler doesn't interfere with existing panic-halt behavior
  - Test panic logging functionality with intentional panic scenarios
  - _Requirements: 5.4, 7.3_

- [x] 12. Create Python host utility for log message reception
  - Implement HidLogReceiver class using hidapi for device communication
  - Add log message parsing and formatting for human-readable display
  - Implement real-time log display with timestamp formatting
  - Add command-line options for device selection and log filtering
  - _Requirements: 6.1, 6.2, 6.4, 11.1, 11.2_

- [x] 13. Add log file saving and device management to host utility
  - Implement log file writing with timestamps and proper formatting
  - Add support for multiple device detection and selection by serial number
  - Create command-line interface for log verbosity control
  - Add raw HID report inspection mode for debugging
  - _Requirements: 6.4, 6.5, 11.3, 11.4, 11.5_

- [x] 14. Implement compile-time and runtime configuration system
  - Create LogConfig struct with compile-time feature flags for log levels
  - Add conditional compilation for different logging categories (battery, pEMF, system)
  - Implement runtime log level control via USB control commands
  - Add configuration validation and error handling
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_

- [x] 15. Write comprehensive integration tests for USB HID functionality
  - Create integration tests for USB device enumeration and HID report transmission
  - Implement end-to-end communication tests between device and host utility
  - Add performance tests to measure timing impact on existing pEMF/battery tasks
  - Create error recovery tests for USB disconnection/reconnection scenarios
  - _Requirements: 10.2, 10.3, 10.4, 10.5_

- [x] 16. Create hardware validation tests and setup documentation
  - Write hardware-in-loop tests for real RP2040 device testing
  - Create timing validation tests to confirm pEMF pulse accuracy with USB logging active
  - Implement battery monitoring integration tests with actual ADC readings
  - Document complete setup process including Arch Linux package installation
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_

- [x] 17. Add performance monitoring and optimization
  - Implement CPU usage monitoring for USB tasks
  - Add memory usage tracking and optimization for message queues
  - Create performance benchmarks comparing system behavior with/without USB logging
  - Optimize message formatting and transmission for minimal CPU overhead
  - _Requirements: 7.1, 7.2, 7.5_

- [x] 18. Create comprehensive documentation and usage examples
  - Write detailed setup instructions for Arch Linux development environment
  - Create usage examples showing how to monitor different types of log messages
  - Document bootloader mode activation process and firmware flashing steps
  - Add troubleshooting guide for common USB HID issues
  - _Requirements: 9.1, 9.3, 9.5_