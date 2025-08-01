# Implementation Plan

- [x] 1. Fix compilation issues and complete project setup
  - [x] 1.1 Resolve RTIC atomic CAS compilation error
    - Add portable-atomic feature to fix compare_exchange error on thumbv6m target
    - Update Cargo.toml with required atomic features for single-core operation
    - Verify compilation succeeds without errors
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [x] 1.2 Basic project structure (completed but needs compilation fix)
    - Configure Cargo.toml with all required embedded dependencies (rtic, rp2040-hal, etc.)
    - Set up .cargo/config.toml for thumbv6m-none-eabi target and probe-rs runner
    - Create basic main.rs structure with RTIC app skeleton
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [x] 2. Implement hardware initialization and resource setup
  - [x] 2.1 Configure clocks and PLL with 12MHz external crystal
    - Initialize clocks using init_clocks_and_plls() with standard configuration
    - Enable required peripherals (GPIO, ADC, TIMER)
    - _Requirements: 1.4_

  - [x] 2.2 Set up GPIO pin configurations
    - Configure GPIO 15 as push-pull output for MOSFET control
    - Configure GPIO 25 as push-pull output for LED control
    - Configure GPIO 26 as floating input for ADC battery monitoring
    - _Requirements: 5.1, 5.2, 5.3_

  - [x] 2.3 Initialize ADC peripheral for battery monitoring
    - Configure ADC with 12-bit resolution and 3.3V reference
    - Set up ADC pin for GPIO 26 with proper input configuration
    - _Requirements: 5.4, 5.5_

- [x] 3. Implement battery state management system
  - [x] 3.1 Create BatteryState enum and state machine logic
    - Define BatteryState enum with Low, Normal, and Charging variants
    - Implement state transition logic with ADC threshold comparisons
    - Write unit tests for state machine transitions
    - _Requirements: 3.2, 3.3, 3.4_

  - [x] 3.2 Implement ADC reading and voltage conversion functions
    - Create function to read ADC value from GPIO 26
    - Implement voltage divider calculation and battery state determination
    - Add error handling for ADC read failures
    - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 4. Implement high-priority pEMF pulse generation task
  - [x] 4.1 Create pemf_pulse_task with hardware timer scheduling
    - Implement task with highest priority using hardware timer interrupt
    - Create pulse state machine (2ms HIGH, 498ms LOW cycle)
    - Use spawn_after() for precise self-scheduling
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

  - [x] 4.2 Implement precise timing logic for 2Hz square wave
    - Calculate exact timing values for 2ms and 498ms intervals
    - Implement pulse_active boolean state tracking
    - Add GPIO 15 control logic for MOSFET switching
    - Write timing validation tests
    - _Requirements: 2.1, 2.2, 2.3_

- [x] 5. Implement medium-priority battery monitoring task
  - [x] 5.1 Create battery_monitor_task with 100ms periodic execution
    - Implement task with medium priority and 10Hz sampling rate
    - Read ADC value and update shared adc_reading resource
    - Update battery_state based on threshold logic
    - _Requirements: 3.1, 3.5_

  - [x] 5.2 Implement battery state change detection and updates
    - Compare new state with previous state to detect changes
    - Update shared battery_state resource with proper locking
    - Ensure state updates complete within 200ms requirement
    - _Requirements: 3.5_

- [x] 6. Implement low-priority LED control task
  - [x] 6.1 Create led_control_task with variable scheduling
    - Implement task with lowest priority for visual feedback
    - Read battery_state from shared resource
    - Implement LED control logic for each battery state
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

  - [x] 6.2 Implement LED flash patterns and timing
    - Create 2Hz flash pattern for low battery (250ms ON/OFF)
    - Implement solid ON for charging state
    - Implement OFF for normal state
    - Use spawn_after() for flash timing control
    - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [x] 7. Implement RTIC resource sharing and task coordination
  - [x] 7.1 Set up shared resources with proper locking
    - Define Shared struct with led, adc_reading, and battery_state
    - Define Local struct with mosfet_pin, adc, adc_pin, alarm, pulse_active
    - Implement resource access patterns with minimal lock duration
    - _Requirements: 6.1, 6.4_

  - [x] 7.2 Configure task priorities and dispatchers
    - Set up RTIC dispatchers and monotonic timer configuration
    - Assign correct priority levels to each task
    - Verify priority hierarchy prevents timing conflicts
    - _Requirements: 6.1, 6.2_

- [x] 8. Implement error handling and system safety
  - [x] 8.1 Add panic handling and error recovery
    - Configure panic-halt for unrecoverable errors
    - Implement graceful error handling for non-critical operations
    - Add error logging for debugging purposes
    - _Requirements: 7.1, 7.5_

  - [x] 8.2 Implement memory safety and resource protection
    - Ensure all hardware resources are properly moved into RTIC structures
    - Verify no global mutable state outside RTIC framework
    - Use RTIC resource sharing for thread safety
    - _Requirements: 7.2, 7.3, 7.4_

- [ ] 9. Create comprehensive testing and validation
  - [x] 9.1 Implement unit tests for core functionality
    - Write tests for battery state machine logic
    - Test ADC value conversion and threshold detection
    - Test timing calculations and pulse generation logic
    - _Requirements: 2.3, 3.2, 3.3, 3.4_

go watch -x run  - [ ] 9.2 Create integration tests for hardware interaction
    - Test GPIO pin control and ADC reading functionality
    - Validate task scheduling and priority behavior
    - Test resource sharing and concurrent access patterns
    - _Requirements: 6.1, 6.2, 6.3_

- [x] 10. Create comprehensive DIY tutorial documentation
  - [x] 10.1 Create detailed README with project overview and specifications
    - Write project introduction explaining pEMF device functionality
    - Create specifications table with technical parameters
    - Document hardware requirements and component list
    - Explain basic concepts of pEMF therapy and battery monitoring
    - _Requirements: All requirements for user understanding_

  - [x] 10.2 Document complete wiring schematics and assembly instructions
    - Create detailed wiring diagrams for Raspberry Pi Pico connections
    - Document MOSFET driver module connections with pin mappings
    - Create voltage divider circuit schematic with component values
    - Provide step-by-step assembly instructions with photos/diagrams
    - Include troubleshooting section for common wiring issues
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

  - [x] 10.3 Create software setup and flashing instructions
    - Document Rust embedded development environment setup
    - Provide probe-rs installation and configuration instructions
    - Create step-by-step build and flash process
    - Include debugging and monitoring instructions
    - Add section on customizing timing parameters
    - _Requirements: 1.1, 1.2, 1.3_

  - [x] 10.4 Add usage guide and safety information
    - Document device operation and LED status indicators
    - Create battery management and charging guidelines
    - Include safety warnings for electrical connections
    - Provide maintenance and troubleshooting guide
    - Add performance validation procedures
    - _Requirements: 4.1, 4.2, 4.3, 7.1_

- [x] 11. Optimize performance and validate timing requirements
  - [x] 11.1 Profile task execution times and system performance
    - Measure pEMF pulse timing accuracy and jitter
    - Validate battery monitoring latency requirements
    - Test LED response time requirements
    - _Requirements: 2.3, 3.5, 4.4_

  - [x] 11.2 Conduct long-term stability testing
    - Run continuous operation tests for extended periods
    - Monitor for timing drift or system lockups
    - Validate system behavior under various battery conditions
    - _Requirements: 6.3, 7.4_