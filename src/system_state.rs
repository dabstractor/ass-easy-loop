//! System State Query Handler Module
//! 
//! This module implements system state query handlers for the automated testing framework.
//! It provides comprehensive system health data collection, hardware status reporting,
//! and configuration dump functionality.
//! 
//! Requirements: 3.1, 3.2, 3.3, 3.4, 3.5

use heapless::Vec;
use crate::config::LogConfig;
use crate::battery::BatteryState;
use crate::command::parsing::{CommandReport, TestResponse, ErrorCode};
use crate::error_handling::{SystemError, SystemResult};
use core::option::Option::{self, Some, None};
use core::result::Result::{self, Ok};
use core::iter::Iterator;
use core::clone::Clone;

/// System state query types
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum StateQueryType {
    SystemHealth = 0x01,
    TaskPerformance = 0x02,
    HardwareStatus = 0x03,
    ConfigurationDump = 0x04,
    ErrorHistory = 0x05,
}

impl StateQueryType {
    /// Convert from u8 to StateQueryType
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(StateQueryType::SystemHealth),
            0x02 => Some(StateQueryType::TaskPerformance),
            0x03 => Some(StateQueryType::HardwareStatus),
            0x04 => Some(StateQueryType::ConfigurationDump),
            0x05 => Some(StateQueryType::ErrorHistory),
            _ => None,
        }
    }
}

/// System health data structure
/// Requirements: 3.1 (system status including battery state, pEMF status, and task health)
#[derive(Clone, Copy, Debug)]
pub struct SystemHealthData {
    pub uptime_ms: u32,
    pub battery_state: BatteryState,
    pub battery_voltage_mv: u32,
    pub pemf_active: bool,
    pub pemf_cycle_count: u32,
    pub task_health_status: TaskHealthStatus,
    pub memory_usage: MemoryUsageStats,
    pub error_counts: ErrorCounters,
    pub system_temperature: Option<u16>, // In 0.1°C units
}

/// Task health status for all system tasks
/// Requirements: 3.1 (task health monitoring)
#[derive(Clone, Copy, Debug)]
pub struct TaskHealthStatus {
    pub pemf_task_healthy: bool,
    pub battery_task_healthy: bool,
    pub led_task_healthy: bool,
    pub usb_poll_task_healthy: bool,
    pub usb_hid_task_healthy: bool,
    pub command_handler_healthy: bool,
    pub last_health_check_ms: u32,
}

/// Memory usage statistics
/// Requirements: 3.1 (memory usage data)
#[derive(Clone, Copy, Debug)]
pub struct MemoryUsageStats {
    pub stack_usage_bytes: u32,
    pub heap_usage_bytes: u32, // Should be 0 for no_std
    pub log_queue_usage_bytes: u32,
    pub command_queue_usage_bytes: u32,
    pub total_ram_usage_bytes: u32,
    pub peak_ram_usage_bytes: u32,
    pub memory_fragmentation_percent: u8,
}

/// Error counters for system monitoring
/// Requirements: 3.1 (error counts for system health)
#[derive(Clone, Copy, Debug)]
pub struct ErrorCounters {
    pub adc_read_errors: u32,
    pub gpio_operation_errors: u32,
    pub usb_transmission_errors: u32,
    pub command_parsing_errors: u32,
    pub timing_violations: u32,
    pub bootloader_entry_failures: u32,
    pub total_error_count: u32,
}

/// Task performance data
/// Requirements: 3.2 (timing statistics, error counts, and resource usage data)
#[derive(Clone, Copy, Debug)]
pub struct TaskPerformanceData {
    pub task_execution_times: TaskExecutionTimes,
    pub timing_statistics: TimingStatistics,
    pub resource_usage: ResourceUsageData,
    pub performance_metrics: PerformanceMetrics,
}

/// Task execution times in microseconds
#[derive(Clone, Copy, Debug)]
pub struct TaskExecutionTimes {
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

/// Timing statistics for system validation
#[derive(Clone, Copy, Debug)]
pub struct TimingStatistics {
    pub pemf_frequency_hz: f32,
    pub pemf_timing_accuracy_percent: f32,
    pub battery_sampling_rate_hz: f32,
    pub max_timing_deviation_us: u32,
    pub timing_violations_count: u32,
    pub jitter_measurements_us: JitterMeasurements,
}

/// Jitter measurements for timing validation
#[derive(Clone, Copy, Debug)]
pub struct JitterMeasurements {
    pub pemf_jitter_us: u32,
    pub battery_jitter_us: u32,
    pub usb_jitter_us: u32,
    pub max_system_jitter_us: u32,
}

/// Resource usage data
#[derive(Clone, Copy, Debug)]
pub struct ResourceUsageData {
    pub cpu_usage_percent: u8,
    pub memory_usage_percent: u8,
    pub queue_utilization_percent: u8,
    pub usb_bandwidth_usage_percent: u8,
    pub interrupt_load_percent: u8,
}

/// Performance metrics
#[derive(Clone, Copy, Debug)]
pub struct PerformanceMetrics {
    pub messages_per_second: u32,
    pub commands_processed_per_second: u32,
    pub average_response_time_ms: u32,
    pub throughput_bytes_per_second: u32,
    pub system_efficiency_percent: u8,
}

/// Hardware status data
/// Requirements: 3.3 (GPIO states, ADC readings, USB status)
#[derive(Clone, Copy, Debug)]
pub struct HardwareStatusData {
    pub gpio_states: GpioStates,
    pub adc_readings: AdcReadings,
    pub usb_status: UsbStatus,
    pub power_status: PowerStatus,
    pub sensor_readings: SensorReadings,
}

/// GPIO pin states
#[derive(Clone, Copy, Debug)]
pub struct GpioStates {
    pub mosfet_pin_state: bool,     // GPIO 15
    pub led_pin_state: bool,        // GPIO 25
    pub adc_pin_voltage_mv: u32,    // GPIO 26
    pub bootsel_pin_state: bool,    // BOOTSEL button
    pub gpio_error_count: u32,
}

/// ADC readings and calibration data
#[derive(Clone, Copy, Debug)]
pub struct AdcReadings {
    pub battery_adc_raw: u16,
    pub battery_voltage_mv: u32,
    pub internal_temperature_c: i16,
    pub vref_voltage_mv: u32,
    pub adc_calibration_offset: i16,
    pub adc_error_count: u32,
}

/// USB connection and communication status
#[derive(Clone, Copy, Debug)]
pub struct UsbStatus {
    pub connected: bool,
    pub configured: bool,
    pub suspended: bool,
    pub enumerated: bool,
    pub hid_reports_sent: u32,
    pub hid_reports_received: u32,
    pub usb_errors: u32,
    pub last_activity_ms: u32,
}

/// Power management status
#[derive(Clone, Copy, Debug)]
pub struct PowerStatus {
    pub supply_voltage_mv: u32,
    pub core_voltage_mv: u32,
    pub power_consumption_mw: u32,
    pub battery_charging: bool,
    pub low_power_mode: bool,
    pub power_good: bool,
}

/// Sensor readings and environmental data
#[derive(Clone, Copy, Debug)]
pub struct SensorReadings {
    pub internal_temperature_c: i16,
    pub cpu_temperature_c: i16,
    pub ambient_light_level: u16,
    pub magnetic_field_strength: u16,
    pub vibration_level: u16,
}

/// Configuration dump data
/// Requirements: 3.4 (current device settings)
#[derive(Clone, Debug)]
pub struct ConfigurationData {
    pub log_config: LogConfig,
    pub timing_config: TimingConfiguration,
    pub hardware_config: HardwareConfiguration,
    pub feature_flags: FeatureFlags,
    pub calibration_data: CalibrationData,
}

/// Timing configuration parameters
#[derive(Clone, Copy, Debug)]
pub struct TimingConfiguration {
    pub pemf_frequency_hz: f32,
    pub pemf_high_duration_ms: u64,
    pub pemf_low_duration_ms: u64,
    pub battery_sampling_interval_ms: u64,
    pub led_flash_rate_hz: f32,
    pub usb_poll_interval_ms: u64,
    pub timing_tolerance_percent: f32,
}

/// Hardware configuration settings
#[derive(Clone, Copy, Debug)]
pub struct HardwareConfiguration {
    pub mosfet_pin: u8,
    pub led_pin: u8,
    pub adc_pin: u8,
    pub adc_resolution_bits: u8,
    pub adc_reference_mv: u32,
    pub system_clock_hz: u32,
    pub external_crystal_hz: u32,
}

/// Feature flags and compile-time options
#[derive(Clone, Copy, Debug)]
pub struct FeatureFlags {
    pub usb_hid_logging_enabled: bool,
    pub battery_monitoring_enabled: bool,
    pub pemf_generation_enabled: bool,
    pub led_control_enabled: bool,
    pub performance_monitoring_enabled: bool,
    pub debug_mode_enabled: bool,
    pub panic_logging_enabled: bool,
}

/// Calibration data for sensors and ADC
#[derive(Clone, Copy, Debug)]
pub struct CalibrationData {
    pub adc_offset: i16,
    pub adc_gain: f32,
    pub temperature_offset: i16,
    pub voltage_divider_ratio: f32,
    pub timing_calibration_factor: f32,
    pub last_calibration_timestamp: u32,
}

/// System State Handler for processing state queries
/// Requirements: 3.1, 3.2, 3.3, 3.4, 3.5
pub struct SystemStateHandler {
    performance_monitor: PerformanceMonitor,
    diagnostic_collector: DiagnosticCollector,
    config_manager: ConfigurationManager,
    error_history: ErrorHistory,
    last_query_timestamp: u32,
    query_count: u32,
}

impl Default for SystemStateHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemStateHandler {
    /// Create a new system state handler
    pub const fn new() -> Self {
        Self {
            performance_monitor: PerformanceMonitor::new(),
            diagnostic_collector: DiagnosticCollector::new(),
            config_manager: ConfigurationManager::new(),
            error_history: ErrorHistory::new(),
            last_query_timestamp: 0,
            query_count: 0,
        }
    }

    /// Process a system state query and return response data
    /// Requirements: 3.1, 3.2, 3.3, 3.4, 3.5
    pub fn process_state_query(
        &mut self,
        query_type: StateQueryType,
        timestamp_ms: u32,
    ) -> SystemResult<Vec<u8, 60>> {
        self.last_query_timestamp = timestamp_ms;
        self.query_count = self.query_count.saturating_add(1);

        match query_type {
            StateQueryType::SystemHealth => {
                let health_data = self.collect_system_health_data(timestamp_ms)?;
                self.serialize_system_health(&health_data)
            }
            StateQueryType::TaskPerformance => {
                let performance_data = self.collect_task_performance_data()?;
                self.serialize_task_performance(&performance_data)
            }
            StateQueryType::HardwareStatus => {
                let hardware_data = self.collect_hardware_status_data()?;
                self.serialize_hardware_status(&hardware_data)
            }
            StateQueryType::ConfigurationDump => {
                let config_data = self.collect_configuration_data()?;
                self.serialize_configuration(&config_data)
            }
            StateQueryType::ErrorHistory => {
                let error_data = self.collect_error_history_data()?;
                self.serialize_error_history(&error_data)
            }
        }
    }

    /// Collect comprehensive system health data
    /// Requirements: 3.1 (system status including battery state, pEMF status, and task health)
    fn collect_system_health_data(&mut self, timestamp_ms: u32) -> SystemResult<SystemHealthData> {
        // Collect uptime information
        let uptime_ms = timestamp_ms; // Assuming timestamp is uptime since boot

        // Collect battery information (placeholder - would integrate with actual battery monitoring)
        let battery_state = BatteryState::Normal; // Would get from shared RTIC resource
        let battery_voltage_mv = 3300; // Would get from actual ADC reading

        // Collect pEMF status (placeholder - would integrate with actual pEMF task)
        let pemf_active = true; // Would get from shared RTIC resource
        let pemf_cycle_count = uptime_ms / 500; // Approximate cycles at 2Hz

        // Collect task health status
        let task_health_status = self.collect_task_health_status(timestamp_ms)?;

        // Collect memory usage statistics
        let memory_usage = self.collect_memory_usage_stats()?;

        // Collect error counters
        let error_counts = self.collect_error_counters()?;

        // System temperature (if available)
        let system_temperature = Some(250); // 25.0°C in 0.1°C units

        Ok(SystemHealthData {
            uptime_ms,
            battery_state,
            battery_voltage_mv,
            pemf_active,
            pemf_cycle_count,
            task_health_status,
            memory_usage,
            error_counts,
            system_temperature,
        })
    }

    /// Collect task health status for all system tasks
    fn collect_task_health_status(&self, timestamp_ms: u32) -> SystemResult<TaskHealthStatus> {
        // In a real implementation, this would check actual task status
        // For now, we'll assume all tasks are healthy
        Ok(TaskHealthStatus {
            pemf_task_healthy: true,
            battery_task_healthy: true,
            led_task_healthy: true,
            usb_poll_task_healthy: true,
            usb_hid_task_healthy: true,
            command_handler_healthy: true,
            last_health_check_ms: timestamp_ms,
        })
    }

    /// Collect memory usage statistics
    fn collect_memory_usage_stats(&self) -> SystemResult<MemoryUsageStats> {
        // Estimate memory usage based on known data structures
        let log_queue_usage_bytes = 32 * 64; // 32 messages * 64 bytes each
        let command_queue_usage_bytes = 8 * 64; // 8 commands * 64 bytes each
        let stack_usage_bytes = 4096; // Estimated stack usage
        let heap_usage_bytes = 0; // No heap in no_std
        let total_ram_usage_bytes = log_queue_usage_bytes + command_queue_usage_bytes + stack_usage_bytes;
        let peak_ram_usage_bytes = total_ram_usage_bytes; // Simplified
        let memory_fragmentation_percent = 0; // No fragmentation in static allocation

        Ok(MemoryUsageStats {
            stack_usage_bytes,
            heap_usage_bytes,
            log_queue_usage_bytes,
            command_queue_usage_bytes,
            total_ram_usage_bytes,
            peak_ram_usage_bytes,
            memory_fragmentation_percent,
        })
    }

    /// Collect error counters from various system components
    fn collect_error_counters(&self) -> SystemResult<ErrorCounters> {
        // In a real implementation, these would be collected from actual error tracking
        Ok(ErrorCounters {
            adc_read_errors: 0,
            gpio_operation_errors: 0,
            usb_transmission_errors: 0,
            command_parsing_errors: 0,
            timing_violations: 0,
            bootloader_entry_failures: 0,
            total_error_count: 0,
        })
    }

    /// Collect task performance data
    /// Requirements: 3.2 (timing statistics, error counts, and resource usage data)
    fn collect_task_performance_data(&mut self) -> SystemResult<TaskPerformanceData> {
        let task_execution_times = self.performance_monitor.get_task_execution_times();
        let timing_statistics = self.performance_monitor.get_timing_statistics();
        let resource_usage = self.performance_monitor.get_resource_usage();
        let performance_metrics = self.performance_monitor.get_performance_metrics();

        Ok(TaskPerformanceData {
            task_execution_times,
            timing_statistics,
            resource_usage,
            performance_metrics,
        })
    }

    /// Collect hardware status data
    /// Requirements: 3.3 (GPIO states, ADC readings, USB status)
    fn collect_hardware_status_data(&mut self) -> SystemResult<HardwareStatusData> {
        let gpio_states = self.diagnostic_collector.get_gpio_states();
        let adc_readings = self.diagnostic_collector.get_adc_readings();
        let usb_status = self.diagnostic_collector.get_usb_status();
        let power_status = self.diagnostic_collector.get_power_status();
        let sensor_readings = self.diagnostic_collector.get_sensor_readings();

        Ok(HardwareStatusData {
            gpio_states,
            adc_readings,
            usb_status,
            power_status,
            sensor_readings,
        })
    }

    /// Collect configuration data
    /// Requirements: 3.4 (current device settings)
    fn collect_configuration_data(&mut self) -> SystemResult<ConfigurationData> {
        let log_config = self.config_manager.get_log_config();
        let timing_config = self.config_manager.get_timing_config();
        let hardware_config = self.config_manager.get_hardware_config();
        let feature_flags = self.config_manager.get_feature_flags();
        let calibration_data = self.config_manager.get_calibration_data();

        Ok(ConfigurationData {
            log_config,
            timing_config,
            hardware_config,
            feature_flags,
            calibration_data,
        })
    }

    /// Collect error history data
    /// Requirements: 3.5 (system health checks and validation)
    fn collect_error_history_data(&mut self) -> SystemResult<Vec<u8, 60>> {
        let error_history = self.error_history.get_recent_errors();
        let mut serialized = Vec::new();

        // Serialize error history (simplified format)
        let error_count = core::cmp::min(error_history.len(), 15); // Limit to fit in payload
        serialized.push(error_count as u8).map_err(|_| SystemError::SystemBusy)?;

        for error in error_history.iter().take(error_count) {
            serialized.push(error.error_type as u8).map_err(|_| SystemError::SystemBusy)?;
            serialized.push((error.timestamp_ms & 0xFF) as u8).map_err(|_| SystemError::SystemBusy)?;
            serialized.push(((error.timestamp_ms >> 8) & 0xFF) as u8).map_err(|_| SystemError::SystemBusy)?;
            serialized.push(((error.timestamp_ms >> 16) & 0xFF) as u8).map_err(|_| SystemError::SystemBusy)?;
        }

        Ok(serialized)
    }

    /// Serialize system health data to bytes
    fn serialize_system_health(&self, data: &SystemHealthData) -> SystemResult<Vec<u8, 60>> {
        let mut serialized = Vec::new();

        // Serialize uptime (4 bytes)
        let uptime_bytes = data.uptime_ms.to_le_bytes();
        for &byte in &uptime_bytes {
            serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
        }

        // Serialize battery state (1 byte)
        let battery_state_byte = match data.battery_state {
            BatteryState::Low => 0,
            BatteryState::Normal => 1,
            BatteryState::Charging => 2,
        };
        serialized.push(battery_state_byte).map_err(|_| SystemError::SystemBusy)?;

        // Serialize battery voltage (4 bytes)
        let voltage_bytes = data.battery_voltage_mv.to_le_bytes();
        for &byte in &voltage_bytes {
            serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
        }

        // Serialize pEMF status (1 byte + 4 bytes)
        serialized.push(if data.pemf_active { 1 } else { 0 }).map_err(|_| SystemError::SystemBusy)?;
        let cycle_bytes = data.pemf_cycle_count.to_le_bytes();
        for &byte in &cycle_bytes {
            serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
        }

        // Serialize task health (1 byte bitfield)
        let mut task_health_byte = 0u8;
        if data.task_health_status.pemf_task_healthy { task_health_byte |= 0x01; }
        if data.task_health_status.battery_task_healthy { task_health_byte |= 0x02; }
        if data.task_health_status.led_task_healthy { task_health_byte |= 0x04; }
        if data.task_health_status.usb_poll_task_healthy { task_health_byte |= 0x08; }
        if data.task_health_status.usb_hid_task_healthy { task_health_byte |= 0x10; }
        if data.task_health_status.command_handler_healthy { task_health_byte |= 0x20; }
        serialized.push(task_health_byte).map_err(|_| SystemError::SystemBusy)?;

        // Serialize memory usage (simplified - 8 bytes)
        let memory_bytes = data.memory_usage.total_ram_usage_bytes.to_le_bytes();
        for &byte in &memory_bytes {
            serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
        }
        let peak_bytes = data.memory_usage.peak_ram_usage_bytes.to_le_bytes();
        for &byte in &peak_bytes {
            serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
        }

        // Serialize error counts (4 bytes)
        let error_bytes = data.error_counts.total_error_count.to_le_bytes();
        for &byte in &error_bytes {
            serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
        }

        // Serialize temperature if available (2 bytes)
        if let Some(temp) = data.system_temperature {
            let temp_bytes = temp.to_le_bytes();
            for &byte in &temp_bytes {
                serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
            }
        } else {
            serialized.push(0xFF).map_err(|_| SystemError::SystemBusy)?;
            serialized.push(0xFF).map_err(|_| SystemError::SystemBusy)?;
        }

        Ok(serialized)
    }

    /// Serialize task performance data to bytes
    fn serialize_task_performance(&self, data: &TaskPerformanceData) -> SystemResult<Vec<u8, 60>> {
        let mut serialized = Vec::new();

        // Serialize task execution times (20 bytes - 4 bytes per task average)
        let times = [
            data.task_execution_times.pemf_task_avg_us,
            data.task_execution_times.battery_task_avg_us,
            data.task_execution_times.led_task_avg_us,
            data.task_execution_times.usb_poll_avg_us,
            data.task_execution_times.usb_hid_avg_us,
        ];

        for time in &times {
            let time_bytes = time.to_le_bytes();
            for &byte in &time_bytes {
                serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
            }
        }

        // Serialize timing statistics (12 bytes)
        let freq_bytes = (data.timing_statistics.pemf_frequency_hz * 1000.0) as u32;
        let freq_le_bytes = freq_bytes.to_le_bytes();
        for &byte in &freq_le_bytes {
            serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
        }

        let accuracy_bytes = (data.timing_statistics.pemf_timing_accuracy_percent * 100.0) as u32;
        let accuracy_le_bytes = accuracy_bytes.to_le_bytes();
        for &byte in &accuracy_le_bytes {
            serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
        }

        let deviation_bytes = data.timing_statistics.max_timing_deviation_us.to_le_bytes();
        for &byte in &deviation_bytes {
            serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
        }

        // Serialize resource usage (5 bytes)
        serialized.push(data.resource_usage.cpu_usage_percent).map_err(|_| SystemError::SystemBusy)?;
        serialized.push(data.resource_usage.memory_usage_percent).map_err(|_| SystemError::SystemBusy)?;
        serialized.push(data.resource_usage.queue_utilization_percent).map_err(|_| SystemError::SystemBusy)?;
        serialized.push(data.resource_usage.usb_bandwidth_usage_percent).map_err(|_| SystemError::SystemBusy)?;
        serialized.push(data.resource_usage.interrupt_load_percent).map_err(|_| SystemError::SystemBusy)?;

        // Serialize performance metrics (13 bytes)
        let metrics = [
            data.performance_metrics.messages_per_second,
            data.performance_metrics.commands_processed_per_second,
            data.performance_metrics.average_response_time_ms,
        ];

        for metric in &metrics {
            let metric_bytes = metric.to_le_bytes();
            for &byte in &metric_bytes {
                serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
            }
        }

        serialized.push(data.performance_metrics.system_efficiency_percent).map_err(|_| SystemError::SystemBusy)?;

        Ok(serialized)
    }

    /// Serialize hardware status data to bytes
    fn serialize_hardware_status(&self, data: &HardwareStatusData) -> SystemResult<Vec<u8, 60>> {
        let mut serialized = Vec::new();

        // Serialize GPIO states (9 bytes)
        serialized.push(if data.gpio_states.mosfet_pin_state { 1 } else { 0 }).map_err(|_| SystemError::SystemBusy)?;
        serialized.push(if data.gpio_states.led_pin_state { 1 } else { 0 }).map_err(|_| SystemError::SystemBusy)?;
        
        let adc_voltage_bytes = data.gpio_states.adc_pin_voltage_mv.to_le_bytes();
        for &byte in &adc_voltage_bytes {
            serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
        }

        serialized.push(if data.gpio_states.bootsel_pin_state { 1 } else { 0 }).map_err(|_| SystemError::SystemBusy)?;

        let gpio_error_bytes = data.gpio_states.gpio_error_count.to_le_bytes();
        for &byte in &gpio_error_bytes {
            serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
        }

        // Serialize ADC readings (14 bytes)
        let adc_raw_bytes = data.adc_readings.battery_adc_raw.to_le_bytes();
        for &byte in &adc_raw_bytes {
            serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
        }

        let battery_voltage_bytes = data.adc_readings.battery_voltage_mv.to_le_bytes();
        for &byte in &battery_voltage_bytes {
            serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
        }

        let temp_bytes = data.adc_readings.internal_temperature_c.to_le_bytes();
        for &byte in &temp_bytes {
            serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
        }

        let vref_bytes = data.adc_readings.vref_voltage_mv.to_le_bytes();
        for &byte in &vref_bytes {
            serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
        }

        let adc_error_bytes = data.adc_readings.adc_error_count.to_le_bytes();
        for &byte in &adc_error_bytes {
            serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
        }

        // Serialize USB status (16 bytes)
        let mut usb_status_byte = 0u8;
        if data.usb_status.connected { usb_status_byte |= 0x01; }
        if data.usb_status.configured { usb_status_byte |= 0x02; }
        if data.usb_status.suspended { usb_status_byte |= 0x04; }
        if data.usb_status.enumerated { usb_status_byte |= 0x08; }
        serialized.push(usb_status_byte).map_err(|_| SystemError::SystemBusy)?;

        let usb_metrics = [
            data.usb_status.hid_reports_sent,
            data.usb_status.hid_reports_received,
            data.usb_status.usb_errors,
            data.usb_status.last_activity_ms,
        ];

        for metric in &usb_metrics {
            let metric_bytes = metric.to_le_bytes();
            for &byte in &metric_bytes {
                serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
            }
        }

        // Serialize power status (12 bytes)
        let power_metrics = [
            data.power_status.supply_voltage_mv,
            data.power_status.core_voltage_mv,
            data.power_status.power_consumption_mw,
        ];

        for metric in &power_metrics {
            let metric_bytes = metric.to_le_bytes();
            for &byte in &metric_bytes {
                serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
            }
        }

        Ok(serialized)
    }

    /// Serialize configuration data to bytes
    fn serialize_configuration(&self, data: &ConfigurationData) -> SystemResult<Vec<u8, 60>> {
        let mut serialized = Vec::new();

        // Serialize log configuration (16 bytes)
        let log_config_bytes = data.log_config.serialize();
        for &byte in &log_config_bytes {
            serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
        }

        // Serialize timing configuration (16 bytes)
        let freq_bytes = (data.timing_config.pemf_frequency_hz * 1000.0) as u32;
        let freq_le_bytes = freq_bytes.to_le_bytes();
        for &byte in &freq_le_bytes {
            serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
        }

        let high_duration_bytes = data.timing_config.pemf_high_duration_ms.to_le_bytes();
        for &byte in &high_duration_bytes {
            serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
        }

        let low_duration_bytes = data.timing_config.pemf_low_duration_ms.to_le_bytes();
        for &byte in &low_duration_bytes {
            serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
        }

        // Serialize hardware configuration (12 bytes)
        serialized.push(data.hardware_config.mosfet_pin).map_err(|_| SystemError::SystemBusy)?;
        serialized.push(data.hardware_config.led_pin).map_err(|_| SystemError::SystemBusy)?;
        serialized.push(data.hardware_config.adc_pin).map_err(|_| SystemError::SystemBusy)?;
        serialized.push(data.hardware_config.adc_resolution_bits).map_err(|_| SystemError::SystemBusy)?;

        let adc_ref_bytes = data.hardware_config.adc_reference_mv.to_le_bytes();
        for &byte in &adc_ref_bytes {
            serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
        }

        let sys_clock_bytes = data.hardware_config.system_clock_hz.to_le_bytes();
        for &byte in &sys_clock_bytes {
            serialized.push(byte).map_err(|_| SystemError::SystemBusy)?;
        }

        // Serialize feature flags (1 byte)
        let mut feature_byte = 0u8;
        if data.feature_flags.usb_hid_logging_enabled { feature_byte |= 0x01; }
        if data.feature_flags.battery_monitoring_enabled { feature_byte |= 0x02; }
        if data.feature_flags.pemf_generation_enabled { feature_byte |= 0x04; }
        if data.feature_flags.led_control_enabled { feature_byte |= 0x08; }
        if data.feature_flags.performance_monitoring_enabled { feature_byte |= 0x10; }
        if data.feature_flags.debug_mode_enabled { feature_byte |= 0x20; }
        if data.feature_flags.panic_logging_enabled { feature_byte |= 0x40; }
        serialized.push(feature_byte).map_err(|_| SystemError::SystemBusy)?;

        Ok(serialized)
    }

    /// Serialize error history data to bytes
    fn serialize_error_history(&self, data: &Vec<u8, 60>) -> SystemResult<Vec<u8, 60>> {
        // Error history is already serialized in collect_error_history_data
        Ok(data.clone())
    }

    /// Create a command response for state query results
    pub fn create_state_response(
        &self,
        command_id: u8,
        query_type: StateQueryType,
        data: &[u8],
    ) -> Result<CommandReport, ErrorCode> {
        let mut payload: Vec<u8, 60> = Vec::new();
        
        // Add query type to payload
        payload.push(query_type as u8).map_err(|_| ErrorCode::PayloadTooLarge)?;
        
        // Add data length
        payload.push(data.len() as u8).map_err(|_| ErrorCode::PayloadTooLarge)?;
        
        // Add data (truncate if necessary)
        let max_data_len = core::cmp::min(data.len(), 58); // Reserve 2 bytes for type and length
        for &byte in &data[..max_data_len] {
            payload.push(byte).map_err(|_| ErrorCode::PayloadTooLarge)?;
        }

        CommandReport::new(TestResponse::StateData as u8, command_id, &payload)
    }

    /// Get handler statistics
    pub fn get_stats(&self) -> StateHandlerStats {
        StateHandlerStats {
            queries_processed: self.query_count,
            last_query_timestamp: self.last_query_timestamp,
            performance_queries: self.performance_monitor.get_query_count(),
            hardware_queries: self.diagnostic_collector.get_query_count(),
            config_queries: self.config_manager.get_query_count(),
            error_queries: self.error_history.get_query_count(),
        }
    }
}

/// Statistics for the system state handler
#[derive(Clone, Copy, Debug)]
pub struct StateHandlerStats {
    pub queries_processed: u32,
    pub last_query_timestamp: u32,
    pub performance_queries: u32,
    pub hardware_queries: u32,
    pub config_queries: u32,
    pub error_queries: u32,
}

/// Performance monitor for collecting task execution and timing data
pub struct PerformanceMonitor {
    query_count: u32,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMonitor {
    pub const fn new() -> Self {
        Self { query_count: 0 }
    }

    pub fn get_task_execution_times(&mut self) -> TaskExecutionTimes {
        self.query_count = self.query_count.saturating_add(1);
        // In a real implementation, this would collect actual timing data
        TaskExecutionTimes {
            pemf_task_avg_us: 50,
            pemf_task_max_us: 100,
            battery_task_avg_us: 200,
            battery_task_max_us: 500,
            led_task_avg_us: 30,
            led_task_max_us: 80,
            usb_poll_avg_us: 100,
            usb_poll_max_us: 300,
            usb_hid_avg_us: 150,
            usb_hid_max_us: 400,
        }
    }

    pub fn get_timing_statistics(&self) -> TimingStatistics {
        TimingStatistics {
            pemf_frequency_hz: 2.0,
            pemf_timing_accuracy_percent: 99.5,
            battery_sampling_rate_hz: 10.0,
            max_timing_deviation_us: 100,
            timing_violations_count: 0,
            jitter_measurements_us: JitterMeasurements {
                pemf_jitter_us: 10,
                battery_jitter_us: 50,
                usb_jitter_us: 200,
                max_system_jitter_us: 200,
            },
        }
    }

    pub fn get_resource_usage(&self) -> ResourceUsageData {
        ResourceUsageData {
            cpu_usage_percent: 15,
            memory_usage_percent: 25,
            queue_utilization_percent: 30,
            usb_bandwidth_usage_percent: 10,
            interrupt_load_percent: 20,
        }
    }

    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            messages_per_second: 100,
            commands_processed_per_second: 5,
            average_response_time_ms: 10,
            throughput_bytes_per_second: 6400,
            system_efficiency_percent: 85,
        }
    }

    pub fn get_query_count(&self) -> u32 {
        self.query_count
    }
}

/// Diagnostic collector for hardware status information
pub struct DiagnosticCollector {
    query_count: u32,
}

impl Default for DiagnosticCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticCollector {
    pub const fn new() -> Self {
        Self { query_count: 0 }
    }

    pub fn get_gpio_states(&mut self) -> GpioStates {
        self.query_count = self.query_count.saturating_add(1);
        // In a real implementation, this would read actual GPIO states
        GpioStates {
            mosfet_pin_state: false,
            led_pin_state: true,
            adc_pin_voltage_mv: 3300,
            bootsel_pin_state: false,
            gpio_error_count: 0,
        }
    }

    pub fn get_adc_readings(&self) -> AdcReadings {
        AdcReadings {
            battery_adc_raw: 1500,
            battery_voltage_mv: 3300,
            internal_temperature_c: 25,
            vref_voltage_mv: 3300,
            adc_calibration_offset: 0,
            adc_error_count: 0,
        }
    }

    pub fn get_usb_status(&self) -> UsbStatus {
        UsbStatus {
            connected: true,
            configured: true,
            suspended: false,
            enumerated: true,
            hid_reports_sent: 1000,
            hid_reports_received: 50,
            usb_errors: 0,
            last_activity_ms: 12345,
        }
    }

    pub fn get_power_status(&self) -> PowerStatus {
        PowerStatus {
            supply_voltage_mv: 3300,
            core_voltage_mv: 1200,
            power_consumption_mw: 150,
            battery_charging: false,
            low_power_mode: false,
            power_good: true,
        }
    }

    pub fn get_sensor_readings(&self) -> SensorReadings {
        SensorReadings {
            internal_temperature_c: 25,
            cpu_temperature_c: 30,
            ambient_light_level: 500,
            magnetic_field_strength: 100,
            vibration_level: 10,
        }
    }

    pub fn get_query_count(&self) -> u32 {
        self.query_count
    }
}

/// Configuration manager for device settings
pub struct ConfigurationManager {
    query_count: u32,
}

impl Default for ConfigurationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigurationManager {
    pub const fn new() -> Self {
        Self { query_count: 0 }
    }

    pub fn get_log_config(&mut self) -> LogConfig {
        self.query_count = self.query_count.saturating_add(1);
        LogConfig::new()
    }

    pub fn get_timing_config(&self) -> TimingConfiguration {
        TimingConfiguration {
            pemf_frequency_hz: 2.0,
            pemf_high_duration_ms: 2,
            pemf_low_duration_ms: 498,
            battery_sampling_interval_ms: 100,
            led_flash_rate_hz: 2.0,
            usb_poll_interval_ms: 1,
            timing_tolerance_percent: 0.01,
        }
    }

    pub fn get_hardware_config(&self) -> HardwareConfiguration {
        HardwareConfiguration {
            mosfet_pin: 15,
            led_pin: 25,
            adc_pin: 26,
            adc_resolution_bits: 12,
            adc_reference_mv: 3300,
            system_clock_hz: 125_000_000,
            external_crystal_hz: 12_000_000,
        }
    }

    pub fn get_feature_flags(&self) -> FeatureFlags {
        FeatureFlags {
            usb_hid_logging_enabled: true,
            battery_monitoring_enabled: true,
            pemf_generation_enabled: true,
            led_control_enabled: true,
            performance_monitoring_enabled: true,
            debug_mode_enabled: cfg!(debug_assertions),
            panic_logging_enabled: true,
        }
    }

    pub fn get_calibration_data(&self) -> CalibrationData {
        CalibrationData {
            adc_offset: 0,
            adc_gain: 1.0,
            temperature_offset: 0,
            voltage_divider_ratio: 0.337,
            timing_calibration_factor: 1.0,
            last_calibration_timestamp: 0,
        }
    }

    pub fn get_query_count(&self) -> u32 {
        self.query_count
    }
}

/// Error history tracker for system diagnostics
pub struct ErrorHistory {
    errors: Vec<ErrorRecord, 16>,
    query_count: u32,
}

impl Default for ErrorHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorHistory {
    pub const fn new() -> Self {
        Self {
            errors: Vec::new(),
            query_count: 0,
        }
    }

    pub fn add_error(&mut self, error_type: SystemError, timestamp_ms: u32) {
        let error_record = ErrorRecord {
            error_type,
            timestamp_ms,
        };

        // Add error, removing oldest if full
        if self.errors.is_full() {
            self.errors.remove(0);
        }
        let _ = self.errors.push(error_record);
    }

    pub fn get_recent_errors(&mut self) -> &Vec<ErrorRecord, 16> {
        self.query_count = self.query_count.saturating_add(1);
        &self.errors
    }

    pub fn get_query_count(&self) -> u32 {
        self.query_count
    }

    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }
}

/// Error record for history tracking
#[derive(Clone, Copy, Debug)]
pub struct ErrorRecord {
    pub error_type: SystemError,
    pub timestamp_ms: u32,
}