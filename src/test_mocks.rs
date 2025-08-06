//! Mock components and test utilities for no_std testing environment
//! 
//! This module provides mock implementations of hardware components and system
//! interfaces that can be used in no_std unit tests. These mocks accurately
//! represent real hardware behavior for automated testing validation.
//! 
//! Requirements: 1.3, 5.2, 5.3, 6.1

use heapless::{Vec, FnvIndexMap};
use core::option::Option::{self, Some, None};
use core::result::Result::{self, Ok, Err};
use core::default::Default;
use core::clone::Clone;
use core::cmp::{PartialEq, Eq, PartialOrd, Ord};
use core::convert::From;
use core::iter::Iterator;

use crate::battery::BatteryState;
use crate::system_state::{
    SystemHealthData, TaskHealthStatus, MemoryUsageStats, ErrorCounters,
    HardwareStatusData, GpioStates, AdcReadings, UsbStatus
};
use crate::bootloader::{HardwareState, TaskPriority, TaskShutdownStatus};
use crate::error_handling::SystemError;

/// Maximum number of USB HID reports that can be stored in mock device
pub const MAX_USB_REPORTS: usize = 32;

/// Maximum size of a USB HID report in bytes
pub const MAX_REPORT_SIZE: usize = 64;

/// Maximum number of mock system events
pub const MAX_SYSTEM_EVENTS: usize = 16;

/// Maximum length for mock error messages
pub const MAX_ERROR_MESSAGE_LENGTH: usize = 64;

/// Mock error types for testing error handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockError {
    /// Device is not connected
    NotConnected,
    /// Buffer is full, cannot accept more data
    BufferFull,
    /// Invalid data provided
    InvalidData,
    /// Operation timed out
    Timeout,
    /// Hardware failure simulation
    HardwareFailure,
    /// Resource exhausted
    ResourceExhausted,
}

impl From<MockError> for SystemError {
    fn from(error: MockError) -> Self {
        match error {
            MockError::NotConnected => SystemError::HardwareError,
            MockError::BufferFull => SystemError::SystemBusy,
            MockError::InvalidData => SystemError::InvalidParameter,
            MockError::Timeout => SystemError::SystemBusy,
            MockError::HardwareFailure => SystemError::HardwareError,
            MockError::ResourceExhausted => SystemError::SystemBusy,
        }
    }
}

/// USB HID report structure for mock device
#[derive(Debug, Clone, PartialEq)]
pub struct MockUsbReport {
    /// Report data payload
    pub data: Vec<u8, MAX_REPORT_SIZE>,
    /// Timestamp when report was created (in milliseconds)
    pub timestamp_ms: u32,
    /// Report type identifier
    pub report_type: u8,
}

impl MockUsbReport {
    /// Create a new USB HID report
    pub fn new(data: &[u8], timestamp_ms: u32, report_type: u8) -> Result<Self, MockError> {
        let mut report_data = Vec::new();
        for &byte in data {
            report_data.push(byte).map_err(|_| MockError::BufferFull)?;
        }
        
        Ok(MockUsbReport {
            data: report_data,
            timestamp_ms,
            report_type,
        })
    }

    /// Get the size of the report data
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Check if report is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Mock USB HID device for testing USB communication
/// Requirements: 6.1 (USB HID communication testing)
#[derive(Debug, Clone)]
pub struct MockUsbHidDevice {
    /// Device connection state
    connected: bool,
    /// Device configuration state
    configured: bool,
    /// Device suspended state
    suspended: bool,
    /// Queue of sent reports
    sent_reports: Vec<MockUsbReport, MAX_USB_REPORTS>,
    /// Queue of received reports
    received_reports: Vec<MockUsbReport, MAX_USB_REPORTS>,
    /// Error injection settings
    error_injection_enabled: bool,
    /// Error injection rate (0-100%)
    error_injection_rate: u8,
    /// Current error injection counter
    error_counter: u32,
    /// Device statistics
    reports_sent_count: u32,
    reports_received_count: u32,
    transmission_errors: u32,
    last_activity_timestamp: u32,
}

impl Default for MockUsbHidDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl MockUsbHidDevice {
    /// Create a new mock USB HID device
    pub fn new() -> Self {
        Self {
            connected: false,
            configured: false,
            suspended: false,
            sent_reports: Vec::new(),
            received_reports: Vec::new(),
            error_injection_enabled: false,
            error_injection_rate: 0,
            error_counter: 0,
            reports_sent_count: 0,
            reports_received_count: 0,
            transmission_errors: 0,
            last_activity_timestamp: 0,
        }
    }

    /// Connect the mock device
    pub fn connect(&mut self) {
        self.connected = true;
        self.configured = false;
        self.suspended = false;
    }

    /// Disconnect the mock device
    pub fn disconnect(&mut self) {
        self.connected = false;
        self.configured = false;
        self.suspended = false;
        self.sent_reports.clear();
        self.received_reports.clear();
    }

    /// Configure the mock device
    pub fn configure(&mut self) -> Result<(), MockError> {
        if !self.connected {
            return Err(MockError::NotConnected);
        }
        self.configured = true;
        Ok(())
    }

    /// Suspend the mock device
    pub fn suspend(&mut self) {
        self.suspended = true;
    }

    /// Resume the mock device
    pub fn resume(&mut self) {
        self.suspended = false;
    }

    /// Send a USB HID report
    pub fn send_report(&mut self, data: &[u8], timestamp_ms: u32) -> Result<(), MockError> {
        if !self.connected {
            return Err(MockError::NotConnected);
        }

        if self.suspended {
            return Err(MockError::NotConnected);
        }

        // Check for error injection
        if self.should_inject_error() {
            self.transmission_errors += 1;
            return Err(MockError::HardwareFailure);
        }

        let report = MockUsbReport::new(data, timestamp_ms, 0x01)?;
        self.sent_reports.push(report).map_err(|_| MockError::BufferFull)?;
        
        self.reports_sent_count += 1;
        self.last_activity_timestamp = timestamp_ms;
        
        Ok(())
    }

    /// Receive a USB HID report (simulate host sending data to device)
    pub fn receive_report(&mut self, data: &[u8], timestamp_ms: u32) -> Result<(), MockError> {
        if !self.connected {
            return Err(MockError::NotConnected);
        }

        let report = MockUsbReport::new(data, timestamp_ms, 0x02)?;
        self.received_reports.push(report).map_err(|_| MockError::BufferFull)?;
        
        self.reports_received_count += 1;
        self.last_activity_timestamp = timestamp_ms;
        
        Ok(())
    }

    /// Get the next sent report (for testing)
    pub fn get_sent_report(&mut self) -> Option<MockUsbReport> {
        if self.sent_reports.is_empty() {
            None
        } else {
            Some(self.sent_reports.remove(0))
        }
    }

    /// Get the next received report (for device processing)
    pub fn get_received_report(&mut self) -> Option<MockUsbReport> {
        if self.received_reports.is_empty() {
            None
        } else {
            Some(self.received_reports.remove(0))
        }
    }

    /// Check if device is connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Check if device is configured
    pub fn is_configured(&self) -> bool {
        self.configured
    }

    /// Check if device is suspended
    pub fn is_suspended(&self) -> bool {
        self.suspended
    }

    /// Get number of sent reports
    pub fn sent_report_count(&self) -> usize {
        self.sent_reports.len()
    }

    /// Get number of received reports
    pub fn received_report_count(&self) -> usize {
        self.received_reports.len()
    }

    /// Enable error injection for testing error handling
    pub fn enable_error_injection(&mut self, rate_percent: u8) {
        self.error_injection_enabled = true;
        self.error_injection_rate = rate_percent.min(100);
        self.error_counter = 0;
    }

    /// Disable error injection
    pub fn disable_error_injection(&mut self) {
        self.error_injection_enabled = false;
        self.error_injection_rate = 0;
    }

    /// Check if an error should be injected based on the configured rate
    fn should_inject_error(&mut self) -> bool {
        if !self.error_injection_enabled || self.error_injection_rate == 0 {
            return false;
        }

        self.error_counter += 1;
        (self.error_counter % 100) < self.error_injection_rate as u32
    }

    /// Get USB status for system state queries
    pub fn get_usb_status(&self) -> UsbStatus {
        UsbStatus {
            connected: self.connected,
            configured: self.configured,
            suspended: self.suspended,
            enumerated: self.connected && self.configured,
            hid_reports_sent: self.reports_sent_count,
            hid_reports_received: self.reports_received_count,
            usb_errors: self.transmission_errors,
            last_activity_ms: self.last_activity_timestamp,
        }
    }

    /// Clear all reports and reset counters
    pub fn clear(&mut self) {
        self.sent_reports.clear();
        self.received_reports.clear();
        self.reports_sent_count = 0;
        self.reports_received_count = 0;
        self.transmission_errors = 0;
        self.error_counter = 0;
    }
}

/// Mock system state for testing system monitoring
/// Requirements: 5.2, 5.3 (mock system components)
#[derive(Debug, Clone)]
pub struct MockSystemState {
    /// Current system uptime in milliseconds
    uptime_ms: u32,
    /// Battery state and voltage
    battery_state: BatteryState,
    /// Battery voltage in millivolts
    battery_voltage_mv: u32,
    /// pEMF generation status
    pemf_active: bool,
    /// pEMF cycle count
    pemf_cycle_count: u32,
    /// System temperature in 0.1°C units
    system_temperature: Option<u16>,
    /// Task health status
    task_health: TaskHealthStatus,
    /// Memory usage statistics
    memory_usage: MemoryUsageStats,
    /// Error counters
    error_counters: ErrorCounters,
    /// GPIO states
    gpio_states: GpioStates,
    /// ADC readings
    adc_readings: AdcReadings,
}

impl Default for MockSystemState {
    fn default() -> Self {
        Self::new()
    }
}

impl MockSystemState {
    /// Create a new mock system state with default values
    pub fn new() -> Self {
        Self {
            uptime_ms: 0,
            battery_state: BatteryState::Normal,
            battery_voltage_mv: 3300,
            pemf_active: false,
            pemf_cycle_count: 0,
            system_temperature: Some(250), // 25.0°C
            task_health: TaskHealthStatus {
                pemf_task_healthy: true,
                battery_task_healthy: true,
                led_task_healthy: true,
                usb_poll_task_healthy: true,
                usb_hid_task_healthy: true,
                command_handler_healthy: true,
                last_health_check_ms: 0,
            },
            memory_usage: MemoryUsageStats {
                stack_usage_bytes: 2048,
                heap_usage_bytes: 0, // No heap in no_std
                log_queue_usage_bytes: 1024,
                command_queue_usage_bytes: 512,
                total_ram_usage_bytes: 3584,
                peak_ram_usage_bytes: 4096,
                memory_fragmentation_percent: 0,
            },
            error_counters: ErrorCounters {
                adc_read_errors: 0,
                gpio_operation_errors: 0,
                usb_transmission_errors: 0,
                command_parsing_errors: 0,
                timing_violations: 0,
                bootloader_entry_failures: 0,
                total_error_count: 0,
            },
            gpio_states: GpioStates {
                mosfet_pin_state: false,
                led_pin_state: false,
                adc_pin_voltage_mv: 3300,
                bootsel_pin_state: false,
                gpio_error_count: 0,
            },
            adc_readings: AdcReadings {
                battery_adc_raw: 1500,
                battery_voltage_mv: 3300,
                internal_temperature_c: 25,
                vref_voltage_mv: 3300,
                adc_calibration_offset: 0,
                adc_error_count: 0,
            },
        }
    }

    /// Update system uptime
    pub fn set_uptime(&mut self, uptime_ms: u32) {
        self.uptime_ms = uptime_ms;
        self.task_health.last_health_check_ms = uptime_ms;
    }

    /// Set battery state and voltage
    pub fn set_battery_state(&mut self, state: BatteryState, voltage_mv: u32) {
        self.battery_state = state;
        self.battery_voltage_mv = voltage_mv;
        
        // Update ADC reading to match voltage
        // Assuming 3.3V reference and voltage divider ratio of ~0.337
        let adc_raw = ((voltage_mv as f32 * 0.337 / 3300.0) * 4095.0) as u16;
        self.adc_readings.battery_adc_raw = adc_raw;
        self.adc_readings.battery_voltage_mv = voltage_mv;
    }

    /// Set pEMF generation status
    pub fn set_pemf_active(&mut self, active: bool) {
        self.pemf_active = active;
        if active {
            self.pemf_cycle_count += 1;
        }
        // Update GPIO state to reflect MOSFET control
        self.gpio_states.mosfet_pin_state = active;
    }

    /// Set system temperature
    pub fn set_temperature(&mut self, temperature_c: i16) {
        self.system_temperature = Some((temperature_c * 10) as u16);
        self.adc_readings.internal_temperature_c = temperature_c;
    }

    /// Set task health status
    pub fn set_task_health(&mut self, task: &str, healthy: bool) {
        match task {
            "pemf" => self.task_health.pemf_task_healthy = healthy,
            "battery" => self.task_health.battery_task_healthy = healthy,
            "led" => self.task_health.led_task_healthy = healthy,
            "usb_poll" => self.task_health.usb_poll_task_healthy = healthy,
            "usb_hid" => self.task_health.usb_hid_task_healthy = healthy,
            "command" => self.task_health.command_handler_healthy = healthy,
            _ => {} // Unknown task, ignore
        }
    }

    /// Increment error counter
    pub fn increment_error(&mut self, error_type: &str) {
        match error_type {
            "adc" => {
                self.error_counters.adc_read_errors += 1;
                self.adc_readings.adc_error_count += 1;
            }
            "gpio" => {
                self.error_counters.gpio_operation_errors += 1;
                self.gpio_states.gpio_error_count += 1;
            }
            "usb" => self.error_counters.usb_transmission_errors += 1,
            "command" => self.error_counters.command_parsing_errors += 1,
            "timing" => self.error_counters.timing_violations += 1,
            "bootloader" => self.error_counters.bootloader_entry_failures += 1,
            _ => {} // Unknown error type
        }
        self.error_counters.total_error_count += 1;
    }

    /// Get system health data
    pub fn get_system_health(&self) -> SystemHealthData {
        SystemHealthData {
            uptime_ms: self.uptime_ms,
            battery_state: self.battery_state.clone(),
            battery_voltage_mv: self.battery_voltage_mv,
            pemf_active: self.pemf_active,
            pemf_cycle_count: self.pemf_cycle_count,
            task_health_status: self.task_health.clone(),
            memory_usage: self.memory_usage.clone(),
            error_counts: self.error_counters.clone(),
            system_temperature: self.system_temperature,
        }
    }

    /// Get hardware status data
    pub fn get_hardware_status(&self) -> HardwareStatusData {
        use crate::system_state::{PowerStatus, SensorReadings};
        
        HardwareStatusData {
            gpio_states: self.gpio_states,
            adc_readings: self.adc_readings,
            usb_status: UsbStatus {
                connected: true,
                configured: true,
                suspended: false,
                enumerated: true,
                hid_reports_sent: 100,
                hid_reports_received: 50,
                usb_errors: self.error_counters.usb_transmission_errors,
                last_activity_ms: self.uptime_ms,
            },
            power_status: PowerStatus {
                supply_voltage_mv: 3300,
                core_voltage_mv: 1100,
                power_consumption_mw: 150,
                battery_charging: self.battery_state == BatteryState::Charging,
                low_power_mode: false,
                power_good: true,
            },
            sensor_readings: SensorReadings {
                internal_temperature_c: self.adc_readings.internal_temperature_c,
                cpu_temperature_c: self.adc_readings.internal_temperature_c + 5,
                ambient_light_level: 512,
                magnetic_field_strength: 100,
                vibration_level: 10,
            },
        }
    }

    /// Get current uptime
    pub fn get_uptime_ms(&self) -> u32 {
        self.uptime_ms
    }

    /// Get battery voltage
    pub fn get_battery_voltage(&self) -> u32 {
        self.battery_voltage_mv
    }

    /// Get pEMF status
    pub fn is_pemf_active(&self) -> bool {
        self.pemf_active
    }
}

/// Mock bootloader hardware state for testing bootloader entry
/// Requirements: 5.3 (accurate hardware behavior representation)
#[derive(Debug, Clone)]
pub struct MockBootloaderHardware {
    /// Current hardware state
    hardware_state: HardwareState,
    /// Task shutdown states
    task_states: FnvIndexMap<TaskPriority, TaskShutdownStatus, 8>,
    /// Bootloader entry simulation settings
    entry_delay_ms: u32,
    /// Force failure simulation
    force_failure: bool,
}

impl Default for MockBootloaderHardware {
    fn default() -> Self {
        Self::new()
    }
}

impl MockBootloaderHardware {
    /// Create new mock bootloader hardware
    pub fn new() -> Self {
        let mut task_states = FnvIndexMap::new();
        let _ = task_states.insert(TaskPriority::High, TaskShutdownStatus::Running);
        let _ = task_states.insert(TaskPriority::Medium, TaskShutdownStatus::Running);
        let _ = task_states.insert(TaskPriority::Low, TaskShutdownStatus::Running);

        Self {
            hardware_state: HardwareState {
                mosfet_state: false,
                led_state: false,
                adc_active: false,
                usb_transmitting: false,
                pemf_pulse_active: false,
            },
            task_states,
            entry_delay_ms: 0,
            force_failure: false,
        }
    }

    /// Set hardware component state
    pub fn set_hardware_state(&mut self, state: HardwareState) {
        self.hardware_state = state;
    }

    /// Get current hardware state
    pub fn get_hardware_state(&self) -> HardwareState {
        self.hardware_state.clone()
    }

    /// Set task shutdown status
    pub fn set_task_status(&mut self, priority: TaskPriority, status: TaskShutdownStatus) {
        let _ = self.task_states.insert(priority, status);
    }

    /// Get task shutdown status
    pub fn get_task_status(&self, priority: TaskPriority) -> TaskShutdownStatus {
        self.task_states.get(&priority).copied().unwrap_or(TaskShutdownStatus::Running)
    }

    /// Simulate pEMF pulse completion
    pub fn complete_pemf_pulse(&mut self) {
        self.hardware_state.pemf_pulse_active = false;
        self.hardware_state.mosfet_state = false;
    }

    /// Simulate task shutdown completion
    pub fn complete_task_shutdown(&mut self, priority: TaskPriority) {
        let _ = self.task_states.insert(priority, TaskShutdownStatus::ShutdownComplete);
    }

    /// Set bootloader entry delay for testing timing
    pub fn set_entry_delay(&mut self, delay_ms: u32) {
        self.entry_delay_ms = delay_ms;
    }

    /// Force bootloader entry failure for testing error handling
    pub fn force_entry_failure(&mut self, force: bool) {
        self.force_failure = force;
    }

    /// Check if bootloader entry should fail
    pub fn should_fail_entry(&self) -> bool {
        self.force_failure
    }

    /// Get entry delay
    pub fn get_entry_delay(&self) -> u32 {
        self.entry_delay_ms
    }
}
/// Test data generation utilities for embedded-friendly testing
/// Requirements: 1.3 (embedded-friendly test data generation)
#[derive(Debug, Clone)]
pub struct TestDataGenerator {
    /// Seed for pseudo-random number generation
    seed: u32,
    /// Counter for generating sequential test data
    counter: u32,
}

impl Default for TestDataGenerator {
    fn default() -> Self {
        Self::new(12345)
    }
}

impl TestDataGenerator {
    /// Create a new test data generator with a seed
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            counter: 0,
        }
    }

    /// Generate a pseudo-random u32 value
    pub fn next_u32(&mut self) -> u32 {
        // Simple linear congruential generator
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        self.counter += 1;
        self.seed
    }

    /// Generate a pseudo-random u16 value
    pub fn next_u16(&mut self) -> u16 {
        (self.next_u32() >> 16) as u16
    }

    /// Generate a pseudo-random u8 value
    pub fn next_u8(&mut self) -> u8 {
        (self.next_u32() >> 24) as u8
    }

    /// Generate a pseudo-random boolean value
    pub fn next_bool(&mut self) -> bool {
        (self.next_u32() & 1) == 1
    }

    /// Generate a pseudo-random value in a range
    pub fn next_range(&mut self, min: u32, max: u32) -> u32 {
        if min >= max {
            return min;
        }
        min + (self.next_u32() % (max - min))
    }

    /// Generate realistic battery ADC reading
    pub fn generate_battery_adc(&mut self) -> u16 {
        // Generate ADC values in realistic range (1000-2000)
        self.next_range(1000, 2000) as u16
    }

    /// Generate realistic battery voltage in millivolts
    pub fn generate_battery_voltage(&mut self) -> u32 {
        // Generate voltage values in realistic range (2.8V - 4.2V)
        self.next_range(2800, 4200)
    }

    /// Generate realistic temperature reading in Celsius
    pub fn generate_temperature(&mut self) -> i16 {
        // Generate temperature in realistic range (-10°C to 60°C)
        (self.next_range(0, 70) as i16) - 10
    }

    /// Generate test USB HID report data
    pub fn generate_usb_report(&mut self, size: usize) -> Vec<u8, MAX_REPORT_SIZE> {
        let mut data = Vec::new();
        let actual_size = size.min(MAX_REPORT_SIZE);
        
        for _ in 0..actual_size {
            let _ = data.push(self.next_u8());
        }
        
        data
    }

    /// Generate test command data
    pub fn generate_command_data(&mut self) -> Vec<u8, 32> {
        let mut data = Vec::new();
        let size = self.next_range(4, 32) as usize;
        
        for _ in 0..size {
            let _ = data.push(self.next_u8());
        }
        
        data
    }

    /// Generate realistic timing measurement
    pub fn generate_timing_us(&mut self) -> u32 {
        // Generate timing values in realistic range (10us - 10ms)
        self.next_range(10, 10000)
    }

    /// Reset the generator with a new seed
    pub fn reset(&mut self, seed: u32) {
        self.seed = seed;
        self.counter = 0;
    }

    /// Get current counter value
    pub fn get_counter(&self) -> u32 {
        self.counter
    }
}

/// Test validation utilities for verifying test results
/// Requirements: 1.3 (test data validation utilities)
#[derive(Debug, Clone)]
pub struct TestValidator {
    /// Tolerance for floating-point comparisons (in percent)
    float_tolerance_percent: f32,
    /// Tolerance for timing comparisons (in microseconds)
    timing_tolerance_us: u32,
}

impl Default for TestValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl TestValidator {
    /// Create a new test validator with default tolerances
    pub fn new() -> Self {
        Self {
            float_tolerance_percent: 1.0, // 1% tolerance
            timing_tolerance_us: 100,      // 100us tolerance
        }
    }

    /// Create a validator with custom tolerances
    pub fn with_tolerances(float_tolerance_percent: f32, timing_tolerance_us: u32) -> Self {
        Self {
            float_tolerance_percent,
            timing_tolerance_us,
        }
    }

    /// Validate that two floating-point values are approximately equal
    pub fn validate_float_approx(&self, actual: f32, expected: f32) -> bool {
        if expected == 0.0 {
            actual.abs() < 0.001 // Small absolute tolerance for zero
        } else {
            let tolerance = expected.abs() * (self.float_tolerance_percent / 100.0);
            (actual - expected).abs() <= tolerance
        }
    }

    /// Validate that two timing values are within tolerance
    pub fn validate_timing(&self, actual_us: u32, expected_us: u32) -> bool {
        let diff = if actual_us > expected_us {
            actual_us - expected_us
        } else {
            expected_us - actual_us
        };
        diff <= self.timing_tolerance_us
    }

    /// Validate that a value is within a specified range
    pub fn validate_range<T>(&self, value: T, min: T, max: T) -> bool
    where
        T: PartialOrd,
    {
        value >= min && value <= max
    }

    /// Validate battery voltage is in acceptable range
    pub fn validate_battery_voltage(&self, voltage_mv: u32) -> bool {
        self.validate_range(voltage_mv, 2500, 4500) // 2.5V to 4.5V
    }

    /// Validate ADC reading is in acceptable range
    pub fn validate_adc_reading(&self, adc_value: u16) -> bool {
        self.validate_range(adc_value, 0, 4095) // 12-bit ADC range
    }

    /// Validate temperature reading is reasonable
    pub fn validate_temperature(&self, temp_c: i16) -> bool {
        self.validate_range(temp_c, -40, 85) // Industrial temperature range
    }

    /// Validate USB report size
    pub fn validate_usb_report_size(&self, size: usize) -> bool {
        self.validate_range(size, 1, MAX_REPORT_SIZE)
    }

    /// Validate system uptime is monotonic
    pub fn validate_uptime_monotonic(&self, current_ms: u32, previous_ms: u32) -> bool {
        current_ms >= previous_ms
    }
}

/// Mock event system for testing event-driven behavior
/// Requirements: 5.2 (mock components for testing)
#[derive(Debug, Clone, PartialEq)]
pub struct MockSystemEvent {
    /// Event type identifier
    pub event_type: u8,
    /// Event timestamp
    pub timestamp_ms: u32,
    /// Event data payload
    pub data: Vec<u8, 16>,
    /// Event priority
    pub priority: u8,
}

impl MockSystemEvent {
    /// Create a new system event
    pub fn new(event_type: u8, timestamp_ms: u32, data: &[u8], priority: u8) -> Result<Self, MockError> {
        let mut event_data = Vec::new();
        for &byte in data.iter().take(16) {
            event_data.push(byte).map_err(|_| MockError::BufferFull)?;
        }

        Ok(MockSystemEvent {
            event_type,
            timestamp_ms,
            data: event_data,
            priority,
        })
    }

    /// Check if this is a high-priority event
    pub fn is_high_priority(&self) -> bool {
        self.priority >= 200
    }

    /// Get event age in milliseconds
    pub fn age_ms(&self, current_time_ms: u32) -> u32 {
        current_time_ms.saturating_sub(self.timestamp_ms)
    }
}

/// Mock event queue for testing event processing
#[derive(Debug, Clone)]
pub struct MockEventQueue {
    /// Queue of pending events
    events: Vec<MockSystemEvent, MAX_SYSTEM_EVENTS>,
    /// Total events processed
    events_processed: u32,
    /// Events dropped due to full queue
    events_dropped: u32,
}

impl Default for MockEventQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl MockEventQueue {
    /// Create a new mock event queue
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            events_processed: 0,
            events_dropped: 0,
        }
    }

    /// Add an event to the queue
    pub fn push_event(&mut self, event: MockSystemEvent) -> Result<(), MockError> {
        match self.events.push(event) {
            Ok(()) => Ok(()),
            Err(_) => {
                self.events_dropped += 1;
                Err(MockError::BufferFull)
            }
        }
    }

    /// Get the next event from the queue
    pub fn pop_event(&mut self) -> Option<MockSystemEvent> {
        if self.events.is_empty() {
            None
        } else {
            self.events_processed += 1;
            Some(self.events.remove(0))
        }
    }

    /// Get the number of pending events
    pub fn pending_count(&self) -> usize {
        self.events.len()
    }

    /// Get the number of processed events
    pub fn processed_count(&self) -> u32 {
        self.events_processed
    }

    /// Get the number of dropped events
    pub fn dropped_count(&self) -> u32 {
        self.events_dropped
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Check if queue is full
    pub fn is_full(&self) -> bool {
        self.events.len() >= MAX_SYSTEM_EVENTS
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Comprehensive mock test environment that combines all mock components
/// Requirements: 6.1 (integration with automated testing infrastructure)
#[derive(Debug, Clone)]
pub struct MockTestEnvironment {
    /// Mock USB HID device
    pub usb_device: MockUsbHidDevice,
    /// Mock system state
    pub system_state: MockSystemState,
    /// Mock bootloader hardware
    pub bootloader_hardware: MockBootloaderHardware,
    /// Test data generator
    pub data_generator: TestDataGenerator,
    /// Test validator
    pub validator: TestValidator,
    /// Mock event queue
    pub event_queue: MockEventQueue,
    /// Environment timestamp
    current_time_ms: u32,
}

impl Default for MockTestEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl MockTestEnvironment {
    /// Create a new mock test environment
    pub fn new() -> Self {
        Self {
            usb_device: MockUsbHidDevice::new(),
            system_state: MockSystemState::new(),
            bootloader_hardware: MockBootloaderHardware::new(),
            data_generator: TestDataGenerator::new(42), // Fixed seed for reproducible tests
            validator: TestValidator::new(),
            event_queue: MockEventQueue::new(),
            current_time_ms: 0,
        }
    }

    /// Initialize the test environment with connected USB device
    pub fn initialize(&mut self) {
        self.usb_device.connect();
        let _ = self.usb_device.configure();
        self.system_state.set_uptime(self.current_time_ms);
    }

    /// Advance the mock time
    pub fn advance_time(&mut self, delta_ms: u32) {
        self.current_time_ms += delta_ms;
        self.system_state.set_uptime(self.current_time_ms);
    }

    /// Get current mock time
    pub fn get_current_time(&self) -> u32 {
        self.current_time_ms
    }

    /// Simulate a complete pEMF cycle
    pub fn simulate_pemf_cycle(&mut self, duration_ms: u32) {
        // Start pEMF pulse
        self.system_state.set_pemf_active(true);
        self.bootloader_hardware.set_hardware_state(HardwareState {
            mosfet_state: true,
            led_state: false,
            adc_active: false,
            usb_transmitting: false,
            pemf_pulse_active: true,
        });

        // Advance time for pulse duration
        self.advance_time(duration_ms);

        // End pEMF pulse
        self.system_state.set_pemf_active(false);
        self.bootloader_hardware.complete_pemf_pulse();
    }

    /// Simulate battery voltage change
    pub fn simulate_battery_change(&mut self, new_voltage_mv: u32) {
        let battery_state = if new_voltage_mv <= 3100 {
            BatteryState::Low
        } else if new_voltage_mv >= 3600 {
            BatteryState::Charging
        } else {
            BatteryState::Normal
        };

        self.system_state.set_battery_state(battery_state, new_voltage_mv);
    }

    /// Simulate system error
    pub fn simulate_error(&mut self, error_type: &str) {
        self.system_state.increment_error(error_type);
    }

    /// Simulate USB communication
    pub fn simulate_usb_communication(&mut self, message_count: usize) -> Result<(), MockError> {
        for i in 0..message_count {
            let data = self.data_generator.generate_usb_report(32);
            self.usb_device.send_report(&data, self.current_time_ms + i as u32)?;
        }
        Ok(())
    }

    /// Reset the entire test environment
    pub fn reset(&mut self) {
        self.usb_device = MockUsbHidDevice::new();
        self.system_state = MockSystemState::new();
        self.bootloader_hardware = MockBootloaderHardware::new();
        self.data_generator.reset(42);
        self.event_queue.clear();
        self.current_time_ms = 0;
    }

    /// Validate the current state of the test environment
    pub fn validate_state(&self) -> Result<(), MockError> {
        // Validate battery voltage
        if !self.validator.validate_battery_voltage(self.system_state.get_battery_voltage()) {
            return Err(MockError::InvalidData);
        }

        // Validate uptime is reasonable
        if self.current_time_ms > 86400000 { // More than 24 hours
            return Err(MockError::InvalidData);
        }

        // Validate USB device state consistency
        if self.usb_device.is_configured() && !self.usb_device.is_connected() {
            return Err(MockError::InvalidData);
        }

        Ok(())
    }
}

/// Utility macros for common mock operations
#[macro_export]
macro_rules! mock_assert_usb_report {
    ($mock_device:expr, $expected_data:expr) => {
        if let Some(report) = $mock_device.get_sent_report() {
            $crate::assert_eq_no_std!(report.data.as_slice(), $expected_data);
        } else {
            return $crate::test_framework::TestResult::fail("No USB report found");
        }
    };
}

#[macro_export]
macro_rules! mock_assert_battery_state {
    ($mock_state:expr, $expected_state:expr) => {
        let health = $mock_state.get_system_health();
        $crate::assert_eq_no_std!(health.battery_state, $expected_state);
    };
}

#[macro_export]
macro_rules! mock_assert_pemf_active {
    ($mock_state:expr, $expected_active:expr) => {
        let health = $mock_state.get_system_health();
        $crate::assert_eq_no_std!(health.pemf_active, $expected_active);
    };
}

#[macro_export]
macro_rules! mock_setup_test_env {
    () => {{
        let mut env = $crate::test_mocks::MockTestEnvironment::new();
        env.initialize();
        env
    }};
}

/// Test helper functions for common test patterns
pub mod test_helpers {
    use super::*;

    /// Create a standard test environment for USB communication tests
    pub fn create_usb_test_env() -> MockTestEnvironment {
        let mut env = MockTestEnvironment::new();
        env.initialize();
        env.usb_device.connect();
        let _ = env.usb_device.configure();
        env
    }

    /// Create a test environment with simulated errors
    pub fn create_error_test_env() -> MockTestEnvironment {
        let mut env = MockTestEnvironment::new();
        env.initialize();
        env.usb_device.enable_error_injection(10); // 10% error rate
        env
    }

    /// Create a test environment for bootloader testing
    pub fn create_bootloader_test_env() -> MockTestEnvironment {
        let mut env = MockTestEnvironment::new();
        env.initialize();
        // Set up safe hardware state for bootloader entry
        env.bootloader_hardware.set_hardware_state(HardwareState {
            mosfet_state: false,
            led_state: false,
            adc_active: false,
            usb_transmitting: false,
            pemf_pulse_active: false,
        });
        env
    }

    /// Validate test environment consistency
    pub fn validate_test_env(env: &MockTestEnvironment) -> Result<(), MockError> {
        env.validate_state()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_framework::TestResult;

    fn test_mock_usb_device_basic() -> TestResult {
        let mut device = MockUsbHidDevice::new();
        
        // Test initial state
        if device.is_connected() {
            return TestResult::fail("Device should not be connected initially");
        }

        // Test connection
        device.connect();
        if !device.is_connected() {
            return TestResult::fail("Device should be connected after connect()");
        }

        // Test configuration
        if device.configure().is_err() {
            return TestResult::fail("Device configuration should succeed when connected");
        }

        TestResult::pass()
    }

    fn test_mock_system_state_basic() -> TestResult {
        let mut state = MockSystemState::new();
        
        // Test initial state
        if state.get_uptime_ms() != 0 {
            return TestResult::fail("Initial uptime should be 0");
        }

        // Test uptime update
        state.set_uptime(1000);
        if state.get_uptime_ms() != 1000 {
            return TestResult::fail("Uptime should be updated to 1000ms");
        }

        // Test battery state
        state.set_battery_state(BatteryState::Low, 2900);
        if state.get_battery_voltage() != 2900 {
            return TestResult::fail("Battery voltage should be updated to 2900mV");
        }

        TestResult::pass()
    }

    fn test_data_generator() -> TestResult {
        let mut gen = TestDataGenerator::new(12345);
        
        // Test that generator produces values
        let val1 = gen.next_u32();
        let val2 = gen.next_u32();
        
        if val1 == val2 {
            return TestResult::fail("Generator should produce different values");
        }

        // Test range generation
        let range_val = gen.next_range(10, 20);
        if range_val < 10 || range_val >= 20 {
            return TestResult::fail("Range value should be within specified bounds");
        }

        TestResult::pass()
    }

    fn test_validator() -> TestResult {
        let validator = TestValidator::new();
        
        // Test float validation
        if !validator.validate_float_approx(1.0, 1.01) {
            return TestResult::fail("Float values within tolerance should validate");
        }

        if validator.validate_float_approx(1.0, 1.1) {
            return TestResult::fail("Float values outside tolerance should not validate");
        }

        // Test range validation
        if !validator.validate_range(5, 1, 10) {
            return TestResult::fail("Value within range should validate");
        }

        if validator.validate_range(15, 1, 10) {
            return TestResult::fail("Value outside range should not validate");
        }

        TestResult::pass()
    }

    fn test_mock_test_environment() -> TestResult {
        let mut env = MockTestEnvironment::new();
        env.initialize();
        
        // Test initial state
        if !env.usb_device.is_connected() {
            return TestResult::fail("USB device should be connected after initialization");
        }

        // Test time advancement
        env.advance_time(1000);
        if env.get_current_time() != 1000 {
            return TestResult::fail("Time should advance correctly");
        }

        // Test pEMF simulation
        env.simulate_pemf_cycle(250);
        let health = env.system_state.get_system_health();
        if health.pemf_cycle_count == 0 {
            return TestResult::fail("pEMF cycle count should increment");
        }

        TestResult::pass()
    }
}