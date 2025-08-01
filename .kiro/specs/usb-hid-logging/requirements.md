# Requirements Document

## Introduction

This document outlines the requirements for adding USB HID logging capability to the existing RP2040 pEMF/battery monitoring device. The system will provide a way to send debug and status information from the embedded device to a host computer via USB HID interface, allowing developers to monitor device behavior and debug issues in real-time without requiring additional hardware like a debug probe.

## Requirements

### Requirement 1: USB HID Interface Setup

**User Story:** As a developer, I want the device to enumerate as a USB HID device when connected to a computer, so that I can establish communication without requiring special drivers.

#### Acceptance Criteria

1. WHEN the device is connected via USB THEN it SHALL enumerate as a standard HID device
2. WHEN the host queries device descriptors THEN the device SHALL present itself with appropriate vendor/product IDs
3. WHEN the device initializes THEN it SHALL configure USB peripheral with proper clock settings
4. WHEN USB is connected THEN the device SHALL maintain all existing functionality (pEMF, battery monitoring, LED control)
5. IF USB connection is lost THEN the device SHALL continue operating normally without USB logging

### Requirement 2: Logging Message Format

**User Story:** As a developer, I want structured log messages with timestamps and severity levels, so that I can effectively debug and monitor device behavior.

#### Acceptance Criteria

1. WHEN sending log messages THEN each message SHALL include a timestamp in milliseconds since boot
2. WHEN logging events THEN messages SHALL include severity levels (DEBUG, INFO, WARN, ERROR)
3. WHEN formatting messages THEN they SHALL be human-readable text with consistent structure
4. WHEN message length exceeds HID report size THEN messages SHALL be properly truncated or split
5. WHEN multiple messages are queued THEN they SHALL be sent in chronological order

### Requirement 3: Battery Status Logging

**User Story:** As a developer, I want to monitor battery voltage readings and state changes via USB, so that I can verify battery monitoring functionality and debug voltage measurement issues.

#### Acceptance Criteria

1. WHEN battery state changes THEN the device SHALL log the state transition with ADC reading
2. WHEN ADC readings are sampled THEN periodic voltage readings SHALL be logged at configurable intervals
3. WHEN battery reaches critical thresholds THEN warning messages SHALL be logged immediately
4. WHEN ADC errors occur THEN error messages SHALL be logged with diagnostic information
5. WHEN logging battery data THEN messages SHALL include both raw ADC values and calculated voltages

### Requirement 4: pEMF Pulse Monitoring

**User Story:** As a developer, I want to monitor pEMF pulse generation timing and status via USB, so that I can verify timing accuracy and detect any pulse generation issues.

#### Acceptance Criteria

1. WHEN pEMF pulse timing deviates from specification THEN timing error messages SHALL be logged
2. WHEN pulse generation starts THEN initialization status SHALL be logged
3. WHEN system detects timing conflicts THEN conflict warnings SHALL be logged
4. WHEN pulse generation encounters errors THEN error details SHALL be logged
5. WHEN requested THEN pulse timing statistics SHALL be logged periodically

### Requirement 5: System Status and Diagnostics

**User Story:** As a developer, I want to monitor overall system health and performance metrics via USB, so that I can identify potential issues and optimize system performance.

#### Acceptance Criteria

1. WHEN the system boots THEN initialization status and configuration SHALL be logged
2. WHEN RTIC tasks experience delays THEN timing warnings SHALL be logged
3. WHEN memory usage approaches limits THEN resource warnings SHALL be logged
4. WHEN system errors occur THEN detailed error information SHALL be logged
5. WHEN requested THEN system uptime and task statistics SHALL be logged

### Requirement 6: Host-Side Log Reception

**User Story:** As a developer, I want a simple way to receive and display log messages on my computer, so that I can monitor device behavior in real-time.

#### Acceptance Criteria

1. WHEN the device sends HID reports THEN a host application SHALL receive and decode them
2. WHEN log messages are received THEN they SHALL be displayed with proper formatting
3. WHEN multiple devices are connected THEN the host SHALL distinguish between different devices
4. WHEN log data is received THEN it SHALL optionally be saved to a file for later analysis
5. WHEN the host application starts THEN it SHALL automatically detect and connect to the logging device

### Requirement 7: Performance and Resource Management

**User Story:** As a developer, I want USB logging to have minimal impact on existing system performance, so that critical timing requirements are not compromised.

#### Acceptance Criteria

1. WHEN USB logging is active THEN pEMF pulse timing SHALL remain within ±1% tolerance
2. WHEN log messages are queued THEN memory usage SHALL not exceed available resources
3. WHEN USB communication fails THEN the system SHALL continue operating without degradation
4. WHEN log queue is full THEN oldest messages SHALL be discarded to prevent memory overflow
5. WHEN USB logging is disabled THEN there SHALL be no performance impact on core functionality

### Requirement 8: Configuration and Control

**User Story:** As a developer, I want to control logging verbosity and enable/disable different log categories, so that I can focus on specific aspects of system behavior.

#### Acceptance Criteria

1. WHEN compile-time flags are set THEN different log levels SHALL be included or excluded
2. WHEN runtime commands are received THEN log verbosity SHALL be adjustable via USB
3. WHEN specific subsystems are debugged THEN individual logging categories SHALL be controllable
4. WHEN logging is disabled THEN USB HID interface SHALL remain available for control commands
5. WHEN configuration changes are made THEN they SHALL take effect immediately without restart

### Requirement 9: Development Environment and Testing Setup

**User Story:** As a developer on Arch Linux, I want clear instructions for setting up the necessary tools and validating the USB HID logging functionality, so that I can effectively develop and test the logging system.

#### Acceptance Criteria

1. WHEN setting up the development environment THEN the system SHALL provide instructions for installing required Arch Linux packages
2. WHEN installing tools THEN the setup SHALL include hidapi-based utilities for HID communication testing
3. WHEN validating USB connection THEN the system SHALL provide step-by-step instructions including bootloader mode activation
4. WHEN testing HID communication THEN validation tools SHALL confirm proper device enumeration and data reception
5. WHEN flashing firmware THEN instructions SHALL specify when to put the device in bootloader mode (holding BOOTSEL while connecting USB)

### Requirement 10: Automated Testing and Validation

**User Story:** As a developer, I want automated tests that validate USB HID logging functionality, so that I can ensure the system works correctly across different scenarios.

#### Acceptance Criteria

1. WHEN running tests THEN the system SHALL include unit tests for log message formatting and queuing
2. WHEN validating USB functionality THEN integration tests SHALL verify HID report generation and transmission
3. WHEN testing host communication THEN automated tests SHALL validate log message reception and parsing
4. WHEN running performance tests THEN timing validation SHALL confirm minimal impact on existing functionality
5. WHEN testing error conditions THEN tests SHALL validate graceful handling of USB disconnection and reconnection

### Requirement 11: Host-Side Tooling and Utilities

**User Story:** As a developer, I want simple command-line tools to interact with the USB HID logging device, so that I can easily monitor and test the logging functionality.

#### Acceptance Criteria

1. WHEN monitoring logs THEN a simple Python or Rust utility SHALL display real-time log messages
2. WHEN testing HID communication THEN command-line tools SHALL allow sending control commands to the device
3. WHEN debugging issues THEN utilities SHALL provide raw HID report inspection capabilities
4. WHEN saving logs THEN tools SHALL support writing log data to files with timestamps
5. WHEN multiple devices are connected THEN tools SHALL allow selecting specific devices by serial number or path