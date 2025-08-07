//! Performance Monitoring Integration Tests
//! 
//! This module contains comprehensive tests for the USB HID logging performance monitoring system.
//! Tests validate CPU usage monitoring, memory usage tracking, message performance metrics,
//! and timing impact measurements.
//! 
//! Requirements: 7.1, 7.2, 7.5

#![no_std]
#![no_main]

use panic_halt as _;
use ass_easy_loop::logging::{
    PerformanceStats, CpuUsageStats, MemoryUsageStats, MessagePerformanceStats, 
    TimingImpactStats, PerformanceMonitor, LogMessage, LogLevel, LogQueue
};
use ass_easy_loop::config;

/// Test CPU usage monitoring functionality
/// Requirements: 7.1
#[test]
fn test_cpu_usage_monitoring() {
    let mut perf_stats = PerformanceStats::new();
    
    // Test initial state
    assert_eq!(perf_stats.usb_cpu_usage.usb_poll_cpu_percent, 0);
    assert_eq!(perf_stats.usb_cpu_usage.usb_hid_cpu_percent, 0);
    assert_eq!(perf_stats.usb_cpu_usage.total_usb_cpu_percent, 0);
    assert_eq!(perf_stats.usb_cpu_usage.peak_usb_cpu_percent, 0);
    
    // Test CPU usage updates
    perf_stats.update_cpu_usage(5, 3); // 5% poll, 3% HID
    assert_eq!(perf_stats.usb_cpu_usage.usb_poll_cpu_percent, 5);
    assert_eq!(perf_stats.usb_cpu_usage.usb_hid_cpu_percent, 3);
    assert_eq!(perf_stats.usb_cpu_usage.total_usb_cpu_percent, 8);
    assert_eq!(perf_stats.usb_cpu_usage.peak_usb_cpu_percent, 8);
    assert_eq!(perf_stats.usb_cpu_usage.measurement_count, 1);
    assert_eq!(perf_stats.usb_cpu_usage.average_cpu_percent, 8);
    
    // Test peak tracking
    perf_stats.update_cpu_usage(2, 4); // 2% poll, 4% HID = 6% total
    assert_eq!(perf_stats.usb_cpu_usage.total_usb_cpu_percent, 6);
    assert_eq!(perf_stats.usb_cpu_usage.peak_usb_cpu_percent, 8); // Should remain 8
    assert_eq!(perf_stats.usb_cpu_usage.measurement_count, 2);
    assert_eq!(perf_stats.usb_cpu_usage.average_cpu_percent, 7); // (8 + 6) / 2 = 7
    
    // Test higher peak
    perf_stats.update_cpu_usage(8, 4); // 8% poll, 4% HID = 12% total
    assert_eq!(perf_stats.usb_cpu_usage.total_usb_cpu_percent, 12);
    assert_eq!(perf_stats.usb_cpu_usage.peak_usb_cpu_percent, 12); // Should update to 12
    
    // Test saturation (values should not exceed 100%)
    perf_stats.update_cpu_usage(200, 200); // Extreme values
    assert_eq!(perf_stats.usb_cpu_usage.total_usb_cpu_percent, 255); // Saturated addition
    assert_eq!(perf_stats.usb_cpu_usage.peak_usb_cpu_percent, 255);
}

/// Test CPU usage calculation utility
/// Requirements: 7.1
#[test]
fn test_cpu_usage_calculation() {
    // Test normal cases
    assert_eq!(PerformanceMonitor::calculate_cpu_usage(1000, 10000), 10); // 10%
    assert_eq!(PerformanceMonitor::calculate_cpu_usage(5000, 10000), 50); // 50%
    assert_eq!(PerformanceMonitor::calculate_cpu_usage(10000, 10000), 100); // 100%
    
    // Test edge cases
    assert_eq!(PerformanceMonitor::calculate_cpu_usage(0, 10000), 0); // 0%
    assert_eq!(PerformanceMonitor::calculate_cpu_usage(15000, 10000), 100); // Clamped to 100%
    assert_eq!(PerformanceMonitor::calculate_cpu_usage(1000, 0), 0); // Division by zero protection
    
    // Test realistic USB task scenarios
    const USB_POLL_INTERVAL_US: u32 = 10_000; // 10ms
    const USB_HID_INTERVAL_US: u32 = 20_000; // 20ms
    
    // Typical execution times
    assert_eq!(PerformanceMonitor::calculate_cpu_usage(100, USB_POLL_INTERVAL_US), 1); // 1% for 100us execution
    assert_eq!(PerformanceMonitor::calculate_cpu_usage(500, USB_HID_INTERVAL_US), 2); // 2.5% rounded down for 500us execution
    
    // Performance threshold testing
    let max_allowed_cpu = config::system::MAX_USB_CPU_USAGE_PERCENT;
    let max_execution_time_us = (USB_POLL_INTERVAL_US * max_allowed_cpu as u32) / 100;
    assert!(PerformanceMonitor::calculate_cpu_usage(max_execution_time_us, USB_POLL_INTERVAL_US) <= max_allowed_cpu);
}

/// Test memory usage tracking functionality
/// Requirements: 7.2
#[test]
fn test_memory_usage_tracking() {
    let mut perf_stats = PerformanceStats::new();
    
    // Test initial state
    assert_eq!(perf_stats.memory_usage.queue_memory_bytes, 0);
    assert_eq!(perf_stats.memory_usage.usb_buffer_memory_bytes, 0);
    assert_eq!(perf_stats.memory_usage.total_memory_bytes, 0);
    assert_eq!(perf_stats.memory_usage.peak_queue_memory_bytes, 0);
    assert_eq!(perf_stats.memory_usage.memory_utilization_percent, 0);
    
    // Test memory usage updates
    perf_stats.update_memory_usage(2048, 1024); // 2KB queue, 1KB USB buffers
    assert_eq!(perf_stats.memory_usage.queue_memory_bytes, 2048);
    assert_eq!(perf_stats.memory_usage.usb_buffer_memory_bytes, 1024);
    assert_eq!(perf_stats.memory_usage.total_memory_bytes, 3072);
    assert_eq!(perf_stats.memory_usage.peak_queue_memory_bytes, 2048);
    
    // Calculate expected utilization percentage (assuming 264KB total RAM)
    const TOTAL_RAM_BYTES: usize = 264 * 1024;
    let expected_utilization = (3072 * 100) / TOTAL_RAM_BYTES;
    assert_eq!(perf_stats.memory_usage.memory_utilization_percent, expected_utilization as u8);
    
    // Test peak tracking
    perf_stats.update_memory_usage(1536, 1024); // Reduced queue memory
    assert_eq!(perf_stats.memory_usage.queue_memory_bytes, 1536);
    assert_eq!(perf_stats.memory_usage.peak_queue_memory_bytes, 2048); // Should remain 2048
    
    // Test higher peak
    perf_stats.update_memory_usage(4096, 1024); // Increased queue memory
    assert_eq!(perf_stats.memory_usage.queue_memory_bytes, 4096);
    assert_eq!(perf_stats.memory_usage.peak_queue_memory_bytes, 4096); // Should update to 4096
}

/// Test queue memory usage calculation
/// Requirements: 7.2
#[test]
fn test_queue_memory_calculation() {
    // Test empty queue
    assert_eq!(PerformanceMonitor::calculate_queue_memory_usage::<32>(0), 
               core::mem::size_of::<LogQueue<32>>() - (32 * core::mem::size_of::<LogMessage>()));
    
    // Test partially filled queue
    let half_full = PerformanceMonitor::calculate_queue_memory_usage::<32>(16);
    let expected_half = core::mem::size_of::<LogQueue<32>>() - (16 * core::mem::size_of::<LogMessage>());
    assert_eq!(half_full, expected_half);
    
    // Test full queue
    let full = PerformanceMonitor::calculate_queue_memory_usage::<32>(32);
    let expected_full = core::mem::size_of::<LogQueue<32>>();
    assert_eq!(full, expected_full);
    
    // Test different queue sizes
    assert!(PerformanceMonitor::calculate_queue_memory_usage::<64>(32) > 
            PerformanceMonitor::calculate_queue_memory_usage::<32>(32));
}

/// Test message performance tracking
/// Requirements: 7.1, 7.5
#[test]
fn test_message_performance_tracking() {
    let mut perf_stats = PerformanceStats::new();
    
    // Test initial state
    assert_eq!(perf_stats.message_performance.avg_format_time_us, 0);
    assert_eq!(perf_stats.message_performance.avg_enqueue_time_us, 0);
    assert_eq!(perf_stats.message_performance.avg_transmission_time_us, 0);
    assert_eq!(perf_stats.message_performance.peak_processing_time_us, 0);
    assert_eq!(perf_stats.message_performance.messages_processed, 0);
    assert_eq!(perf_stats.message_performance.transmission_failures, 0);
    
    // Test first message performance update
    perf_stats.update_message_performance(100, 50, 200); // 100us format, 50us enqueue, 200us transmission
    assert_eq!(perf_stats.message_performance.avg_format_time_us, 100);
    assert_eq!(perf_stats.message_performance.avg_enqueue_time_us, 50);
    assert_eq!(perf_stats.message_performance.avg_transmission_time_us, 200);
    assert_eq!(perf_stats.message_performance.peak_processing_time_us, 350); // 100 + 50 + 200
    assert_eq!(perf_stats.message_performance.messages_processed, 1);
    
    // Test second message (running average)
    perf_stats.update_message_performance(200, 100, 300); // Higher times
    assert_eq!(perf_stats.message_performance.avg_format_time_us, 150); // (100 + 200) / 2
    assert_eq!(perf_stats.message_performance.avg_enqueue_time_us, 75); // (50 + 100) / 2
    assert_eq!(perf_stats.message_performance.avg_transmission_time_us, 250); // (200 + 300) / 2
    assert_eq!(perf_stats.message_performance.peak_processing_time_us, 600); // 200 + 100 + 300
    assert_eq!(perf_stats.message_performance.messages_processed, 2);
    
    // Test transmission failure recording
    perf_stats.record_transmission_failure();
    assert_eq!(perf_stats.message_performance.transmission_failures, 1);
    
    perf_stats.record_transmission_failure();
    assert_eq!(perf_stats.message_performance.transmission_failures, 2);
}

/// Test timing impact measurements
/// Requirements: 7.1, 7.5
#[test]
fn test_timing_impact_measurements() {
    let mut perf_stats = PerformanceStats::new();
    
    // Test initial state
    assert_eq!(perf_stats.timing_impact.pemf_timing_deviation_us, 0);
    assert_eq!(perf_stats.timing_impact.battery_timing_deviation_us, 0);
    assert_eq!(perf_stats.timing_impact.max_timing_deviation_us, 0);
    assert_eq!(perf_stats.timing_impact.timing_accuracy_percent, 100);
    assert_eq!(perf_stats.timing_impact.timing_violations, 0);
    
    // Test timing impact within tolerance
    perf_stats.update_timing_impact(5000, 3000); // 5ms pEMF, 3ms battery deviation
    assert_eq!(perf_stats.timing_impact.pemf_timing_deviation_us, 5000);
    assert_eq!(perf_stats.timing_impact.battery_timing_deviation_us, 3000);
    assert_eq!(perf_stats.timing_impact.max_timing_deviation_us, 5000);
    assert_eq!(perf_stats.timing_impact.timing_accuracy_percent, 100); // Within tolerance
    assert_eq!(perf_stats.timing_impact.timing_violations, 0);
    
    // Test timing impact exceeding tolerance
    perf_stats.update_timing_impact(15000, 8000); // 15ms pEMF, 8ms battery deviation (exceeds 10ms tolerance)
    assert_eq!(perf_stats.timing_impact.pemf_timing_deviation_us, 15000);
    assert_eq!(perf_stats.timing_impact.battery_timing_deviation_us, 8000);
    assert_eq!(perf_stats.timing_impact.max_timing_deviation_us, 15000);
    assert_eq!(perf_stats.timing_impact.timing_violations, 1);
    
    // Test maximum deviation tracking
    perf_stats.update_timing_impact(12000, 20000); // Battery deviation higher than pEMF
    assert_eq!(perf_stats.timing_impact.max_timing_deviation_us, 20000); // Should update to 20000
    assert_eq!(perf_stats.timing_impact.timing_violations, 2);
}

/// Test performance summary generation
/// Requirements: 7.1, 7.2, 7.5
#[test]
fn test_performance_summary() {
    let mut perf_stats = PerformanceStats::new();
    
    // Test initial state (should be OK)
    let summary = perf_stats.get_performance_summary();
    assert!(summary.cpu_usage_ok);
    assert!(summary.memory_usage_ok);
    assert!(summary.timing_impact_ok);
    assert!(summary.overall_performance_ok);
    
    // Test CPU usage exceeding threshold
    perf_stats.update_cpu_usage(10, 0); // 10% > MAX_USB_CPU_USAGE_PERCENT (5%)
    let summary = perf_stats.get_performance_summary();
    assert!(!summary.cpu_usage_ok);
    assert!(!summary.overall_performance_ok);
    
    // Reset CPU usage
    perf_stats.update_cpu_usage(2, 2); // 4% total, within limits
    
    // Test memory usage exceeding threshold
    const HIGH_MEMORY_BYTES: usize = 30 * 1024; // 30KB should exceed 10% threshold
    perf_stats.update_memory_usage(HIGH_MEMORY_BYTES, 1024);
    let summary = perf_stats.get_performance_summary();
    assert!(!summary.memory_usage_ok);
    assert!(!summary.overall_performance_ok);
    
    // Reset memory usage
    perf_stats.update_memory_usage(2048, 1024); // Reasonable memory usage
    
    // Test timing accuracy below threshold
    perf_stats.timing_impact.timing_accuracy_percent = 90; // Below 95% threshold
    let summary = perf_stats.get_performance_summary();
    assert!(!summary.timing_impact_ok);
    assert!(!summary.overall_performance_ok);
}

/// Test optimized message formatting performance
/// Requirements: 7.5
#[test]
fn test_optimized_message_formatting() {
    let test_message = LogMessage::new(
        12345,
        LogLevel::Info,
        "TEST",
        "Optimized formatting test message"
    );
    
    // Test optimized formatting
    let optimized_result = PerformanceMonitor::format_message_optimized(&test_message);
    
    // Verify format structure
    assert_eq!(optimized_result[0], LogLevel::Info as u8); // Level
    assert_eq!(&optimized_result[1..5], b"TEST"); // Module (first 4 chars)
    assert_eq!(&optimized_result[57..61], &12345u32.to_le_bytes()); // Timestamp
    
    // Test that optimized formatting produces same result as standard serialization
    let standard_result = test_message.serialize();
    assert_eq!(optimized_result, standard_result);
}

/// Test batch message processing
/// Requirements: 7.5
#[test]
fn test_batch_message_processing() {
    let test_messages = [
        LogMessage::new(1000, LogLevel::Debug, "MOD1", "Message 1"),
        LogMessage::new(2000, LogLevel::Info, "MOD2", "Message 2"),
        LogMessage::new(3000, LogLevel::Warn, "MOD3", "Message 3"),
    ];
    
    // Test batch formatting
    let batch_result = PerformanceMonitor::batch_format_messages(&test_messages);
    
    // Verify batch size
    assert_eq!(batch_result.len(), 3);
    
    // Verify each message in batch
    for (i, formatted) in batch_result.iter().enumerate() {
        let expected = test_messages[i].serialize();
        assert_eq!(*formatted, expected);
    }
    
    // Test batch size limiting (create more than 32 messages)
    let large_batch: [LogMessage; 40] = [LogMessage::new(0, LogLevel::Debug, "TEST", "msg"); 40];
    let limited_result = PerformanceMonitor::batch_format_messages(&large_batch);
    
    // Should be limited to 32 messages
    assert_eq!(limited_result.len(), 32);
}

/// Test performance monitoring integration with queue statistics
/// Requirements: 7.1, 7.2, 7.5
#[test]
fn test_queue_performance_integration() {
    let mut queue: LogQueue<8> = LogQueue::new();
    
    // Test initial queue statistics
    let stats = queue.stats();
    assert_eq!(stats.messages_sent, 0);
    assert_eq!(stats.messages_dropped, 0);
    assert_eq!(stats.peak_utilization, 0);
    assert_eq!(stats.current_utilization_percent, 0);
    
    // Add some messages
    for i in 0..5 {
        let message = LogMessage::new(i * 1000, LogLevel::Info, "TEST", "Queue test message");
        assert!(queue.enqueue(message));
    }
    
    // Check statistics after adding messages
    let stats = queue.stats();
    assert_eq!(stats.messages_sent, 5);
    assert_eq!(stats.messages_dropped, 0);
    assert_eq!(stats.peak_utilization, 5);
    assert_eq!(stats.current_utilization_percent, 62); // 5/8 * 100 = 62.5% rounded down
    
    // Fill queue to capacity
    for i in 5..8 {
        let message = LogMessage::new(i * 1000, LogLevel::Info, "TEST", "Queue test message");
        assert!(queue.enqueue(message));
    }
    
    // Check full queue statistics
    let stats = queue.stats();
    assert_eq!(stats.messages_sent, 8);
    assert_eq!(stats.messages_dropped, 0);
    assert_eq!(stats.peak_utilization, 8);
    assert_eq!(stats.current_utilization_percent, 100);
    
    // Test overflow behavior (should drop oldest messages)
    let overflow_message = LogMessage::new(9000, LogLevel::Error, "TEST", "Overflow message");
    assert!(queue.enqueue(overflow_message)); // Should succeed by dropping oldest
    
    let stats = queue.stats();
    assert_eq!(stats.messages_sent, 9);
    assert_eq!(stats.messages_dropped, 1); // One message should be dropped
    assert_eq!(stats.current_utilization_percent, 100); // Still full
    
    // Test memory usage calculation
    let memory_usage = PerformanceMonitor::calculate_queue_memory_usage::<8>(queue.len());
    assert!(memory_usage > 0);
    assert!(memory_usage <= core::mem::size_of::<LogQueue<8>>());
}

/// Test performance thresholds and alerts
/// Requirements: 7.1, 7.2
#[test]
fn test_performance_thresholds() {
    // Test CPU usage threshold
    let max_cpu_percent = config::system::MAX_USB_CPU_USAGE_PERCENT;
    assert!(max_cpu_percent > 0);
    assert!(max_cpu_percent <= 50); // Should be reasonable limit
    
    // Test execution time that would exceed CPU threshold
    const TEST_INTERVAL_US: u32 = 10_000; // 10ms
    let max_execution_time_us = (TEST_INTERVAL_US * max_cpu_percent as u32) / 100;
    
    assert_eq!(
        PerformanceMonitor::calculate_cpu_usage(max_execution_time_us, TEST_INTERVAL_US),
        max_cpu_percent
    );
    
    // Test that exceeding the threshold is detected
    let excessive_execution_time = max_execution_time_us + 1000; // Add 1ms
    assert!(
        PerformanceMonitor::calculate_cpu_usage(excessive_execution_time, TEST_INTERVAL_US) > max_cpu_percent
    );
    
    // Test memory utilization thresholds
    const MEMORY_WARNING_THRESHOLD: u8 = 10; // 10% of total RAM
    const TOTAL_RAM_BYTES: usize = 264 * 1024; // 264KB
    let warning_memory_bytes = (TOTAL_RAM_BYTES * MEMORY_WARNING_THRESHOLD as usize) / 100;
    
    let mut perf_stats = PerformanceStats::new();
    perf_stats.update_memory_usage(warning_memory_bytes, 0);
    
    // Should be at the threshold
    assert_eq!(perf_stats.memory_usage.memory_utilization_percent, MEMORY_WARNING_THRESHOLD);
    
    // Exceeding threshold should trigger warning
    perf_stats.update_memory_usage(warning_memory_bytes + 1024, 0);
    assert!(perf_stats.memory_usage.memory_utilization_percent > MEMORY_WARNING_THRESHOLD);
}

/// Test timing tolerance validation
/// Requirements: 7.1, 7.5
#[test]
fn test_timing_tolerance_validation() {
    const TIMING_TOLERANCE_US: u32 = 10_000; // 10ms tolerance
    
    let mut perf_stats = PerformanceStats::new();
    
    // Test timing within tolerance
    perf_stats.update_timing_impact(5000, 3000); // 5ms and 3ms deviations
    assert_eq!(perf_stats.timing_impact.timing_violations, 0);
    
    // Test timing at tolerance boundary
    perf_stats.update_timing_impact(TIMING_TOLERANCE_US, TIMING_TOLERANCE_US);
    assert_eq!(perf_stats.timing_impact.timing_violations, 0); // Should still be OK
    
    // Test timing exceeding tolerance
    perf_stats.update_timing_impact(TIMING_TOLERANCE_US + 1000, 5000); // Exceed by 1ms
    assert_eq!(perf_stats.timing_impact.timing_violations, 1);
    
    // Test large timing violation
    perf_stats.update_timing_impact(50000, 30000); // 50ms and 30ms deviations
    assert_eq!(perf_stats.timing_impact.timing_violations, 2);
    assert_eq!(perf_stats.timing_impact.max_timing_deviation_us, 50000);
}

/// Benchmark test comparing system performance with and without USB logging
/// This test simulates the performance impact measurement
/// Requirements: 7.1, 7.2, 7.5
#[test]
fn test_performance_benchmark_simulation() {
    // Simulate baseline performance (without USB logging)
    const BASELINE_PEMF_TIME_US: u32 = 2000; // 2ms for pEMF pulse
    const BASELINE_BATTERY_TIME_US: u32 = 100; // 100us for battery ADC read
    
    // Simulate current performance (with USB logging active)
    const CURRENT_PEMF_TIME_US: u32 = 2050; // 2.05ms (2.5% increase)
    const CURRENT_BATTERY_TIME_US: u32 = 105; // 105us (5% increase)
    
    // Calculate performance impact
    let pemf_impact_percent = if BASELINE_PEMF_TIME_US > 0 {
        ((CURRENT_PEMF_TIME_US - BASELINE_PEMF_TIME_US) * 100) / BASELINE_PEMF_TIME_US
    } else {
        0
    };
    
    let battery_impact_percent = if BASELINE_BATTERY_TIME_US > 0 {
        ((CURRENT_BATTERY_TIME_US - BASELINE_BATTERY_TIME_US) * 100) / BASELINE_BATTERY_TIME_US
    } else {
        0
    };
    
    // Verify impact calculations
    assert_eq!(pemf_impact_percent, 2); // 2.5% rounded down to 2%
    assert_eq!(battery_impact_percent, 5); // 5% exact
    
    // Test that impact is within acceptable limits (±1% tolerance)
    const ACCEPTABLE_IMPACT_PERCENT: u32 = 1;
    
    // pEMF impact exceeds tolerance (should trigger warning)
    assert!(pemf_impact_percent > ACCEPTABLE_IMPACT_PERCENT);
    
    // Battery impact exceeds tolerance (should trigger warning)
    assert!(battery_impact_percent > ACCEPTABLE_IMPACT_PERCENT);
    
    // Record timing impact in performance stats
    let mut perf_stats = PerformanceStats::new();
    let pemf_deviation_us = CURRENT_PEMF_TIME_US - BASELINE_PEMF_TIME_US;
    let battery_deviation_us = CURRENT_BATTERY_TIME_US - BASELINE_BATTERY_TIME_US;
    
    perf_stats.update_timing_impact(pemf_deviation_us, battery_deviation_us);
    
    // Verify timing impact recording
    assert_eq!(perf_stats.timing_impact.pemf_timing_deviation_us, pemf_deviation_us);
    assert_eq!(perf_stats.timing_impact.battery_timing_deviation_us, battery_deviation_us);
    assert_eq!(perf_stats.timing_impact.max_timing_deviation_us, pemf_deviation_us); // pEMF deviation is larger
}