# USB HID Logging Requirements

## Overview

USB HID logging provides real-time debug information and status monitoring from the RP2040 device to host computers without requiring special drivers. This feature must be **configurable at build time** and only enabled during testing, not in production builds.

## Core Requirements

### Build-Time Configuration

**Priority**: Critical
**Source**: Project requirements - "USB HID logging must be configurable at build time"

- USB HID logging MUST be disabled by default in production builds
- Logging infrastructure MUST be conditionally compiled using feature flags:
  - `usb-logs` - Enable USB HID communication logs
  - `battery-logs` - Enable battery monitoring logs
  - `pemf-logs` - Enable pEMF pulse monitoring logs
  - `system-logs` - Enable system status and diagnostic logs
- When logging is disabled, there MUST be zero performance impact on core functionality
- Build configurations MUST support:
  - `production` - No logging, optimized for performance
  - `development` - All logging enabled with debug features
  - `testing` - Full logging with test command support

### USB HID Interface Specifications

**Hardware Requirements**:
- USB Device Class: HID (Human Interface Device)
- Vendor ID: 0x1234 (development/testing only)
- Product ID: 0x5678
- HID Report Size: 64 bytes
- No special drivers required on host systems

**Enumeration Requirements**:
- Device MUST enumerate as standard HID device
- Device MUST maintain all existing functionality when USB connected
- Device MUST continue operating normally if USB disconnected
- Device descriptors MUST present proper vendor/product identification

### Message Format and Structure

**Log Message Structure**:
```
[TIMESTAMP_MS][LEVEL] Category: Message Content
```

**Required Fields**:
- **Timestamp**: Milliseconds since device boot (u32)
- **Severity Level**: DEBUG, INFO, WARN, ERROR
- **Category**: BATTERY, PEMF, SYSTEM, USB
- **Message Content**: Human-readable text

**Message Constraints**:
- Maximum message length: 60 bytes (accounting for HID report overhead)
- Messages exceeding length MUST be truncated with "..." indicator
- Multiple queued messages MUST be sent chronologically
- Queue size: 32 messages maximum

### Logging Categories

#### Battery Status Logging
**When enabled via `battery-logs` feature**:

- Battery state transitions (Low/Normal/Charging) with ADC readings
- Periodic voltage readings at configurable intervals (default: every 5 seconds)
- Critical threshold warnings (immediate logging)
- ADC read errors with diagnostic information
- Both raw ADC values (0-4095) and calculated voltages

**ADC Voltage Mapping** (from pico-dual-function-device specs):
- Voltage divider: R1=10kΩ, R2=5.1kΩ
- Scale factor: 0.337 (3.7V battery → 1.25V ADC)
- Thresholds:
  - Low: ADC ≤ 1425 (≈ 3.1V battery)
  - Normal: 1425 < ADC < 1675 (3.1V - 3.6V)
  - Charging: ADC ≥ 1675 (> 3.6V)

#### pEMF Pulse Monitoring
**When enabled via `pemf-logs` feature**:

- Pulse generation initialization status
- Timing deviations from specification (±1% tolerance)
- Frequency calculation vs target frequency
- Timing conflict warnings with other tasks
- Pulse generation errors and recovery actions

**pEMF Specifications** (from design specs):
- Target frequency: 2.0 Hz (500ms period)
- Pulse timing: 2ms HIGH, 498ms LOW
- Tolerance: ±1% timing accuracy
- Priority: Highest (cannot be preempted)

#### System Status and Diagnostics
**When enabled via `system-logs` feature**:

- Boot sequence and hardware initialization status
- RTIC task timing warnings and delays
- Memory usage approaching limits
- System errors and recovery actions
- Uptime and task execution statistics
- Performance metrics and resource usage

#### USB Communication Logs
**When enabled via `usb-logs` feature**:

- USB connection/disconnection events
- HID report transmission status
- Queue overflow warnings
- Communication errors and retry attempts
- Host-side connection validation

### Performance Requirements

**Critical Constraints**:
- pEMF pulse timing MUST remain within ±1% tolerance when logging active
- Memory usage MUST NOT exceed available resources
- When log queue full, oldest messages MUST be discarded (FIFO)
- Log transmission timeout: 100ms maximum
- Maximum retry attempts: 3

**Resource Management**:
- Global log queue: 32 message capacity
- Stack-allocated structures only (no dynamic allocation)
- Compile-time resource sizing
- Minimal RAM footprint (<2KB for logging infrastructure)

### Host-Side Requirements

**Host Application Must**:
- Automatically detect and connect to logging device
- Receive and decode HID reports in real-time
- Display formatted log messages with timestamps
- Support saving logs to files for analysis
- Handle multiple devices with unique identification
- Provide raw HID report inspection for debugging

**Development Environment**:
- Target platform: Arch Linux
- Required packages: hidapi-based utilities
- Python or Rust utilities for log monitoring
- Command-line tools for device control

### Configuration and Control

**Compile-Time Control**:
- Feature flags determine included log categories
- Debug vs release build configurations
- Conditional compilation of logging infrastructure

**Runtime Control** (via USB commands):
- Adjustable log verbosity levels
- Individual category enable/disable
- Configuration changes take effect immediately
- Control interface remains available when logging disabled

### Error Handling and Recovery

**USB Communication Failures**:
- System MUST continue operating without degradation
- Failed transmissions logged locally for debugging
- Automatic retry with exponential backoff
- Graceful degradation when host disconnected

**Resource Exhaustion**:
- Queue overflow: discard oldest messages
- Memory pressure: reduce logging verbosity
- Transmission failures: local buffering with limits

### Testing and Validation

**Automated Tests Required**:
- Unit tests for message formatting and queuing
- Integration tests for HID report generation
- Performance tests confirming minimal impact
- Error condition testing (USB disconnect/reconnect)
- Multi-device testing scenarios

**Validation Tools**:
- HID communication test utilities
- Raw report inspection capabilities
- Timing validation with oscilloscope verification
- Long-term stability testing (24+ hours)

## Implementation Notes

### RTIC Integration
- Logging tasks have lowest priority (priority 3)
- Shared resources protected by RTIC resource sharing
- USB transmission in separate task from message generation
- Non-blocking queue operations to prevent task blocking

### Memory Safety
- All operations use memory-safe Rust patterns
- No dynamic allocation in embedded context
- Compile-time resource verification
- Stack-based data structures only

### Security Considerations
- Development/testing VID/PID only
- No sensitive information in log messages
- Rate limiting for log message generation
- Input validation for control commands

## Dependencies

**Embedded Dependencies**:
- `usbd-hid = "0.8.2"` - USB HID class implementation
- `usb-device = "0.3.2"` - USB device framework
- `rtic = "2.2.0"` - Real-time interrupt-driven concurrency
- `heapless = "0.8.0"` - Collections without allocation

**Host-Side Dependencies**:
- `hidapi` - Cross-platform HID library access
- Python 3.x with `hid` module for utilities
- Standard development tools (rustc, cargo, probe-rs)

## Success Criteria

1. **Build System**: Clean separation of production/development/testing builds
2. **Zero Impact**: No performance degradation when logging disabled
3. **Real-Time**: Logging never interferes with ±1% pEMF timing requirement
4. **Reliability**: System continues operating regardless of USB/logging state
5. **Developer Experience**: Simple command-line tools for log monitoring
6. **Test Coverage**: Comprehensive automated validation of logging functionality