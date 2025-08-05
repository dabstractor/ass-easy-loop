# USB HID Logging Usage Examples

This document provides comprehensive examples of how to monitor different types of log messages from the RP2040 pEMF/Battery monitoring device using the USB HID logging system.

## Table of Contents

1. [Basic Usage Examples](#basic-usage-examples)
2. [Battery Monitoring Examples](#battery-monitoring-examples)
3. [pEMF Timing Monitoring Examples](#pemf-timing-monitoring-examples)
4. [System Diagnostics Examples](#system-diagnostics-examples)
5. [Advanced Filtering and Analysis](#advanced-filtering-and-analysis)
6. [Automated Monitoring Scripts](#automated-monitoring-scripts)
7. [Integration with Development Workflow](#integration-with-development-workflow)

## Basic Usage Examples

### Example 1: First-Time Setup and Connection

```bash
# Activate Python virtual environment
source ~/rp2040-development/venv/bin/activate

# List available HID devices to verify connection
python3 hidlog.py --list

# Expected output:
# Available HID devices:
# Device 1:
#   Path: /dev/hidraw0
#   VID:PID: 1234:5678
#   Manufacturer: Custom Electronics
#   Product: pEMF Battery Monitor
#   Serial: 123456789

# Start basic monitoring with all log levels
python3 hidlog.py

# Expected initial output:
# Connected to device:
#   Manufacturer: Custom Electronics
#   Product: pEMF Battery Monitor
#   Serial: 123456789
#   VID:PID: 1234:5678
# 
# Monitoring logs (min level: DEBUG, module filter: none)
# Press Ctrl+C to stop
# --------------------------------------------------------------------------------
# [000.001s] [INFO ] [SYSTEM ] RP2040 boot started, firmware v1.2.3
# [000.010s] [INFO ] [SYSTEM ] Clock configuration: 125MHz system, 48MHz USB
# [000.025s] [INFO ] [SYSTEM ] GPIO initialization complete
# [000.040s] [INFO ] [BATTERY] ADC initialization complete, calibration loaded
# [000.055s] [INFO ] [PEMF   ] Timer initialization complete, 2Hz target set
# [000.070s] [INFO ] [USB    ] HID device enumeration started
# [000.250s] [INFO ] [USB    ] HID device enumerated successfully
# [000.300s] [INFO ] [SYSTEM ] All systems operational, entering main loop
```

### Example 2: Filtering by Log Level

```bash
# Monitor only important messages (INFO and above)
python3 hidlog.py --level INFO

# Expected output (less verbose):
# [000.001s] [INFO ] [SYSTEM ] RP2040 boot started, firmware v1.2.3
# [000.040s] [INFO ] [BATTERY] ADC initialization complete
# [000.055s] [INFO ] [PEMF   ] Timer initialization complete, 2Hz target set
# [000.300s] [INFO ] [SYSTEM ] All systems operational, entering main loop
# [001.456s] [INFO ] [BATTERY] State: Normal, Voltage: 3.45V
# [002.789s] [INFO ] [PEMF   ] Pulse generation started, frequency: 2Hz

# Monitor only warnings and errors
python3 hidlog.py --level WARN

# Expected output (only issues):
# [010.123s] [WARN ] [BATTERY] Voltage drop detected: 3.45V -> 3.12V
# [015.456s] [ERROR] [PEMF   ] Pulse timing deviation: +2.1ms (+5.0%)
# [020.789s] [WARN ] [USB    ] Log queue 80% full, consider reducing verbosity
```

### Example 3: Saving Logs to Files

```bash
# Save all logs to timestamped file
python3 hidlog.py --log-file logs/device_$(date +%Y%m%d_%H%M%S).log

# Monitor in terminal AND save to file simultaneously
python3 hidlog.py --log-file device.log | tee terminal_output.log

# Save only errors to separate file for analysis
python3 hidlog.py --level ERROR --log-file error_log_$(date +%Y%m%d).log

# Example log file content:
# [2024-01-15 14:30:01] [000.123s] [INFO ] [SYSTEM ] System boot complete
# [2024-01-15 14:30:01] [000.456s] [DEBUG] [BATTERY] ADC reading: 1650 (3.21V)
# [2024-01-15 14:30:01] [000.789s] [INFO ] [PEMF   ] Pulse generation started
```

## Battery Monitoring Examples

### Example 4: Real-Time Battery State Monitoring

```bash
# Monitor battery-specific logs with INFO level and above
python3 hidlog.py --module BATTERY --level INFO

# Simulate different battery conditions and observe output:

# Normal operation (3.1V - 3.6V):
# [010.123s] [INFO ] [BATTERY] State: Normal, Voltage: 3.45V, ADC: 1612
# [011.123s] [INFO ] [BATTERY] Periodic reading: 3.44V, ADC: 1608
# [012.123s] [INFO ] [BATTERY] Periodic reading: 3.46V, ADC: 1615

# Connect charger (voltage rises above 3.6V):
# [015.456s] [INFO ] [BATTERY] State changed: Normal -> Charging
# [015.457s] [INFO ] [BATTERY] State: Charging, Voltage: 3.78V, ADC: 1702
# [016.456s] [INFO ] [BATTERY] Charging detected: 3.82V, ADC: 1720
# [017.456s] [INFO ] [BATTERY] Charging progress: 3.95V, ADC: 1779

# Disconnect charger and let battery drain:
# [025.789s] [INFO ] [BATTERY] State changed: Charging -> Normal
# [025.790s] [INFO ] [BATTERY] State: Normal, Voltage: 3.55V, ADC: 1650
# [035.123s] [INFO ] [BATTERY] Voltage declining: 3.25V, ADC: 1512

# Low battery condition (below 3.1V):
# [045.456s] [WARN ] [BATTERY] State changed: Normal -> Low
# [045.457s] [WARN ] [BATTERY] State: Low, Voltage: 3.05V, ADC: 1398
# [046.456s] [WARN ] [BATTERY] Low battery warning: 3.02V, ADC: 1385
# [047.456s] [ERROR] [BATTERY] Critical battery level: 2.98V, ADC: 1365
```

### Example 5: Battery Calibration and ADC Analysis

```bash
# Monitor raw ADC values for calibration purposes
python3 hidlog.py --module BATTERY --level DEBUG

# Use a precision multimeter to measure actual battery voltage
# Compare with device readings for calibration:

# [050.123s] [DEBUG] [BATTERY] Raw ADC: 1650, Calculated: 3.21V, Filtered: 3.20V
# [050.223s] [DEBUG] [BATTERY] Raw ADC: 1651, Calculated: 3.21V, Filtered: 3.21V
# [050.323s] [DEBUG] [BATTERY] Raw ADC: 1649, Calculated: 3.20V, Filtered: 3.21V
# [050.423s] [DEBUG] [BATTERY] ADC statistics: Min=1648, Max=1652, Avg=1650.2

# Calibration calculation example:
# Multimeter reading: 3.25V
# Device calculated: 3.21V
# Calibration factor: 3.25 / 3.21 = 1.012

# Save calibration data for analysis
python3 hidlog.py --module BATTERY --level DEBUG --log-file battery_calibration.log

# Process calibration data with external tools:
# grep "Raw ADC" battery_calibration.log | awk '{print $6, $8}' > adc_voltage_pairs.csv
```

### Example 6: Battery Performance Analysis

```bash
# Long-term battery monitoring for performance analysis
python3 hidlog.py --module BATTERY --level INFO --log-file battery_performance_$(date +%Y%m%d).log &

# Let run for several hours, then analyze:
# - Voltage stability over time
# - Charging/discharging patterns
# - State transition frequency
# - ADC noise characteristics

# Example analysis commands:
grep "State changed" battery_performance_*.log | wc -l  # Count state changes
grep "Voltage:" battery_performance_*.log | tail -100   # Recent voltage readings
grep "ERROR\|WARN" battery_performance_*.log            # Battery issues
```

## pEMF Timing Monitoring Examples

### Example 7: Pulse Timing Accuracy Verification

```bash
# Monitor pEMF timing with high precision
python3 hidlog.py --module PEMF --level DEBUG

# Expected output showing timing measurements:
# [060.100s] [INFO ] [PEMF   ] Pulse generation started, target: 2Hz
# [060.200s] [DEBUG] [PEMF   ] Pulse timing: HIGH=2.0ms, LOW=498.0ms, Period=500.0ms
# [060.700s] [DEBUG] [PEMF   ] Pulse timing: HIGH=2.0ms, LOW=498.1ms, Period=500.1ms
# [061.200s] [DEBUG] [PEMF   ] Pulse timing: HIGH=1.9ms, LOW=498.1ms, Period=500.0ms
# [061.700s] [DEBUG] [PEMF   ] Pulse timing: HIGH=2.0ms, LOW=498.0ms, Period=500.0ms
# [062.200s] [DEBUG] [PEMF   ] Timing statistics: Avg=500.02ms, StdDev=0.05ms

# Monitor for timing deviations:
python3 hidlog.py --module PEMF --level WARN

# Expected warnings for timing issues:
# [065.123s] [WARN ] [PEMF   ] Timing deviation: HIGH +0.1ms (+5.0%)
# [065.623s] [WARN ] [PEMF   ] Timing deviation: LOW -0.2ms (-0.04%)
# [066.123s] [ERROR] [PEMF   ] Timing deviation exceeds tolerance: +1.2ms
# [066.623s] [WARN ] [PEMF   ] Timing recovered: HIGH=2.0ms, LOW=498.0ms
```

### Example 8: Performance Impact Analysis

```bash
# Monitor system performance while pEMF is running
python3 hidlog.py --module SYSTEM --level INFO

# Look for performance-related messages:
# [070.123s] [INFO ] [SYSTEM ] CPU usage: 15%, Free RAM: 180KB
# [070.623s] [INFO ] [SYSTEM ] Task timing: pEMF=0.1ms, Battery=2.3ms, USB=1.8ms
# [071.123s] [INFO ] [SYSTEM ] RTIC task delays: pEMF=0.0ms, Battery=0.1ms
# [071.623s] [INFO ] [SYSTEM ] Interrupt latency: Timer=0.05ms, USB=0.12ms

# Monitor for performance warnings:
python3 hidlog.py --module SYSTEM --level WARN

# Expected warnings for performance issues:
# [075.123s] [WARN ] [SYSTEM ] USB task overrun: 12.5ms (target: 10ms)
# [075.623s] [WARN ] [SYSTEM ] High interrupt rate: 1250 Hz (USB polling)
# [076.123s] [ERROR] [SYSTEM ] Task deadline missed: Battery monitor +5.2ms
# [076.623s] [WARN ] [SYSTEM ] Memory usage high: 85% (220KB used)
```

### Example 9: Load Testing and Stress Analysis

```bash
# Monitor pEMF under different load conditions
python3 hidlog.py --module PEMF --level INFO --log-file pemf_load_test.log &

# Simulate different load conditions:
# 1. No load (open circuit)
# 2. Light load (high impedance)
# 3. Heavy load (low impedance)
# 4. Variable load (changing impedance)

# Expected output for different loads:

# No load condition:
# [080.100s] [INFO ] [PEMF   ] Load impedance: >10kΩ (no load detected)
# [080.600s] [DEBUG] [PEMF   ] Pulse timing stable: HIGH=2.0ms, LOW=498.0ms

# Light load condition:
# [085.100s] [INFO ] [PEMF   ] Load impedance: ~5kΩ (light load)
# [085.600s] [DEBUG] [PEMF   ] Pulse timing: HIGH=2.0ms, LOW=498.0ms
# [086.100s] [INFO ] [PEMF   ] Load current: ~0.66mA

# Heavy load condition:
# [090.100s] [INFO ] [PEMF   ] Load impedance: ~100Ω (heavy load)
# [090.600s] [WARN ] [PEMF   ] High load current: ~33mA
# [091.100s] [WARN ] [PEMF   ] Timing affected by load: HIGH=2.1ms
# [091.600s] [INFO ] [PEMF   ] Load compensation applied

# Variable load condition:
# [095.100s] [WARN ] [PEMF   ] Load impedance changing: 1kΩ -> 500Ω
# [095.600s] [INFO ] [PEMF   ] Adaptive timing enabled
# [096.100s] [DEBUG] [PEMF   ] Timing adjustment: HIGH=2.0ms (compensated)
```

## System Diagnostics Examples

### Example 10: Boot Sequence Analysis

```bash
# Monitor complete system initialization
python3 hidlog.py --level INFO

# Power cycle the device and observe boot sequence:
# [000.001s] [INFO ] [SYSTEM ] RP2040 boot started, firmware v1.2.3
# [000.005s] [INFO ] [SYSTEM ] Hardware ID: RP2040-B2, Silicon rev: 2
# [000.010s] [INFO ] [SYSTEM ] Clock configuration: 125MHz system, 48MHz USB
# [000.015s] [INFO ] [SYSTEM ] Memory test: 264KB RAM available
# [000.020s] [INFO ] [SYSTEM ] Flash configuration: 2MB, XIP enabled
# [000.025s] [INFO ] [SYSTEM ] GPIO initialization complete
# [000.030s] [INFO ] [SYSTEM ] ADC calibration: Offset=0x12, Gain=0x0FFF
# [000.035s] [INFO ] [SYSTEM ] Timer initialization: 1MHz tick rate
# [000.040s] [INFO ] [BATTERY] ADC initialization complete, calibration loaded
# [000.045s] [INFO ] [BATTERY] Voltage divider ratio: 0.337 (10kΩ/5.1kΩ)
# [000.050s] [INFO ] [BATTERY] Initial reading: 3.45V, State: Normal
# [000.055s] [INFO ] [PEMF   ] Timer initialization complete, 2Hz target set
# [000.060s] [INFO ] [PEMF   ] GPIO 15 configured for pulse output
# [000.065s] [INFO ] [PEMF   ] Pulse generation ready, waiting for enable
# [000.070s] [INFO ] [USB    ] HID device enumeration started
# [000.150s] [INFO ] [USB    ] USB device configured, VID:PID=1234:5678
# [000.200s] [INFO ] [USB    ] HID report descriptor sent
# [000.250s] [INFO ] [USB    ] HID device enumerated successfully
# [000.280s] [INFO ] [SYSTEM ] RTIC scheduler started, 4 tasks active
# [000.290s] [INFO ] [SYSTEM ] Watchdog configured: 8s timeout
# [000.300s] [INFO ] [SYSTEM ] All systems operational, entering main loop
# [000.350s] [INFO ] [PEMF   ] Pulse generation started
```

### Example 11: Error Condition Monitoring

```bash
# Monitor for system errors and recovery
python3 hidlog.py --level WARN --log-file system_errors.log

# Trigger various error conditions and observe responses:

# ADC communication failure:
# [100.123s] [WARN ] [BATTERY] ADC timeout, retrying... (attempt 1/3)
# [100.223s] [WARN ] [BATTERY] ADC timeout, retrying... (attempt 2/3)
# [100.323s] [ERROR] [BATTERY] ADC communication failed after 3 retries
# [100.423s] [INFO ] [BATTERY] Using last known voltage: 3.45V
# [100.523s] [INFO ] [BATTERY] ADC recovery attempt in 5 seconds
# [105.523s] [INFO ] [BATTERY] ADC communication restored

# USB communication issues:
# [110.123s] [WARN ] [USB    ] HID report transmission failed, retrying
# [110.223s] [WARN ] [USB    ] USB device not configured, waiting
# [110.323s] [INFO ] [USB    ] USB device reconfigured successfully
# [110.423s] [INFO ] [USB    ] HID communication restored

# Memory/resource warnings:
# [120.123s] [WARN ] [SYSTEM ] Log queue 75% full (24/32 messages)
# [120.623s] [WARN ] [SYSTEM ] High memory usage: 85% (220KB/264KB)
# [121.123s] [ERROR] [SYSTEM ] Log queue overflow, dropping oldest messages
# [121.623s] [INFO ] [SYSTEM ] Log queue utilization reduced to 60%

# Timing/performance issues:
# [130.123s] [WARN ] [SYSTEM ] Task overrun: Battery monitor +2.5ms
# [130.623s] [ERROR] [PEMF   ] Timing deadline missed: +5.2ms
# [131.123s] [WARN ] [PEMF   ] Pulse generation paused for recovery
# [131.623s] [INFO ] [PEMF   ] Timing recovered, pulse generation resumed
```

### Example 12: Resource Usage Monitoring

```bash
# Monitor system resource usage over time
python3 hidlog.py --module SYSTEM --level DEBUG --log-file resource_usage.log

# Expected resource monitoring output:
# [140.123s] [DEBUG] [SYSTEM ] Memory usage: Stack=2.1KB, Heap=0KB, Static=45KB
# [140.623s] [DEBUG] [SYSTEM ] CPU usage: pEMF=5%, Battery=8%, USB=12%, Idle=75%
# [141.123s] [DEBUG] [SYSTEM ] Task execution times: pEMF=0.05ms, Battery=0.8ms
# [141.623s] [DEBUG] [SYSTEM ] Interrupt counts: Timer=280, USB=156, ADC=28
# [142.123s] [DEBUG] [SYSTEM ] Queue utilization: Log=45%, USB=12%
# [142.623s] [DEBUG] [SYSTEM ] Flash usage: Code=128KB, Data=4KB, Free=1920KB

# Analyze resource usage trends:
grep "Memory usage" resource_usage.log | tail -20    # Recent memory usage
grep "CPU usage" resource_usage.log | awk '{print $8}' | sed 's/%//' | sort -n | tail -1  # Peak CPU
```

## Advanced Filtering and Analysis

### Example 13: Complex Filtering Scenarios

```bash
# Monitor multiple modules with different priorities
python3 hidlog.py --module "BATTERY|PEMF" --level INFO

# Monitor errors from all modules
python3 hidlog.py --level ERROR

# Monitor specific subsystem during testing
python3 hidlog.py --module USB --level DEBUG

# Combine filtering with file output for analysis
python3 hidlog.py --module BATTERY --level WARN --log-file battery_issues.log
```

### Example 14: JSON Output for Data Analysis

```bash
# Output logs in JSON format for programmatic analysis
python3 hidlog.py --json-output --log-file logs.json

# Expected JSON output:
# {"timestamp": 0.123, "level": "INFO", "module": "SYSTEM", "message": "System boot complete"}
# {"timestamp": 0.456, "level": "DEBUG", "module": "BATTERY", "message": "ADC reading: 1650 (3.21V)"}
# {"timestamp": 0.789, "level": "INFO", "module": "PEMF", "message": "Pulse generation started"}

# Analyze JSON logs with jq:
# Filter only errors:
cat logs.json | jq 'select(.level == "ERROR")'

# Extract battery voltage readings:
cat logs.json | jq 'select(.module == "BATTERY" and (.message | contains("Voltage:"))) | {timestamp, voltage: (.message | split(" ")[2])}'

# Count messages by module:
cat logs.json | jq -r '.module' | sort | uniq -c

# Find timing deviations:
cat logs.json | jq 'select(.message | contains("deviation"))'
```

### Example 15: Real-Time Analysis with External Tools

```bash
# Pipe logs to external analysis tools
python3 hidlog.py --json-output | jq 'select(.level == "ERROR")' | while read line; do
    echo "ERROR DETECTED: $line"
    # Send notification, trigger alert, etc.
done

# Monitor battery voltage trends
python3 hidlog.py --module BATTERY --level INFO | grep "Voltage:" | while read line; do
    voltage=$(echo $line | awk '{print $6}' | sed 's/V,//')
    echo "$(date): Battery voltage: ${voltage}V"
    # Log to database, trigger alerts for low voltage, etc.
done

# Real-time timing analysis
python3 hidlog.py --module PEMF --level DEBUG | grep "Pulse timing" | while read line; do
    high_time=$(echo $line | grep -o 'HIGH=[0-9.]*ms' | cut -d= -f2 | sed 's/ms//')
    if (( $(echo "$high_time > 2.02" | bc -l) )); then
        echo "WARNING: Pulse timing deviation detected: ${high_time}ms"
    fi
done
```

## Automated Monitoring Scripts

### Example 16: Automated Test Script

```bash
#!/bin/bash
# automated_device_test.sh - Comprehensive device testing script

set -e

LOG_DIR="test_logs_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$LOG_DIR"

echo "Starting automated device test..."

# Test 1: Basic connectivity
echo "Test 1: Device connectivity"
timeout 10s python3 hidlog.py --list > "$LOG_DIR/connectivity.log"
if grep -q "1234:5678" "$LOG_DIR/connectivity.log"; then
    echo "✓ Device detected"
else
    echo "✗ Device not detected"
    exit 1
fi

# Test 2: Boot sequence monitoring
echo "Test 2: Boot sequence analysis"
echo "Please power cycle the device now..."
timeout 30s python3 hidlog.py --level INFO > "$LOG_DIR/boot_sequence.log" &
MONITOR_PID=$!
sleep 25
kill $MONITOR_PID 2>/dev/null || true

if grep -q "All systems operational" "$LOG_DIR/boot_sequence.log"; then
    echo "✓ Boot sequence completed successfully"
else
    echo "✗ Boot sequence incomplete or failed"
fi

# Test 3: Battery monitoring
echo "Test 3: Battery monitoring"
timeout 15s python3 hidlog.py --module BATTERY --level INFO > "$LOG_DIR/battery_test.log" &
MONITOR_PID=$!
sleep 10
kill $MONITOR_PID 2>/dev/null || true

battery_readings=$(grep -c "Voltage:" "$LOG_DIR/battery_test.log" || echo 0)
if [ "$battery_readings" -gt 5 ]; then
    echo "✓ Battery monitoring active ($battery_readings readings)"
else
    echo "✗ Insufficient battery readings ($battery_readings)"
fi

# Test 4: pEMF timing verification
echo "Test 4: pEMF timing verification"
timeout 15s python3 hidlog.py --module PEMF --level DEBUG > "$LOG_DIR/pemf_test.log" &
MONITOR_PID=$!
sleep 10
kill $MONITOR_PID 2>/dev/null || true

timing_measurements=$(grep -c "Pulse timing:" "$LOG_DIR/pemf_test.log" || echo 0)
if [ "$timing_measurements" -gt 3 ]; then
    echo "✓ pEMF timing measurements active ($timing_measurements measurements)"
else
    echo "✗ Insufficient pEMF timing data ($timing_measurements)"
fi

# Test 5: Error handling
echo "Test 5: Error handling verification"
timeout 10s python3 hidlog.py --level WARN > "$LOG_DIR/error_test.log" &
MONITOR_PID=$!
sleep 5
kill $MONITOR_PID 2>/dev/null || true

echo "Test completed. Results saved in $LOG_DIR/"
echo "Summary:"
echo "- Connectivity: $([ -f "$LOG_DIR/connectivity.log" ] && echo "PASS" || echo "FAIL")"
echo "- Boot sequence: $(grep -q "All systems operational" "$LOG_DIR/boot_sequence.log" && echo "PASS" || echo "FAIL")"
echo "- Battery monitoring: $([ "$battery_readings" -gt 5 ] && echo "PASS" || echo "FAIL")"
echo "- pEMF timing: $([ "$timing_measurements" -gt 3 ] && echo "PASS" || echo "FAIL")"
```

### Example 17: Continuous Monitoring Service

```bash
#!/bin/bash
# continuous_monitor.sh - Long-term device monitoring service

LOG_DIR="/var/log/rp2040-monitor"
ROTATE_SIZE="10M"
ROTATE_COUNT=5

# Create log directory
sudo mkdir -p "$LOG_DIR"
sudo chown $USER:$USER "$LOG_DIR"

# Function to start monitoring with log rotation
start_monitoring() {
    local module=$1
    local level=$2
    local logfile="$LOG_DIR/${module,,}_$(date +%Y%m%d).log"
    
    echo "Starting $module monitoring (level: $level)"
    
    while true; do
        # Check log file size and rotate if needed
        if [ -f "$logfile" ] && [ $(stat -f%z "$logfile" 2>/dev/null || stat -c%s "$logfile") -gt 10485760 ]; then
            mv "$logfile" "${logfile}.$(date +%H%M%S)"
            gzip "${logfile}.$(date +%H%M%S)" &
        fi
        
        # Start monitoring
        python3 hidlog.py --module "$module" --level "$level" --log-file "$logfile" || {
            echo "Monitoring failed, restarting in 10 seconds..."
            sleep 10
        }
    done
}

# Start monitoring processes in background
start_monitoring "BATTERY" "INFO" &
start_monitoring "PEMF" "WARN" &
start_monitoring "SYSTEM" "ERROR" &

echo "Continuous monitoring started. Logs in $LOG_DIR"
echo "Press Ctrl+C to stop all monitoring"

# Wait for interrupt
trap 'kill $(jobs -p); exit' INT
wait
```

## Integration with Development Workflow

### Example 18: Development Testing Integration

```bash
#!/bin/bash
# dev_test_integration.sh - Integration with development workflow

PROJECT_DIR="~/rp2040-projects/ass-easy-loop"
TEST_LOG_DIR="dev_test_logs"

cd "$PROJECT_DIR"
mkdir -p "$TEST_LOG_DIR"

echo "=== Development Test Integration ==="

# Step 1: Build firmware
echo "Building firmware..."
cargo build --release

# Step 2: Flash firmware
echo "Flashing firmware (ensure device is in bootloader mode)..."
cargo run --release

# Step 3: Wait for device to start
echo "Waiting for device to start..."
sleep 5

# Step 4: Verify device is running
if ! lsusb | grep -q "1234:5678"; then
    echo "ERROR: Device not detected after flashing"
    exit 1
fi

# Step 5: Run automated tests
echo "Running automated functionality tests..."

# Test boot sequence
echo "Testing boot sequence..."
timeout 30s python3 hidlog.py --level INFO > "$TEST_LOG_DIR/boot_test.log" &
BOOT_PID=$!

# Power cycle device for boot test
echo "Please power cycle the device now for boot sequence test..."
sleep 25
kill $BOOT_PID 2>/dev/null || true

# Test battery monitoring
echo "Testing battery monitoring..."
timeout 20s python3 hidlog.py --module BATTERY --level DEBUG > "$TEST_LOG_DIR/battery_test.log" &
BATTERY_PID=$!
sleep 15
kill $BATTERY_PID 2>/dev/null || true

# Test pEMF timing
echo "Testing pEMF timing..."
timeout 20s python3 hidlog.py --module PEMF --level DEBUG > "$TEST_LOG_DIR/pemf_test.log" &
PEMF_PID=$!
sleep 15
kill $PEMF_PID 2>/dev/null || true

# Analyze results
echo "Analyzing test results..."

# Check boot sequence
if grep -q "All systems operational" "$TEST_LOG_DIR/boot_test.log"; then
    echo "✓ Boot sequence test PASSED"
else
    echo "✗ Boot sequence test FAILED"
fi

# Check battery monitoring
battery_count=$(grep -c "ADC reading" "$TEST_LOG_DIR/battery_test.log" || echo 0)
if [ "$battery_count" -gt 10 ]; then
    echo "✓ Battery monitoring test PASSED ($battery_count readings)"
else
    echo "✗ Battery monitoring test FAILED ($battery_count readings)"
fi

# Check pEMF timing
timing_count=$(grep -c "Pulse timing" "$TEST_LOG_DIR/pemf_test.log" || echo 0)
if [ "$timing_count" -gt 5 ]; then
    echo "✓ pEMF timing test PASSED ($timing_count measurements)"
else
    echo "✗ pEMF timing test FAILED ($timing_count measurements)"
fi

# Check for errors
error_count=$(grep -c "ERROR" "$TEST_LOG_DIR"/*.log || echo 0)
if [ "$error_count" -eq 0 ]; then
    echo "✓ No errors detected"
else
    echo "⚠ $error_count errors detected, check logs"
fi

echo "Development test completed. Logs saved in $TEST_LOG_DIR/"
```

### Example 19: Performance Regression Testing

```bash
#!/bin/bash
# performance_regression_test.sh - Test for performance regressions

BASELINE_DIR="performance_baseline"
CURRENT_DIR="performance_current"

mkdir -p "$BASELINE_DIR" "$CURRENT_DIR"

echo "=== Performance Regression Test ==="

# Capture current performance data
echo "Capturing current performance data..."
timeout 60s python3 hidlog.py --module SYSTEM --level DEBUG > "$CURRENT_DIR/system_performance.log" &
timeout 60s python3 hidlog.py --module PEMF --level DEBUG > "$CURRENT_DIR/pemf_performance.log" &

wait

# Analyze performance metrics
echo "Analyzing performance metrics..."

# Extract timing data
grep "Task execution times" "$CURRENT_DIR/system_performance.log" | \
    awk '{print $6}' | sed 's/pEMF=//' | sed 's/ms,//' > "$CURRENT_DIR/pemf_execution_times.txt"

grep "CPU usage" "$CURRENT_DIR/system_performance.log" | \
    awk '{print $6}' | sed 's/pEMF=//' | sed 's/%,//' > "$CURRENT_DIR/cpu_usage.txt"

grep "Pulse timing" "$CURRENT_DIR/pemf_performance.log" | \
    grep -o 'HIGH=[0-9.]*ms' | sed 's/HIGH=//' | sed 's/ms//' > "$CURRENT_DIR/pulse_timing.txt"

# Calculate statistics
if [ -f "$CURRENT_DIR/pemf_execution_times.txt" ]; then
    avg_exec_time=$(awk '{sum+=$1} END {print sum/NR}' "$CURRENT_DIR/pemf_execution_times.txt")
    echo "Average pEMF execution time: ${avg_exec_time}ms"
fi

if [ -f "$CURRENT_DIR/cpu_usage.txt" ]; then
    avg_cpu=$(awk '{sum+=$1} END {print sum/NR}' "$CURRENT_DIR/cpu_usage.txt")
    echo "Average CPU usage: ${avg_cpu}%"
fi

if [ -f "$CURRENT_DIR/pulse_timing.txt" ]; then
    timing_stddev=$(awk '{sum+=$1; sumsq+=$1*$1} END {print sqrt(sumsq/NR - (sum/NR)^2)}' "$CURRENT_DIR/pulse_timing.txt")
    echo "Pulse timing standard deviation: ${timing_stddev}ms"
fi

# Compare with baseline if available
if [ -d "$BASELINE_DIR" ] && [ -f "$BASELINE_DIR/pemf_execution_times.txt" ]; then
    echo "Comparing with baseline..."
    baseline_exec=$(awk '{sum+=$1} END {print sum/NR}' "$BASELINE_DIR/pemf_execution_times.txt")
    current_exec=$(awk '{sum+=$1} END {print sum/NR}' "$CURRENT_DIR/pemf_execution_times.txt")
    
    improvement=$(echo "scale=2; ($baseline_exec - $current_exec) / $baseline_exec * 100" | bc)
    echo "Execution time change: ${improvement}% (positive = improvement)"
fi

echo "Performance test completed."
```

This comprehensive usage examples document demonstrates the full range of monitoring capabilities available with the USB HID logging system, from basic connectivity testing to advanced performance analysis and integration with development workflows.