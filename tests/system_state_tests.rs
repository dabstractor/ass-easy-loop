//! No-std integration tests for system state query handlers
//!
//! This module tests the system state query functionality including data serialization,
//! performance monitoring, hardware status reporting, and configuration management.
//!
//! Requirements: 3.1, 3.2, 3.3, 3.4, 3.5

// Test file - uses std for host-side testing
use ass_easy_loop::battery::BatteryState;
use ass_easy_loop::config::LogConfig;
use ass_easy_loop::error_handling::SystemError;
use ass_easy_loop::system_state::*;
use ass_easy_loop::test_framework::{create_test_suite, TestResult};
use ass_easy_loop::{assert_eq_no_std, assert_no_std};

fn test_state_query_type_conversion() -> TestResult {
    // Test conversion from u8 to StateQueryType
    assert_eq!(
        StateQueryType::from_u8(0x01),
        Some(StateQueryType::SystemHealth)
    );
    assert_eq!(
        StateQueryType::from_u8(0x02),
        Some(StateQueryType::TaskPerformance)
    );
    assert_eq!(
        StateQueryType::from_u8(0x03),
        Some(StateQueryType::HardwareStatus)
    );
    assert_eq!(
        StateQueryType::from_u8(0x04),
        Some(StateQueryType::ConfigurationDump)
    );
    assert_eq!(
        StateQueryType::from_u8(0x05),
        Some(StateQueryType::ErrorHistory)
    );
    assert_eq!(StateQueryType::from_u8(0xFF), None);
    TestResult::pass()
}

fn test_system_health_data_creation() -> TestResult {
    let health_data = SystemHealthData {
        uptime_ms: 12345,
        battery_state: BatteryState::Normal,
        battery_voltage_mv: 3300,
        pemf_active: true,
        pemf_cycle_count: 100,
        task_health_status: TaskHealthStatus {
            pemf_task_healthy: true,
            battery_task_healthy: true,
            led_task_healthy: true,
            usb_poll_task_healthy: true,
            usb_hid_task_healthy: true,
            command_handler_healthy: true,
            last_health_check_ms: 12345,
        },
        memory_usage: MemoryUsageStats {
            stack_usage_bytes: 4096,
            heap_usage_bytes: 0,
            log_queue_usage_bytes: 2048,
            command_queue_usage_bytes: 512,
            total_ram_usage_bytes: 6656,
            peak_ram_usage_bytes: 7000,
            memory_fragmentation_percent: 0,
        },
        error_counts: ErrorCounters {
            adc_read_errors: 0,
            gpio_operation_errors: 1,
            usb_transmission_errors: 0,
            command_parsing_errors: 0,
            timing_violations: 0,
            bootloader_entry_failures: 0,
            total_error_count: 1,
        },
        system_temperature: Some(250), // 25.0°C
    };

    // Verify data structure integrity
    assert_eq!(health_data.uptime_ms, 12345);
    assert_eq!(health_data.battery_state, BatteryState::Normal);
    assert_eq!(health_data.battery_voltage_mv, 3300);
    assert!(health_data.pemf_active);
    assert_eq!(health_data.pemf_cycle_count, 100);
    assert!(health_data.task_health_status.pemf_task_healthy);
    assert_eq!(health_data.memory_usage.total_ram_usage_bytes, 6656);
    assert_eq!(health_data.error_counts.total_error_count, 1);
    assert_eq!(health_data.system_temperature, Some(250));
    TestResult::pass()
}

fn test_task_performance_data_creation() -> TestResult {
    let performance_data = TaskPerformanceData {
        task_execution_times: TaskExecutionTimes {
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
        },
        timing_statistics: TimingStatistics {
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
        },
        resource_usage: ResourceUsageData {
            cpu_usage_percent: 15,
            memory_usage_percent: 25,
            queue_utilization_percent: 30,
            usb_bandwidth_usage_percent: 10,
            interrupt_load_percent: 20,
        },
        performance_metrics: PerformanceMetrics {
            messages_per_second: 100,
            commands_processed_per_second: 5,
            average_response_time_ms: 10,
            throughput_bytes_per_second: 6400,
            system_efficiency_percent: 85,
        },
    };

    // Verify performance data integrity
    assert_eq!(performance_data.task_execution_times.pemf_task_avg_us, 50);
    assert_eq!(performance_data.timing_statistics.pemf_frequency_hz, 2.0);
    assert_eq!(performance_data.resource_usage.cpu_usage_percent, 15);
    assert_eq!(
        performance_data.performance_metrics.messages_per_second,
        100
    );
    TestResult::pass()
}

fn test_hardware_status_data_creation() -> TestResult {
    let hardware_data = HardwareStatusData {
        gpio_states: GpioStates {
            mosfet_pin_state: false,
            led_pin_state: true,
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
        usb_status: UsbStatus {
            connected: true,
            configured: true,
            suspended: false,
            enumerated: true,
            hid_reports_sent: 1000,
            hid_reports_received: 50,
            usb_errors: 0,
            last_activity_ms: 12345,
        },
        power_status: PowerStatus {
            supply_voltage_mv: 3300,
            core_voltage_mv: 1200,
            power_consumption_mw: 150,
            battery_charging: false,
            low_power_mode: false,
            power_good: true,
        },
        sensor_readings: SensorReadings {
            internal_temperature_c: 25,
            cpu_temperature_c: 30,
            ambient_light_level: 500,
            magnetic_field_strength: 100,
            vibration_level: 10,
        },
    };

    // Verify hardware data integrity
    assert!(!hardware_data.gpio_states.mosfet_pin_state);
    assert!(hardware_data.gpio_states.led_pin_state);
    assert_eq!(hardware_data.adc_readings.battery_adc_raw, 1500);
    assert!(hardware_data.usb_status.connected);
    assert_eq!(hardware_data.power_status.supply_voltage_mv, 3300);
    assert_eq!(hardware_data.sensor_readings.internal_temperature_c, 25);
    TestResult::pass()
}

fn test_configuration_data_creation() -> TestResult {
    let config_data = ConfigurationData {
        log_config: LogConfig::new(),
        timing_config: TimingConfiguration {
            pemf_frequency_hz: 2.0,
            pemf_high_duration_ms: 2,
            pemf_low_duration_ms: 498,
            battery_sampling_interval_ms: 100,
            led_flash_rate_hz: 2.0,
            usb_poll_interval_ms: 1,
            timing_tolerance_percent: 0.01,
        },
        hardware_config: HardwareConfiguration {
            mosfet_pin: 15,
            led_pin: 25,
            adc_pin: 26,
            adc_resolution_bits: 12,
            adc_reference_mv: 3300,
            system_clock_hz: 125_000_000,
            external_crystal_hz: 12_000_000,
        },
        feature_flags: FeatureFlags {
            usb_hid_logging_enabled: true,
            battery_monitoring_enabled: true,
            pemf_generation_enabled: true,
            led_control_enabled: true,
            performance_monitoring_enabled: true,
            debug_mode_enabled: false,
            panic_logging_enabled: true,
        },
        calibration_data: CalibrationData {
            adc_offset: 0,
            adc_gain: 1.0,
            temperature_offset: 0,
            voltage_divider_ratio: 0.337,
            timing_calibration_factor: 1.0,
            last_calibration_timestamp: 0,
        },
    };

    // Verify configuration data integrity
    assert_eq!(config_data.timing_config.pemf_frequency_hz, 2.0);
    assert_eq!(config_data.hardware_config.mosfet_pin, 15);
    assert!(config_data.feature_flags.usb_hid_logging_enabled);
    assert_eq!(config_data.calibration_data.voltage_divider_ratio, 0.337);
    TestResult::pass()
}

fn test_system_state_handler_creation() -> TestResult {
    let handler = SystemStateHandler::new();
    let stats = handler.get_stats();

    // Verify initial state
    assert_eq!(stats.queries_processed, 0);
    assert_eq!(stats.last_query_timestamp, 0);
    assert_eq!(stats.performance_queries, 0);
    assert_eq!(stats.hardware_queries, 0);
    assert_eq!(stats.config_queries, 0);
    assert_eq!(stats.error_queries, 0);
    TestResult::pass()
}

fn test_system_health_data_serialization() -> TestResult {
    let mut handler = SystemStateHandler::new();

    // Test system health query
    let result = handler.process_state_query(StateQueryType::SystemHealth, 12345);
    assert!(result.is_ok());

    let serialized_data = result.unwrap();
    assert!(!serialized_data.is_empty());
    assert!(serialized_data.len() <= 60); // Must fit in command payload

    // Verify handler statistics updated
    let stats = handler.get_stats();
    assert_eq!(stats.queries_processed, 1);
    assert_eq!(stats.last_query_timestamp, 12345);
    TestResult::pass()
}

fn test_task_performance_data_serialization() -> TestResult {
    let mut handler = SystemStateHandler::new();

    // Test task performance query
    let result = handler.process_state_query(StateQueryType::TaskPerformance, 12345);
    assert!(result.is_ok());

    let serialized_data = result.unwrap();
    assert!(!serialized_data.is_empty());
    assert!(serialized_data.len() <= 60); // Must fit in command payload

    // Verify the serialized data contains expected performance information
    // First 20 bytes should be task execution times (5 tasks * 4 bytes each)
    assert!(serialized_data.len() >= 20);
    TestResult::pass()
}

fn test_hardware_status_data_serialization() -> TestResult {
    let mut handler = SystemStateHandler::new();

    // Test hardware status query
    let result = handler.process_state_query(StateQueryType::HardwareStatus, 12345);
    assert!(result.is_ok());

    let serialized_data = result.unwrap();
    assert!(!serialized_data.is_empty());
    assert!(serialized_data.len() <= 60); // Must fit in command payload

    // Verify the serialized data contains GPIO, ADC, and USB status information
    assert!(serialized_data.len() >= 30); // Minimum expected size for hardware data
    TestResult::pass()
}

fn test_configuration_data_serialization() -> TestResult {
    let mut handler = SystemStateHandler::new();

    // Test configuration dump query
    let result = handler.process_state_query(StateQueryType::ConfigurationDump, 12345);
    assert!(result.is_ok());

    let serialized_data = result.unwrap();
    assert!(!serialized_data.is_empty());
    assert!(serialized_data.len() <= 60); // Must fit in command payload

    // Verify the serialized data contains configuration information
    // Should include log config (16 bytes) + timing config + hardware config
    assert!(serialized_data.len() >= 16);
    TestResult::pass()
}

fn test_error_history_data_serialization() -> TestResult {
    let mut handler = SystemStateHandler::new();

    // Test error history query
    let result = handler.process_state_query(StateQueryType::ErrorHistory, 12345);
    assert!(result.is_ok());

    let serialized_data = result.unwrap();
    assert!(serialized_data.len() <= 60); // Must fit in command payload

    // Error history might be empty initially, so just verify it doesn't fail
    // First byte should be error count
    if !serialized_data.is_empty() {
        let error_count = serialized_data[0];
        assert!(error_count <= 15); // Maximum errors that fit in payload
    }
    TestResult::pass()
}

fn test_state_response_creation() -> TestResult {
    let handler = SystemStateHandler::new();
    let test_data = [0x01, 0x02, 0x03, 0x04];

    // Test creating a state response
    let response = handler.create_state_response(
        0x42, // command_id
        StateQueryType::SystemHealth,
        &test_data,
    );

    assert!(response.is_ok());
    let command_report = response.unwrap();

    // Verify response structure
    assert_eq!(command_report.command_id, 0x42);
    assert_eq!(command_report.payload.len(), 6); // 1 byte type + 1 byte length + 4 bytes data
    assert_eq!(
        command_report.payload[0],
        StateQueryType::SystemHealth as u8
    );
    assert_eq!(command_report.payload[1], 4); // data length
    assert_eq!(command_report.payload[2], 0x01);
    assert_eq!(command_report.payload[3], 0x02);
    assert_eq!(command_report.payload[4], 0x03);
    assert_eq!(command_report.payload[5], 0x04);
    TestResult::pass()
}

fn test_performance_monitor() -> TestResult {
    let mut monitor = PerformanceMonitor::new();

    // Test initial state
    assert_eq!(monitor.get_query_count(), 0);

    // Test task execution times collection
    let execution_times = monitor.get_task_execution_times();
    assert!(execution_times.pemf_task_avg_us > 0);
    assert!(execution_times.pemf_task_max_us >= execution_times.pemf_task_avg_us);
    assert_eq!(monitor.get_query_count(), 1);

    // Test timing statistics collection
    let timing_stats = monitor.get_timing_statistics();
    assert_eq!(timing_stats.pemf_frequency_hz, 2.0);
    assert!(timing_stats.pemf_timing_accuracy_percent > 90.0);

    // Test resource usage collection
    let resource_usage = monitor.get_resource_usage();
    assert!(resource_usage.cpu_usage_percent <= 100);
    assert!(resource_usage.memory_usage_percent <= 100);

    // Test performance metrics collection
    let performance_metrics = monitor.get_performance_metrics();
    assert!(performance_metrics.messages_per_second > 0);
    assert!(performance_metrics.system_efficiency_percent <= 100);
    TestResult::pass()
}

fn test_diagnostic_collector() -> TestResult {
    let mut collector = DiagnosticCollector::new();

    // Test initial state
    assert_eq!(collector.get_query_count(), 0);

    // Test GPIO states collection
    let gpio_states = collector.get_gpio_states();
    assert_eq!(gpio_states.gpio_error_count, 0);
    assert_eq!(collector.get_query_count(), 1);

    // Test ADC readings collection
    let adc_readings = collector.get_adc_readings();
    assert!(adc_readings.battery_adc_raw > 0);
    assert!(adc_readings.battery_voltage_mv > 0);
    assert_eq!(adc_readings.adc_error_count, 0);

    // Test USB status collection
    let usb_status = collector.get_usb_status();
    assert!(usb_status.connected);
    assert!(usb_status.configured);
    assert_eq!(usb_status.usb_errors, 0);

    // Test power status collection
    let power_status = collector.get_power_status();
    assert!(power_status.supply_voltage_mv > 0);
    assert!(power_status.power_good);

    // Test sensor readings collection
    let sensor_readings = collector.get_sensor_readings();
    assert!(sensor_readings.internal_temperature_c > -40);
    assert!(sensor_readings.internal_temperature_c < 85);
    TestResult::pass()
}

fn test_configuration_manager() -> TestResult {
    let mut manager = ConfigurationManager::new();

    // Test initial state
    assert_eq!(manager.get_query_count(), 0);

    // Test log configuration retrieval
    let _log_config = manager.get_log_config();
    assert_eq!(manager.get_query_count(), 1);

    // Test timing configuration retrieval
    let timing_config = manager.get_timing_config();
    assert_eq!(timing_config.pemf_frequency_hz, 2.0);
    assert_eq!(timing_config.pemf_high_duration_ms, 2);
    assert_eq!(timing_config.pemf_low_duration_ms, 498);

    // Test hardware configuration retrieval
    let hardware_config = manager.get_hardware_config();
    assert_eq!(hardware_config.mosfet_pin, 15);
    assert_eq!(hardware_config.led_pin, 25);
    assert_eq!(hardware_config.adc_pin, 26);

    // Test feature flags retrieval
    let feature_flags = manager.get_feature_flags();
    assert!(feature_flags.usb_hid_logging_enabled);
    assert!(feature_flags.battery_monitoring_enabled);
    assert!(feature_flags.pemf_generation_enabled);

    // Test calibration data retrieval
    let calibration_data = manager.get_calibration_data();
    assert_eq!(calibration_data.adc_offset, 0);
    assert_eq!(calibration_data.adc_gain, 1.0);
    assert_eq!(calibration_data.voltage_divider_ratio, 0.337);
    TestResult::pass()
}

fn test_error_history() -> TestResult {
    let mut error_history = ErrorHistory::new();

    // Test initial state
    assert_eq!(error_history.get_query_count(), 0);
    let errors = error_history.get_recent_errors();
    assert!(errors.is_empty());
    assert_eq!(error_history.get_query_count(), 1);

    // Test adding errors
    error_history.add_error(SystemError::AdcReadFailed, 1000);
    error_history.add_error(SystemError::GpioOperationFailed, 2000);

    let errors = error_history.get_recent_errors();
    assert_eq!(errors.len(), 2);
    assert_eq!(errors[0].timestamp_ms, 1000);
    assert_eq!(errors[1].timestamp_ms, 2000);

    // Test error history overflow (should remove oldest)
    for i in 0..20 {
        error_history.add_error(SystemError::SystemBusy, 3000 + i);
    }

    let errors = error_history.get_recent_errors();
    assert_eq!(errors.len(), 16); // Maximum capacity

    // Test clearing errors
    error_history.clear_errors();
    let errors = error_history.get_recent_errors();
    assert!(errors.is_empty());
    TestResult::pass()
}

fn test_serialization_bounds() -> TestResult {
    let mut handler = SystemStateHandler::new();

    // Test all query types to ensure serialization stays within bounds
    let query_types = [
        StateQueryType::SystemHealth,
        StateQueryType::TaskPerformance,
        StateQueryType::HardwareStatus,
        StateQueryType::ConfigurationDump,
        StateQueryType::ErrorHistory,
    ];

    for query_type in &query_types {
        let result = handler.process_state_query(*query_type, 12345);
        assert!(result.is_ok(), "Query type failed");

        let serialized_data = result.unwrap();
        assert!(
            serialized_data.len() <= 60,
            "Query type serialized data too large"
        );
    }
    TestResult::pass()
}

fn test_handler_statistics() -> TestResult {
    let mut handler = SystemStateHandler::new();

    // Process multiple queries
    let _ = handler.process_state_query(StateQueryType::SystemHealth, 1000);
    let _ = handler.process_state_query(StateQueryType::TaskPerformance, 2000);
    let _ = handler.process_state_query(StateQueryType::HardwareStatus, 3000);

    let stats = handler.get_stats();
    assert_eq!(stats.queries_processed, 3);
    assert_eq!(stats.last_query_timestamp, 3000);
    assert!(stats.performance_queries > 0);
    assert!(stats.hardware_queries > 0);
    TestResult::pass()
}

fn test_data_structure_sizes() -> TestResult {
    use core::mem::size_of;

    // Verify data structures are reasonably sized for embedded use
    assert!(size_of::<SystemHealthData>() < 1024);
    assert!(size_of::<TaskPerformanceData>() < 1024);
    assert!(size_of::<HardwareStatusData>() < 1024);
    assert!(size_of::<ConfigurationData>() < 1024);
    assert!(size_of::<SystemStateHandler>() < 2048);

    // Verify critical structures fit in expected memory constraints
    assert!(size_of::<TaskExecutionTimes>() <= 40); // 10 u32 values
    assert!(size_of::<GpioStates>() <= 20);
    assert!(size_of::<AdcReadings>() <= 24);
    assert!(size_of::<UsbStatus>() <= 32);
    TestResult::pass()
}

// Test registration array for no_std environment
const SYSTEM_STATE_TESTS: &[(&str, fn() -> TestResult)] = &[
    (
        "test_state_query_type_conversion",
        test_state_query_type_conversion,
    ),
    (
        "test_system_health_data_creation",
        test_system_health_data_creation,
    ),
    (
        "test_task_performance_data_creation",
        test_task_performance_data_creation,
    ),
    (
        "test_hardware_status_data_creation",
        test_hardware_status_data_creation,
    ),
    (
        "test_configuration_data_creation",
        test_configuration_data_creation,
    ),
    (
        "test_system_state_handler_creation",
        test_system_state_handler_creation,
    ),
    (
        "test_system_health_data_serialization",
        test_system_health_data_serialization,
    ),
    (
        "test_task_performance_data_serialization",
        test_task_performance_data_serialization,
    ),
    (
        "test_hardware_status_data_serialization",
        test_hardware_status_data_serialization,
    ),
    (
        "test_configuration_data_serialization",
        test_configuration_data_serialization,
    ),
    (
        "test_error_history_data_serialization",
        test_error_history_data_serialization,
    ),
    ("test_state_response_creation", test_state_response_creation),
    ("test_performance_monitor", test_performance_monitor),
    ("test_diagnostic_collector", test_diagnostic_collector),
    ("test_configuration_manager", test_configuration_manager),
    ("test_error_history", test_error_history),
    ("test_serialization_bounds", test_serialization_bounds),
    ("test_handler_statistics", test_handler_statistics),
    ("test_data_structure_sizes", test_data_structure_sizes),
];

/// Run all system state integration tests
pub fn run_system_state_tests() -> TestResult {
    let runner = create_test_suite("System State Integration Tests", SYSTEM_STATE_TESTS);
    let results = runner.run_all();

    if results.stats.has_failures() {
        TestResult::fail("Some system state tests failed")
    } else {
        TestResult::pass()
    }
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() -> ! {
    let _result = run_system_state_tests();

    // In a real embedded environment, this would communicate results via USB HID
    // For now, we'll just loop indefinitely
    loop {
        // Wait for watchdog or external reset
    }
}
