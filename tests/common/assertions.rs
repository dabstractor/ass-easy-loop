//! Custom assertions and test utilities
//!
//! This module provides custom assertion macros and helper functions
//! that make testing more convenient and provide better error messages.

/// Assert that a value is within a specified range
#[macro_export]
macro_rules! assert_in_range {
    ($value:expr, $min:expr, $max:expr) => {
        assert!(
            $value >= $min && $value <= $max,
            "Value {} is not in range [{}, {}]",
            $value,
            $min,
            $max
        )
    };
    ($value:expr, $min:expr, $max:expr, $msg:expr) => {
        assert!(
            $value >= $min && $value <= $max,
            "{}: Value {} is not in range [{}, {}]",
            $msg,
            $value,
            $min,
            $max
        )
    };
}

/// Assert that a floating point value is approximately equal to another
#[macro_export]
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr, $epsilon:expr) => {
        let diff = (($left as f64) - ($right as f64)).abs();
        assert!(
            diff <= $epsilon,
            "Values are not approximately equal: {} != {} (diff: {}, epsilon: {})",
            $left,
            $right,
            diff,
            $epsilon
        );
    };
    ($left:expr, $right:expr) => {
        assert_approx_eq!($left, $right, 1e-6);
    };
}

/// Assert that a duration is within expected bounds
#[macro_export]
macro_rules! assert_duration_in_range {
    ($duration:expr, $min:expr, $max:expr) => {
        assert!(
            $duration >= $min && $duration <= $max,
            "Duration {:?} is not in range [{:?}, {:?}]",
            $duration,
            $min,
            $max
        );
    };
}

/// Assert that a collection contains all expected items
#[macro_export]
macro_rules! assert_contains_all {
    ($collection:expr, $expected:expr) => {
        for item in $expected {
            assert!(
                $collection.contains(&item),
                "Collection does not contain expected item: {:?}",
                item
            );
        }
    };
}

/// Assert that a collection has the expected length
#[macro_export]
macro_rules! assert_len {
    ($collection:expr, $expected_len:expr) => {
        assert_eq!(
            $collection.len(),
            $expected_len,
            "Collection has length {}, expected {}",
            $collection.len(),
            $expected_len
        );
    };
}

/// Assert that a result is Ok and return the value
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {
        match $result {
            Ok(val) => val,
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
    };
}

/// Assert that a result is Err and return the error
#[macro_export]
macro_rules! assert_err {
    ($result:expr) => {
        match $result {
            Ok(val) => panic!("Expected Err, got Ok: {:?}", val),
            Err(e) => e,
        }
    };
}

/// Assert that a value matches a pattern
#[macro_export]
macro_rules! assert_matches {
    ($value:expr, $pattern:pat) => {
        match $value {
            $pattern => {}
            ref v => panic!(
                "Value {:?} does not match pattern {}",
                v,
                stringify!($pattern)
            ),
        }
    };
    ($value:expr, $pattern:pat, $msg:expr) => {
        match $value {
            $pattern => {}
            ref v => panic!(
                "{}: Value {:?} does not match pattern {}",
                $msg,
                v,
                stringify!($pattern)
            ),
        }
    };
}

/// Assert that a closure eventually returns true within a timeout
#[macro_export]
macro_rules! assert_eventually {
    ($condition:expr, $timeout:expr) => {{
        let start = std::time::Instant::now();
        let timeout_duration = $timeout;

        loop {
            if $condition {
                break;
            }

            if start.elapsed() > timeout_duration {
                panic!(
                    "Condition did not become true within {:?}",
                    timeout_duration
                );
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }};
    ($condition:expr, $timeout:expr, $msg:expr) => {{
        let start = std::time::Instant::now();
        let timeout_duration = $timeout;

        loop {
            if $condition {
                break;
            }

            if start.elapsed() > timeout_duration {
                panic!(
                    "{}: Condition did not become true within {:?}",
                    $msg, timeout_duration
                );
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }};
}

/// Assert that a value is monotonically increasing
#[macro_export]
macro_rules! assert_monotonic_increasing {
    ($values:expr) => {
        for window in $values.windows(2) {
            assert!(
                window[1] >= window[0],
                "Values are not monotonically increasing: {} followed by {}",
                window[0],
                window[1]
            );
        }
    };
}

/// Assert that a value is monotonically decreasing
#[macro_export]
macro_rules! assert_monotonic_decreasing {
    ($values:expr) => {
        for window in $values.windows(2) {
            assert!(
                window[1] <= window[0],
                "Values are not monotonically decreasing: {} followed by {}",
                window[0],
                window[1]
            );
        }
    };
}

/// Assert that all values in a collection satisfy a predicate
#[macro_export]
macro_rules! assert_all {
    ($collection:expr, $predicate:expr) => {
        for (i, item) in $collection.iter().enumerate() {
            assert!(
                $predicate(item),
                "Item at index {} does not satisfy predicate: {:?}",
                i,
                item
            );
        }
    };
}

/// Assert that any value in a collection satisfies a predicate
#[macro_export]
macro_rules! assert_any {
    ($collection:expr, $predicate:expr) => {
        let found = $collection.iter().any($predicate);
        assert!(found, "No item in collection satisfies predicate");
    };
}

/// Assert that a collection is sorted
#[macro_export]
macro_rules! assert_sorted {
    ($collection:expr) => {
        for window in $collection.windows(2) {
            assert!(
                window[0] <= window[1],
                "Collection is not sorted: {:?} > {:?}",
                window[0],
                window[1]
            );
        }
    };
}

/// Assert that two collections have the same elements (order doesn't matter)
#[macro_export]
macro_rules! assert_same_elements {
    ($left:expr, $right:expr) => {{
        let mut left_sorted = $left.clone();
        let mut right_sorted = $right.clone();
        left_sorted.sort();
        right_sorted.sort();
        assert_eq!(
            left_sorted, right_sorted,
            "Collections do not have the same elements"
        );
    }};
}

/// Battery-specific assertions
pub mod battery {
    // use std::time::Duration;

    /// Assert that battery voltage is in a valid range
    pub fn assert_valid_battery_voltage(voltage_mv: u32) {
        assert_in_range!(voltage_mv, 2500, 4500, "Battery voltage out of valid range");
    }

    /// Assert that battery state matches expected voltage range
    pub fn assert_battery_state_consistent(voltage_mv: u32, state: &str) {
        match state {
            "low" => assert!(
                voltage_mv < 3100,
                "Low state but voltage {} is too high",
                voltage_mv
            ),
            "normal" => {
                assert_in_range!(voltage_mv, 3100, 3600, "Normal state voltage inconsistent")
            }
            "charging" => assert!(
                voltage_mv >= 3600,
                "Charging state but voltage {} is too low",
                voltage_mv
            ),
            _ => panic!("Unknown battery state: {}", state),
        }
    }

    /// Assert that battery readings show expected trend
    pub fn assert_battery_trend(readings: &[u32], expected_trend: BatteryTrend) {
        if readings.len() < 2 {
            return;
        }

        let mut increasing = 0;
        let mut decreasing = 0;

        for window in readings.windows(2) {
            if window[1] > window[0] {
                increasing += 1;
            } else if window[1] < window[0] {
                decreasing += 1;
            }
        }

        match expected_trend {
            BatteryTrend::Charging => assert!(
                increasing > decreasing,
                "Expected charging trend but found more decreasing readings"
            ),
            BatteryTrend::Discharging => assert!(
                decreasing > increasing,
                "Expected discharging trend but found more increasing readings"
            ),
            BatteryTrend::Stable => assert!(
                (increasing as i32 - decreasing as i32).abs() <= 1,
                "Expected stable trend but found significant changes"
            ),
        }
    }

    /// Assert that ADC readings are within expected noise levels
    pub fn assert_adc_noise_within_bounds(readings: &[u16], max_noise: u16) {
        if readings.len() < 2 {
            return;
        }

        for window in readings.windows(2) {
            let diff = if window[1] > window[0] {
                window[1] - window[0]
            } else {
                window[0] - window[1]
            };

            assert!(
                diff <= max_noise,
                "ADC noise {} exceeds maximum allowed noise {}",
                diff,
                max_noise
            );
        }
    }

    /// Assert that battery state transitions are valid
    pub fn assert_valid_state_transition(from_state: &str, to_state: &str) {
        let valid_transitions = [
            ("low", "normal"),
            ("normal", "low"),
            ("normal", "charging"),
            ("charging", "normal"),
            // Same state transitions are always valid
            ("low", "low"),
            ("normal", "normal"),
            ("charging", "charging"),
        ];

        let transition_valid = valid_transitions
            .iter()
            .any(|(from, to)| *from == from_state && *to == to_state);

        assert!(
            transition_valid,
            "Invalid battery state transition from '{}' to '{}'",
            from_state, to_state
        );
    }

    /// Assert that battery monitoring frequency is within expected range
    pub fn assert_monitoring_frequency(
        timestamps: &[u32],
        expected_interval_ms: u32,
        tolerance_percent: f32,
    ) {
        if timestamps.len() < 2 {
            return;
        }

        let tolerance_ms = (expected_interval_ms as f32 * tolerance_percent / 100.0) as u32;
        let min_interval = expected_interval_ms.saturating_sub(tolerance_ms);
        let max_interval = expected_interval_ms + tolerance_ms;

        for window in timestamps.windows(2) {
            let actual_interval = window[1] - window[0];
            assert_in_range!(
                actual_interval,
                min_interval,
                max_interval,
                "Battery monitoring interval out of tolerance"
            );
        }
    }

    /// Assert that battery discharge curve is realistic
    pub fn assert_realistic_discharge_curve(voltage_readings: &[(u32, u32)]) {
        // voltage_readings: (timestamp_ms, voltage_mv)
        if voltage_readings.len() < 3 {
            return;
        }

        // Check that voltage generally decreases over time (allowing for some noise)
        let start_voltage = voltage_readings[0].1;
        let end_voltage = voltage_readings[voltage_readings.len() - 1].1;

        // Allow for some measurement noise, but overall trend should be downward
        let max_allowed_increase = start_voltage / 20; // 5% increase allowed for noise

        assert!(
            end_voltage <= start_voltage + max_allowed_increase,
            "Battery discharge curve shows unrealistic voltage increase: {} -> {}",
            start_voltage,
            end_voltage
        );

        // Check that discharge rate is reasonable (not too fast)
        let duration_hours = (voltage_readings[voltage_readings.len() - 1].0
            - voltage_readings[0].0) as f32
            / (1000.0 * 3600.0);
        let voltage_drop = start_voltage.saturating_sub(end_voltage);
        let discharge_rate_mv_per_hour = voltage_drop as f32 / duration_hours;

        // Typical lithium battery shouldn't discharge faster than 500mV/hour under normal load
        assert!(
            discharge_rate_mv_per_hour <= 500.0,
            "Battery discharge rate {} mV/hour is unrealistically fast",
            discharge_rate_mv_per_hour
        );
    }

    /// Assert that battery calibration is accurate
    pub fn assert_calibration_accuracy(
        measured_voltages: &[u32],
        reference_voltages: &[u32],
        max_error_percent: f32,
    ) {
        assert_eq!(
            measured_voltages.len(),
            reference_voltages.len(),
            "Measured and reference voltage arrays must have same length"
        );

        for (i, (&measured, &reference)) in measured_voltages
            .iter()
            .zip(reference_voltages.iter())
            .enumerate()
        {
            let error_percent =
                ((measured as f32 - reference as f32).abs() / reference as f32) * 100.0;

            assert!(
                error_percent <= max_error_percent,
                "Battery calibration error at index {}: {:.2}% exceeds maximum {:.2}%",
                i,
                error_percent,
                max_error_percent
            );
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum BatteryTrend {
        Charging,
        Discharging,
        Stable,
    }
}

/// USB HID specific assertions
pub mod usb_hid {
    use std::time::Duration;

    /// Assert that a USB HID message has the expected format
    pub fn assert_valid_hid_message(message: &[u8]) {
        assert!(!message.is_empty(), "HID message cannot be empty");
        assert!(
            message.len() <= 64,
            "HID message too long: {} bytes",
            message.len()
        );
    }

    /// Assert that USB HID command is valid
    pub fn assert_valid_hid_command(command: u8) {
        assert_in_range!(command, 0x01, 0x07, "Invalid HID command");
    }

    /// Assert that USB HID response matches expected pattern
    pub fn assert_hid_response_format(response: &[u8], expected_command: u8) {
        assert_valid_hid_message(response);
        assert_eq!(response[0], expected_command, "Response command mismatch");
    }

    /// Assert that USB HID message sequence is valid
    pub fn assert_valid_message_sequence(messages: &[Vec<u8>]) {
        assert!(!messages.is_empty(), "Message sequence cannot be empty");

        for (i, message) in messages.iter().enumerate() {
            assert_valid_hid_message(message);

            // Additional sequence validation
            if i > 0 {
                // Check for reasonable timing between messages (if timestamps were included)
                // This is a placeholder for more sophisticated sequence validation
            }
        }
    }

    /// Assert that USB connection state is valid
    pub fn assert_valid_connection_state(connected: bool, configured: bool, enumerated: bool) {
        if configured {
            assert!(
                connected,
                "Device cannot be configured without being connected"
            );
        }

        if enumerated {
            assert!(
                connected,
                "Device cannot be enumerated without being connected"
            );
        }
    }

    /// Assert that USB bandwidth usage is within limits
    pub fn assert_bandwidth_within_limits(bytes_per_second: u32, max_bandwidth: u32) {
        assert!(
            bytes_per_second <= max_bandwidth,
            "USB bandwidth usage {} bytes/s exceeds limit {} bytes/s",
            bytes_per_second,
            max_bandwidth
        );
    }

    /// Assert that USB HID report has correct structure
    pub fn assert_hid_report_structure(
        report: &[u8],
        expected_report_id: u8,
        min_data_length: usize,
    ) {
        assert_valid_hid_message(report);
        assert_eq!(report[0], expected_report_id, "HID report ID mismatch");
        assert!(
            report.len() >= min_data_length + 1, // +1 for report ID
            "HID report data too short: {} bytes, expected at least {}",
            report.len() - 1,
            min_data_length
        );
    }

    /// Assert that USB error recovery is working
    pub fn assert_error_recovery(
        error_count_before: u32,
        error_count_after: u32,
        max_allowed_errors: u32,
    ) {
        let new_errors = error_count_after.saturating_sub(error_count_before);
        assert!(
            new_errors <= max_allowed_errors,
            "Too many USB errors occurred: {} new errors, maximum allowed: {}",
            new_errors,
            max_allowed_errors
        );
    }

    /// Assert that USB communication latency is acceptable
    pub fn assert_communication_latency(latencies: &[Duration], max_latency: Duration) {
        for (i, &latency) in latencies.iter().enumerate() {
            assert!(
                latency <= max_latency,
                "USB communication latency at index {} ({:?}) exceeds maximum ({:?})",
                i,
                latency,
                max_latency
            );
        }
    }

    /// Assert that USB HID configuration is valid
    pub fn assert_valid_hid_config(vendor_id: u16, product_id: u16, report_size: usize) {
        assert_ne!(vendor_id, 0, "Vendor ID cannot be zero");
        assert_ne!(product_id, 0, "Product ID cannot be zero");
        assert_in_range!(report_size, 1, 64, "HID report size out of valid range");
    }

    /// Assert that USB message throughput meets requirements
    pub fn assert_message_throughput(
        message_count: u32,
        duration: Duration,
        min_messages_per_second: f32,
    ) {
        let actual_throughput = message_count as f32 / duration.as_secs_f32();
        assert!(
            actual_throughput >= min_messages_per_second,
            "USB message throughput {:.2} msg/s is below minimum {:.2} msg/s",
            actual_throughput,
            min_messages_per_second
        );
    }

    /// Assert that USB HID descriptor is valid
    pub fn assert_valid_hid_descriptor(descriptor: &[u8]) {
        assert!(!descriptor.is_empty(), "HID descriptor cannot be empty");
        assert!(descriptor.len() >= 9, "HID descriptor too short"); // Minimum HID descriptor length

        // Check HID descriptor header
        assert_eq!(descriptor[0], 0x09, "Invalid HID descriptor length");
        assert_eq!(descriptor[1], 0x21, "Invalid HID descriptor type");
    }

    /// Assert that USB enumeration sequence is correct
    pub fn assert_enumeration_sequence(states: &[(Duration, &str)]) {
        let expected_sequence = ["connected", "enumerated", "configured"];
        let mut sequence_index = 0;

        for (_timestamp, state) in states {
            if sequence_index < expected_sequence.len()
                && *state == expected_sequence[sequence_index]
            {
                sequence_index += 1;
            }
        }

        assert_eq!(
            sequence_index,
            expected_sequence.len(),
            "USB enumeration sequence incomplete or incorrect"
        );
    }
}

/// Timing-specific assertions
pub mod timing {
    use std::time::Duration;

    /// Assert that timing measurements are within expected bounds
    pub fn assert_timing_bounds(
        measurements: &[Duration],
        min_duration: Duration,
        max_duration: Duration,
    ) {
        for (i, &measurement) in measurements.iter().enumerate() {
            assert!(
                measurement >= min_duration && measurement <= max_duration,
                "Measurement {} ({:?}) is not in range [{:?}, {:?}]",
                i,
                measurement,
                min_duration,
                max_duration
            );
        }
    }

    /// Assert that timing measurements have acceptable variance
    pub fn assert_timing_variance(measurements: &[Duration], max_variance_percent: f32) {
        if measurements.len() < 2 {
            return;
        }

        let sum: Duration = measurements.iter().sum();
        let mean = sum / measurements.len() as u32;
        let mean_nanos = mean.as_nanos() as f64;

        for (i, &measurement) in measurements.iter().enumerate() {
            let diff = (measurement.as_nanos() as f64 - mean_nanos).abs();
            let variance_percent = (diff / mean_nanos) * 100.0;

            assert!(
                variance_percent <= max_variance_percent as f64,
                "Measurement {} has variance {:.2}%, expected <= {:.2}%",
                i,
                variance_percent,
                max_variance_percent
            );
        }
    }

    /// Assert that PEMF timing is within tolerance
    pub fn assert_pemf_timing(
        actual_frequency: f32,
        expected_frequency: f32,
        tolerance_percent: f32,
    ) {
        let tolerance = expected_frequency * (tolerance_percent / 100.0);
        let min_freq = expected_frequency - tolerance;
        let max_freq = expected_frequency + tolerance;

        assert_in_range!(
            actual_frequency,
            min_freq,
            max_freq,
            "PEMF frequency out of tolerance"
        );
    }

    /// Assert that real-time deadlines are met
    pub fn assert_deadlines_met(execution_times: &[Duration], deadline: Duration) {
        for (i, &execution_time) in execution_times.iter().enumerate() {
            assert!(
                execution_time <= deadline,
                "Task {} missed deadline: {:?} > {:?}",
                i,
                execution_time,
                deadline
            );
        }
    }

    /// Assert that timing jitter is within acceptable bounds
    pub fn assert_jitter_bounds(timestamps: &[u32], expected_interval_us: u32, max_jitter_us: u32) {
        if timestamps.len() < 2 {
            return;
        }

        for window in timestamps.windows(2) {
            let actual_interval = window[1] - window[0];
            let jitter = if actual_interval > expected_interval_us {
                actual_interval - expected_interval_us
            } else {
                expected_interval_us - actual_interval
            };

            assert!(
                jitter <= max_jitter_us,
                "Timing jitter {} µs exceeds maximum {} µs (interval: {} µs, expected: {} µs)",
                jitter,
                max_jitter_us,
                actual_interval,
                expected_interval_us
            );
        }
    }

    /// Assert that interrupt latency is acceptable
    pub fn assert_interrupt_latency(latencies: &[Duration], max_latency: Duration) {
        for (i, &latency) in latencies.iter().enumerate() {
            assert!(
                latency <= max_latency,
                "Interrupt latency {} ({:?}) exceeds maximum ({:?})",
                i,
                latency,
                max_latency
            );
        }
    }

    /// Assert that task scheduling is deterministic
    pub fn assert_deterministic_scheduling(
        scheduled_times: &[u32],
        actual_times: &[u32],
        max_deviation_us: u32,
    ) {
        assert_eq!(
            scheduled_times.len(),
            actual_times.len(),
            "Scheduled and actual time arrays must have same length"
        );

        for (i, (&scheduled, &actual)) in
            scheduled_times.iter().zip(actual_times.iter()).enumerate()
        {
            let deviation = if actual > scheduled {
                actual - scheduled
            } else {
                scheduled - actual
            };

            assert!(
                deviation <= max_deviation_us,
                "Task {} scheduling deviation {} µs exceeds maximum {} µs",
                i,
                deviation,
                max_deviation_us
            );
        }
    }

    /// Assert that timing measurements show expected periodicity
    pub fn assert_periodic_timing(
        timestamps: &[u32],
        expected_period_us: u32,
        tolerance_percent: f32,
    ) {
        if timestamps.len() < 3 {
            return;
        }

        let tolerance_us = (expected_period_us as f32 * tolerance_percent / 100.0) as u32;
        let min_period = expected_period_us.saturating_sub(tolerance_us);
        let max_period = expected_period_us + tolerance_us;

        for window in timestamps.windows(2) {
            let actual_period = window[1] - window[0];
            assert_in_range!(
                actual_period,
                min_period,
                max_period,
                "Period out of tolerance"
            );
        }
    }

    /// Assert that timing distribution is within expected parameters
    pub fn assert_timing_distribution(
        measurements: &[Duration],
        expected_mean: Duration,
        max_std_deviation: Duration,
    ) {
        if measurements.len() < 2 {
            return;
        }

        // Calculate mean
        let sum: Duration = measurements.iter().sum();
        let mean = sum / measurements.len() as u32;

        // Check mean is close to expected
        let mean_diff = if mean > expected_mean {
            mean - expected_mean
        } else {
            expected_mean - mean
        };

        assert!(
            mean_diff <= max_std_deviation,
            "Timing mean {:?} differs from expected {:?} by more than {:?}",
            mean,
            expected_mean,
            max_std_deviation
        );

        // Calculate standard deviation
        let variance_sum: f64 = measurements
            .iter()
            .map(|&d| {
                let diff = d.as_nanos() as f64 - mean.as_nanos() as f64;
                diff * diff
            })
            .sum();

        let variance = variance_sum / measurements.len() as f64;
        let std_dev = Duration::from_nanos(variance.sqrt() as u64);

        assert!(
            std_dev <= max_std_deviation,
            "Timing standard deviation {:?} exceeds maximum {:?}",
            std_dev,
            max_std_deviation
        );
    }

    /// Assert that timing measurements are monotonically increasing
    pub fn assert_monotonic_timestamps(timestamps: &[u32]) {
        for window in timestamps.windows(2) {
            assert!(
                window[1] >= window[0],
                "Timestamps are not monotonic: {} followed by {}",
                window[0],
                window[1]
            );
        }
    }

    /// Assert that system timing accuracy meets requirements
    pub fn assert_timing_accuracy(
        measured_intervals: &[u32],
        reference_intervals: &[u32],
        max_error_percent: f32,
    ) {
        assert_eq!(
            measured_intervals.len(),
            reference_intervals.len(),
            "Measured and reference interval arrays must have same length"
        );

        for (i, (&measured, &reference)) in measured_intervals
            .iter()
            .zip(reference_intervals.iter())
            .enumerate()
        {
            let error_percent =
                ((measured as f32 - reference as f32).abs() / reference as f32) * 100.0;

            assert!(
                error_percent <= max_error_percent,
                "Timing accuracy error at index {}: {:.2}% exceeds maximum {:.2}%",
                i,
                error_percent,
                max_error_percent
            );
        }
    }
}

/// Performance-specific assertions
pub mod performance {
    use std::time::Duration;

    /// Assert that memory usage is within acceptable bounds
    pub fn assert_memory_usage(current_bytes: u32, max_bytes: u32) {
        assert!(
            current_bytes <= max_bytes,
            "Memory usage {} bytes exceeds maximum {} bytes",
            current_bytes,
            max_bytes
        );
    }

    /// Assert that performance metrics meet requirements
    pub fn assert_performance_requirements(
        execution_time: Duration,
        memory_usage: u32,
        max_time: Duration,
        max_memory: u32,
    ) {
        assert_duration_in_range!(execution_time, Duration::ZERO, max_time);
        assert_memory_usage(memory_usage, max_memory);
    }

    /// Assert that performance has not regressed
    pub fn assert_no_performance_regression(
        current_time: Duration,
        baseline_time: Duration,
        max_regression_percent: f32,
    ) {
        let baseline_nanos = baseline_time.as_nanos() as f64;
        let current_nanos = current_time.as_nanos() as f64;
        let regression_percent = ((current_nanos - baseline_nanos) / baseline_nanos) * 100.0;

        assert!(
            regression_percent <= max_regression_percent as f64,
            "Performance regression of {:.2}% exceeds maximum allowed {:.2}%",
            regression_percent,
            max_regression_percent
        );
    }
}

/// System state assertions
pub mod system_state {
    use std::collections::HashMap;

    /// Assert that system state contains required fields
    pub fn assert_required_state_fields(state: &HashMap<String, String>, required_fields: &[&str]) {
        for field in required_fields {
            assert!(
                state.contains_key(*field),
                "System state missing required field: {}",
                field
            );
        }
    }

    /// Assert that system state values are valid
    pub fn assert_valid_state_values(state: &HashMap<String, String>) {
        if let Some(uptime) = state.get("uptime_ms") {
            let _uptime_val: u32 = uptime.parse().expect("Invalid uptime value");
            // Uptime is u32, so it's always >= 0
        }

        if let Some(battery) = state.get("battery_voltage") {
            let battery_val: u32 = battery.parse().expect("Invalid battery voltage");
            super::battery::assert_valid_battery_voltage(battery_val);
        }
    }
}

/// Test result validation
pub mod results {
    /// Assert that test results are complete and valid
    pub fn assert_test_results_valid<T>(results: &[T])
    where
        T: std::fmt::Debug,
    {
        assert!(!results.is_empty(), "Test results cannot be empty");

        // Additional validation can be added here based on result type
        for (_i, result) in results.iter().enumerate() {
            // Basic validation that result can be formatted (not corrupted)
            let _ = format!("{:?}", result);
        }
    }

    /// Assert that test coverage meets minimum requirements
    pub fn assert_test_coverage(
        covered_items: usize,
        total_items: usize,
        min_coverage_percent: f32,
    ) {
        let coverage_percent = (covered_items as f32 / total_items as f32) * 100.0;
        assert!(
            coverage_percent >= min_coverage_percent,
            "Test coverage {:.2}% is below minimum required {:.2}%",
            coverage_percent,
            min_coverage_percent
        );
    }
}
