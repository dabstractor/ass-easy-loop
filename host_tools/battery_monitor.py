#!/usr/bin/env python3
"""
Battery Monitoring and Validation Tool

Continuous monitoring and validation of battery charging system
Integrates with existing USB HID logging infrastructure
"""

try:
    import hid
except ImportError:
    print("hidapi library not found. Please install it with: pip install hidapi")
    exit(1)

import argparse
import time
import json
import csv
import struct
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass, asdict
from datetime import datetime, timedelta
import threading
import queue
import statistics

# Use same VID/PID as main device
VENDOR_ID = 0xfade
PRODUCT_ID = 0x1212

@dataclass
class BatteryReading:
    timestamp: float
    adc_value: int
    voltage_mv: int
    battery_state: str
    charge_phase: str
    current_ma: int
    temperature_c: float
    
@dataclass
class ValidationResult:
    test_name: str
    passed: bool
    timestamp: float
    error_message: Optional[str] = None
    data: Optional[Dict] = None

class BatteryValidator:
    """Real-time battery system validator"""
    
    def __init__(self):
        self.readings: List[BatteryReading] = []
        self.validation_results: List[ValidationResult] = []
        self.safety_limits = {
            'max_voltage_mv': 4250,      # 4.25V absolute maximum
            'min_voltage_mv': 2900,      # 2.9V minimum for safety
            'max_charge_current_ma': 1100,  # 1.1A maximum
            'max_temperature_c': 50,     # 50°C thermal limit
            'voltage_change_rate_mv_per_sec': 50,  # Max 50mV/s change
        }
        self.timing_validator = TimingValidator()
        
    def add_reading(self, reading: BatteryReading) -> List[ValidationResult]:
        """Add new reading and perform validations"""
        self.readings.append(reading)
        results = []
        
        # Perform all validation checks
        results.extend(self._validate_safety_limits(reading))
        results.extend(self._validate_state_transitions(reading))
        results.extend(self._validate_voltage_consistency(reading))
        results.extend(self._validate_charge_progression(reading))
        
        # Keep only last 1000 readings for memory management
        if len(self.readings) > 1000:
            self.readings = self.readings[-1000:]
            
        self.validation_results.extend(results)
        return results
    
    def _validate_safety_limits(self, reading: BatteryReading) -> List[ValidationResult]:
        """Validate critical safety limits"""
        results = []
        
        # Over-voltage protection
        if reading.voltage_mv > self.safety_limits['max_voltage_mv']:
            results.append(ValidationResult(
                test_name="Over-voltage Protection",
                passed=False,
                timestamp=reading.timestamp,
                error_message=f"Voltage {reading.voltage_mv}mV exceeds limit {self.safety_limits['max_voltage_mv']}mV",
                data={'voltage_mv': reading.voltage_mv, 'limit_mv': self.safety_limits['max_voltage_mv']}
            ))
        
        # Under-voltage protection  
        if reading.voltage_mv < self.safety_limits['min_voltage_mv']:
            results.append(ValidationResult(
                test_name="Under-voltage Protection",
                passed=False,
                timestamp=reading.timestamp,
                error_message=f"Voltage {reading.voltage_mv}mV below limit {self.safety_limits['min_voltage_mv']}mV",
                data={'voltage_mv': reading.voltage_mv, 'limit_mv': self.safety_limits['min_voltage_mv']}
            ))
            
        # Over-current protection
        if abs(reading.current_ma) > self.safety_limits['max_charge_current_ma']:
            results.append(ValidationResult(
                test_name="Over-current Protection", 
                passed=False,
                timestamp=reading.timestamp,
                error_message=f"Current {reading.current_ma}mA exceeds limit {self.safety_limits['max_charge_current_ma']}mA",
                data={'current_ma': reading.current_ma, 'limit_ma': self.safety_limits['max_charge_current_ma']}
            ))
            
        # Thermal protection
        if reading.temperature_c > self.safety_limits['max_temperature_c']:
            results.append(ValidationResult(
                test_name="Thermal Protection",
                passed=False,
                timestamp=reading.timestamp,
                error_message=f"Temperature {reading.temperature_c}°C exceeds limit {self.safety_limits['max_temperature_c']}°C",
                data={'temperature_c': reading.temperature_c, 'limit_c': self.safety_limits['max_temperature_c']}
            ))
            
        return results
    
    def _validate_state_transitions(self, reading: BatteryReading) -> List[ValidationResult]:
        """Validate battery state transitions are logical"""
        results = []
        
        if len(self.readings) < 2:
            return results
            
        prev_reading = self.readings[-2]
        
        # Define valid state transitions
        valid_transitions = {
            'Low': ['Normal', 'Low'],
            'Normal': ['Charging', 'Low', 'Normal'],  
            'Charging': ['Full', 'Normal', 'Charging'],
            'Full': ['Normal', 'Full'],
            'Fault': [],  # Fault state should not transition automatically
        }
        
        if prev_reading.battery_state != reading.battery_state:
            valid_next_states = valid_transitions.get(prev_reading.battery_state, [])
            if reading.battery_state not in valid_next_states:
                results.append(ValidationResult(
                    test_name="State Transition Validation",
                    passed=False,
                    timestamp=reading.timestamp,
                    error_message=f"Invalid state transition: {prev_reading.battery_state} -> {reading.battery_state}",
                    data={
                        'prev_state': prev_reading.battery_state,
                        'curr_state': reading.battery_state,
                        'valid_transitions': valid_next_states
                    }
                ))
        
        return results
    
    def _validate_voltage_consistency(self, reading: BatteryReading) -> List[ValidationResult]:
        """Validate voltage readings are consistent with ADC values"""
        results = []
        
        # ADC to voltage conversion: (adc_value * 3300 / 4095) / 0.337
        expected_voltage = int((reading.adc_value * 3300 / 4095) / 0.337)
        voltage_error = abs(reading.voltage_mv - expected_voltage)
        
        # Allow 5% tolerance for ADC conversion
        tolerance = expected_voltage * 0.05
        
        if voltage_error > tolerance:
            results.append(ValidationResult(
                test_name="Voltage Consistency",
                passed=False,
                timestamp=reading.timestamp,
                error_message=f"Voltage inconsistency: ADC {reading.adc_value} -> expected {expected_voltage}mV, got {reading.voltage_mv}mV",
                data={
                    'adc_value': reading.adc_value,
                    'expected_voltage_mv': expected_voltage,
                    'actual_voltage_mv': reading.voltage_mv,
                    'error_mv': voltage_error,
                    'tolerance_mv': tolerance
                }
            ))
        
        return results
    
    def _validate_charge_progression(self, reading: BatteryReading) -> List[ValidationResult]:
        """Validate charge progression is reasonable"""
        results = []
        
        if len(self.readings) < 10:  # Need some history
            return results
            
        # Look at voltage trend over last 10 readings
        recent_readings = self.readings[-10:]
        voltages = [r.voltage_mv for r in recent_readings]
        time_span = recent_readings[-1].timestamp - recent_readings[0].timestamp
        
        if time_span > 0:
            voltage_rate = (voltages[-1] - voltages[0]) / time_span  # mV per second
            
            # During charging, voltage should increase (positive rate)
            # But not too fast (safety limit)
            if reading.battery_state == 'Charging':
                if voltage_rate < 0:
                    results.append(ValidationResult(
                        test_name="Charge Progression",
                        passed=False,
                        timestamp=reading.timestamp,
                        error_message=f"Voltage decreasing during charging: {voltage_rate:.2f} mV/s",
                        data={'voltage_rate_mv_per_sec': voltage_rate}
                    ))
                elif voltage_rate > self.safety_limits['voltage_change_rate_mv_per_sec']:
                    results.append(ValidationResult(
                        test_name="Charge Progression",
                        passed=False,
                        timestamp=reading.timestamp,
                        error_message=f"Voltage rising too fast: {voltage_rate:.2f} mV/s",
                        data={'voltage_rate_mv_per_sec': voltage_rate, 'limit': self.safety_limits['voltage_change_rate_mv_per_sec']}
                    ))
        
        return results
    
    def get_statistics(self) -> Dict:
        """Get validation statistics"""
        if not self.validation_results:
            return {}
            
        total_validations = len(self.validation_results)
        failed_validations = len([r for r in self.validation_results if not r.passed])
        
        # Group by test type
        test_stats = {}
        for result in self.validation_results:
            if result.test_name not in test_stats:
                test_stats[result.test_name] = {'total': 0, 'failed': 0}
            test_stats[result.test_name]['total'] += 1
            if not result.passed:
                test_stats[result.test_name]['failed'] += 1
        
        return {
            'total_validations': total_validations,
            'failed_validations': failed_validations, 
            'success_rate': (total_validations - failed_validations) / total_validations * 100,
            'test_statistics': test_stats,
            'total_readings': len(self.readings)
        }

class TimingValidator:
    """Validates pEMF timing accuracy during charging"""
    
    def __init__(self):
        self.timing_measurements = []
        self.target_frequency_hz = 1000  # Example target frequency
        self.tolerance_percent = 1.0     # ±1% tolerance requirement
        
    def add_timing_measurement(self, timestamp: float, measured_frequency_hz: float, charge_phase: str) -> ValidationResult:
        """Add timing measurement and validate"""
        timing_error_percent = ((measured_frequency_hz - self.target_frequency_hz) / self.target_frequency_hz) * 100
        
        measurement = {
            'timestamp': timestamp,
            'measured_frequency_hz': measured_frequency_hz,
            'timing_error_percent': timing_error_percent,
            'charge_phase': charge_phase
        }
        
        self.timing_measurements.append(measurement)
        
        # Validate timing is within tolerance
        if abs(timing_error_percent) <= self.tolerance_percent:
            return ValidationResult(
                test_name="pEMF Timing Accuracy",
                passed=True,
                timestamp=timestamp,
                data=measurement
            )
        else:
            return ValidationResult(
                test_name="pEMF Timing Accuracy", 
                passed=False,
                timestamp=timestamp,
                error_message=f"Timing error {timing_error_percent:.2f}% exceeds tolerance ±{self.tolerance_percent}%",
                data=measurement
            )

class BatteryMonitor:
    """Main battery monitoring application"""
    
    def __init__(self, config_file: Optional[str] = None):
        self.validator = BatteryValidator()
        self.device = None
        self.monitoring = False
        self.data_queue = queue.Queue()
        self.log_file = None
        self.csv_writer = None
        
        if config_file:
            self.load_config(config_file)
    
    def load_config(self, config_file: str):
        """Load monitoring configuration"""
        try:
            with open(config_file, 'r') as f:
                config = json.load(f)
            
            # Update safety limits if provided
            if 'safety_limits' in config:
                self.validator.safety_limits.update(config['safety_limits'])
                
            # Update timing parameters if provided
            if 'timing' in config:
                if 'target_frequency_hz' in config['timing']:
                    self.validator.timing_validator.target_frequency_hz = config['timing']['target_frequency_hz']
                if 'tolerance_percent' in config['timing']:
                    self.validator.timing_validator.tolerance_percent = config['timing']['tolerance_percent']
                    
        except Exception as e:
            print(f"Warning: Could not load config file {config_file}: {e}")
    
    def connect(self) -> bool:
        """Connect to the battery monitoring device"""
        try:
            self.device = hid.Device(VENDOR_ID, PRODUCT_ID)
            self.device.nonblocking = True
            return True
        except Exception as e:
            print(f"Failed to connect to device: {e}")
            return False
    
    def disconnect(self):
        """Disconnect from device"""
        if self.device:
            self.device.close()
            self.device = None
    
    def start_monitoring(self, duration_minutes: Optional[int] = None, log_file: Optional[str] = None):
        """Start continuous battery monitoring"""
        if not self.device:
            if not self.connect():
                return False
        
        self.monitoring = True
        
        # Setup logging if requested
        if log_file:
            self.log_file = open(log_file, 'w', newline='')
            self.csv_writer = csv.DictWriter(self.log_file, fieldnames=[
                'timestamp', 'adc_value', 'voltage_mv', 'battery_state', 
                'charge_phase', 'current_ma', 'temperature_c'
            ])
            self.csv_writer.writeheader()
        
        # Start monitoring thread
        monitor_thread = threading.Thread(target=self._monitoring_loop, daemon=True)
        monitor_thread.start()
        
        # Start validation thread  
        validation_thread = threading.Thread(target=self._validation_loop, daemon=True)
        validation_thread.start()
        
        try:
            start_time = time.time()
            
            print("Battery monitoring started... Press Ctrl+C to stop")
            print("=" * 60)
            
            while self.monitoring:
                # Print periodic statistics
                time.sleep(10)  # Update every 10 seconds
                
                stats = self.validator.get_statistics()
                if stats:
                    print(f"\nStatistics (Runtime: {time.time() - start_time:.1f}s):")
                    print(f"  Total readings: {stats['total_readings']}")
                    print(f"  Validations: {stats['total_validations']} (Success rate: {stats['success_rate']:.1f}%)")
                    if stats['failed_validations'] > 0:
                        print(f"  ⚠️  Failed validations: {stats['failed_validations']}")
                
                # Check duration limit
                if duration_minutes and (time.time() - start_time) > duration_minutes * 60:
                    print(f"\nMonitoring duration ({duration_minutes} minutes) completed.")
                    break
                    
        except KeyboardInterrupt:
            print("\nStopping battery monitoring...")
        finally:
            self.stop_monitoring()
        
        return True
    
    def stop_monitoring(self):
        """Stop monitoring"""
        self.monitoring = False
        
        if self.csv_writer:
            self.csv_writer = None
        if self.log_file:
            self.log_file.close()
            self.log_file = None
        
        self.disconnect()
    
    def _monitoring_loop(self):
        """Main monitoring loop - reads from device"""
        while self.monitoring and self.device:
            try:
                data = self.device.read(64, timeout_ms=100)
                if data and len(data) == 64:
                    reading = self._parse_battery_data(bytes(data))
                    if reading:
                        self.data_queue.put(reading)
                        
                time.sleep(0.01)  # Small delay to prevent excessive CPU usage
                
            except Exception as e:
                print(f"Error reading from device: {e}")
                time.sleep(0.1)
    
    def _validation_loop(self):
        """Validation loop - processes readings and performs validations"""
        while self.monitoring:
            try:
                reading = self.data_queue.get(timeout=1.0)
                
                # Log to CSV if enabled
                if self.csv_writer:
                    self.csv_writer.writerow(asdict(reading))
                    self.log_file.flush()
                
                # Perform validations
                validation_results = self.validator.add_reading(reading)
                
                # Print any failed validations immediately
                for result in validation_results:
                    if not result.passed:
                        timestamp_str = datetime.fromtimestamp(result.timestamp).strftime('%H:%M:%S')
                        print(f"🚨 VALIDATION FAILURE [{timestamp_str}] {result.test_name}: {result.error_message}")
                
            except queue.Empty:
                continue
            except Exception as e:
                print(f"Error in validation loop: {e}")
    
    def _parse_battery_data(self, data: bytes) -> Optional[BatteryReading]:
        """Parse battery data from HID report"""
        try:
            # This is a placeholder - actual implementation would parse 
            # the specific HID report format used by your device
            
            # Example parsing (adjust based on actual report format):
            timestamp = time.time()
            adc_value = struct.unpack('<H', data[0:2])[0]  # 16-bit ADC value
            voltage_mv = int((adc_value * 3300 / 4095) / 0.337)  # Convert to voltage
            
            # Determine battery state from voltage
            if voltage_mv < 3100:
                battery_state = "Low"
            elif voltage_mv < 3600:
                battery_state = "Normal" 
            elif voltage_mv < 4200:
                battery_state = "Charging"
            else:
                battery_state = "Full"
            
            # Placeholder values - replace with actual parsing
            charge_phase = "Unknown"
            current_ma = 0
            temperature_c = 25.0
            
            return BatteryReading(
                timestamp=timestamp,
                adc_value=adc_value,
                voltage_mv=voltage_mv,
                battery_state=battery_state,
                charge_phase=charge_phase,
                current_ma=current_ma,
                temperature_c=temperature_c
            )
            
        except Exception as e:
            print(f"Error parsing battery data: {e}")
            return None

def main():
    parser = argparse.ArgumentParser(description="Battery Monitoring and Validation Tool")
    parser.add_argument('-d', '--duration', type=int, help='Monitoring duration in minutes')
    parser.add_argument('-l', '--log', type=str, help='CSV log file path')
    parser.add_argument('-c', '--config', type=str, help='Configuration file path')
    parser.add_argument('--test-connection', action='store_true', help='Test device connection')
    parser.add_argument('--generate-config', type=str, help='Generate sample config file')
    
    args = parser.parse_args()
    
    if args.generate_config:
        generate_sample_config(args.generate_config)
        return
    
    monitor = BatteryMonitor(args.config)
    
    if args.test_connection:
        if monitor.connect():
            print("✅ Device connection successful")
            monitor.disconnect()
        else:
            print("❌ Device connection failed")
        return
    
    # Start monitoring
    monitor.start_monitoring(args.duration, args.log)

def generate_sample_config(filename: str):
    """Generate sample configuration file"""
    config = {
        "safety_limits": {
            "max_voltage_mv": 4250,
            "min_voltage_mv": 2900,
            "max_charge_current_ma": 1100,
            "max_temperature_c": 50,
            "voltage_change_rate_mv_per_sec": 50
        },
        "timing": {
            "target_frequency_hz": 1000,
            "tolerance_percent": 1.0
        },
        "monitoring": {
            "update_interval_seconds": 10,
            "history_length": 1000
        }
    }
    
    with open(filename, 'w') as f:
        json.dump(config, f, indent=2)
    
    print(f"Sample configuration generated: {filename}")

if __name__ == "__main__":
    main()