//! Mock implementations for hardware interfaces and system components
//!
//! This module provides mock implementations that can be used in host-side tests
//! to simulate hardware behavior without requiring actual embedded hardware.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

// Re-export actual types when available for testing
#[cfg(any(test, feature = "std-testing"))]
pub use ass_easy_loop::BatteryState;
#[cfg(any(test, feature = "std-testing"))]
pub use ass_easy_loop::UsbControlCommand;

// Fallback types when library is not available
#[cfg(not(any(test, feature = "std-testing")))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryState {
    Low,
    Normal,
    Charging,
}

#[cfg(not(any(test, feature = "std-testing")))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbControlCommand {
    GetConfig = 0x01,
    SetConfig = 0x02,
    SetLogLevel = 0x03,
    EnableCategory = 0x04,
    DisableCategory = 0x05,
    ResetConfig = 0x06,
    GetStats = 0x07,
}

/// Mock battery monitor for testing battery-related functionality
/// Simulates the behavior of the actual BatteryMonitor from src/battery.rs
#[derive(Debug, Clone)]
pub struct MockBatteryMonitor {
    voltage_mv: Arc<Mutex<u32>>,
    adc_value: Arc<Mutex<u16>>,
    state: Arc<Mutex<BatteryState>>,
    readings: Arc<Mutex<Vec<(u32, u16, u32)>>>, // (timestamp_ms, adc_value, voltage_mv)
    error_count: Arc<Mutex<u32>>,
    simulate_errors: Arc<Mutex<bool>>,
}

impl MockBatteryMonitor {
    /// Create a new mock battery monitor with default values
    pub fn new() -> Self {
        Self {
            voltage_mv: Arc::new(Mutex::new(3700)), // Default to 3.7V (normal state)
            adc_value: Arc::new(Mutex::new(1550)),  // Corresponding ADC value for 3.7V
            state: Arc::new(Mutex::new(BatteryState::Normal)),
            readings: Arc::new(Mutex::new(Vec::new())),
            error_count: Arc::new(Mutex::new(0)),
            simulate_errors: Arc::new(Mutex::new(false)),
        }
    }

    /// Set the simulated battery voltage in millivolts
    /// Automatically calculates corresponding ADC value and battery state
    pub fn set_voltage(&self, voltage_mv: u32) {
        let adc_value = self.voltage_to_adc(voltage_mv);
        let state = self.adc_to_battery_state(adc_value);

        *self.voltage_mv.lock().unwrap() = voltage_mv;
        *self.adc_value.lock().unwrap() = adc_value;
        *self.state.lock().unwrap() = state;

        // Record the reading with timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u32;

        self.readings
            .lock()
            .unwrap()
            .push((timestamp, adc_value, voltage_mv));
    }

    /// Set the simulated ADC value directly
    /// Automatically calculates corresponding voltage and battery state
    pub fn set_adc_value(&self, adc_value: u16) {
        let voltage_mv = self.adc_to_voltage(adc_value);
        let state = self.adc_to_battery_state(adc_value);

        *self.adc_value.lock().unwrap() = adc_value;
        *self.voltage_mv.lock().unwrap() = voltage_mv;
        *self.state.lock().unwrap() = state;

        // Record the reading with timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u32;

        self.readings
            .lock()
            .unwrap()
            .push((timestamp, adc_value, voltage_mv));
    }

    /// Get the current simulated voltage in millivolts
    pub fn get_voltage(&self) -> u32 {
        *self.voltage_mv.lock().unwrap()
    }

    /// Get the current simulated ADC value
    pub fn get_adc_value(&self) -> u16 {
        *self.adc_value.lock().unwrap()
    }

    /// Get the current battery state
    pub fn get_state(&self) -> BatteryState {
        *self.state.lock().unwrap()
    }

    /// Simulate a battery reading (mimics actual hardware ADC read)
    pub fn read_battery(&self) -> Result<(u16, u32, BatteryState), MockAdcError> {
        if *self.simulate_errors.lock().unwrap() {
            *self.error_count.lock().unwrap() += 1;
            return Err(MockAdcError::ReadFailed);
        }

        let adc_value = *self.adc_value.lock().unwrap();
        let voltage_mv = *self.voltage_mv.lock().unwrap();
        let state = *self.state.lock().unwrap();

        // Record the reading
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u32;

        self.readings
            .lock()
            .unwrap()
            .push((timestamp, adc_value, voltage_mv));

        Ok((adc_value, voltage_mv, state))
    }

    /// Get all battery readings with timestamps
    pub fn get_readings(&self) -> Vec<(u32, u16, u32)> {
        self.readings.lock().unwrap().clone()
    }

    /// Clear all readings
    pub fn clear_readings(&self) {
        self.readings.lock().unwrap().clear();
    }

    /// Get error count
    pub fn get_error_count(&self) -> u32 {
        *self.error_count.lock().unwrap()
    }

    /// Enable/disable error simulation
    pub fn set_simulate_errors(&self, simulate: bool) {
        *self.simulate_errors.lock().unwrap() = simulate;
    }

    /// Simulate battery state transitions over time
    pub fn simulate_battery_discharge(&self, duration_ms: u32, steps: u32) {
        let start_voltage = *self.voltage_mv.lock().unwrap();
        let end_voltage = 3000; // Discharge to 3.0V (low battery)

        for i in 0..=steps {
            let progress = i as f32 / steps as f32;
            let current_voltage =
                start_voltage as f32 - (start_voltage as f32 - end_voltage as f32) * progress;

            self.set_voltage(current_voltage as u32);

            // Simulate time passing
            std::thread::sleep(std::time::Duration::from_millis(
                duration_ms as u64 / steps as u64,
            ));
        }
    }

    /// Convert voltage to ADC value (mimics actual hardware conversion)
    fn voltage_to_adc(&self, voltage_mv: u32) -> u16 {
        // Reverse of the actual conversion: adc_value = voltage_mv * 1000 / 2386
        let adc_value = (voltage_mv * 1000) / 2386;
        if adc_value > 4095 {
            4095
        } else {
            adc_value as u16
        }
    }

    /// Convert ADC value to voltage (mimics actual hardware conversion)
    fn adc_to_voltage(&self, adc_value: u16) -> u32 {
        // Actual conversion: voltage_mv = adc_value * 2386 / 1000
        (adc_value as u32 * 2386) / 1000
    }

    /// Convert ADC value to battery state (mimics actual logic)
    fn adc_to_battery_state(&self, adc_value: u16) -> BatteryState {
        if adc_value <= 1425 {
            BatteryState::Low
        } else if adc_value < 1675 {
            BatteryState::Normal
        } else {
            BatteryState::Charging
        }
    }

    /// Reset battery monitor to initial state
    pub fn reset_state(&self) {
        *self.voltage_mv.lock().unwrap() = 3700;
        *self.adc_value.lock().unwrap() = 1550;
        *self.state.lock().unwrap() = BatteryState::Normal;
        self.clear_readings();
        *self.error_count.lock().unwrap() = 0;
        *self.simulate_errors.lock().unwrap() = false;
    }
}

impl Default for MockBatteryMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock ADC error type for testing error conditions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MockAdcError {
    ReadFailed,
    InvalidReading,
    HardwareError,
}

/// Mock USB HID device for testing USB communication
/// Simulates the behavior of the actual USB HID interface
#[derive(Debug, Clone)]
pub struct MockUsbHidDevice {
    sent_reports: Arc<Mutex<Vec<HidReport>>>,
    received_reports: Arc<Mutex<Vec<HidReport>>>,
    command_queue: Arc<Mutex<Vec<UsbControlCommand>>>,
    connection_state: Arc<Mutex<UsbConnectionState>>,
    device_config: Arc<Mutex<MockUsbConfig>>,
    error_count: Arc<Mutex<u32>>,
    simulate_errors: Arc<Mutex<bool>>,
    bandwidth_usage: Arc<Mutex<u32>>, // bytes per second
}

/// USB connection state for realistic simulation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UsbConnectionState {
    pub connected: bool,
    pub configured: bool,
    pub suspended: bool,
    pub enumerated: bool,
    pub last_activity_ms: u32,
}

/// Mock USB configuration
#[derive(Debug, Clone)]
pub struct MockUsbConfig {
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: String,
    pub product: String,
    pub serial_number: String,
    pub report_size: usize,
}

/// HID report structure for testing
#[derive(Debug, Clone, PartialEq)]
pub struct HidReport {
    pub timestamp_ms: u32,
    pub report_id: u8,
    pub data: Vec<u8>,
    pub report_type: HidReportType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HidReportType {
    Input,
    Output,
    Feature,
}

impl MockUsbHidDevice {
    /// Create a new mock USB HID device
    pub fn new() -> Self {
        Self {
            sent_reports: Arc::new(Mutex::new(Vec::new())),
            received_reports: Arc::new(Mutex::new(Vec::new())),
            command_queue: Arc::new(Mutex::new(Vec::new())),
            connection_state: Arc::new(Mutex::new(UsbConnectionState {
                connected: true,
                configured: true,
                suspended: false,
                enumerated: true,
                last_activity_ms: 0,
            })),
            device_config: Arc::new(Mutex::new(MockUsbConfig {
                vendor_id: 0x2E8A,  // Raspberry Pi Foundation
                product_id: 0x000A, // Pico
                manufacturer: "Test Manufacturer".to_string(),
                product: "Test Device".to_string(),
                serial_number: "TEST123456".to_string(),
                report_size: 64,
            })),
            error_count: Arc::new(Mutex::new(0)),
            simulate_errors: Arc::new(Mutex::new(false)),
            bandwidth_usage: Arc::new(Mutex::new(0)),
        }
    }

    /// Simulate sending a HID report
    pub fn send_report(&self, report_id: u8, data: Vec<u8>) -> Result<(), MockUsbError> {
        if *self.simulate_errors.lock().unwrap() {
            *self.error_count.lock().unwrap() += 1;
            return Err(MockUsbError::TransmissionFailed);
        }

        if !self.connection_state.lock().unwrap().connected {
            return Err(MockUsbError::NotConnected);
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u32;

        let report = HidReport {
            timestamp_ms: timestamp,
            report_id,
            data: data.clone(),
            report_type: HidReportType::Input,
        };

        self.sent_reports.lock().unwrap().push(report);

        // Update bandwidth usage
        let data_size = data.len() as u32;
        *self.bandwidth_usage.lock().unwrap() += data_size;

        // Update last activity
        self.connection_state.lock().unwrap().last_activity_ms = timestamp;

        Ok(())
    }

    /// Simulate receiving a HID report
    pub fn receive_report(&self, report_id: u8, data: Vec<u8>) -> Result<(), MockUsbError> {
        if !self.connection_state.lock().unwrap().connected {
            return Err(MockUsbError::NotConnected);
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u32;

        let report = HidReport {
            timestamp_ms: timestamp,
            report_id,
            data,
            report_type: HidReportType::Output,
        };

        self.received_reports.lock().unwrap().push(report);

        // Update last activity
        self.connection_state.lock().unwrap().last_activity_ms = timestamp;

        Ok(())
    }

    /// Simulate processing a USB control command
    pub fn process_control_command(
        &self,
        command: UsbControlCommand,
        data: Option<Vec<u8>>,
    ) -> Result<Vec<u8>, MockUsbError> {
        if !self.connection_state.lock().unwrap().configured {
            return Err(MockUsbError::NotConfigured);
        }

        self.command_queue.lock().unwrap().push(command);

        // Simulate command processing
        match command {
            UsbControlCommand::GetConfig => {
                // Return mock configuration data
                Ok(vec![0x01, 0x02, 0x03, 0x04]) // Mock config bytes
            }
            UsbControlCommand::SetConfig => {
                if let Some(_config_data) = data {
                    // Process configuration update
                    Ok(vec![0x00]) // Success response
                } else {
                    Err(MockUsbError::InvalidData)
                }
            }
            UsbControlCommand::SetLogLevel => {
                if let Some(level_data) = data {
                    if !level_data.is_empty() {
                        // Process log level change
                        Ok(vec![0x00]) // Success response
                    } else {
                        Err(MockUsbError::InvalidData)
                    }
                } else {
                    Err(MockUsbError::InvalidData)
                }
            }
            UsbControlCommand::EnableCategory | UsbControlCommand::DisableCategory => {
                if let Some(category_data) = data {
                    if !category_data.is_empty() {
                        // Process category change
                        Ok(vec![0x00]) // Success response
                    } else {
                        Err(MockUsbError::InvalidData)
                    }
                } else {
                    Err(MockUsbError::InvalidData)
                }
            }
            UsbControlCommand::ResetConfig => {
                // Reset to default configuration
                Ok(vec![0x00]) // Success response
            }
            UsbControlCommand::GetStats => {
                // Return mock statistics
                let stats = vec![
                    0x10,
                    0x20,
                    0x30,
                    0x40, // Mock stats bytes
                    (self.sent_reports.lock().unwrap().len() as u8),
                    (self.received_reports.lock().unwrap().len() as u8),
                    (*self.error_count.lock().unwrap() & 0xFF) as u8,
                ];
                Ok(stats)
            }
        }
    }

    /// Get all sent reports
    pub fn get_sent_reports(&self) -> Vec<HidReport> {
        self.sent_reports.lock().unwrap().clone()
    }

    /// Get all received reports
    pub fn get_received_reports(&self) -> Vec<HidReport> {
        self.received_reports.lock().unwrap().clone()
    }

    /// Get processed commands
    pub fn get_processed_commands(&self) -> Vec<UsbControlCommand> {
        self.command_queue.lock().unwrap().clone()
    }

    /// Set connection state
    pub fn set_connection_state(&self, state: UsbConnectionState) {
        *self.connection_state.lock().unwrap() = state;
    }

    /// Get connection state
    pub fn get_connection_state(&self) -> UsbConnectionState {
        *self.connection_state.lock().unwrap()
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connection_state.lock().unwrap().connected
    }

    /// Check if configured
    pub fn is_configured(&self) -> bool {
        self.connection_state.lock().unwrap().configured
    }

    /// Simulate USB disconnection
    pub fn disconnect(&self) {
        let mut state = self.connection_state.lock().unwrap();
        state.connected = false;
        state.configured = false;
        state.enumerated = false;
    }

    /// Simulate USB reconnection
    pub fn reconnect(&self) {
        let mut state = self.connection_state.lock().unwrap();
        state.connected = true;
        state.configured = true;
        state.enumerated = true;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u32;
        state.last_activity_ms = timestamp;
    }

    /// Get error count
    pub fn get_error_count(&self) -> u32 {
        *self.error_count.lock().unwrap()
    }

    /// Enable/disable error simulation
    pub fn set_simulate_errors(&self, simulate: bool) {
        *self.simulate_errors.lock().unwrap() = simulate;
    }

    /// Get bandwidth usage
    pub fn get_bandwidth_usage(&self) -> u32 {
        *self.bandwidth_usage.lock().unwrap()
    }

    /// Reset bandwidth usage counter
    pub fn reset_bandwidth_usage(&self) {
        *self.bandwidth_usage.lock().unwrap() = 0;
    }

    /// Send a message (convenience method for tests)
    pub fn send_message(&self, data: Vec<u8>) -> Result<(), MockUsbError> {
        self.send_report(0, data)
    }

    /// Get sent messages (convenience method for tests)
    pub fn get_sent_messages(&self) -> Vec<HidReport> {
        self.sent_reports.lock().unwrap().clone()
    }

    /// Clear all message history
    pub fn clear_messages(&self) {
        self.sent_reports.lock().unwrap().clear();
        self.received_reports.lock().unwrap().clear();
        self.command_queue.lock().unwrap().clear();
    }

    /// Get device configuration
    pub fn get_device_config(&self) -> MockUsbConfig {
        self.device_config.lock().unwrap().clone()
    }

    /// Update device configuration
    pub fn set_device_config(&self, config: MockUsbConfig) {
        *self.device_config.lock().unwrap() = config;
    }
}

impl Default for MockUsbHidDevice {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock USB error types for testing error conditions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MockUsbError {
    NotConnected,
    NotConfigured,
    TransmissionFailed,
    InvalidData,
    BufferFull,
    Timeout,
}

/// Mock system state for testing system state management
/// Simulates the comprehensive system state from src/system_state.rs
#[derive(Debug, Clone)]
pub struct MockSystemState {
    // Core system state
    uptime_ms: Arc<Mutex<u32>>,
    system_health: Arc<Mutex<MockSystemHealth>>,
    task_performance: Arc<Mutex<MockTaskPerformance>>,
    hardware_status: Arc<Mutex<MockHardwareStatus>>,
    configuration: Arc<Mutex<HashMap<String, String>>>,
    error_history: Arc<Mutex<Vec<MockSystemError>>>,

    // State tracking
    state_queries: Arc<Mutex<Vec<(u32, String)>>>, // (timestamp, query_type)
    last_update_ms: Arc<Mutex<u32>>,
}

/// Mock system health data
#[derive(Debug, Clone)]
pub struct MockSystemHealth {
    pub battery_state: BatteryState,
    pub battery_voltage_mv: u32,
    pub pemf_active: bool,
    pub pemf_cycle_count: u32,
    pub task_health_status: MockTaskHealthStatus,
    pub memory_usage: MockMemoryUsage,
    pub error_counts: MockErrorCounters,
    pub system_temperature_c: Option<i16>,
}

/// Mock task health status
#[derive(Debug, Clone)]
pub struct MockTaskHealthStatus {
    pub pemf_task_healthy: bool,
    pub battery_task_healthy: bool,
    pub led_task_healthy: bool,
    pub usb_poll_task_healthy: bool,
    pub usb_hid_task_healthy: bool,
    pub command_handler_healthy: bool,
    pub last_health_check_ms: u32,
}

/// Mock memory usage statistics
#[derive(Debug, Clone)]
pub struct MockMemoryUsage {
    pub stack_usage_bytes: u32,
    pub heap_usage_bytes: u32,
    pub log_queue_usage_bytes: u32,
    pub command_queue_usage_bytes: u32,
    pub total_ram_usage_bytes: u32,
    pub peak_ram_usage_bytes: u32,
    pub memory_fragmentation_percent: u8,
}

/// Mock error counters
#[derive(Debug, Clone)]
pub struct MockErrorCounters {
    pub adc_read_errors: u32,
    pub gpio_operation_errors: u32,
    pub usb_transmission_errors: u32,
    pub command_parsing_errors: u32,
    pub timing_violations: u32,
    pub bootloader_entry_failures: u32,
    pub total_error_count: u32,
}

/// Mock task performance data
#[derive(Debug, Clone)]
pub struct MockTaskPerformance {
    pub task_execution_times: MockTaskExecutionTimes,
    pub timing_statistics: MockTimingStatistics,
    pub resource_usage: MockResourceUsage,
    pub performance_metrics: MockPerformanceMetrics,
}

/// Mock task execution times
#[derive(Debug, Clone)]
pub struct MockTaskExecutionTimes {
    pub pemf_task_avg_us: u32,
    pub pemf_task_max_us: u32,
    pub battery_task_avg_us: u32,
    pub battery_task_max_us: u32,
    pub led_task_avg_us: u32,
    pub led_task_max_us: u32,
    pub usb_poll_avg_us: u32,
    pub usb_poll_max_us: u32,
    pub usb_hid_avg_us: u32,
    pub usb_hid_max_us: u32,
}

/// Mock timing statistics
#[derive(Debug, Clone)]
pub struct MockTimingStatistics {
    pub pemf_frequency_hz: f32,
    pub pemf_timing_accuracy_percent: f32,
    pub battery_sampling_rate_hz: f32,
    pub max_timing_deviation_us: u32,
    pub timing_violations_count: u32,
}

/// Mock resource usage
#[derive(Debug, Clone)]
pub struct MockResourceUsage {
    pub cpu_usage_percent: u8,
    pub memory_usage_percent: u8,
    pub queue_utilization_percent: u8,
    pub usb_bandwidth_usage_percent: u8,
    pub interrupt_load_percent: u8,
}

/// Mock performance metrics
#[derive(Debug, Clone)]
pub struct MockPerformanceMetrics {
    pub messages_per_second: u32,
    pub commands_processed_per_second: u32,
    pub average_response_time_ms: u32,
    pub throughput_bytes_per_second: u32,
    pub system_efficiency_percent: u8,
}

/// Mock hardware status
#[derive(Debug, Clone)]
pub struct MockHardwareStatus {
    pub gpio_states: MockGpioStates,
    pub adc_readings: MockAdcReadings,
    pub usb_status: MockUsbStatus,
    pub power_status: MockPowerStatus,
}

/// Mock GPIO states
#[derive(Debug, Clone)]
pub struct MockGpioStates {
    pub mosfet_pin_state: bool,
    pub led_pin_state: bool,
    pub adc_pin_voltage_mv: u32,
    pub bootsel_pin_state: bool,
    pub gpio_error_count: u32,
}

/// Mock ADC readings
#[derive(Debug, Clone)]
pub struct MockAdcReadings {
    pub battery_adc_raw: u16,
    pub battery_voltage_mv: u32,
    pub internal_temperature_c: i16,
    pub vref_voltage_mv: u32,
    pub adc_calibration_offset: i16,
    pub adc_error_count: u32,
}

/// Mock USB status
#[derive(Debug, Clone)]
pub struct MockUsbStatus {
    pub connected: bool,
    pub configured: bool,
    pub suspended: bool,
    pub enumerated: bool,
    pub hid_reports_sent: u32,
    pub hid_reports_received: u32,
    pub usb_errors: u32,
    pub last_activity_ms: u32,
}

/// Mock power status
#[derive(Debug, Clone)]
pub struct MockPowerStatus {
    pub supply_voltage_mv: u32,
    pub core_voltage_mv: u32,
    pub power_consumption_mw: u32,
    pub battery_charging: bool,
    pub low_power_mode: bool,
    pub power_good: bool,
}

/// Mock system error
#[derive(Debug, Clone)]
pub struct MockSystemError {
    pub error_type: u8,
    pub timestamp_ms: u32,
    pub description: String,
    pub severity: MockErrorSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MockErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl MockSystemState {
    /// Create a new mock system state with realistic defaults
    pub fn new() -> Self {
        let system_health = MockSystemHealth {
            battery_state: BatteryState::Normal,
            battery_voltage_mv: 3700,
            pemf_active: false,
            pemf_cycle_count: 0,
            task_health_status: MockTaskHealthStatus {
                pemf_task_healthy: true,
                battery_task_healthy: true,
                led_task_healthy: true,
                usb_poll_task_healthy: true,
                usb_hid_task_healthy: true,
                command_handler_healthy: true,
                last_health_check_ms: 0,
            },
            memory_usage: MockMemoryUsage {
                stack_usage_bytes: 4096,
                heap_usage_bytes: 0,
                log_queue_usage_bytes: 2048,
                command_queue_usage_bytes: 512,
                total_ram_usage_bytes: 6656,
                peak_ram_usage_bytes: 7000,
                memory_fragmentation_percent: 0,
            },
            error_counts: MockErrorCounters {
                adc_read_errors: 0,
                gpio_operation_errors: 0,
                usb_transmission_errors: 0,
                command_parsing_errors: 0,
                timing_violations: 0,
                bootloader_entry_failures: 0,
                total_error_count: 0,
            },
            system_temperature_c: Some(25),
        };

        let task_performance = MockTaskPerformance {
            task_execution_times: MockTaskExecutionTimes {
                pemf_task_avg_us: 50,
                pemf_task_max_us: 100,
                battery_task_avg_us: 200,
                battery_task_max_us: 500,
                led_task_avg_us: 10,
                led_task_max_us: 50,
                usb_poll_avg_us: 100,
                usb_poll_max_us: 300,
                usb_hid_avg_us: 150,
                usb_hid_max_us: 400,
            },
            timing_statistics: MockTimingStatistics {
                pemf_frequency_hz: 2.0,
                pemf_timing_accuracy_percent: 99.5,
                battery_sampling_rate_hz: 1.0,
                max_timing_deviation_us: 50,
                timing_violations_count: 0,
            },
            resource_usage: MockResourceUsage {
                cpu_usage_percent: 25,
                memory_usage_percent: 30,
                queue_utilization_percent: 15,
                usb_bandwidth_usage_percent: 10,
                interrupt_load_percent: 20,
            },
            performance_metrics: MockPerformanceMetrics {
                messages_per_second: 100,
                commands_processed_per_second: 10,
                average_response_time_ms: 5,
                throughput_bytes_per_second: 6400,
                system_efficiency_percent: 85,
            },
        };

        let hardware_status = MockHardwareStatus {
            gpio_states: MockGpioStates {
                mosfet_pin_state: false,
                led_pin_state: false,
                adc_pin_voltage_mv: 3700,
                bootsel_pin_state: false,
                gpio_error_count: 0,
            },
            adc_readings: MockAdcReadings {
                battery_adc_raw: 1550,
                battery_voltage_mv: 3700,
                internal_temperature_c: 25,
                vref_voltage_mv: 3300,
                adc_calibration_offset: 0,
                adc_error_count: 0,
            },
            usb_status: MockUsbStatus {
                connected: true,
                configured: true,
                suspended: false,
                enumerated: true,
                hid_reports_sent: 0,
                hid_reports_received: 0,
                usb_errors: 0,
                last_activity_ms: 0,
            },
            power_status: MockPowerStatus {
                supply_voltage_mv: 5000,
                core_voltage_mv: 3300,
                power_consumption_mw: 150,
                battery_charging: false,
                low_power_mode: false,
                power_good: true,
            },
        };

        Self {
            uptime_ms: Arc::new(Mutex::new(0)),
            system_health: Arc::new(Mutex::new(system_health)),
            task_performance: Arc::new(Mutex::new(task_performance)),
            hardware_status: Arc::new(Mutex::new(hardware_status)),
            configuration: Arc::new(Mutex::new(HashMap::new())),
            error_history: Arc::new(Mutex::new(Vec::new())),
            state_queries: Arc::new(Mutex::new(Vec::new())),
            last_update_ms: Arc::new(Mutex::new(0)),
        }
    }

    /// Set system uptime
    pub fn set_uptime(&self, uptime_ms: u32) {
        *self.uptime_ms.lock().unwrap() = uptime_ms;
        *self.last_update_ms.lock().unwrap() = uptime_ms;
    }

    /// Get system uptime
    pub fn get_uptime(&self) -> u32 {
        *self.uptime_ms.lock().unwrap()
    }

    /// Update system health data
    pub fn update_system_health(&self, health: MockSystemHealth) {
        *self.system_health.lock().unwrap() = health;
    }

    /// Get system health data
    pub fn get_system_health(&self) -> MockSystemHealth {
        self.system_health.lock().unwrap().clone()
    }

    /// Update task performance data
    pub fn update_task_performance(&self, performance: MockTaskPerformance) {
        *self.task_performance.lock().unwrap() = performance;
    }

    /// Get task performance data
    pub fn get_task_performance(&self) -> MockTaskPerformance {
        self.task_performance.lock().unwrap().clone()
    }

    /// Update hardware status
    pub fn update_hardware_status(&self, status: MockHardwareStatus) {
        *self.hardware_status.lock().unwrap() = status;
    }

    /// Get hardware status
    pub fn get_hardware_status(&self) -> MockHardwareStatus {
        self.hardware_status.lock().unwrap().clone()
    }

    /// Set a configuration value
    pub fn set_config(&self, key: &str, value: &str) {
        self.configuration
            .lock()
            .unwrap()
            .insert(key.to_string(), value.to_string());
    }

    /// Get a configuration value
    pub fn get_config(&self, key: &str) -> Option<String> {
        self.configuration.lock().unwrap().get(key).cloned()
    }

    /// Add a system error to history
    pub fn add_error(&self, error: MockSystemError) {
        let mut history = self.error_history.lock().unwrap();
        history.push(error);

        // Keep only the last 100 errors
        if history.len() > 100 {
            history.remove(0);
        }

        // Update error counters
        let mut health = self.system_health.lock().unwrap();
        health.error_counts.total_error_count += 1;
    }

    /// Get error history
    pub fn get_error_history(&self) -> Vec<MockSystemError> {
        self.error_history.lock().unwrap().clone()
    }

    /// Record a state query
    pub fn record_state_query(&self, query_type: &str) {
        let timestamp = *self.uptime_ms.lock().unwrap();
        self.state_queries
            .lock()
            .unwrap()
            .push((timestamp, query_type.to_string()));
    }

    /// Get state query history
    pub fn get_state_queries(&self) -> Vec<(u32, String)> {
        self.state_queries.lock().unwrap().clone()
    }

    /// Simulate system operation over time
    pub fn simulate_operation(&self, duration_ms: u32) {
        let start_uptime = *self.uptime_ms.lock().unwrap();
        let end_uptime = start_uptime + duration_ms;

        // Update uptime
        *self.uptime_ms.lock().unwrap() = end_uptime;

        // Update performance metrics
        let mut performance = self.task_performance.lock().unwrap();
        performance.performance_metrics.messages_per_second = (duration_ms / 10).min(200); // Simulate message rate

        // Update cycle counts
        let mut health = self.system_health.lock().unwrap();
        health.pemf_cycle_count += duration_ms / 500; // 2Hz pEMF

        // Update USB activity
        let mut hardware = self.hardware_status.lock().unwrap();
        hardware.usb_status.last_activity_ms = end_uptime;
        hardware.usb_status.hid_reports_sent += duration_ms / 100; // Simulate reports
    }

    /// Set a state value (convenience method for tests)
    pub fn set_state(&self, key: &str, value: &str) {
        self.configuration
            .lock()
            .unwrap()
            .insert(key.to_string(), value.to_string());
    }

    /// Get a state value (convenience method for tests)
    pub fn get_state(&self, key: &str) -> Option<String> {
        self.configuration.lock().unwrap().get(key).cloned()
    }

    /// Clear all state data
    pub fn clear_state(&self) {
        self.configuration.lock().unwrap().clear();
        self.error_history.lock().unwrap().clear();
        self.state_queries.lock().unwrap().clear();
        *self.uptime_ms.lock().unwrap() = 0;
        *self.last_update_ms.lock().unwrap() = 0;
    }
}

impl Default for MockSystemState {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock bootloader hardware for testing bootloader functionality
/// Simulates the behavior of the actual bootloader system from src/bootloader.rs
#[derive(Debug, Clone)]
pub struct MockBootloaderHardware {
    // Bootloader state
    entry_state: Arc<Mutex<MockBootloaderEntryState>>,
    hardware_state: Arc<Mutex<MockHardwareState>>,
    task_shutdown_status: Arc<Mutex<MockTaskShutdownStatus>>,

    // Flash operations
    flash_data: Arc<Mutex<Vec<u8>>>,
    flash_operations: Arc<Mutex<Vec<MockFlashOperation>>>,

    // Entry management
    entry_timeout_ms: Arc<Mutex<u32>>,
    entry_start_time: Arc<Mutex<Option<u32>>>,
    shutdown_sequence_active: Arc<Mutex<bool>>,

    // Error simulation
    simulate_errors: Arc<Mutex<bool>>,
    error_count: Arc<Mutex<u32>>,
}

/// Mock bootloader entry state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MockBootloaderEntryState {
    Normal,
    EntryRequested,
    ValidatingHardware,
    ShuttingDownTasks,
    FinalSafetyCheck,
    ReadyForBootloader,
    EntryFailed,
}

/// Mock hardware state for bootloader validation
#[derive(Debug, Clone)]
pub struct MockHardwareState {
    pub mosfet_state: bool,
    pub led_state: bool,
    pub adc_active: bool,
    pub usb_transmitting: bool,
    pub pemf_pulse_active: bool,
    pub bootsel_pin_state: bool,
}

/// Mock task shutdown status
#[derive(Debug, Clone)]
pub struct MockTaskShutdownStatus {
    pub high_priority_status: MockTaskShutdownState,
    pub medium_priority_status: MockTaskShutdownState,
    pub low_priority_status: MockTaskShutdownState,
    pub shutdown_start_time: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MockTaskShutdownState {
    Running,
    ShutdownRequested,
    ShutdownAcknowledged,
    ShutdownComplete,
    ShutdownFailed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MockTaskPriority {
    Low = 1,
    Medium = 2,
    High = 3,
}

/// Mock flash operation record
#[derive(Debug, Clone)]
pub struct MockFlashOperation {
    pub timestamp_ms: u32,
    pub operation_type: String,
    pub data_size: usize,
    pub success: bool,
}

impl MockBootloaderHardware {
    /// Create a new mock bootloader hardware
    pub fn new() -> Self {
        let hardware_state = MockHardwareState {
            mosfet_state: false,
            led_state: false,
            adc_active: false,
            usb_transmitting: false,
            pemf_pulse_active: false,
            bootsel_pin_state: false,
        };

        let task_shutdown_status = MockTaskShutdownStatus {
            high_priority_status: MockTaskShutdownState::Running,
            medium_priority_status: MockTaskShutdownState::Running,
            low_priority_status: MockTaskShutdownState::Running,
            shutdown_start_time: None,
        };

        Self {
            entry_state: Arc::new(Mutex::new(MockBootloaderEntryState::Normal)),
            hardware_state: Arc::new(Mutex::new(hardware_state)),
            task_shutdown_status: Arc::new(Mutex::new(task_shutdown_status)),
            flash_data: Arc::new(Mutex::new(Vec::new())),
            flash_operations: Arc::new(Mutex::new(Vec::new())),
            entry_timeout_ms: Arc::new(Mutex::new(2000)),
            entry_start_time: Arc::new(Mutex::new(None)),
            shutdown_sequence_active: Arc::new(Mutex::new(false)),
            simulate_errors: Arc::new(Mutex::new(false)),
            error_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Request bootloader mode entry
    pub fn request_bootloader_entry(
        &self,
        timeout_ms: u32,
        current_time_ms: u32,
    ) -> Result<(), MockBootloaderError> {
        if *self.entry_state.lock().unwrap() != MockBootloaderEntryState::Normal {
            return Err(MockBootloaderError::SystemBusy);
        }

        *self.entry_state.lock().unwrap() = MockBootloaderEntryState::EntryRequested;
        *self.entry_timeout_ms.lock().unwrap() = timeout_ms.min(2000);
        *self.entry_start_time.lock().unwrap() = Some(current_time_ms);

        Ok(())
    }

    /// Update bootloader entry progress
    pub fn update_entry_progress(
        &self,
        current_time_ms: u32,
    ) -> Result<MockBootloaderEntryState, MockBootloaderError> {
        // Check for timeout
        if let Some(start_time) = *self.entry_start_time.lock().unwrap() {
            let timeout = *self.entry_timeout_ms.lock().unwrap();
            if current_time_ms.saturating_sub(start_time) > timeout {
                *self.entry_state.lock().unwrap() = MockBootloaderEntryState::EntryFailed;
                return Err(MockBootloaderError::TaskShutdownFailed);
            }
        }

        let current_state = *self.entry_state.lock().unwrap();

        match current_state {
            MockBootloaderEntryState::EntryRequested => {
                *self.entry_state.lock().unwrap() = MockBootloaderEntryState::ValidatingHardware;
            }
            MockBootloaderEntryState::ValidatingHardware => {
                if self.is_hardware_safe_for_bootloader()? {
                    *self.entry_state.lock().unwrap() = MockBootloaderEntryState::ShuttingDownTasks;
                    self.start_task_shutdown(current_time_ms);
                }
            }
            MockBootloaderEntryState::ShuttingDownTasks => {
                if self.update_task_shutdown_progress(current_time_ms)? {
                    *self.entry_state.lock().unwrap() = MockBootloaderEntryState::FinalSafetyCheck;
                }
            }
            MockBootloaderEntryState::FinalSafetyCheck => {
                if self.is_hardware_safe_for_bootloader()? {
                    *self.entry_state.lock().unwrap() =
                        MockBootloaderEntryState::ReadyForBootloader;
                } else {
                    self.force_safe_state()?;
                    *self.entry_state.lock().unwrap() =
                        MockBootloaderEntryState::ReadyForBootloader;
                }
            }
            _ => {}
        }

        Ok(*self.entry_state.lock().unwrap())
    }

    /// Check if hardware state is safe for bootloader entry
    pub fn is_hardware_safe_for_bootloader(&self) -> Result<bool, MockBootloaderError> {
        let hardware = self.hardware_state.lock().unwrap();

        // MOSFET must be OFF and pEMF pulse must not be active
        let safe = !hardware.mosfet_state && !hardware.pemf_pulse_active;

        if !safe && *self.simulate_errors.lock().unwrap() {
            *self.error_count.lock().unwrap() += 1;
            return Err(MockBootloaderError::UnsafeHardwareState);
        }

        Ok(safe)
    }

    /// Start task shutdown sequence
    pub fn start_task_shutdown(&self, current_time_ms: u32) {
        let mut status = self.task_shutdown_status.lock().unwrap();
        status.shutdown_start_time = Some(current_time_ms);
        status.high_priority_status = MockTaskShutdownState::ShutdownRequested;
        *self.shutdown_sequence_active.lock().unwrap() = true;
    }

    /// Update task shutdown progress
    pub fn update_task_shutdown_progress(
        &self,
        _current_time_ms: u32,
    ) -> Result<bool, MockBootloaderError> {
        let mut status = self.task_shutdown_status.lock().unwrap();

        // Simulate task shutdown progression
        match (
            status.high_priority_status,
            status.medium_priority_status,
            status.low_priority_status,
        ) {
            (
                MockTaskShutdownState::ShutdownRequested,
                MockTaskShutdownState::Running,
                MockTaskShutdownState::Running,
            ) => {
                status.high_priority_status = MockTaskShutdownState::ShutdownComplete;
                status.medium_priority_status = MockTaskShutdownState::ShutdownRequested;
            }
            (
                MockTaskShutdownState::ShutdownComplete,
                MockTaskShutdownState::ShutdownRequested,
                MockTaskShutdownState::Running,
            ) => {
                status.medium_priority_status = MockTaskShutdownState::ShutdownComplete;
                status.low_priority_status = MockTaskShutdownState::ShutdownRequested;
            }
            (
                MockTaskShutdownState::ShutdownComplete,
                MockTaskShutdownState::ShutdownComplete,
                MockTaskShutdownState::ShutdownRequested,
            ) => {
                status.low_priority_status = MockTaskShutdownState::ShutdownComplete;
                return Ok(true); // All tasks shut down
            }
            (
                MockTaskShutdownState::ShutdownComplete,
                MockTaskShutdownState::ShutdownComplete,
                MockTaskShutdownState::ShutdownComplete,
            ) => {
                return Ok(true); // All tasks shut down
            }
            _ => {}
        }

        Ok(false) // Shutdown still in progress
    }

    /// Force hardware into safe state
    pub fn force_safe_state(&self) -> Result<(), MockBootloaderError> {
        let mut hardware = self.hardware_state.lock().unwrap();
        hardware.mosfet_state = false;
        hardware.pemf_pulse_active = false;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u32;

        let operation = MockFlashOperation {
            timestamp_ms: timestamp,
            operation_type: "force_safe_state".to_string(),
            data_size: 0,
            success: true,
        };

        self.flash_operations.lock().unwrap().push(operation);

        Ok(())
    }

    /// Simulate entering bootloader mode
    pub fn enter_bootloader_mode(&self) -> Result<(), MockBootloaderError> {
        if *self.entry_state.lock().unwrap() != MockBootloaderEntryState::ReadyForBootloader {
            return Err(MockBootloaderError::EntryInterrupted);
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u32;

        let operation = MockFlashOperation {
            timestamp_ms: timestamp,
            operation_type: "enter_bootloader".to_string(),
            data_size: 0,
            success: true,
        };

        self.flash_operations.lock().unwrap().push(operation);

        // Reset state after bootloader entry
        self.reset_entry_state();

        Ok(())
    }

    /// Reset bootloader entry state
    pub fn reset_entry_state(&self) {
        *self.entry_state.lock().unwrap() = MockBootloaderEntryState::Normal;
        *self.entry_start_time.lock().unwrap() = None;
        *self.shutdown_sequence_active.lock().unwrap() = false;

        let mut status = self.task_shutdown_status.lock().unwrap();
        status.high_priority_status = MockTaskShutdownState::Running;
        status.medium_priority_status = MockTaskShutdownState::Running;
        status.low_priority_status = MockTaskShutdownState::Running;
        status.shutdown_start_time = None;
    }

    /// Simulate writing flash data
    pub fn write_flash(&self, data: Vec<u8>) -> Result<(), MockBootloaderError> {
        if *self.simulate_errors.lock().unwrap() {
            *self.error_count.lock().unwrap() += 1;
            return Err(MockBootloaderError::HardwareValidationFailed);
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u32;

        let operation = MockFlashOperation {
            timestamp_ms: timestamp,
            operation_type: "write_flash".to_string(),
            data_size: data.len(),
            success: true,
        };

        *self.flash_data.lock().unwrap() = data;
        self.flash_operations.lock().unwrap().push(operation);

        Ok(())
    }

    /// Get flash data
    pub fn get_flash_data(&self) -> Vec<u8> {
        self.flash_data.lock().unwrap().clone()
    }

    /// Get all flash operations
    pub fn get_flash_operations(&self) -> Vec<MockFlashOperation> {
        self.flash_operations.lock().unwrap().clone()
    }

    /// Get current entry state
    pub fn get_entry_state(&self) -> MockBootloaderEntryState {
        *self.entry_state.lock().unwrap()
    }

    /// Get hardware state
    pub fn get_hardware_state(&self) -> MockHardwareState {
        self.hardware_state.lock().unwrap().clone()
    }

    /// Set hardware state for testing
    pub fn set_hardware_state(&self, state: MockHardwareState) {
        *self.hardware_state.lock().unwrap() = state;
    }

    /// Get task shutdown status
    pub fn get_task_shutdown_status(&self) -> MockTaskShutdownStatus {
        self.task_shutdown_status.lock().unwrap().clone()
    }

    /// Mark task shutdown complete
    pub fn mark_task_shutdown_complete(&self, priority: MockTaskPriority) {
        let mut status = self.task_shutdown_status.lock().unwrap();
        match priority {
            MockTaskPriority::High => {
                status.high_priority_status = MockTaskShutdownState::ShutdownComplete
            }
            MockTaskPriority::Medium => {
                status.medium_priority_status = MockTaskShutdownState::ShutdownComplete
            }
            MockTaskPriority::Low => {
                status.low_priority_status = MockTaskShutdownState::ShutdownComplete
            }
        }
    }

    /// Check if shutdown is requested for a priority level
    pub fn is_shutdown_requested(&self, priority: MockTaskPriority) -> bool {
        let status = self.task_shutdown_status.lock().unwrap();
        match priority {
            MockTaskPriority::High => {
                status.high_priority_status == MockTaskShutdownState::ShutdownRequested
            }
            MockTaskPriority::Medium => {
                status.medium_priority_status == MockTaskShutdownState::ShutdownRequested
            }
            MockTaskPriority::Low => {
                status.low_priority_status == MockTaskShutdownState::ShutdownRequested
            }
        }
    }

    /// Get error count
    pub fn get_error_count(&self) -> u32 {
        *self.error_count.lock().unwrap()
    }

    /// Enable/disable error simulation
    pub fn set_simulate_errors(&self, simulate: bool) {
        *self.simulate_errors.lock().unwrap() = simulate;
    }

    /// Clear flash operations history
    pub fn clear_operations(&self) {
        self.flash_operations.lock().unwrap().clear();
        *self.error_count.lock().unwrap() = 0;
    }

    /// Check if bootloader entry is in progress
    pub fn is_entry_in_progress(&self) -> bool {
        *self.entry_state.lock().unwrap() != MockBootloaderEntryState::Normal
    }

    /// Get remaining time for bootloader entry
    pub fn get_remaining_time_ms(&self, current_time_ms: u32) -> Option<u32> {
        if let Some(start_time) = *self.entry_start_time.lock().unwrap() {
            let timeout = *self.entry_timeout_ms.lock().unwrap();
            let elapsed = current_time_ms.saturating_sub(start_time);
            Some(timeout.saturating_sub(elapsed))
        } else {
            None
        }
    }

    /// Check if currently in bootloader mode (convenience method for tests)
    pub fn is_in_bootloader_mode(&self) -> bool {
        *self.entry_state.lock().unwrap() == MockBootloaderEntryState::ReadyForBootloader
    }

    /// Prepare for bootloader entry (convenience method for tests)
    pub fn prepare_for_bootloader_entry(&self) -> Result<(), MockBootloaderError> {
        *self.entry_state.lock().unwrap() = MockBootloaderEntryState::ReadyForBootloader;
        Ok(())
    }
}

impl Default for MockBootloaderHardware {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock bootloader error types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MockBootloaderError {
    UnsafeHardwareState,
    TaskShutdownFailed,
    HardwareValidationFailed,
    SystemBusy,
    EntryInterrupted,
}

/// Mock test environment that combines all mock components
/// Provides a comprehensive testing environment that simulates the entire embedded system
#[derive(Debug, Clone)]
pub struct MockTestEnvironment {
    pub battery: MockBatteryMonitor,
    pub usb_hid: MockUsbHidDevice,
    pub system_state: MockSystemState,
    pub bootloader: MockBootloaderHardware,

    // Environment state
    current_time_ms: Arc<Mutex<u32>>,
    test_scenarios: Arc<Mutex<Vec<MockTestScenario>>>,
    active_scenario: Arc<Mutex<Option<String>>>,

    // Test configuration
    error_injection_enabled: Arc<Mutex<bool>>,
    performance_monitoring: Arc<Mutex<bool>>,
    test_results: Arc<Mutex<Vec<MockTestResult>>>,
}

/// Mock test scenario for automated testing
#[derive(Debug, Clone)]
pub struct MockTestScenario {
    pub name: String,
    pub description: String,
    pub duration_ms: u32,
    pub steps: Vec<MockTestStep>,
    pub expected_outcomes: Vec<String>,
}

/// Mock test step within a scenario
#[derive(Debug, Clone)]
pub struct MockTestStep {
    pub timestamp_ms: u32,
    pub action: MockTestAction,
    pub expected_result: Option<String>,
}

/// Mock test actions that can be performed
#[derive(Debug, Clone)]
pub enum MockTestAction {
    SetBatteryVoltage(u32),
    SetBatteryState(BatteryState),
    SendUsbCommand(UsbControlCommand, Option<Vec<u8>>),
    SimulateUsbDisconnect,
    SimulateUsbReconnect,
    RequestBootloaderEntry(u32),
    UpdateSystemTime(u32),
    InjectError(String),
    ValidateSystemState,
    CheckPerformanceMetrics,
}

/// Mock test result
#[derive(Debug, Clone)]
pub struct MockTestResult {
    pub timestamp_ms: u32,
    pub test_name: String,
    pub action: String,
    pub result: MockTestResultType,
    pub details: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MockTestResultType {
    Pass,
    Fail,
    Warning,
    Info,
}

impl MockTestEnvironment {
    /// Create a new mock test environment with all components
    pub fn new() -> Self {
        Self {
            battery: MockBatteryMonitor::new(),
            usb_hid: MockUsbHidDevice::new(),
            system_state: MockSystemState::new(),
            bootloader: MockBootloaderHardware::new(),
            current_time_ms: Arc::new(Mutex::new(0)),
            test_scenarios: Arc::new(Mutex::new(Vec::new())),
            active_scenario: Arc::new(Mutex::new(None)),
            error_injection_enabled: Arc::new(Mutex::new(false)),
            performance_monitoring: Arc::new(Mutex::new(true)),
            test_results: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a test environment with specific configuration
    pub fn with_config(
        battery_voltage_mv: u32,
        usb_connected: bool,
        error_injection: bool,
    ) -> Self {
        let env = Self::new();

        // Configure battery
        env.battery.set_voltage(battery_voltage_mv);

        // Configure USB
        if !usb_connected {
            env.usb_hid.disconnect();
        }

        // Configure error injection
        *env.error_injection_enabled.lock().unwrap() = error_injection;
        env.battery.set_simulate_errors(error_injection);
        env.usb_hid.set_simulate_errors(error_injection);
        env.bootloader.set_simulate_errors(error_injection);

        env
    }

    /// Advance system time and update all components
    pub fn advance_time(&self, delta_ms: u32) {
        let new_time = {
            let mut current_time = self.current_time_ms.lock().unwrap();
            *current_time += delta_ms;
            *current_time
        };

        // Update system state with new time
        self.system_state.set_uptime(new_time);
        self.system_state.simulate_operation(delta_ms);

        // Update bootloader if entry is in progress
        if self.bootloader.is_entry_in_progress() {
            let _ = self.bootloader.update_entry_progress(new_time);
        }
    }

    /// Get current system time
    pub fn get_current_time(&self) -> u32 {
        *self.current_time_ms.lock().unwrap()
    }

    /// Execute a test scenario
    pub fn execute_scenario(
        &self,
        scenario: MockTestScenario,
    ) -> Result<Vec<MockTestResult>, String> {
        *self.active_scenario.lock().unwrap() = Some(scenario.name.clone());
        let mut results = Vec::new();

        let start_time = self.get_current_time();

        for step in &scenario.steps {
            // Advance time to step timestamp
            let target_time = start_time + step.timestamp_ms;
            let current_time = self.get_current_time();
            if target_time > current_time {
                self.advance_time(target_time - current_time);
            }

            // Execute the action
            let result = self.execute_test_action(&step.action);

            // Record result
            let test_result = MockTestResult {
                timestamp_ms: self.get_current_time(),
                test_name: scenario.name.clone(),
                action: format!("{:?}", step.action),
                result: if result.is_ok() {
                    MockTestResultType::Pass
                } else {
                    MockTestResultType::Fail
                },
                details: result.unwrap_or_else(|e| e),
            };

            results.push(test_result.clone());
            self.test_results.lock().unwrap().push(test_result);
        }

        *self.active_scenario.lock().unwrap() = None;
        Ok(results)
    }

    /// Execute a single test action
    pub fn execute_test_action(&self, action: &MockTestAction) -> Result<String, String> {
        match action {
            MockTestAction::SetBatteryVoltage(voltage_mv) => {
                self.battery.set_voltage(*voltage_mv);
                Ok(format!("Battery voltage set to {}mV", voltage_mv))
            }
            MockTestAction::SetBatteryState(state) => {
                // Calculate voltage for the desired state
                let voltage_mv = match state {
                    BatteryState::Low => 3000,
                    BatteryState::Normal => 3500,
                    BatteryState::Charging => 4000,
                };
                self.battery.set_voltage(voltage_mv);
                Ok(format!("Battery state set to {:?}", state))
            }
            MockTestAction::SendUsbCommand(command, data) => {
                match self.usb_hid.process_control_command(*command, data.clone()) {
                    Ok(response) => Ok(format!(
                        "USB command {:?} sent, response: {:?}",
                        command, response
                    )),
                    Err(e) => Err(format!("USB command failed: {:?}", e)),
                }
            }
            MockTestAction::SimulateUsbDisconnect => {
                self.usb_hid.disconnect();
                Ok("USB disconnected".to_string())
            }
            MockTestAction::SimulateUsbReconnect => {
                self.usb_hid.reconnect();
                Ok("USB reconnected".to_string())
            }
            MockTestAction::RequestBootloaderEntry(timeout_ms) => {
                let current_time = self.get_current_time();
                match self
                    .bootloader
                    .request_bootloader_entry(*timeout_ms, current_time)
                {
                    Ok(()) => Ok(format!(
                        "Bootloader entry requested with {}ms timeout",
                        timeout_ms
                    )),
                    Err(e) => Err(format!("Bootloader entry failed: {:?}", e)),
                }
            }
            MockTestAction::UpdateSystemTime(time_ms) => {
                *self.current_time_ms.lock().unwrap() = *time_ms;
                self.system_state.set_uptime(*time_ms);
                Ok(format!("System time updated to {}ms", time_ms))
            }
            MockTestAction::InjectError(error_type) => {
                // Enable error simulation on relevant components
                match error_type.as_str() {
                    "battery" => self.battery.set_simulate_errors(true),
                    "usb" => self.usb_hid.set_simulate_errors(true),
                    "bootloader" => self.bootloader.set_simulate_errors(true),
                    _ => return Err(format!("Unknown error type: {}", error_type)),
                }
                Ok(format!("Error injection enabled for {}", error_type))
            }
            MockTestAction::ValidateSystemState => {
                let health = self.system_state.get_system_health();
                let battery_state = self.battery.get_state();
                let usb_connected = self.usb_hid.is_connected();

                let validation_result = format!(
                    "System validation: Battery={:?}, USB={}, Health={:?}",
                    battery_state, usb_connected, health.battery_state
                );

                Ok(validation_result)
            }
            MockTestAction::CheckPerformanceMetrics => {
                let performance = self.system_state.get_task_performance();
                let metrics_result = format!(
                    "Performance: CPU={}%, Memory={}%, Messages/sec={}",
                    performance.resource_usage.cpu_usage_percent,
                    performance.resource_usage.memory_usage_percent,
                    performance.performance_metrics.messages_per_second
                );
                Ok(metrics_result)
            }
        }
    }

    /// Reset all mock components to initial state
    pub fn reset(&self) {
        self.battery.reset_state();
        self.usb_hid.clear_messages();
        self.system_state.clear_state();
        self.bootloader.reset_entry_state();
        *self.current_time_ms.lock().unwrap() = 0;
    }
} // Close MockTestEnvironment impl block
