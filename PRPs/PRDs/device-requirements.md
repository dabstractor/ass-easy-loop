# Requirements Document

## Introduction

This document outlines the requirements for the ass-easy-loop application - a Raspberry Pi Pico-based system that combines a precise pEMF (pulsed Electromagnetic Field) driver with a battery monitoring loop. The application consists of three main functional areas: environment configuration, the pEMF driver subsystem, and the battery monitoring loop. The system must operate with real-time constraints using the RTIC framework in Rust.

## Requirements

### Requirement 1: Development Environment Configuration

**User Story:** As a developer, I want a properly configured embedded Rust environment, so that I can build and deploy the application to the Raspberry Pi Pico.

#### Acceptance Criteria

1. WHEN setting up the project THEN the system SHALL use RTIC 2.0 with thumbv6-backend features
2. WHEN building the project THEN it SHALL target thumbv6m-none-eabi architecture
3. WHEN flashing the device THEN it SHALL use probe-rs with Picoprobe configuration
4. WHEN initializing hardware THEN it SHALL configure 12MHz external crystal with standard PLL settings
5. WHEN managing dependencies THEN it SHALL include all required embedded HAL crates

### Requirement 2: pEMF Driver Subsystem

**User Story:** As a user, I want the device to generate precise pEMF waveforms, so that I can drive electromagnetic field devices with accurate timing.

#### Acceptance Criteria

1. WHEN the system starts THEN the device SHALL generate a continuous 2 Hz square wave on GPIO 15
2. WHEN generating pulses THEN the device SHALL maintain a pulse width of exactly 2ms HIGH followed by 498ms LOW
3. WHEN operating under any system load THEN the pulse timing SHALL remain accurate within ±1% tolerance
4. WHEN other tasks are executing THEN the pEMF pulse generation SHALL NOT be interrupted or delayed
5. IF the system encounters timing conflicts THEN the pEMF pulse generation SHALL have highest priority

### Requirement 3: Battery Monitoring Loop

**User Story:** As a user, I want the device to monitor battery voltage levels, so that I can be alerted to low battery conditions and charging status.

#### Acceptance Criteria

1. WHEN the system is running THEN the device SHALL continuously sample battery voltage on GPIO 26 at 10 Hz
2. WHEN ADC reading is ≤ 1425 THEN the system SHALL classify battery state as Low
3. WHEN ADC reading is > 1425 AND < 1675 THEN the system SHALL classify battery state as Normal
4. WHEN ADC reading is ≥ 1675 THEN the system SHALL classify battery state as Charging
5. WHEN battery state changes THEN the system SHALL update the status within 200ms

### Requirement 4: Visual Status Indication

**User Story:** As a user, I want visual feedback about battery status, so that I can quickly assess the device's power state without external tools.

#### Acceptance Criteria

1. WHEN battery state is Low THEN the onboard LED SHALL flash at 2 Hz (250ms ON, 250ms OFF)
2. WHEN battery state is Charging THEN the onboard LED SHALL remain solid ON continuously
3. WHEN battery state is Normal THEN the onboard LED SHALL remain OFF continuously
4. WHEN battery state changes THEN the LED behavior SHALL update within 500ms
5. IF LED control conflicts with other tasks THEN LED updates SHALL have lowest priority

### Requirement 5: Hardware Interface Configuration

**User Story:** As a developer, I want proper hardware pin assignments and configurations, so that the device interfaces correctly with external components.

#### Acceptance Criteria

1. WHEN the system initializes THEN GPIO 15 SHALL be configured as output for MOSFET control
2. WHEN the system initializes THEN GPIO 25 SHALL be configured as output for LED control
3. WHEN the system initializes THEN GPIO 26 SHALL be configured as ADC input for battery monitoring
4. WHEN reading battery voltage THEN the ADC SHALL use 12-bit resolution with 3.3V reference
5. WHEN interfacing with external components THEN the voltage divider SHALL scale 3.7V battery to safe ADC range

### Requirement 6: Real-Time System Constraints

**User Story:** As a user, I want the device to operate reliably under real-time constraints, so that timing-critical operations are never compromised.

#### Acceptance Criteria

1. WHEN multiple tasks are running THEN the system SHALL maintain task priority hierarchy (pEMF highest, battery medium, LED lowest)
2. WHEN system resources are constrained THEN timing-critical tasks SHALL NOT be affected by lower priority operations
3. WHEN the system operates continuously THEN it SHALL NOT experience lockups or timing drift
4. WHEN using RTIC framework THEN all shared resources SHALL be properly protected from race conditions
5. IF system encounters errors THEN it SHALL handle them gracefully without affecting critical timing

### Requirement 7: System Reliability and Safety

**User Story:** As a user, I want the device to operate safely and reliably, so that it can run continuously without supervision.

#### Acceptance Criteria

1. WHEN the system encounters unrecoverable errors THEN it SHALL use panic-halt for safe shutdown
2. WHEN accessing hardware resources THEN the system SHALL use memory-safe operations
3. WHEN sharing resources between tasks THEN the system SHALL use RTIC's resource sharing mechanisms
4. WHEN operating continuously THEN the system SHALL maintain stable operation without memory leaks
5. IF hardware operations fail THEN the system SHALL handle failures appropriately for the operation type