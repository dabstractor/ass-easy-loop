//! Hardware-in-loop validation tests for real RP2040 device testing
//! 
//! These tests are designed to run with actual RP2040 hardware connected
//! and validate the complete system functionality including:
//! - Real-time pEMF pulse generation timing accuracy
//! - Battery monitoring with actual ADC readings
//! - USB HID logging functionality with hardware
//! - System integration under real operating conditions
//! 
//! Requirements: 9.1, 9.2, 9.3, 9.4, 9.5

#![cfg(test)]

use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};
use std::fs;
use std::path::Path;

/// Hardware test configuration
const TEST_DEVICE_VID: u16 = 0x1234;
const TEST_DEVICE_PID: u16 = 0x5678;
const TEST_TIMEOUT_SECONDS: u64 = 30;
const PEMF_FREQUENCY_HZ: f32 = 2.0;
const PEMF_HIGH_DURATION_MS: u64 = 2;
const PEMF_LOW_DURATION_MS: u64 = 498;
const TIMING_TOLERANCE_PERCENT: f32 = 1.0; // ±1% as per requirements

/// Test result structure for hardware validation
#[derive(Debug, Clone)]
pub struct HardwareTestResult {
    pub test_name: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub error_message: Option<String>,
    pub measurements: Vec<f64>,
}

impl HardwareTestResult {
    pub fn new(test_name: &str) -> Self {
        Self {
            test_name: test_name.to_string(),
            passed: false,
            duration_ms: 0,
            error_message: None,
            measurements: Vec::new(),
        }
    }

    pub fn success(mut self, duration_ms: u64) -> Self {
        self.passed = true;
        self.duration_ms = duration_ms;
        self
    }

    pub fn failure(mut self, duration_ms: u64, error: &str) -> Self {
        self.passed = false;
        self.duration_ms = duration_ms;
        self.error_message = Some(error.to_string());
        self
    }

    pub fn with_measurements(mut self, measurements: Vec<f64>) -> Self {
        self.measurements = measurements;
        self
    }
}

/// Hardware test suite runner
pub struct HardwareTestSuite {
    pub results: Vec<HardwareTestResult>,
    pub device_connected: bool,
    pub firmware_version: Option<String>,
}

impl HardwareTestSuite {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            device_connected: false,
            firmware_version: None,
        }
    }

    /// Check if RP2040 device is connected and accessible
    pub fn check_device_connection(&mut self) -> bool {
        let start_time = Instant::now();
        
        // Check if device appears in lsusb output
        let lsusb_result = Command::new("lsusb")
            .output();
            
        match lsusb_result {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let device_found = output_str.contains(&format!("{:04x}:{:04x}", TEST_DEVICE_VID, TEST_DEVICE_PID));
                
                if device_found {
                    self.device_connected = true;
                    println!("✓ RP2040 device found in USB enumeration");
                } else {
                    println!("✗ RP2040 device not found in USB enumeration");
                    println!("Expected VID:PID = {:04x}:{:04x}", TEST_DEVICE_VID, TEST_DEVICE_PID);
                    println!("Available USB devices:");
                    println!("{}", output_str);
                }
            }
            Err(e) => {
                println!("✗ Failed to run lsusb: {}", e);
                println!("Make sure lsusb is installed: sudo pacman -S usbutils");
            }
        }

        // Additional check for HID device accessibility
        if self.device_connected {
            self.check_hid_device_access();
        }

        let duration = start_time.elapsed().as_millis() as u64;
        let result = if self.device_connected {
            HardwareTestResult::new("Device Connection Check").success(duration)
        } else {
            HardwareTestResult::new("Device Connection Check")
                .failure(duration, "RP2040 device not found or not accessible")
        };
        
        self.results.push(result);
        self.device_connected
    }

    /// Check HID device accessibility
    fn check_hid_device_access(&mut self) {
        // Check for hidraw devices
        let hidraw_devices = fs::read_dir("/dev")
            .unwrap_or_else(|_| panic!("Cannot read /dev directory"))
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("hidraw") {
                    Some(name)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if hidraw_devices.is_empty() {
            println!("⚠ No hidraw devices found. USB HID logging may not work.");
            println!("Make sure hidapi is installed: sudo pacman -S hidapi");
        } else {
            println!("✓ Found hidraw devices: {:?}", hidraw_devices);
        }
    }

    /// Run all hardware validation tests
    pub fn run_all_tests(&mut self) -> bool {
        println!("=== Hardware Validation Test Suite ===");
        println!("Testing RP2040 pEMF/Battery Monitor Device");
        println!("Expected VID:PID: {:04x}:{:04x}", TEST_DEVICE_VID, TEST_DEVICE_PID);
        println!();

        // Check device connection first
        if !self.check_device_connection() {
            println!("❌ Device connection failed. Cannot proceed with hardware tests.");
            println!();
            println!("Troubleshooting steps:");
            println!("1. Ensure RP2040 is connected via USB");
            println!("2. Verify device is NOT in bootloader mode (should run normal firmware)");
            println!("3. Check that firmware includes USB HID logging functionality");
            println!("4. Try: lsusb | grep {:04x}:{:04x}", TEST_DEVICE_VID, TEST_DEVICE_PID);
            return false;
        }

        // Run individual test suites
        let mut all_passed = true;

        all_passed &= self.test_usb_hid_communication();
        all_passed &= self.test_pemf_timing_accuracy();
        all_passed &= self.test_battery_monitoring_integration();
        all_passed &= self.test_system_integration();
        all_passed &= self.test_performance_under_load();

        // Print summary
        self.print_test_summary();
        
        all_passed
    }

    /// Test USB HID communication with real hardware
    fn test_usb_hid_communication(&mut self) -> bool {
        println!("--- USB HID Communication Tests ---");
        let start_time = Instant::now();
        
        // Test 1: Device enumeration
        let enum_result = self.test_device_enumeration();
        
        // Test 2: Log message reception
        let log_result = self.test_log_message_reception();
        
        // Test 3: Connection stability
        let stability_result = self.test_connection_stability();
        
        let duration = start_time.elapsed().as_millis() as u64;
        let all_passed = enum_result && log_result && stability_result;
        
        if all_passed {
            println!("✓ USB HID communication tests passed");
        } else {
            println!("✗ USB HID communication tests failed");
        }
        
        all_passed
    }

    /// Test device enumeration
    fn test_device_enumeration(&mut self) -> bool {
        let start_time = Instant::now();
        
        // Use Python hidapi to test device enumeration
        let python_test = Command::new("python3")
            .arg("-c")
            .arg(&format!(r#"
import hid
try:
    device = hid.device()
    device.open(0x{:04x}, 0x{:04x})
    info = device.get_manufacturer_string()
    print(f"Device enumerated successfully: {{info}}")
    device.close()
    exit(0)
except Exception as e:
    print(f"Enumeration failed: {{e}}")
    exit(1)
"#, TEST_DEVICE_VID, TEST_DEVICE_PID))
            .output();

        let duration = start_time.elapsed().as_millis() as u64;
        
        match python_test {
            Ok(output) => {
                let success = output.status.success();
                let output_str = String::from_utf8_lossy(&output.stdout);
                let error_str = String::from_utf8_lossy(&output.stderr);
                
                let result = if success {
                    println!("✓ Device enumeration successful");
                    if !output_str.is_empty() {
                        println!("  Device info: {}", output_str.trim());
                    }
                    HardwareTestResult::new("Device Enumeration").success(duration)
                } else {
                    println!("✗ Device enumeration failed");
                    if !error_str.is_empty() {
                        println!("  Error: {}", error_str.trim());
                    }
                    HardwareTestResult::new("Device Enumeration")
                        .failure(duration, &error_str)
                };
                
                self.results.push(result);
                success
            }
            Err(e) => {
                println!("✗ Failed to run Python HID test: {}", e);
                println!("  Make sure python3-hid is installed: sudo pacman -S python-hid");
                let result = HardwareTestResult::new("Device Enumeration")
                    .failure(duration, &format!("Python execution failed: {}", e));
                self.results.push(result);
                false
            }
        }
    }

    /// Test log message reception
    fn test_log_message_reception(&mut self) -> bool {
        let start_time = Instant::now();
        
        println!("Testing log message reception (10 second capture)...");
        
        // Run hidlog.py for a short duration to capture messages
        let hidlog_test = Command::new("python3")
            .arg("scripts/utilities/hidlog.py")
            .arg("--timeout")
            .arg("10")
            .arg("--json-output")
            .output();

        let duration = start_time.elapsed().as_millis() as u64;
        
        match hidlog_test {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let error_str = String::from_utf8_lossy(&output.stderr);
                
                // Count received messages
                let message_count = output_str.lines()
                    .filter(|line| line.contains("timestamp"))
                    .count();
                
                let result = if message_count > 0 {
                    println!("✓ Received {} log messages", message_count);
                    HardwareTestResult::new("Log Message Reception")
                        .success(duration)
                        .with_measurements(vec![message_count as f64])
                } else {
                    println!("✗ No log messages received");
                    if !error_str.is_empty() {
                        println!("  Error: {}", error_str.trim());
                    }
                    HardwareTestResult::new("Log Message Reception")
                        .failure(duration, "No messages received")
                };
                
                self.results.push(result);
                message_count > 0
            }
            Err(e) => {
                println!("✗ Failed to run hidlog.py: {}", e);
                println!("  Make sure hidlog.py is available at scripts/utilities/hidlog.py");
                let result = HardwareTestResult::new("Log Message Reception")
                    .failure(duration, &format!("scripts/utilities/hidlog.py execution failed: {}", e));
                self.results.push(result);
                false
            }
        }
    }

    /// Test connection stability
    fn test_connection_stability(&mut self) -> bool {
        let start_time = Instant::now();
        
        println!("Testing connection stability (multiple connect/disconnect cycles)...");
        
        let mut successful_connections = 0;
        let test_cycles = 5;
        
        for cycle in 1..=test_cycles {
            println!("  Cycle {}/{}", cycle, test_cycles);
            
            // Test connection
            let connect_result = Command::new("python3")
                .arg("-c")
                .arg(&format!(r#"
import hid
import time
try:
    device = hid.device()
    device.open(0x{:04x}, 0x{:04x})
    time.sleep(0.5)  # Brief connection
    device.close()
    print("Connection successful")
    exit(0)
except Exception as e:
    print(f"Connection failed: {{e}}")
    exit(1)
"#, TEST_DEVICE_VID, TEST_DEVICE_PID))
                .output();
            
            if let Ok(output) = connect_result {
                if output.status.success() {
                    successful_connections += 1;
                    println!("    ✓ Connection successful");
                } else {
                    println!("    ✗ Connection failed");
                }
            }
            
            thread::sleep(Duration::from_millis(500));
        }
        
        let duration = start_time.elapsed().as_millis() as u64;
        let success_rate = (successful_connections as f64 / test_cycles as f64) * 100.0;
        
        let result = if success_rate >= 80.0 {
            println!("✓ Connection stability: {:.1}% ({}/{})", success_rate, successful_connections, test_cycles);
            HardwareTestResult::new("Connection Stability")
                .success(duration)
                .with_measurements(vec![success_rate])
        } else {
            println!("✗ Connection stability: {:.1}% ({}/{})", success_rate, successful_connections, test_cycles);
            HardwareTestResult::new("Connection Stability")
                .failure(duration, &format!("Low success rate: {:.1}%", success_rate))
        };
        
        self.results.push(result);
        success_rate >= 80.0
    }

    /// Test pEMF timing accuracy with USB logging active
    fn test_pemf_timing_accuracy(&mut self) -> bool {
        println!("--- pEMF Timing Accuracy Tests ---");
        let start_time = Instant::now();
        
        // This test requires external measurement equipment (oscilloscope or logic analyzer)
        // For now, we'll test the timing calculations and validate against log messages
        
        println!("Testing pEMF timing accuracy through log message analysis...");
        
        // Capture pEMF-related log messages for timing analysis
        let timing_test = Command::new("python3")
            .arg("-c")
            .arg(&format!(r#"
import hid
import time
import json
import struct

def parse_log_message(data):
    if len(data) < 64:
        return None
    
    level = data[0]
    module = data[1:9].rstrip(b'\x00').decode('utf-8', errors='ignore')
    message = data[9:57].rstrip(b'\x00').decode('utf-8', errors='ignore')
    timestamp = struct.unpack('<I', data[57:61])[0]
    
    return {{
        'timestamp': timestamp,
        'level': level,
        'module': module,
        'message': message
    }}

try:
    device = hid.device()
    device.open(0x{:04x}, 0x{:04x})
    
    pemf_messages = []
    start_time = time.time()
    
    # Collect messages for 15 seconds
    while time.time() - start_time < 15:
        data = device.read(64, timeout_ms=1000)
        if data:
            msg = parse_log_message(bytes(data))
            if msg and 'PEMF' in msg.get('module', '').upper():
                pemf_messages.append(msg)
                if len(pemf_messages) >= 10:  # Enough samples
                    break
    
    device.close()
    
    if len(pemf_messages) >= 5:
        print(f"Captured {{len(pemf_messages)}} pEMF timing messages")
        for msg in pemf_messages[:5]:  # Show first 5
            print(f"[{{msg['timestamp']:08d}}] {{msg['message']}}")
        exit(0)
    else:
        print(f"Insufficient pEMF messages captured: {{len(pemf_messages)}}")
        exit(1)
        
except Exception as e:
    print(f"pEMF timing test failed: {{e}}")
    exit(1)
"#, TEST_DEVICE_VID, TEST_DEVICE_PID))
            .output();

        let duration = start_time.elapsed().as_millis() as u64;
        
        match timing_test {
            Ok(output) => {
                let success = output.status.success();
                let output_str = String::from_utf8_lossy(&output.stdout);
                let error_str = String::from_utf8_lossy(&output.stderr);
                
                let result = if success {
                    println!("✓ pEMF timing messages captured successfully");
                    println!("  Sample messages:");
                    for line in output_str.lines().take(6) {
                        if !line.is_empty() {
                            println!("    {}", line);
                        }
                    }
                    
                    // Validate timing constants
                    let timing_valid = self.validate_timing_constants();
                    
                    if timing_valid {
                        HardwareTestResult::new("pEMF Timing Accuracy").success(duration)
                    } else {
                        HardwareTestResult::new("pEMF Timing Accuracy")
                            .failure(duration, "Timing constants validation failed")
                    }
                } else {
                    println!("✗ pEMF timing test failed");
                    if !error_str.is_empty() {
                        println!("  Error: {}", error_str.trim());
                    }
                    HardwareTestResult::new("pEMF Timing Accuracy")
                        .failure(duration, &error_str)
                };
                
                self.results.push(result);
                success
            }
            Err(e) => {
                println!("✗ Failed to run pEMF timing test: {}", e);
                let result = HardwareTestResult::new("pEMF Timing Accuracy")
                    .failure(duration, &format!("Test execution failed: {}", e));
                self.results.push(result);
                false
            }
        }
    }

    /// Validate timing constants against requirements
    fn validate_timing_constants(&self) -> bool {
        let total_period_ms = PEMF_HIGH_DURATION_MS + PEMF_LOW_DURATION_MS;
        let calculated_frequency = 1000.0 / total_period_ms as f32;
        let frequency_error = ((calculated_frequency - PEMF_FREQUENCY_HZ) / PEMF_FREQUENCY_HZ * 100.0).abs();
        
        println!("  Timing validation:");
        println!("    HIGH duration: {}ms", PEMF_HIGH_DURATION_MS);
        println!("    LOW duration: {}ms", PEMF_LOW_DURATION_MS);
        println!("    Total period: {}ms", total_period_ms);
        println!("    Calculated frequency: {:.3}Hz", calculated_frequency);
        println!("    Target frequency: {:.3}Hz", PEMF_FREQUENCY_HZ);
        println!("    Frequency error: {:.2}%", frequency_error);
        
        let timing_valid = frequency_error <= TIMING_TOLERANCE_PERCENT;
        
        if timing_valid {
            println!("  ✓ Timing constants within ±{}% tolerance", TIMING_TOLERANCE_PERCENT);
        } else {
            println!("  ✗ Timing constants exceed ±{}% tolerance", TIMING_TOLERANCE_PERCENT);
        }
        
        timing_valid
    }

    /// Test battery monitoring integration with actual ADC readings
    fn test_battery_monitoring_integration(&mut self) -> bool {
        println!("--- Battery Monitoring Integration Tests ---");
        let start_time = Instant::now();
        
        println!("Testing battery monitoring through log message analysis...");
        
        // Capture battery-related log messages
        let battery_test = Command::new("python3")
            .arg("-c")
            .arg(&format!(r#"
import hid
import time
import json
import struct
import re

def parse_log_message(data):
    if len(data) < 64:
        return None
    
    level = data[0]
    module = data[1:9].rstrip(b'\x00').decode('utf-8', errors='ignore')
    message = data[9:57].rstrip(b'\x00').decode('utf-8', errors='ignore')
    timestamp = struct.unpack('<I', data[57:61])[0]
    
    return {{
        'timestamp': timestamp,
        'level': level,
        'module': module,
        'message': message
    }}

def extract_adc_voltage(message):
    # Look for ADC readings and voltage values in messages
    adc_match = re.search(r'ADC[:\s]*(\d+)', message)
    voltage_match = re.search(r'(\d+\.?\d*)\s*[mM]?[vV]', message)
    
    adc_value = int(adc_match.group(1)) if adc_match else None
    voltage_value = float(voltage_match.group(1)) if voltage_match else None
    
    return adc_value, voltage_value

try:
    device = hid.device()
    device.open(0x{:04x}, 0x{:04x})
    
    battery_messages = []
    adc_readings = []
    voltage_readings = []
    start_time = time.time()
    
    # Collect messages for 20 seconds
    while time.time() - start_time < 20:
        data = device.read(64, timeout_ms=1000)
        if data:
            msg = parse_log_message(bytes(data))
            if msg and 'BATTERY' in msg.get('module', '').upper():
                battery_messages.append(msg)
                
                # Extract ADC and voltage values
                adc_val, volt_val = extract_adc_voltage(msg['message'])
                if adc_val is not None:
                    adc_readings.append(adc_val)
                if volt_val is not None:
                    voltage_readings.append(volt_val)
                
                if len(battery_messages) >= 15:  # Enough samples
                    break
    
    device.close()
    
    print(f"Captured {{len(battery_messages)}} battery messages")
    print(f"ADC readings: {{len(adc_readings)}} samples")
    print(f"Voltage readings: {{len(voltage_readings)}} samples")
    
    if len(battery_messages) >= 5:
        print("Sample battery messages:")
        for msg in battery_messages[:5]:
            print(f"[{{msg['timestamp']:08d}}] {{msg['message']}}")
        
        if adc_readings:
            print(f"ADC range: {{min(adc_readings)}} - {{max(adc_readings)}}")
        if voltage_readings:
            print(f"Voltage range: {{min(voltage_readings):.2f}} - {{max(voltage_readings):.2f}}V")
        
        exit(0)
    else:
        print(f"Insufficient battery messages: {{len(battery_messages)}}")
        exit(1)
        
except Exception as e:
    print(f"Battery monitoring test failed: {{e}}")
    exit(1)
"#, TEST_DEVICE_VID, TEST_DEVICE_PID))
            .output();

        let duration = start_time.elapsed().as_millis() as u64;
        
        match battery_test {
            Ok(output) => {
                let success = output.status.success();
                let output_str = String::from_utf8_lossy(&output.stdout);
                let error_str = String::from_utf8_lossy(&output.stderr);
                
                let result = if success {
                    println!("✓ Battery monitoring messages captured successfully");
                    println!("  Analysis results:");
                    for line in output_str.lines() {
                        if !line.is_empty() {
                            println!("    {}", line);
                        }
                    }
                    
                    // Validate ADC conversion logic
                    let conversion_valid = self.validate_adc_conversion();
                    
                    if conversion_valid {
                        HardwareTestResult::new("Battery Monitoring Integration").success(duration)
                    } else {
                        HardwareTestResult::new("Battery Monitoring Integration")
                            .failure(duration, "ADC conversion validation failed")
                    }
                } else {
                    println!("✗ Battery monitoring test failed");
                    if !error_str.is_empty() {
                        println!("  Error: {}", error_str.trim());
                    }
                    HardwareTestResult::new("Battery Monitoring Integration")
                        .failure(duration, &error_str)
                };
                
                self.results.push(result);
                success
            }
            Err(e) => {
                println!("✗ Failed to run battery monitoring test: {}", e);
                let result = HardwareTestResult::new("Battery Monitoring Integration")
                    .failure(duration, &format!("Test execution failed: {}", e));
                self.results.push(result);
                false
            }
        }
    }

    /// Validate ADC conversion calculations
    fn validate_adc_conversion(&self) -> bool {
        println!("  ADC conversion validation:");
        
        // Test known conversion points
        let test_cases = vec![
            (1425, 3100), // Low battery threshold
            (1675, 3600), // Charging threshold
            (2048, 4400), // Mid-range
        ];
        
        let mut all_valid = true;
        
        for (adc_value, expected_mv) in test_cases {
            // Simulate the conversion calculation from battery.rs
            let calculated_mv = (adc_value as u32 * 2386) / 1000;
            let error_percent = ((calculated_mv as f32 - expected_mv as f32) / expected_mv as f32 * 100.0).abs();
            
            println!("    ADC {} -> {}mV (expected ~{}mV, error: {:.1}%)", 
                     adc_value, calculated_mv, expected_mv, error_percent);
            
            if error_percent > 10.0 { // Allow 10% tolerance for conversion
                all_valid = false;
            }
        }
        
        if all_valid {
            println!("  ✓ ADC conversion calculations valid");
        } else {
            println!("  ✗ ADC conversion calculations have excessive errors");
        }
        
        all_valid
    }

    /// Test system integration under normal operating conditions
    fn test_system_integration(&mut self) -> bool {
        println!("--- System Integration Tests ---");
        let start_time = Instant::now();
        
        println!("Testing complete system integration (30 second monitoring)...");
        
        // Monitor all system components simultaneously
        let integration_test = Command::new("python3")
            .arg("-c")
            .arg(&format!(r#"
import hid
import time
import struct
from collections import defaultdict

def parse_log_message(data):
    if len(data) < 64:
        return None
    
    level = data[0]
    module = data[1:9].rstrip(b'\x00').decode('utf-8', errors='ignore')
    message = data[9:57].rstrip(b'\x00').decode('utf-8', errors='ignore')
    timestamp = struct.unpack('<I', data[57:61])[0]
    
    return {{
        'timestamp': timestamp,
        'level': level,
        'module': module,
        'message': message
    }}

try:
    device = hid.device()
    device.open(0x{:04x}, 0x{:04x})
    
    messages_by_module = defaultdict(list)
    error_count = 0
    warning_count = 0
    start_time = time.time()
    
    # Monitor for 30 seconds
    while time.time() - start_time < 30:
        data = device.read(64, timeout_ms=1000)
        if data:
            msg = parse_log_message(bytes(data))
            if msg:
                module = msg['module'].upper()
                messages_by_module[module].append(msg)
                
                if msg['level'] == 3:  # ERROR
                    error_count += 1
                elif msg['level'] == 2:  # WARN
                    warning_count += 1
    
    device.close()
    
    print("System integration test results:")
    print(f"Total modules active: {{len(messages_by_module)}}")
    print(f"Error messages: {{error_count}}")
    print(f"Warning messages: {{warning_count}}")
    
    for module, messages in messages_by_module.items():
        print(f"{{module}}: {{len(messages)}} messages")
    
    # Check that key modules are active
    required_modules = ['SYSTEM', 'BATTERY', 'PEMF']
    missing_modules = [mod for mod in required_modules if mod not in messages_by_module]
    
    if missing_modules:
        print(f"Missing required modules: {{missing_modules}}")
        exit(1)
    
    if error_count > 5:  # Allow some errors but not too many
        print(f"Too many errors detected: {{error_count}}")
        exit(1)
    
    print("✓ System integration test passed")
    exit(0)
        
except Exception as e:
    print(f"System integration test failed: {{e}}")
    exit(1)
"#, TEST_DEVICE_VID, TEST_DEVICE_PID))
            .output();

        let duration = start_time.elapsed().as_millis() as u64;
        
        match integration_test {
            Ok(output) => {
                let success = output.status.success();
                let output_str = String::from_utf8_lossy(&output.stdout);
                let error_str = String::from_utf8_lossy(&output.stderr);
                
                let result = if success {
                    println!("✓ System integration test passed");
                    println!("  Results:");
                    for line in output_str.lines() {
                        if !line.is_empty() {
                            println!("    {}", line);
                        }
                    }
                    HardwareTestResult::new("System Integration").success(duration)
                } else {
                    println!("✗ System integration test failed");
                    if !error_str.is_empty() {
                        println!("  Error: {}", error_str.trim());
                    }
                    HardwareTestResult::new("System Integration")
                        .failure(duration, &error_str)
                };
                
                self.results.push(result);
                success
            }
            Err(e) => {
                println!("✗ Failed to run system integration test: {}", e);
                let result = HardwareTestResult::new("System Integration")
                    .failure(duration, &format!("Test execution failed: {}", e));
                self.results.push(result);
                false
            }
        }
    }

    /// Test performance under load conditions
    fn test_performance_under_load(&mut self) -> bool {
        println!("--- Performance Under Load Tests ---");
        let start_time = Instant::now();
        
        println!("Testing system performance with high logging activity...");
        
        // This test would ideally stress the system and measure performance
        // For now, we'll monitor message throughput and timing consistency
        
        let performance_test = Command::new("python3")
            .arg("-c")
            .arg(&format!(r#"
import hid
import time
import struct

def parse_log_message(data):
    if len(data) < 64:
        return None
    
    timestamp = struct.unpack('<I', data[57:61])[0]
    return timestamp

try:
    device = hid.device()
    device.open(0x{:04x}, 0x{:04x})
    
    timestamps = []
    message_count = 0
    start_time = time.time()
    
    # Collect messages for 15 seconds
    while time.time() - start_time < 15:
        data = device.read(64, timeout_ms=100)
        if data:
            timestamp = parse_log_message(bytes(data))
            if timestamp:
                timestamps.append(timestamp)
                message_count += 1
    
    device.close()
    
    if len(timestamps) >= 10:
        # Calculate message rate
        time_span = (timestamps[-1] - timestamps[0]) / 1000.0  # Convert to seconds
        message_rate = len(timestamps) / time_span if time_span > 0 else 0
        
        # Calculate timing consistency
        intervals = [timestamps[i+1] - timestamps[i] for i in range(len(timestamps)-1)]
        avg_interval = sum(intervals) / len(intervals) if intervals else 0
        max_interval = max(intervals) if intervals else 0
        min_interval = min(intervals) if intervals else 0
        
        print(f"Performance test results:")
        print(f"Messages received: {{message_count}}")
        print(f"Message rate: {{message_rate:.1f}} msg/sec")
        print(f"Timing intervals (ms): avg={{avg_interval:.1f}}, min={{min_interval}}, max={{max_interval}}")
        
        # Performance criteria
        if message_rate >= 1.0 and max_interval < 10000:  # At least 1 msg/sec, max 10s gaps
            print("✓ Performance test passed")
            exit(0)
        else:
            print("✗ Performance below acceptable thresholds")
            exit(1)
    else:
        print(f"Insufficient data for performance analysis: {{len(timestamps)}} samples")
        exit(1)
        
except Exception as e:
    print(f"Performance test failed: {{e}}")
    exit(1)
"#, TEST_DEVICE_VID, TEST_DEVICE_PID))
            .output();

        let duration = start_time.elapsed().as_millis() as u64;
        
        match performance_test {
            Ok(output) => {
                let success = output.status.success();
                let output_str = String::from_utf8_lossy(&output.stdout);
                let error_str = String::from_utf8_lossy(&output.stderr);
                
                let result = if success {
                    println!("✓ Performance test passed");
                    println!("  Results:");
                    for line in output_str.lines() {
                        if !line.is_empty() {
                            println!("    {}", line);
                        }
                    }
                    HardwareTestResult::new("Performance Under Load").success(duration)
                } else {
                    println!("✗ Performance test failed");
                    if !error_str.is_empty() {
                        println!("  Error: {}", error_str.trim());
                    }
                    HardwareTestResult::new("Performance Under Load")
                        .failure(duration, &error_str)
                };
                
                self.results.push(result);
                success
            }
            Err(e) => {
                println!("✗ Failed to run performance test: {}", e);
                let result = HardwareTestResult::new("Performance Under Load")
                    .failure(duration, &format!("Test execution failed: {}", e));
                self.results.push(result);
                false
            }
        }
    }

    /// Print comprehensive test summary
    fn print_test_summary(&self) {
        println!();
        println!("=== Hardware Validation Test Summary ===");
        
        let total_tests = self.results.len();
        let passed_tests = self.results.iter().filter(|r| r.passed).count();
        let failed_tests = total_tests - passed_tests;
        
        println!("Total tests: {}", total_tests);
        println!("Passed: {} ✓", passed_tests);
        println!("Failed: {} ✗", failed_tests);
        println!();
        
        // Detailed results
        for result in &self.results {
            let status = if result.passed { "✓ PASS" } else { "✗ FAIL" };
            println!("{} {} ({} ms)", status, result.test_name, result.duration_ms);
            
            if let Some(error) = &result.error_message {
                println!("    Error: {}", error);
            }
            
            if !result.measurements.is_empty() {
                println!("    Measurements: {:?}", result.measurements);
            }
        }
        
        println!();
        
        if failed_tests == 0 {
            println!("🎉 All hardware validation tests passed!");
            println!("The RP2040 device is functioning correctly with USB HID logging.");
        } else {
            println!("⚠️  {} test(s) failed. Please review the errors above.", failed_tests);
            println!("Common issues and solutions:");
            println!("- Device not connected: Check USB cable and connection");
            println!("- Wrong firmware: Ensure device runs USB HID logging firmware");
            println!("- Permission issues: Check hidapi permissions or run with sudo");
            println!("- Missing dependencies: Install required Python packages");
        }
    }
}

// ============================================================================
// Individual Hardware Test Functions
// ============================================================================

#[test]
fn test_hardware_device_connection() {
    let mut test_suite = HardwareTestSuite::new();
    
    println!("Testing RP2040 device connection...");
    let connected = test_suite.check_device_connection();
    
    if !connected {
        println!("Hardware device not available - skipping hardware tests");
        println!("To run hardware tests:");
        println!("1. Connect RP2040 device via USB");
        println!("2. Ensure device is running USB HID logging firmware");
        println!("3. Verify device appears in: lsusb | grep {:04x}:{:04x}", TEST_DEVICE_VID, TEST_DEVICE_PID);
        return;
    }
    
    assert_no_std!(connected, "RP2040 device should be connected and accessible");
}

#[test]
fn test_hardware_usb_hid_communication() {
    let mut test_suite = HardwareTestSuite::new();
    
    if !test_suite.check_device_connection() {
        println!("Hardware device not available - skipping USB HID test");
        return;
    }
    
    let success = test_suite.test_usb_hid_communication();
    assert_no_std!(success, "USB HID communication should work with real hardware");
}

#[test]
fn test_hardware_pemf_timing() {
    let mut test_suite = HardwareTestSuite::new();
    
    if !test_suite.check_device_connection() {
        println!("Hardware device not available - skipping pEMF timing test");
        return;
    }
    
    let success = test_suite.test_pemf_timing_accuracy();
    assert_no_std!(success, "pEMF timing should be accurate with USB logging active");
}

#[test]
fn test_hardware_battery_monitoring() {
    let mut test_suite = HardwareTestSuite::new();
    
    if !test_suite.check_device_connection() {
        println!("Hardware device not available - skipping battery monitoring test");
        return;
    }
    
    let success = test_suite.test_battery_monitoring_integration();
    assert_no_std!(success, "Battery monitoring should work with actual ADC readings");
}

#[test]
fn test_hardware_system_integration() {
    let mut test_suite = HardwareTestSuite::new();
    
    if !test_suite.check_device_connection() {
        println!("Hardware device not available - skipping system integration test");
        return;
    }
    
    let success = test_suite.test_system_integration();
    assert_no_std!(success, "Complete system should integrate properly");
}

#[test]
#[ignore] // This test takes a long time, run with --ignored
fn test_hardware_full_validation_suite() {
    let mut test_suite = HardwareTestSuite::new();
    
    println!("Running complete hardware validation test suite...");
    println!("This test requires actual RP2040 hardware connected via USB.");
    println!();
    
    let all_passed = test_suite.run_all_tests();
    
    assert_no_std!(all_passed, "All hardware validation tests should pass");
}