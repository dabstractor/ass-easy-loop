#!/usr/bin/env python3
"""
Hardware Validation Test Runner for RP2040 pEMF/Battery Monitor Device

This script provides a comprehensive test runner for validating the hardware
functionality of the RP2040 device with USB HID logging capability.

Requirements: 9.1, 9.2, 9.3, 9.4, 9.5
"""

import argparse
import subprocess
import sys
import time
import json
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Optional, Tuple

# Test configuration
TEST_DEVICE_VID = 0x1234
TEST_DEVICE_PID = 0x5678
DEFAULT_TEST_DURATION = 30

class HardwareTestRunner:
    """Main hardware test runner class"""
    
    def __init__(self, verbose: bool = False):
        self.verbose = verbose
        self.test_results = {}
        self.start_time = datetime.now()
        
    def log(self, message: str, level: str = "INFO"):
        """Log a message with timestamp"""
        timestamp = datetime.now().strftime("%H:%M:%S")
        prefix = f"[{timestamp}] [{level}]"
        print(f"{prefix} {message}")
        
    def run_command(self, cmd: List[str], timeout: Optional[int] = None) -> Tuple[bool, str, str]:
        """Run a command and return success, stdout, stderr"""
        try:
            if self.verbose:
                self.log(f"Running: {' '.join(cmd)}", "DEBUG")
            
            result = subprocess.run(
                cmd, 
                capture_output=True, 
                text=True, 
                timeout=timeout
            )
            
            success = result.returncode == 0
            return success, result.stdout, result.stderr
            
        except subprocess.TimeoutExpired:
            return False, "", f"Command timed out after {timeout} seconds"
        except Exception as e:
            return False, "", str(e)
    
    def check_prerequisites(self) -> bool:
        """Check that all required tools and dependencies are available"""
        self.log("Checking prerequisites...")
        
        prerequisites = [
            ("lsusb", "USB utilities"),
            ("python3", "Python 3"),
            ("cargo", "Rust Cargo"),
            ("rustc", "Rust compiler"),
        ]
        
        missing = []
        for cmd, description in prerequisites:
            success, _, _ = self.run_command(["which", cmd])
            if not success:
                missing.append(f"{cmd} ({description})")
        
        if missing:
            self.log(f"Missing prerequisites: {', '.join(missing)}", "ERROR")
            self.log("Please install missing tools using the setup guide", "ERROR")
            return False
        
        # Check Python packages
        python_packages = ["hid", "struct", "time"]
        for package in python_packages:
            success, _, stderr = self.run_command([
                "python3", "-c", f"import {package}"
            ])
            if not success:
                self.log(f"Missing Python package: {package}", "ERROR")
                self.log("Install with: pip install hidapi", "ERROR")
                return False
        
        self.log("✓ All prerequisites available")
        return True
    
    def check_device_connection(self) -> bool:
        """Check if the RP2040 device is connected and accessible"""
        self.log("Checking device connection...")
        
        # Check USB enumeration
        success, stdout, stderr = self.run_command(["lsusb"])
        if not success:
            self.log(f"Failed to run lsusb: {stderr}", "ERROR")
            return False
        
        device_pattern = f"{TEST_DEVICE_VID:04x}:{TEST_DEVICE_PID:04x}"
        if device_pattern not in stdout:
            self.log(f"Device {device_pattern} not found in USB enumeration", "ERROR")
            self.log("Available USB devices:", "INFO")
            for line in stdout.strip().split('\n'):
                if line.strip():
                    self.log(f"  {line}", "INFO")
            return False
        
        # Check HID accessibility
        hid_test_script = f"""
import hid
try:
    device = hid.device()
    device.open(0x{TEST_DEVICE_VID:04x}, 0x{TEST_DEVICE_PID:04x})
    info = device.get_manufacturer_string()
    device.close()
    print(f"HID device accessible: {{info or 'Unknown'}}")
    exit(0)
except Exception as e:
    print(f"HID access failed: {{e}}")
    exit(1)
"""
        
        success, stdout, stderr = self.run_command([
            "python3", "-c", hid_test_script
        ])
        
        if not success:
            self.log(f"HID device not accessible: {stderr}", "ERROR")
            return False
        
        self.log(f"✓ Device connected and accessible: {stdout.strip()}")
        return True
    
    def run_rust_tests(self, test_name: Optional[str] = None) -> bool:
        """Run Rust hardware validation tests"""
        self.log(f"Running Rust hardware tests{f' ({test_name})' if test_name else ''}...")
        
        cmd = ["cargo", "test"]
        if test_name:
            cmd.extend(["--test", test_name])
        else:
            cmd.extend([
                "--test", "hardware_validation_tests",
                "--test", "pemf_timing_validation_test", 
                "--test", "battery_adc_integration_test"
            ])
        
        cmd.extend(["--", "--nocapture"])
        
        success, stdout, stderr = self.run_command(cmd, timeout=300)  # 5 minute timeout
        
        if success:
            self.log("✓ Rust hardware tests passed")
            if self.verbose:
                self.log("Test output:", "DEBUG")
                for line in stdout.split('\n'):
                    if line.strip():
                        self.log(f"  {line}", "DEBUG")
        else:
            self.log("✗ Rust hardware tests failed", "ERROR")
            self.log("Error output:", "ERROR")
            for line in stderr.split('\n')[-10:]:  # Last 10 lines
                if line.strip():
                    self.log(f"  {line}", "ERROR")
        
        self.test_results['rust_tests'] = {
            'passed': success,
            'output': stdout,
            'error': stderr
        }
        
        return success
    
    def run_usb_communication_test(self, duration: int = 10) -> bool:
        """Test USB HID communication"""
        self.log(f"Testing USB HID communication ({duration}s)...")
        
        comm_test_script = f"""
import hid
import time
import struct

def parse_log_message(data):
    if len(data) < 64:
        return None
    
    level = data[0]
    module = data[1:9].rstrip(b'\\x00').decode('utf-8', errors='ignore')
    message = data[9:57].rstrip(b'\\x00').decode('utf-8', errors='ignore')
    timestamp = struct.unpack('<I', data[57:61])[0]
    
    return {{
        'timestamp': timestamp,
        'level': level,
        'module': module,
        'message': message
    }}

try:
    device = hid.device()
    device.open(0x{TEST_DEVICE_VID:04x}, 0x{TEST_DEVICE_PID:04x})
    
    messages_received = 0
    start_time = time.time()
    
    while time.time() - start_time < {duration}:
        data = device.read(64, timeout_ms=1000)
        if data:
            msg = parse_log_message(bytes(data))
            if msg:
                messages_received += 1
                if messages_received <= 5:  # Show first 5 messages
                    print(f"[{{msg['timestamp']:08d}}] [{{['DEBUG','INFO','WARN','ERROR'][msg['level']]}}] [{{msg['module']}}] {{msg['message']}}")
    
    device.close()
    
    print(f"\\nReceived {{messages_received}} messages in {duration} seconds")
    print(f"Message rate: {{messages_received / {duration}:.1f}} msg/sec")
    
    if messages_received >= 5:
        print("SUCCESS: USB communication working")
        exit(0)
    else:
        print("WARNING: Low message rate")
        exit(1)
        
except Exception as e:
    print(f"USB communication test failed: {{e}}")
    exit(1)
"""
        
        success, stdout, stderr = self.run_command([
            "python3", "-c", comm_test_script
        ], timeout=duration + 10)
        
        if success:
            self.log("✓ USB HID communication test passed")
            if self.verbose:
                for line in stdout.split('\n'):
                    if line.strip():
                        self.log(f"  {line}", "DEBUG")
        else:
            self.log("✗ USB HID communication test failed", "ERROR")
            self.log(f"Error: {stderr}", "ERROR")
        
        self.test_results['usb_communication'] = {
            'passed': success,
            'output': stdout,
            'error': stderr
        }
        
        return success
    
    def run_timing_validation_test(self, duration: int = 30) -> bool:
        """Test pEMF timing accuracy with USB logging active"""
        self.log(f"Testing pEMF timing accuracy ({duration}s)...")
        
        timing_test_script = f"""
import hid
import time
import struct
import re
from collections import defaultdict

def parse_log_message(data):
    if len(data) < 64:
        return None
    
    level = data[0]
    module = data[1:9].rstrip(b'\\x00').decode('utf-8', errors='ignore')
    message = data[9:57].rstrip(b'\\x00').decode('utf-8', errors='ignore')
    timestamp = struct.unpack('<I', data[57:61])[0]
    
    return {{
        'timestamp': timestamp,
        'level': level,
        'module': module,
        'message': message
    }}

def extract_timing_info(message):
    # Look for timing-related information
    patterns = [
        (r'frequency[:\s]*([\\d.]+)\\s*Hz', 'frequency'),
        (r'HIGH[^:]*:[\\s]*([\\d.]+)\\s*ms', 'high_ms'),
        (r'LOW[^:]*:[\\s]*([\\d.]+)\\s*ms', 'low_ms'),
        (r'deviation[^:]*:[\\s]*([\\d.]+)\\s*ms', 'deviation_ms'),
        (r'accuracy[^:]*:[\\s]*([\\d.]+)\\s*%', 'accuracy_percent'),
    ]
    
    extracted = {{}}
    for pattern, key in patterns:
        match = re.search(pattern, message, re.IGNORECASE)
        if match:
            try:
                extracted[key] = float(match.group(1))
            except ValueError:
                pass
    
    return extracted

try:
    device = hid.device()
    device.open(0x{TEST_DEVICE_VID:04x}, 0x{TEST_DEVICE_PID:04x})
    
    pemf_messages = []
    timing_data = defaultdict(list)
    start_time = time.time()
    
    print(f"Capturing pEMF timing data for {duration} seconds...")
    
    while time.time() - start_time < {duration}:
        data = device.read(64, timeout_ms=1000)
        if data:
            msg = parse_log_message(bytes(data))
            if msg and 'PEMF' in msg.get('module', '').upper():
                pemf_messages.append(msg)
                
                # Extract timing information
                timing_info = extract_timing_info(msg['message'])
                for key, value in timing_info.items():
                    timing_data[key].append(value)
                
                if len(pemf_messages) <= 10:  # Show first 10 pEMF messages
                    print(f"[{{msg['timestamp']:08d}}] {{msg['message']}}")
    
    device.close()
    
    print(f"\\nCaptured {{len(pemf_messages)}} pEMF messages")
    
    # Analyze timing data
    if timing_data['frequency']:
        avg_freq = sum(timing_data['frequency']) / len(timing_data['frequency'])
        freq_error = abs(avg_freq - 2.0) / 2.0 * 100
        print(f"Average frequency: {{avg_freq:.3f}}Hz (error: {{freq_error:.2f}}%)")
        
        if freq_error <= 1.0:  # Within 1% tolerance
            print("✓ Frequency accuracy within tolerance")
        else:
            print("✗ Frequency accuracy outside tolerance")
    
    if timing_data['accuracy_percent']:
        avg_accuracy = sum(timing_data['accuracy_percent']) / len(timing_data['accuracy_percent'])
        print(f"Average timing accuracy: {{avg_accuracy:.2f}}%")
    
    if len(pemf_messages) >= 10:
        print("SUCCESS: Sufficient timing data captured")
        exit(0)
    else:
        print("WARNING: Limited timing data captured")
        exit(1)
        
except Exception as e:
    print(f"Timing validation test failed: {{e}}")
    exit(1)
"""
        
        success, stdout, stderr = self.run_command([
            "python3", "-c", timing_test_script
        ], timeout=duration + 20)
        
        if success:
            self.log("✓ pEMF timing validation test passed")
        else:
            self.log("✗ pEMF timing validation test failed", "ERROR")
            self.log(f"Error: {stderr}", "ERROR")
        
        if self.verbose:
            for line in stdout.split('\n'):
                if line.strip():
                    self.log(f"  {line}", "DEBUG")
        
        self.test_results['timing_validation'] = {
            'passed': success,
            'output': stdout,
            'error': stderr
        }
        
        return success
    
    def run_battery_monitoring_test(self, duration: int = 20) -> bool:
        """Test battery monitoring with actual ADC readings"""
        self.log(f"Testing battery monitoring ({duration}s)...")
        
        battery_test_script = f"""
import hid
import time
import struct
import re

def parse_log_message(data):
    if len(data) < 64:
        return None
    
    level = data[0]
    module = data[1:9].rstrip(b'\\x00').decode('utf-8', errors='ignore')
    message = data[9:57].rstrip(b'\\x00').decode('utf-8', errors='ignore')
    timestamp = struct.unpack('<I', data[57:61])[0]
    
    return {{
        'timestamp': timestamp,
        'level': level,
        'module': module,
        'message': message
    }}

def extract_battery_data(message):
    adc_match = re.search(r'ADC[:\\s]*([\\d]+)', message)
    voltage_match = re.search(r'([\\d]+)\\s*mV', message)
    state_match = re.search(r'state[:\\s]*(\\w+)', message, re.IGNORECASE)
    
    return {{
        'adc': int(adc_match.group(1)) if adc_match else None,
        'voltage_mv': int(voltage_match.group(1)) if voltage_match else None,
        'state': state_match.group(1) if state_match else None
    }}

try:
    device = hid.device()
    device.open(0x{TEST_DEVICE_VID:04x}, 0x{TEST_DEVICE_PID:04x})
    
    battery_messages = []
    adc_readings = []
    voltage_readings = []
    states = []
    start_time = time.time()
    
    print(f"Capturing battery data for {duration} seconds...")
    
    while time.time() - start_time < {duration}:
        data = device.read(64, timeout_ms=1000)
        if data:
            msg = parse_log_message(bytes(data))
            if msg and 'BATTERY' in msg.get('module', '').upper():
                battery_messages.append(msg)
                
                battery_data = extract_battery_data(msg['message'])
                if battery_data['adc'] is not None:
                    adc_readings.append(battery_data['adc'])
                if battery_data['voltage_mv'] is not None:
                    voltage_readings.append(battery_data['voltage_mv'])
                if battery_data['state'] is not None:
                    states.append(battery_data['state'])
                
                if len(battery_messages) <= 10:  # Show first 10 battery messages
                    print(f"[{{msg['timestamp']:08d}}] {{msg['message']}}")
    
    device.close()
    
    print(f"\\nCaptured {{len(battery_messages)}} battery messages")
    print(f"ADC readings: {{len(adc_readings)}} samples")
    print(f"Voltage readings: {{len(voltage_readings)}} samples")
    print(f"State readings: {{len(states)}} samples")
    
    if adc_readings:
        print(f"ADC range: {{min(adc_readings)}} - {{max(adc_readings)}}")
        avg_adc = sum(adc_readings) / len(adc_readings)
        print(f"Average ADC: {{avg_adc:.0f}}")
    
    if voltage_readings:
        print(f"Voltage range: {{min(voltage_readings)}} - {{max(voltage_readings)}}mV")
        avg_voltage = sum(voltage_readings) / len(voltage_readings)
        print(f"Average voltage: {{avg_voltage:.0f}}mV")
    
    if states:
        unique_states = set(states)
        print(f"Battery states detected: {{', '.join(unique_states)}}")
    
    if len(battery_messages) >= 5:
        print("SUCCESS: Sufficient battery data captured")
        exit(0)
    else:
        print("WARNING: Limited battery data captured")
        exit(1)
        
except Exception as e:
    print(f"Battery monitoring test failed: {{e}}")
    exit(1)
"""
        
        success, stdout, stderr = self.run_command([
            "python3", "-c", battery_test_script
        ], timeout=duration + 15)
        
        if success:
            self.log("✓ Battery monitoring test passed")
        else:
            self.log("✗ Battery monitoring test failed", "ERROR")
            self.log(f"Error: {stderr}", "ERROR")
        
        if self.verbose:
            for line in stdout.split('\n'):
                if line.strip():
                    self.log(f"  {line}", "DEBUG")
        
        self.test_results['battery_monitoring'] = {
            'passed': success,
            'output': stdout,
            'error': stderr
        }
        
        return success
    
    def generate_test_report(self) -> None:
        """Generate a comprehensive test report"""
        end_time = datetime.now()
        duration = end_time - self.start_time
        
        print("\n" + "="*60)
        print("HARDWARE VALIDATION TEST REPORT")
        print("="*60)
        print(f"Test started: {self.start_time.strftime('%Y-%m-%d %H:%M:%S')}")
        print(f"Test completed: {end_time.strftime('%Y-%m-%d %H:%M:%S')}")
        print(f"Total duration: {duration.total_seconds():.1f} seconds")
        print(f"Device: {TEST_DEVICE_VID:04x}:{TEST_DEVICE_PID:04x}")
        print()
        
        # Test results summary
        total_tests = len(self.test_results)
        passed_tests = sum(1 for result in self.test_results.values() if result['passed'])
        
        print("TEST RESULTS SUMMARY:")
        print(f"  Total tests: {total_tests}")
        print(f"  Passed: {passed_tests} ✓")
        print(f"  Failed: {total_tests - passed_tests} ✗")
        print()
        
        # Detailed results
        print("DETAILED RESULTS:")
        for test_name, result in self.test_results.items():
            status = "✓ PASS" if result['passed'] else "✗ FAIL"
            print(f"  {status} {test_name.replace('_', ' ').title()}")
        print()
        
        # Overall assessment
        if passed_tests == total_tests:
            print("🎉 OVERALL RESULT: ALL TESTS PASSED")
            print("The RP2040 device is functioning correctly with USB HID logging.")
            print("Hardware validation completed successfully.")
        else:
            print("⚠️  OVERALL RESULT: SOME TESTS FAILED")
            print(f"{total_tests - passed_tests} test(s) failed. Please review the errors above.")
            print()
            print("TROUBLESHOOTING STEPS:")
            print("1. Ensure RP2040 device is connected via USB")
            print("2. Verify device is running USB HID logging firmware (not bootloader)")
            print("3. Check that all required Python packages are installed")
            print("4. Try disconnecting and reconnecting the device")
            print("5. Review the detailed error messages above")
        
        print()
        
        # Save detailed report to file
        report_file = f"hardware_validation_report_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
        with open(report_file, 'w') as f:
            json.dump({
                'start_time': self.start_time.isoformat(),
                'end_time': end_time.isoformat(),
                'duration_seconds': duration.total_seconds(),
                'device_vid_pid': f"{TEST_DEVICE_VID:04x}:{TEST_DEVICE_PID:04x}",
                'test_results': self.test_results,
                'summary': {
                    'total_tests': total_tests,
                    'passed_tests': passed_tests,
                    'failed_tests': total_tests - passed_tests,
                    'overall_success': passed_tests == total_tests
                }
            }, f, indent=2)
        
        print(f"Detailed report saved to: {report_file}")

def main():
    parser = argparse.ArgumentParser(
        description="Hardware Validation Test Runner for RP2040 pEMF/Battery Monitor Device",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s --all                    # Run all hardware validation tests
  %(prog)s --quick                  # Run quick validation tests only
  %(prog)s --rust-only              # Run only Rust tests
  %(prog)s --communication-test     # Test USB HID communication only
  %(prog)s --timing-test            # Test pEMF timing accuracy only
  %(prog)s --battery-test           # Test battery monitoring only
  %(prog)s --duration 60            # Set test duration to 60 seconds
  %(prog)s --verbose --all          # Run all tests with verbose output
        """
    )
    
    # Test selection options
    parser.add_argument("--all", action="store_true", 
                       help="Run all hardware validation tests")
    parser.add_argument("--quick", action="store_true",
                       help="Run quick validation tests (shorter duration)")
    parser.add_argument("--rust-only", action="store_true",
                       help="Run only Rust hardware tests")
    parser.add_argument("--communication-test", action="store_true",
                       help="Test USB HID communication only")
    parser.add_argument("--timing-test", action="store_true",
                       help="Test pEMF timing accuracy only")
    parser.add_argument("--battery-test", action="store_true",
                       help="Test battery monitoring only")
    
    # Configuration options
    parser.add_argument("--duration", type=int, default=DEFAULT_TEST_DURATION,
                       help=f"Test duration in seconds (default: {DEFAULT_TEST_DURATION})")
    parser.add_argument("--verbose", "-v", action="store_true",
                       help="Enable verbose output")
    parser.add_argument("--no-prereq-check", action="store_true",
                       help="Skip prerequisite checks")
    
    args = parser.parse_args()
    
    # Validate arguments
    test_options = [args.all, args.quick, args.rust_only, args.communication_test, 
                   args.timing_test, args.battery_test]
    if not any(test_options):
        parser.print_help()
        return 1
    
    # Initialize test runner
    runner = HardwareTestRunner(verbose=args.verbose)
    
    runner.log("Starting RP2040 Hardware Validation Test Suite")
    runner.log(f"Device: {TEST_DEVICE_VID:04x}:{TEST_DEVICE_PID:04x}")
    runner.log(f"Test duration: {args.duration}s")
    
    # Check prerequisites
    if not args.no_prereq_check:
        if not runner.check_prerequisites():
            return 1
    
    # Check device connection
    if not runner.check_device_connection():
        runner.log("Device connection failed. Cannot proceed with hardware tests.", "ERROR")
        runner.log("", "INFO")
        runner.log("TROUBLESHOOTING STEPS:", "INFO")
        runner.log("1. Connect RP2040 device via USB cable", "INFO")
        runner.log("2. Ensure device is NOT in bootloader mode", "INFO")
        runner.log("3. Verify firmware includes USB HID logging functionality", "INFO")
        runner.log(f"4. Check device appears in: lsusb | grep {TEST_DEVICE_VID:04x}:{TEST_DEVICE_PID:04x}", "INFO")
        runner.log("5. Try disconnecting and reconnecting the device", "INFO")
        return 1
    
    # Run selected tests
    all_passed = True
    
    if args.rust_only or args.all:
        all_passed &= runner.run_rust_tests()
    
    if args.communication_test or args.all or args.quick:
        duration = 10 if args.quick else min(args.duration, 15)
        all_passed &= runner.run_usb_communication_test(duration)
    
    if args.timing_test or args.all:
        duration = 15 if args.quick else args.duration
        all_passed &= runner.run_timing_validation_test(duration)
    
    if args.battery_test or args.all:
        duration = 10 if args.quick else min(args.duration, 20)
        all_passed &= runner.run_battery_monitoring_test(duration)
    
    # Generate test report
    runner.generate_test_report()
    
    return 0 if all_passed else 1

if __name__ == "__main__":
    sys.exit(main())