//! Test data generators and utilities
//!
//! This module provides functions to generate test data for various components
//! and scenarios, making it easier to create comprehensive test cases.

// use std::time::Duration;

/// Battery test data generators
pub mod battery {
    #[cfg(any(test, feature = "std-testing"))]
    use ass_easy_loop::BatteryState;

    /// Generate a sequence of battery voltage readings simulating discharge
    pub fn discharge_sequence(start_mv: u32, end_mv: u32, steps: usize) -> Vec<u32> {
        if steps == 0 {
            return vec![];
        }

        let step_size = (start_mv - end_mv) / steps as u32;
        (0..steps)
            .map(|i| start_mv - (i as u32 * step_size))
            .collect()
    }

    /// Generate a sequence of battery voltage readings simulating charge
    pub fn charge_sequence(start_mv: u32, end_mv: u32, steps: usize) -> Vec<u32> {
        if steps == 0 {
            return vec![];
        }

        let step_size = (end_mv - start_mv) / steps as u32;
        (0..steps)
            .map(|i| start_mv + (i as u32 * step_size))
            .collect()
    }

    /// Generate noisy battery readings around a target voltage
    pub fn noisy_readings(target_mv: u32, noise_range: u32, count: usize) -> Vec<u32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        (0..count)
            .map(|i| {
                // Simple pseudo-random noise generation for tests
                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                let hash = hasher.finish();
                let noise = (hash % (noise_range * 2) as u64) as u32;
                let signed_noise = noise as i32 - noise_range as i32;
                ((target_mv as i32) + signed_noise).max(0) as u32
            })
            .collect()
    }

    /// Generate typical battery voltage levels for different states
    pub fn typical_voltage_ranges() -> std::collections::HashMap<&'static str, u32> {
        use std::collections::HashMap;

        let mut voltages = HashMap::new();
        voltages.insert("low", 3000);
        voltages.insert("normal", 3400);
        voltages.insert("charging", 3700);
        voltages
    }

    /// Generate ADC values corresponding to battery voltage readings
    pub fn voltage_to_adc_sequence(voltages_mv: &[u32]) -> Vec<u16> {
        voltages_mv
            .iter()
            .map(|&voltage_mv| {
                // Convert voltage to ADC using the same formula as BatteryMonitor
                // ADC = voltage_mv * 1000 / 2386, clamped to 0-4095
                let adc_value = (voltage_mv * 1000) / 2386;
                if adc_value > 4095 {
                    4095
                } else {
                    adc_value as u16
                }
            })
            .collect()
    }

    /// Generate battery state sequences based on voltage transitions
    #[cfg(any(test, feature = "std-testing"))]
    pub fn state_transition_sequence() -> Vec<(u32, BatteryState)> {
        vec![
            // Start charging
            (3800, BatteryState::Charging),
            (3750, BatteryState::Charging),
            (3700, BatteryState::Charging),
            (3650, BatteryState::Charging),
            // Transition to normal
            (3600, BatteryState::Normal),
            (3500, BatteryState::Normal),
            (3400, BatteryState::Normal),
            (3300, BatteryState::Normal),
            (3200, BatteryState::Normal),
            // Transition to low
            (3100, BatteryState::Low),
            (3050, BatteryState::Low),
            (3000, BatteryState::Low),
            (2950, BatteryState::Low),
        ]
    }

    /// Generate realistic battery discharge curve over time
    pub fn realistic_discharge_curve(
        duration_hours: f32,
        initial_voltage_mv: u32,
    ) -> Vec<(u32, u32)> {
        let steps = (duration_hours * 60.0) as usize; // One reading per minute
        let mut curve = Vec::with_capacity(steps);

        for i in 0..steps {
            let time_fraction = i as f32 / steps as f32;

            // Realistic lithium battery discharge curve (exponential decay)
            let voltage_drop =
                (initial_voltage_mv as f32 - 2800.0) * (1.0 - (-2.0 * time_fraction).exp()) * 0.8;
            let current_voltage = initial_voltage_mv as f32 - voltage_drop;

            let timestamp_ms = (i as f32 * 60.0 * 1000.0) as u32; // Convert minutes to ms
            curve.push((timestamp_ms, current_voltage as u32));
        }

        curve
    }

    /// Generate battery state changes with timestamps for testing state machine
    #[cfg(any(test, feature = "std-testing"))]
    pub fn state_machine_test_sequence() -> Vec<(u32, BatteryState, u16)> {
        vec![
            // timestamp_ms, expected_state, adc_value
            (0, BatteryState::Normal, 1500),
            (1000, BatteryState::Normal, 1450),
            (2000, BatteryState::Low, 1400), // Transition to low
            (3000, BatteryState::Low, 1350),
            (4000, BatteryState::Low, 1300),
            (5000, BatteryState::Normal, 1500), // Recovery to normal
            (6000, BatteryState::Normal, 1600),
            (7000, BatteryState::Charging, 1700), // Transition to charging
            (8000, BatteryState::Charging, 1800),
            (9000, BatteryState::Charging, 1900),
            (10000, BatteryState::Normal, 1600), // Back to normal
        ]
    }

    /// Generate boundary condition test cases for battery state detection
    pub fn boundary_test_cases() -> Vec<(u16, &'static str)> {
        vec![
            // ADC value, expected state description
            (0, "low"),
            (1424, "low"),
            (1425, "low"), // Boundary: Low/Normal
            (1426, "normal"),
            (1500, "normal"),
            (1674, "normal"),
            (1675, "charging"), // Boundary: Normal/Charging
            (1676, "charging"),
            (2000, "charging"),
            (4095, "charging"),
        ]
    }

    /// Generate stress test battery data with rapid fluctuations
    pub fn stress_test_sequence(duration_ms: u32, fluctuation_range: u32) -> Vec<(u32, u32)> {
        let base_voltage = 3400; // Normal state voltage
        let step_ms = 100; // Reading every 100ms
        let steps = duration_ms / step_ms;

        (0..steps)
            .map(|i| {
                let timestamp = i * step_ms;

                // Create fluctuations using simple hash-based pseudo-random
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};

                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                let hash = hasher.finish();
                let fluctuation =
                    (hash % (fluctuation_range * 2) as u64) as i32 - fluctuation_range as i32;

                let voltage = ((base_voltage as i32) + fluctuation).max(2800).min(4200) as u32;
                (timestamp, voltage)
            })
            .collect()
    }
}

/// USB HID test data generators
pub mod usb_hid {
    #[cfg(any(test, feature = "std-testing"))]
    use ass_easy_loop::logging::{LogLevel, LogMessage, LogReport};
    #[cfg(any(test, feature = "std-testing"))]
    use ass_easy_loop::UsbControlCommand;

    /// Generate a test USB HID message with specified command and data
    pub fn create_test_message(command: u8, data: &[u8]) -> Vec<u8> {
        let mut message = vec![command];
        message.extend_from_slice(data);
        // Pad to standard HID report size if needed
        while message.len() < 64 {
            message.push(0);
        }
        message
    }

    /// Generate a sequence of USB HID messages for testing
    pub fn message_sequence(commands: &[u8]) -> Vec<Vec<u8>> {
        commands
            .iter()
            .enumerate()
            .map(|(i, &cmd)| create_test_message(cmd, &[i as u8]))
            .collect()
    }

    /// Generate USB HID configuration messages
    pub fn config_messages() -> Vec<Vec<u8>> {
        vec![
            create_test_message(0x01, &[]),        // GetConfig
            create_test_message(0x02, &[1, 2, 3]), // SetConfig
            create_test_message(0x03, &[2]),       // SetLogLevel
            create_test_message(0x04, &[1]),       // EnableCategory
            create_test_message(0x05, &[1]),       // DisableCategory
            create_test_message(0x06, &[]),        // ResetConfig
            create_test_message(0x07, &[]),        // GetStats
        ]
    }

    /// Generate USB control command messages with proper command codes
    #[cfg(any(test, feature = "std-testing"))]
    pub fn control_command_messages() -> Vec<(UsbControlCommand, Vec<u8>)> {
        vec![
            (UsbControlCommand::GetConfig, create_test_message(0x01, &[])),
            (
                UsbControlCommand::SetConfig,
                create_test_message(0x02, &[1, 2, 3, 4]),
            ),
            (
                UsbControlCommand::SetLogLevel,
                create_test_message(0x03, &[2]),
            ), // Info level
            (
                UsbControlCommand::EnableCategory,
                create_test_message(0x04, &[1]),
            ), // Category 1
            (
                UsbControlCommand::DisableCategory,
                create_test_message(0x05, &[1]),
            ), // Category 1
            (
                UsbControlCommand::ResetConfig,
                create_test_message(0x06, &[]),
            ),
            (UsbControlCommand::GetStats, create_test_message(0x07, &[])),
        ]
    }

    /// Generate log message HID reports for testing
    #[cfg(any(test, feature = "std-testing"))]
    pub fn log_message_reports() -> Vec<LogReport> {
        let messages = vec![
            LogMessage::new(1000, LogLevel::Info, "BATTERY", "Battery voltage: 3400mV"),
            LogMessage::new(2000, LogLevel::Debug, "USB", "USB connected"),
            LogMessage::new(3000, LogLevel::Warn, "PEMF", "Frequency adjusted"),
            LogMessage::new(4000, LogLevel::Error, "SYSTEM", "Memory low"),
            LogMessage::new(5000, LogLevel::Info, "TEST", "Test message"),
        ];

        messages
            .into_iter()
            .map(|msg| LogReport::from_log_message(&msg))
            .collect()
    }

    /// Generate a sequence of log messages with timestamps
    #[cfg(any(test, feature = "std-testing"))]
    pub fn timestamped_log_sequence(
        start_time: u32,
        interval_ms: u32,
        count: usize,
    ) -> Vec<LogMessage> {
        let modules = ["BATTERY", "USB", "PEMF", "SYSTEM", "TEST"];
        let levels = [
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ];
        let messages = [
            "System initialized",
            "Battery check complete",
            "USB connection established",
            "PEMF frequency set",
            "Memory usage normal",
            "Configuration updated",
            "Error condition cleared",
            "Test sequence started",
        ];

        (0..count)
            .map(|i| {
                let timestamp = start_time + (i as u32 * interval_ms);
                let module = modules[i % modules.len()];
                let level = levels[i % levels.len()];
                let message = messages[i % messages.len()];

                LogMessage::new(timestamp, level, module, message)
            })
            .collect()
    }

    /// Generate malformed USB HID messages for error testing
    pub fn malformed_messages() -> Vec<Vec<u8>> {
        vec![
            // Empty message
            vec![],
            // Too short message
            vec![0x01],
            // Invalid command code
            create_test_message(0xFF, &[1, 2, 3]),
            // Message with invalid length
            vec![0x02; 65], // Too long
            // Command with missing required data
            create_test_message(0x02, &[]), // SetConfig without data
            // Command with too much data
            create_test_message(0x03, &[1, 2, 3, 4, 5]), // SetLogLevel with extra data
        ]
    }

    /// Generate USB HID reports with various data patterns for stress testing
    pub fn stress_test_reports(count: usize) -> Vec<Vec<u8>> {
        (0..count)
            .map(|i| {
                let pattern = i % 4;
                match pattern {
                    0 => create_test_message(0x01, &[]),              // GetConfig
                    1 => create_test_message(0x07, &[]),              // GetStats
                    2 => create_test_message(0x03, &[(i % 4) as u8]), // SetLogLevel
                    _ => create_test_message(0x04, &[(i % 8) as u8]), // EnableCategory
                }
            })
            .collect()
    }

    /// Generate USB HID message burst for testing queue overflow
    pub fn message_burst(burst_size: usize, command: u8) -> Vec<Vec<u8>> {
        (0..burst_size)
            .map(|i| create_test_message(command, &[i as u8]))
            .collect()
    }

    /// Generate realistic USB communication scenario
    pub fn realistic_communication_scenario() -> Vec<(String, Vec<u8>)> {
        vec![
            (
                "Initial GetConfig".to_string(),
                create_test_message(0x01, &[]),
            ),
            (
                "Set log level to Info".to_string(),
                create_test_message(0x03, &[1]),
            ),
            (
                "Enable battery category".to_string(),
                create_test_message(0x04, &[1]),
            ),
            (
                "Enable USB category".to_string(),
                create_test_message(0x04, &[2]),
            ),
            ("Get stats".to_string(), create_test_message(0x07, &[])),
            (
                "Update config".to_string(),
                create_test_message(0x02, &[1, 1, 2, 0]),
            ),
            (
                "Get updated config".to_string(),
                create_test_message(0x01, &[]),
            ),
            (
                "Disable debug category".to_string(),
                create_test_message(0x05, &[0]),
            ),
            (
                "Final stats check".to_string(),
                create_test_message(0x07, &[]),
            ),
        ]
    }

    /// Generate USB HID reports with specific timing patterns
    pub fn timed_message_sequence(base_interval_ms: u32, count: usize) -> Vec<(u32, Vec<u8>)> {
        (0..count)
            .map(|i| {
                let timestamp = i as u32 * base_interval_ms;
                let command = match i % 3 {
                    0 => 0x01, // GetConfig
                    1 => 0x07, // GetStats
                    _ => 0x03, // SetLogLevel
                };
                let data = if command == 0x03 {
                    vec![i as u8 % 4]
                } else {
                    vec![]
                };
                let message = create_test_message(command, &data);
                (timestamp, message)
            })
            .collect()
    }

    /// Generate USB HID error response messages
    pub fn error_response_messages() -> Vec<(String, Vec<u8>)> {
        vec![
            (
                "Invalid command error".to_string(),
                create_error_response(0x01, "Invalid command"),
            ),
            (
                "Configuration error".to_string(),
                create_error_response(0x02, "Config failed"),
            ),
            (
                "Parameter error".to_string(),
                create_error_response(0x03, "Bad parameter"),
            ),
            (
                "System error".to_string(),
                create_error_response(0x04, "System fault"),
            ),
            (
                "Timeout error".to_string(),
                create_error_response(0x05, "Timeout"),
            ),
        ]
    }

    /// Helper function to create error response messages
    fn create_error_response(error_code: u8, error_message: &str) -> Vec<u8> {
        let mut response = vec![0xFF, error_code]; // 0xFF indicates error response
        let message_bytes = error_message.as_bytes();
        let max_message_len = 60; // Leave space for error code and padding
        let message_len = std::cmp::min(message_bytes.len(), max_message_len);
        response.extend_from_slice(&message_bytes[..message_len]);

        // Pad to 64 bytes
        while response.len() < 64 {
            response.push(0);
        }

        response
    }

    /// Generate USB HID configuration data for different scenarios
    pub fn configuration_scenarios() -> Vec<(String, Vec<u8>)> {
        vec![
            (
                "Default config".to_string(),
                create_test_message(0x02, &[1, 1, 1, 0]),
            ), // log_level=1, all categories enabled
            (
                "Debug config".to_string(),
                create_test_message(0x02, &[0, 1, 1, 1]),
            ), // log_level=0, all categories enabled
            (
                "Production config".to_string(),
                create_test_message(0x02, &[2, 1, 0, 0]),
            ), // log_level=2, only battery enabled
            (
                "Silent config".to_string(),
                create_test_message(0x02, &[3, 0, 0, 0]),
            ), // log_level=3, no categories
            (
                "Test config".to_string(),
                create_test_message(0x02, &[0, 1, 1, 1]),
            ), // log_level=0, all enabled for testing
        ]
    }
}

/// Timing test data generators
pub mod timing {
    use std::time::Duration;

    /// Generate a sequence of timestamps with specified intervals
    pub fn timestamp_sequence(start_ms: u32, interval_ms: u32, count: usize) -> Vec<u32> {
        (0..count)
            .map(|i| start_ms + (i as u32 * interval_ms))
            .collect()
    }

    /// Generate timing measurements for performance testing
    pub fn performance_measurements(
        base_duration: Duration,
        variance_percent: f32,
        count: usize,
    ) -> Vec<Duration> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let base_nanos = base_duration.as_nanos() as f64;
        let variance_nanos = base_nanos * (variance_percent as f64 / 100.0);

        (0..count)
            .map(|i| {
                // Simple pseudo-random variance for tests
                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                let hash = hasher.finish();
                let variance_factor = (hash % 2000) as f64 / 1000.0 - 1.0; // -1.0 to 1.0
                let adjusted_nanos = base_nanos + (variance_nanos * variance_factor);
                Duration::from_nanos(adjusted_nanos.max(0.0) as u64)
            })
            .collect()
    }

    /// Generate PEMF timing data for testing
    pub fn pemf_timing_data(frequency_hz: f32, duration_ms: u32) -> Vec<u32> {
        let period_ms = 1000.0 / frequency_hz;
        let num_cycles = (duration_ms as f32 / period_ms) as usize;

        (0..num_cycles)
            .map(|i| (i as f32 * period_ms) as u32)
            .collect()
    }

    /// Generate precise PEMF timing measurements with jitter analysis
    pub fn pemf_timing_with_jitter(
        frequency_hz: f32,
        duration_ms: u32,
        max_jitter_us: u32,
    ) -> Vec<(u32, u32, i32)> {
        let period_us = (1_000_000.0 / frequency_hz) as u32;
        let num_cycles = ((duration_ms * 1000) / period_us) as usize;

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        (0..num_cycles)
            .map(|i| {
                let expected_time_us = i as u32 * period_us;

                // Generate jitter using hash-based pseudo-random
                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                let hash = hasher.finish();
                let jitter = (hash % (max_jitter_us * 2) as u64) as i32 - max_jitter_us as i32;

                let actual_time_us = ((expected_time_us as i32) + jitter).max(0) as u32;

                (expected_time_us, actual_time_us, jitter)
            })
            .collect()
    }

    /// Generate battery monitoring timing data
    pub fn battery_monitoring_timing(interval_ms: u32, duration_ms: u32) -> Vec<(u32, u32)> {
        let num_readings = duration_ms / interval_ms;

        (0..num_readings)
            .map(|i| {
                let scheduled_time = i * interval_ms;

                // Simulate small timing variations in battery readings
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};

                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                let hash = hasher.finish();
                let variation = (hash % 20) as u32; // ±10ms variation
                let actual_time = scheduled_time + variation;

                (scheduled_time, actual_time)
            })
            .collect()
    }

    /// Generate USB task timing measurements
    pub fn usb_task_timing(
        base_period_us: u32,
        count: usize,
        load_factor: f32,
    ) -> Vec<(u32, u32, u32)> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        (0..count)
            .map(|i| {
                let scheduled_time = i as u32 * base_period_us;

                // Simulate CPU load affecting timing
                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                let hash = hasher.finish();

                let base_execution_time = (base_period_us as f32 * 0.1) as u32; // 10% of period
                let load_variation = (base_execution_time as f32
                    * load_factor
                    * ((hash % 1000) as f32 / 1000.0)) as u32;
                let execution_time = base_execution_time + load_variation;

                let completion_time = scheduled_time + execution_time;

                (scheduled_time, execution_time, completion_time)
            })
            .collect()
    }

    /// Generate timing violation scenarios for testing
    pub fn timing_violation_scenarios() -> Vec<(String, Vec<(u32, u32, bool)>)> {
        vec![
            (
                "Normal operation".to_string(),
                vec![
                    (1000, 1001, false), // 1ms late, within tolerance
                    (2000, 2002, false), // 2ms late, within tolerance
                    (3000, 2999, false), // 1ms early, within tolerance
                ],
            ),
            (
                "Timing violations".to_string(),
                vec![
                    (1000, 1015, true), // 15ms late, violation
                    (2000, 1980, true), // 20ms early, violation
                    (3000, 3025, true), // 25ms late, violation
                ],
            ),
            (
                "Recovery scenario".to_string(),
                vec![
                    (1000, 1020, true),  // Violation
                    (2000, 2001, false), // Recovery
                    (3000, 3002, false), // Normal
                ],
            ),
        ]
    }

    /// Generate real-time constraint test data
    pub fn realtime_constraint_data(deadline_us: u32, count: usize) -> Vec<(u32, u32, bool)> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        (0..count)
            .map(|i| {
                let start_time = i as u32 * deadline_us;

                // Generate execution times with some that exceed deadline
                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                let hash = hasher.finish();

                let base_execution = deadline_us / 2; // Normally use 50% of deadline
                let variation = (hash % (deadline_us as u64 / 2)) as u32; // Up to 50% variation
                let execution_time = base_execution + variation;

                let deadline_met = execution_time <= deadline_us;

                (start_time, execution_time, deadline_met)
            })
            .collect()
    }

    /// Generate system timing profile for different load conditions
    pub fn system_timing_profile(duration_ms: u32) -> Vec<(u32, f32, f32, f32)> {
        let sample_interval_ms = 100; // Sample every 100ms
        let num_samples = duration_ms / sample_interval_ms;

        (0..num_samples)
            .map(|i| {
                let timestamp = i * sample_interval_ms;

                // Simulate varying system load over time
                let time_factor = (i as f32) / (num_samples as f32);
                let load_cycle = (time_factor * 2.0 * std::f32::consts::PI).sin();

                let cpu_usage = 30.0 + (load_cycle * 20.0); // 10-50% CPU usage
                let memory_usage = 40.0 + (load_cycle * 15.0); // 25-55% memory usage
                let timing_accuracy = 95.0 - (load_cycle.abs() * 10.0); // 85-95% accuracy

                (timestamp, cpu_usage, memory_usage, timing_accuracy)
            })
            .collect()
    }

    /// Generate interrupt latency measurements
    pub fn interrupt_latency_data(count: usize, base_latency_us: u32) -> Vec<(u32, u32, String)> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let interrupt_types = ["Timer", "USB", "ADC", "GPIO", "DMA"];

        (0..count)
            .map(|i| {
                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                let hash = hasher.finish();

                let interrupt_type = interrupt_types[i % interrupt_types.len()];
                let latency_variation = (hash % (base_latency_us as u64 / 2)) as u32;
                let measured_latency = base_latency_us + latency_variation;
                let timestamp = i as u32 * 1000; // Every 1ms

                (timestamp, measured_latency, interrupt_type.to_string())
            })
            .collect()
    }

    /// Generate task scheduling timing data
    pub fn task_scheduling_data(tasks: &[&str], duration_ms: u32) -> Vec<(u32, String, u32, u32)> {
        let mut schedule = Vec::new();
        let mut current_time = 0u32;

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        while current_time < duration_ms * 1000 {
            // Convert to microseconds
            for (task_idx, &task_name) in tasks.iter().enumerate() {
                let mut hasher = DefaultHasher::new();
                (current_time + task_idx as u32).hash(&mut hasher);
                let hash = hasher.finish();

                let base_duration = match task_name {
                    "battery_monitor" => 500u32, // 500µs
                    "usb_poll" => 200u32,        // 200µs
                    "pemf_control" => 100u32,    // 100µs
                    "logging" => 300u32,         // 300µs
                    _ => 250u32,                 // 250µs default
                };

                let duration_variation = (hash % (base_duration as u64 / 2)) as u32;
                let actual_duration = base_duration + duration_variation;

                schedule.push((
                    current_time,
                    task_name.to_string(),
                    base_duration,
                    actual_duration,
                ));
                current_time += actual_duration;

                if current_time >= duration_ms * 1000 {
                    break;
                }
            }
        }

        schedule
    }

    /// Generate timing benchmark results for different operations
    pub fn benchmark_timing_results() -> Vec<(String, Vec<Duration>)> {
        vec![
            (
                "battery_read".to_string(),
                performance_measurements(Duration::from_micros(50), 10.0, 100),
            ),
            (
                "usb_transmit".to_string(),
                performance_measurements(Duration::from_micros(200), 15.0, 100),
            ),
            (
                "log_format".to_string(),
                performance_measurements(Duration::from_micros(30), 5.0, 100),
            ),
            (
                "config_update".to_string(),
                performance_measurements(Duration::from_micros(100), 20.0, 50),
            ),
            (
                "state_machine".to_string(),
                performance_measurements(Duration::from_micros(25), 8.0, 200),
            ),
        ]
    }
}

/// System state test data generators
pub mod system_state {
    use std::collections::HashMap;

    /// Generate typical system state data
    pub fn typical_system_state() -> HashMap<String, String> {
        let mut state = HashMap::new();
        state.insert("uptime_ms".to_string(), "12345".to_string());
        state.insert("battery_voltage".to_string(), "3700".to_string());
        state.insert("usb_connected".to_string(), "true".to_string());
        state.insert("log_level".to_string(), "2".to_string());
        state.insert("pemf_frequency".to_string(), "10.0".to_string());
        state
    }

    /// Generate system state progression over time
    pub fn state_progression(steps: usize) -> Vec<HashMap<String, String>> {
        (0..steps)
            .map(|i| {
                let mut state = typical_system_state();
                state.insert("uptime_ms".to_string(), (i * 1000).to_string());
                state.insert("step".to_string(), i.to_string());
                state
            })
            .collect()
    }
}

/// Performance test data generators
pub mod performance {
    #[cfg(any(test, feature = "std-testing"))]
    use ass_easy_loop::logging::{
        CpuUsageStats, MemoryUsageStats, MessagePerformanceStats, PerformanceStats,
        TimingImpactStats,
    };
    use std::time::Duration;

    /// Generate memory usage data for testing
    pub fn memory_usage_sequence(base_bytes: u32, growth_bytes: u32, steps: usize) -> Vec<u32> {
        (0..steps)
            .map(|i| base_bytes + (i as u32 * growth_bytes))
            .collect()
    }

    /// Generate CPU usage percentages for testing
    pub fn cpu_usage_sequence(base_percent: f32, variance: f32, steps: usize) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        (0..steps)
            .map(|i| {
                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                let hash = hasher.finish();
                let variance_factor = (hash % 2000) as f32 / 1000.0 - 1.0; // -1.0 to 1.0
                (base_percent + (variance * variance_factor))
                    .max(0.0)
                    .min(100.0)
            })
            .collect()
    }

    /// Generate performance benchmark results
    pub fn benchmark_results(operation_name: &str, iterations: usize) -> Vec<(String, Duration)> {
        let base_duration = Duration::from_micros(100);
        let measurements = super::timing::performance_measurements(base_duration, 10.0, iterations);

        measurements
            .into_iter()
            .enumerate()
            .map(|(i, duration)| (format!("{}_{}", operation_name, i), duration))
            .collect()
    }

    /// Generate comprehensive performance statistics for testing
    #[cfg(any(test, feature = "std-testing"))]
    pub fn mock_performance_stats() -> PerformanceStats {
        PerformanceStats {
            usb_cpu_usage: CpuUsageStats {
                usb_poll_cpu_percent: 15,
                usb_hid_cpu_percent: 8,
                total_usb_cpu_percent: 23,
                peak_usb_cpu_percent: 35,
                measurement_count: 1000,
                average_cpu_percent: 20,
            },
            memory_usage: MemoryUsageStats {
                queue_memory_bytes: 2048,
                peak_queue_memory_bytes: 3072,
                usb_buffer_memory_bytes: 512,
                total_memory_bytes: 2560,
                memory_utilization_percent: 1, // ~1% of 264KB
                allocation_count: 0,
            },
            message_performance: MessagePerformanceStats {
                avg_format_time_us: 25,
                avg_enqueue_time_us: 15,
                avg_transmission_time_us: 200,
                peak_processing_time_us: 350,
                messages_processed: 5000,
                transmission_failures: 5,
            },
            timing_impact: TimingImpactStats {
                pemf_timing_deviation_us: 500,
                battery_timing_deviation_us: 200,
                max_timing_deviation_us: 500,
                timing_accuracy_percent: 98,
                timing_violations: 2,
            },
        }
    }

    /// Generate performance statistics progression over time
    #[cfg(any(test, feature = "std-testing"))]
    pub fn performance_stats_progression(duration_minutes: u32) -> Vec<(u32, PerformanceStats)> {
        let samples_per_minute = 6; // Every 10 seconds
        let total_samples = duration_minutes * samples_per_minute;

        (0..total_samples)
            .map(|i| {
                let timestamp_ms = (i * 10 * 1000) / samples_per_minute; // Every 10 seconds
                let time_factor = i as f32 / total_samples as f32;

                // Simulate performance degradation over time
                let cpu_load_increase = (time_factor * 10.0) as u8; // Up to 10% increase
                let memory_growth = (time_factor * 1024.0) as usize; // Up to 1KB growth

                let stats = PerformanceStats {
                    usb_cpu_usage: CpuUsageStats {
                        usb_poll_cpu_percent: 15 + cpu_load_increase,
                        usb_hid_cpu_percent: 8 + (cpu_load_increase / 2),
                        total_usb_cpu_percent: 23 + cpu_load_increase + (cpu_load_increase / 2),
                        peak_usb_cpu_percent: 35 + cpu_load_increase,
                        measurement_count: (i + 1) * 10,
                        average_cpu_percent: 20 + (cpu_load_increase / 2),
                    },
                    memory_usage: MemoryUsageStats {
                        queue_memory_bytes: 2048 + memory_growth,
                        peak_queue_memory_bytes: 3072 + memory_growth,
                        usb_buffer_memory_bytes: 512,
                        total_memory_bytes: 2560 + memory_growth,
                        memory_utilization_percent: ((2560 + memory_growth) * 100 / (264 * 1024))
                            as u8,
                        allocation_count: 0,
                    },
                    message_performance: MessagePerformanceStats {
                        avg_format_time_us: 25 + (time_factor * 5.0) as u32,
                        avg_enqueue_time_us: 15 + (time_factor * 3.0) as u32,
                        avg_transmission_time_us: 200 + (time_factor * 50.0) as u32,
                        peak_processing_time_us: 350 + (time_factor * 100.0) as u32,
                        messages_processed: (i + 1) * 50,
                        transmission_failures: (time_factor * 10.0) as u32,
                    },
                    timing_impact: TimingImpactStats {
                        pemf_timing_deviation_us: 500 + (time_factor * 200.0) as u32,
                        battery_timing_deviation_us: 200 + (time_factor * 100.0) as u32,
                        max_timing_deviation_us: 500 + (time_factor * 200.0) as u32,
                        timing_accuracy_percent: (98.0 - (time_factor * 5.0)) as u8,
                        timing_violations: (time_factor * 10.0) as u32,
                    },
                };

                (timestamp_ms, stats)
            })
            .collect()
    }

    /// Generate stress test performance data
    pub fn stress_test_performance_data(
        load_level: f32,
        duration_seconds: u32,
    ) -> Vec<(u32, f32, f32, u32)> {
        let samples_per_second = 10;
        let total_samples = duration_seconds * samples_per_second;

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        (0..total_samples)
            .map(|i| {
                let timestamp_ms = (i * 1000) / samples_per_second;

                // Generate pseudo-random variations
                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                let hash = hasher.finish();
                let random_factor = (hash % 1000) as f32 / 1000.0;

                // CPU usage increases with load and has random spikes
                let base_cpu = 20.0 + (load_level * 50.0);
                let cpu_spike = if random_factor > 0.9 { 20.0 } else { 0.0 };
                let cpu_usage = (base_cpu + cpu_spike + (random_factor * 10.0)).min(100.0);

                // Memory usage grows over time under stress
                let memory_growth = load_level * (i as f32 / total_samples as f32) * 30.0;
                let memory_usage = 40.0 + memory_growth + (random_factor * 5.0);

                // Response time increases with load
                let base_response = 100 + (load_level * 200.0) as u32;
                let response_variation = (random_factor * 50.0) as u32;
                let response_time_us = base_response + response_variation;

                (timestamp_ms, cpu_usage, memory_usage, response_time_us)
            })
            .collect()
    }

    /// Generate memory leak simulation data
    pub fn memory_leak_simulation(
        leak_rate_bytes_per_sec: u32,
        duration_seconds: u32,
    ) -> Vec<(u32, u32, u32)> {
        let samples_per_second = 4; // Every 250ms
        let total_samples = duration_seconds * samples_per_second;

        (0..total_samples)
            .map(|i| {
                let timestamp_ms = (i * 1000) / samples_per_second;
                let elapsed_seconds = timestamp_ms / 1000;

                let base_memory = 50 * 1024; // 50KB baseline
                let leaked_memory = elapsed_seconds * leak_rate_bytes_per_sec;
                let total_memory = base_memory + leaked_memory;

                (timestamp_ms, base_memory, total_memory)
            })
            .collect()
    }

    /// Generate throughput performance data
    pub fn throughput_performance_data(
        target_ops_per_sec: u32,
        duration_seconds: u32,
    ) -> Vec<(u32, u32, f32)> {
        let samples_per_second = 5; // Every 200ms
        let total_samples = duration_seconds * samples_per_second;

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        (0..total_samples)
            .map(|i| {
                let timestamp_ms = (i * 1000) / samples_per_second;

                // Simulate throughput variations
                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                let hash = hasher.finish();
                let variation_factor = ((hash % 2000) as f32 / 1000.0) - 1.0; // -1.0 to 1.0

                let actual_ops =
                    ((target_ops_per_sec as f32) * (1.0 + variation_factor * 0.2)).max(0.0) as u32;
                let efficiency = (actual_ops as f32 / target_ops_per_sec as f32 * 100.0).min(100.0);

                (timestamp_ms, actual_ops, efficiency)
            })
            .collect()
    }

    /// Generate latency distribution data
    pub fn latency_distribution_data(
        operation: &str,
        sample_count: usize,
    ) -> Vec<(String, Duration, u32)> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let base_latency_us = match operation {
            "usb_transmit" => 200,
            "battery_read" => 50,
            "log_format" => 30,
            "config_update" => 100,
            _ => 75,
        };

        (0..sample_count)
            .map(|i| {
                let mut hasher = DefaultHasher::new();
                (operation, i).hash(&mut hasher);
                let hash = hasher.finish();

                // Generate latency with realistic distribution (mostly low, some high)
                let percentile = (hash % 100) as f32 / 100.0;
                let latency_multiplier = if percentile < 0.5 {
                    0.8 + (percentile * 0.4) // 50% of samples: 0.8x to 1.0x
                } else if percentile < 0.9 {
                    1.0 + ((percentile - 0.5) * 1.0) // 40% of samples: 1.0x to 1.4x
                } else {
                    1.4 + ((percentile - 0.9) * 6.0) // 10% of samples: 1.4x to 2.0x
                };

                let latency_us = (base_latency_us as f32 * latency_multiplier) as u64;
                let frequency = if percentile < 0.5 {
                    100
                } else if percentile < 0.9 {
                    50
                } else {
                    10
                };

                (
                    operation.to_string(),
                    Duration::from_micros(latency_us),
                    frequency,
                )
            })
            .collect()
    }

    /// Generate resource utilization over time
    pub fn resource_utilization_timeline(duration_minutes: u32) -> Vec<(u32, f32, f32, f32, f32)> {
        let samples_per_minute = 12; // Every 5 seconds
        let total_samples = duration_minutes * samples_per_minute;

        (0..total_samples)
            .map(|i| {
                let timestamp_ms = (i * 5 * 1000) / samples_per_minute;
                let time_factor = (i as f32) / (total_samples as f32);

                // Simulate daily usage patterns
                let daily_cycle = (time_factor * 2.0 * std::f32::consts::PI).sin();

                let cpu_usage = 25.0 + (daily_cycle * 15.0) + (time_factor * 5.0); // 10-45%
                let memory_usage = 35.0 + (daily_cycle * 10.0) + (time_factor * 10.0); // 25-55%
                let disk_usage = 60.0 + (time_factor * 20.0); // 60-80% (growing)
                let network_usage = 15.0 + (daily_cycle.abs() * 25.0); // 15-40%

                (
                    timestamp_ms,
                    cpu_usage,
                    memory_usage,
                    disk_usage,
                    network_usage,
                )
            })
            .collect()
    }

    /// Generate performance regression test data
    pub fn performance_regression_data() -> Vec<(String, Duration, Duration, f32)> {
        vec![
            (
                "battery_read".to_string(),
                Duration::from_micros(45),
                Duration::from_micros(50),
                11.1,
            ),
            (
                "usb_transmit".to_string(),
                Duration::from_micros(180),
                Duration::from_micros(200),
                11.1,
            ),
            (
                "log_format".to_string(),
                Duration::from_micros(25),
                Duration::from_micros(30),
                20.0,
            ),
            (
                "config_update".to_string(),
                Duration::from_micros(90),
                Duration::from_micros(100),
                11.1,
            ),
            (
                "state_transition".to_string(),
                Duration::from_micros(20),
                Duration::from_micros(25),
                25.0,
            ),
            (
                "queue_operation".to_string(),
                Duration::from_micros(15),
                Duration::from_micros(15),
                0.0,
            ),
            (
                "error_handling".to_string(),
                Duration::from_micros(35),
                Duration::from_micros(40),
                14.3,
            ),
        ]
    }

    /// Generate load testing scenarios
    pub fn load_testing_scenarios() -> Vec<(String, u32, u32, f32)> {
        vec![
            ("Light load".to_string(), 10, 60, 95.0), // 10 ops/sec for 60s, 95% success
            ("Normal load".to_string(), 50, 300, 98.0), // 50 ops/sec for 5min, 98% success
            ("Heavy load".to_string(), 100, 180, 92.0), // 100 ops/sec for 3min, 92% success
            ("Peak load".to_string(), 200, 60, 85.0), // 200 ops/sec for 1min, 85% success
            ("Burst load".to_string(), 500, 30, 70.0), // 500 ops/sec for 30s, 70% success
        ]
    }
}

/// Error condition test data generators
pub mod errors {
    /// Generate various error conditions for testing
    pub fn error_scenarios() -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "battery_critical",
                "Battery voltage below critical threshold",
            ),
            ("usb_disconnected", "USB connection lost"),
            ("flash_write_failed", "Failed to write to flash memory"),
            ("invalid_command", "Received invalid USB command"),
            ("memory_exhausted", "System memory exhausted"),
            ("timing_violation", "Real-time timing constraint violated"),
        ]
    }

    /// Generate error recovery scenarios
    pub fn recovery_scenarios() -> Vec<(&'static str, Vec<&'static str>)> {
        vec![
            (
                "battery_recovery",
                vec!["detect_low_battery", "enter_power_save", "wait_for_charge"],
            ),
            (
                "usb_recovery",
                vec![
                    "detect_disconnect",
                    "buffer_data",
                    "reconnect",
                    "flush_buffer",
                ],
            ),
            (
                "memory_recovery",
                vec!["detect_low_memory", "garbage_collect", "reduce_buffers"],
            ),
        ]
    }
}

/// Utility functions for test data generation
pub mod utils {
    /// Generate a timestamp for testing
    pub fn test_timestamp() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    /// Generate a unique test ID
    pub fn test_id() -> String {
        format!("test_{}", test_timestamp())
    }

    /// Create test data with specified size
    pub fn test_data_bytes(size: usize) -> Vec<u8> {
        (0..size).map(|i| (i % 256) as u8).collect()
    }

    /// Generate test configuration data
    pub fn test_config_json() -> String {
        r#"{
            "log_level": 2,
            "battery_threshold": 3200,
            "usb_timeout_ms": 5000,
            "pemf_frequency": 10.0,
            "test_mode": true
        }"#
        .to_string()
    }
}
