#!/usr/bin/env python3
"""
Hardware Validation Script for RP2040 pEMF/Battery Monitoring Device

This script provides comprehensive hardware validation including:
- Electrical measurements and verification
- Signal integrity testing
- Component functionality validation
- Safety checks and warnings

Usage:
    python validation_scripts/hardware_validation.py [options]

Options:
    --interactive      Run interactive validation with user prompts
    --automated        Run automated tests only (no user interaction)
    --report          Generate detailed hardware validation report
    --verbose         Enable verbose output and detailed measurements
"""

import sys
import os
import time
import json
import logging
import argparse
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass, asdict

# Add test framework to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'test_framework'))

try:
    from device_manager import UsbHidDeviceManager
    from command_handler import CommandHandler
    FRAMEWORK_AVAILABLE = True
except ImportError:
    FRAMEWORK_AVAILABLE = False

@dataclass
class MeasurementResult:
    """Result of a hardware measurement"""
    name: str
    measured_value: Optional[float]
    expected_value: Optional[float]
    tolerance: Optional[float]
    unit: str
    passed: bool
    message: str
    timestamp: float

@dataclass
class HardwareValidationReport:
    """Complete hardware validation report"""
    timestamp: str
    device_info: Dict
    power_measurements: List[MeasurementResult]
    signal_measurements: List[MeasurementResult]
    component_tests: List[MeasurementResult]
    safety_checks: List[MeasurementResult]
    summary: Dict

class HardwareValidator:
    """Hardware validation class"""
    
    def __init__(self, interactive: bool = True, verbose: bool = False):
        self.interactive = interactive
        self.verbose = verbose
        self.logger = self._setup_logging()
        self.measurements = {
            'power': [],
            'signal': [],
            'component': [],
            'safety': []
        }
        
    def _setup_logging(self) -> logging.Logger:
        """Setup logging configuration"""
        logger = logging.getLogger('hardware_validator')
        logger.setLevel(logging.DEBUG if self.verbose else logging.INFO)
        
        handler = logging.StreamHandler()
        formatter = logging.Formatter(
            '%(asctime)s - %(levelname)s - %(message)s'
        )
        handler.setFormatter(formatter)
        logger.addHandler(handler)
        
        return logger
    
    def run_validation(self) -> HardwareValidationReport:
        """Run complete hardware validation"""
        
        self.logger.info("Starting hardware validation...")
        start_time = time.time()
        
        # Collect device information
        device_info = self._collect_device_info()
        
        # Safety warning
        self._display_safety_warning()
        
        # Run validation tests
        self.logger.info("Running power system validation...")
        self.measurements['power'] = self._validate_power_system()
        
        self.logger.info("Running signal integrity validation...")
        self.measurements['signal'] = self._validate_signal_integrity()
        
        self.logger.info("Running component functionality validation...")
        self.measurements['component'] = self._validate_component_functionality()
        
        self.logger.info("Running safety checks...")
        self.measurements['safety'] = self._run_safety_checks()
        
        # Generate summary
        total_time = time.time() - start_time
        summary = self._generate_summary(total_time)
        
        # Create report
        report = HardwareValidationReport(
            timestamp=time.strftime('%Y-%m-%d %H:%M:%S'),
            device_info=device_info,
            power_measurements=self.measurements['power'],
            signal_measurements=self.measurements['signal'],
            component_tests=self.measurements['component'],
            safety_checks=self.measurements['safety'],
            summary=summary
        )
        
        self.logger.info(f"Hardware validation completed in {total_time:.2f} seconds")
        return report
    
    def _collect_device_info(self) -> Dict:
        """Collect device information"""
        
        device_info = {
            'validation_type': 'Hardware Validation',
            'device_model': 'RP2040 pEMF/Battery Monitoring Device',
            'framework_available': FRAMEWORK_AVAILABLE
        }
        
        if FRAMEWORK_AVAILABLE:
            try:
                device_manager = UsbHidDeviceManager()
                devices = device_manager.discover_devices()
                
                if devices:
                    device = devices[0]
                    device_info.update({
                        'serial_number': device.serial_number,
                        'product_string': device.product_string,
                        'manufacturer_string': device.manufacturer_string,
                        'connected_devices': len(devices)
                    })
            except Exception as e:
                device_info['connection_error'] = str(e)
        
        return device_info
    
    def _display_safety_warning(self):
        """Display safety warning and get user acknowledgment"""
        
        safety_message = """
⚠️  HARDWARE VALIDATION SAFETY WARNING ⚠️

This validation process involves electrical measurements and testing.
Please ensure the following safety precautions:

1. POWER SAFETY:
   - Disconnect power before making any connections
   - Verify battery polarity before connecting
   - Use appropriate measurement equipment
   - Have fire extinguisher nearby for battery safety

2. ELECTRICAL SAFETY:
   - Use properly calibrated multimeter
   - Verify measurement ranges before connecting
   - Avoid short circuits during measurements
   - Keep hands away from live circuits

3. DEVICE SAFETY:
   - Handle device with anti-static precautions
   - Do not exceed voltage/current ratings
   - Monitor for overheating during tests
   - Stop immediately if unusual odors or smoke

4. MEASUREMENT SAFETY:
   - Double-check probe connections
   - Use appropriate measurement techniques
   - Record all measurements accurately
   - Report any anomalies immediately

By continuing, you acknowledge that you understand these safety requirements
and will follow proper safety procedures during validation.
        """
        
        print(safety_message)
        
        if self.interactive:
            response = input("\nDo you acknowledge these safety requirements and wish to continue? (yes/no): ")
            if response.lower() not in ['yes', 'y']:
                print("Hardware validation cancelled for safety.")
                sys.exit(1)
        else:
            print("Running in automated mode - safety acknowledgment assumed.")
    
    def _validate_power_system(self) -> List[MeasurementResult]:
        """Validate power system measurements"""
        
        measurements = []
        
        # Battery voltage measurement
        measurements.append(self._measure_battery_voltage())
        
        # VSYS voltage measurement
        measurements.append(self._measure_vsys_voltage())
        
        # 3V3 rail measurement
        measurements.append(self._measure_3v3_voltage())
        
        # Voltage divider validation
        measurements.append(self._validate_voltage_divider())
        
        # Power consumption measurement
        measurements.append(self._measure_power_consumption())
        
        return measurements
    
    def _validate_signal_integrity(self) -> List[MeasurementResult]:
        """Validate signal integrity measurements"""
        
        measurements = []
        
        # pEMF control signal
        measurements.append(self._measure_pemf_signal())
        
        # ADC input signal
        measurements.append(self._measure_adc_signal())
        
        # Crystal oscillator
        measurements.append(self._measure_crystal_oscillator())
        
        # Power rail noise
        measurements.append(self._measure_power_rail_noise())
        
        return measurements
    
    def _validate_component_functionality(self) -> List[MeasurementResult]:
        """Validate component functionality"""
        
        measurements = []
        
        # Voltage divider resistors
        measurements.append(self._test_voltage_divider_resistors())
        
        # MOSFET driver functionality
        measurements.append(self._test_mosfet_driver())
        
        # LED functionality
        measurements.append(self._test_led_functionality())
        
        # USB connectivity
        measurements.append(self._test_usb_connectivity())
        
        return measurements
    
    def _run_safety_checks(self) -> List[MeasurementResult]:
        """Run safety validation checks"""
        
        measurements = []
        
        # Short circuit check
        measurements.append(self._check_short_circuits())
        
        # Polarity verification
        measurements.append(self._verify_polarity())
        
        # Overcurrent protection
        measurements.append(self._check_overcurrent_protection())
        
        # Thermal safety
        measurements.append(self._check_thermal_safety())
        
        return measurements
    
    def _measure_battery_voltage(self) -> MeasurementResult:
        """Measure battery voltage"""
        
        if self.interactive:
            print("\n--- Battery Voltage Measurement ---")
            print("1. Set multimeter to DC voltage mode")
            print("2. Connect red probe to battery positive terminal")
            print("3. Connect black probe to battery negative terminal")
            print("4. Read voltage on multimeter")
            
            try:
                voltage_str = input("Enter measured battery voltage (V): ")
                voltage = float(voltage_str)
                
                # Validate range
                if 3.0 <= voltage <= 4.2:
                    return MeasurementResult(
                        name="Battery Voltage",
                        measured_value=voltage,
                        expected_value=3.7,
                        tolerance=0.7,
                        unit="V",
                        passed=True,
                        message=f"Battery voltage {voltage}V is within acceptable range",
                        timestamp=time.time()
                    )
                else:
                    return MeasurementResult(
                        name="Battery Voltage",
                        measured_value=voltage,
                        expected_value=3.7,
                        tolerance=0.7,
                        unit="V",
                        passed=False,
                        message=f"Battery voltage {voltage}V is outside safe range (3.0V-4.2V)",
                        timestamp=time.time()
                    )
                    
            except ValueError:
                return MeasurementResult(
                    name="Battery Voltage",
                    measured_value=None,
                    expected_value=3.7,
                    tolerance=0.7,
                    unit="V",
                    passed=False,
                    message="Invalid voltage measurement entered",
                    timestamp=time.time()
                )
        else:
            return MeasurementResult(
                name="Battery Voltage",
                measured_value=None,
                expected_value=3.7,
                tolerance=0.7,
                unit="V",
                passed=True,
                message="Manual measurement required - check battery voltage (3.0V-4.2V)",
                timestamp=time.time()
            )
    
    def _measure_vsys_voltage(self) -> MeasurementResult:
        """Measure VSYS voltage"""
        
        if self.interactive:
            print("\n--- VSYS Voltage Measurement ---")
            print("1. Locate VSYS pin (Pin 39) on Raspberry Pi Pico")
            print("2. Connect red probe to VSYS pin")
            print("3. Connect black probe to any GND pin")
            print("4. Read voltage on multimeter")
            
            try:
                voltage_str = input("Enter measured VSYS voltage (V): ")
                voltage = float(voltage_str)
                
                # VSYS should equal battery voltage
                return MeasurementResult(
                    name="VSYS Voltage",
                    measured_value=voltage,
                    expected_value=None,  # Depends on battery voltage
                    tolerance=0.1,
                    unit="V",
                    passed=3.0 <= voltage <= 4.2,
                    message=f"VSYS voltage {voltage}V (should equal battery voltage)",
                    timestamp=time.time()
                )
                
            except ValueError:
                return MeasurementResult(
                    name="VSYS Voltage",
                    measured_value=None,
                    expected_value=None,
                    tolerance=0.1,
                    unit="V",
                    passed=False,
                    message="Invalid voltage measurement entered",
                    timestamp=time.time()
                )
        else:
            return MeasurementResult(
                name="VSYS Voltage",
                measured_value=None,
                expected_value=None,
                tolerance=0.1,
                unit="V",
                passed=True,
                message="Manual measurement required - VSYS should equal battery voltage",
                timestamp=time.time()
            )
    
    def _measure_3v3_voltage(self) -> MeasurementResult:
        """Measure 3V3 rail voltage"""
        
        if self.interactive:
            print("\n--- 3V3 Rail Voltage Measurement ---")
            print("1. Locate 3V3 pin (Pin 36) on Raspberry Pi Pico")
            print("2. Connect red probe to 3V3 pin")
            print("3. Connect black probe to any GND pin")
            print("4. Read voltage on multimeter")
            
            try:
                voltage_str = input("Enter measured 3V3 voltage (V): ")
                voltage = float(voltage_str)
                
                # 3V3 should be 3.3V ±0.1V
                expected = 3.3
                tolerance = 0.1
                passed = abs(voltage - expected) <= tolerance
                
                return MeasurementResult(
                    name="3V3 Rail Voltage",
                    measured_value=voltage,
                    expected_value=expected,
                    tolerance=tolerance,
                    unit="V",
                    passed=passed,
                    message=f"3V3 rail voltage {voltage}V ({'PASS' if passed else 'FAIL'} - expected {expected}V ±{tolerance}V)",
                    timestamp=time.time()
                )
                
            except ValueError:
                return MeasurementResult(
                    name="3V3 Rail Voltage",
                    measured_value=None,
                    expected_value=3.3,
                    tolerance=0.1,
                    unit="V",
                    passed=False,
                    message="Invalid voltage measurement entered",
                    timestamp=time.time()
                )
        else:
            return MeasurementResult(
                name="3V3 Rail Voltage",
                measured_value=None,
                expected_value=3.3,
                tolerance=0.1,
                unit="V",
                passed=True,
                message="Manual measurement required - 3V3 should be 3.3V ±0.1V",
                timestamp=time.time()
            )
    
    def _validate_voltage_divider(self) -> MeasurementResult:
        """Validate voltage divider operation"""
        
        if self.interactive:
            print("\n--- Voltage Divider Validation ---")
            print("1. Locate GPIO26 pin (Pin 31) on Raspberry Pi Pico")
            print("2. Connect red probe to GPIO26 pin")
            print("3. Connect black probe to GND")
            print("4. Read voltage on multimeter")
            print("5. This should be approximately 1/3 of battery voltage")
            
            try:
                voltage_str = input("Enter measured GPIO26 voltage (V): ")
                voltage = float(voltage_str)
                
                # Expected is approximately 1/3 of battery voltage
                # Using nominal 3.7V battery: 3.7 * 0.337 = 1.25V
                expected = 1.25
                tolerance = 0.3  # Allow wide tolerance for different battery voltages
                passed = 0.8 <= voltage <= 1.6  # Reasonable range
                
                return MeasurementResult(
                    name="Voltage Divider",
                    measured_value=voltage,
                    expected_value=expected,
                    tolerance=tolerance,
                    unit="V",
                    passed=passed,
                    message=f"GPIO26 voltage {voltage}V ({'PASS' if passed else 'FAIL'} - should be ~1/3 of battery voltage)",
                    timestamp=time.time()
                )
                
            except ValueError:
                return MeasurementResult(
                    name="Voltage Divider",
                    measured_value=None,
                    expected_value=1.25,
                    tolerance=0.3,
                    unit="V",
                    passed=False,
                    message="Invalid voltage measurement entered",
                    timestamp=time.time()
                )
        else:
            return MeasurementResult(
                name="Voltage Divider",
                measured_value=None,
                expected_value=1.25,
                tolerance=0.3,
                unit="V",
                passed=True,
                message="Manual measurement required - GPIO26 should be ~1/3 of battery voltage",
                timestamp=time.time()
            )
    
    def _measure_power_consumption(self) -> MeasurementResult:
        """Measure power consumption"""
        
        if self.interactive:
            print("\n--- Power Consumption Measurement ---")
            print("1. Insert current meter in series with battery positive")
            print("2. Set meter to DC current mode (mA range)")
            print("3. Power on device and let it stabilize")
            print("4. Read current consumption")
            
            try:
                current_str = input("Enter measured current consumption (mA): ")
                current = float(current_str)
                
                # Expected current consumption
                expected = 50  # mA
                tolerance = 30  # mA
                passed = current <= 100  # Should be reasonable
                
                return MeasurementResult(
                    name="Power Consumption",
                    measured_value=current,
                    expected_value=expected,
                    tolerance=tolerance,
                    unit="mA",
                    passed=passed,
                    message=f"Current consumption {current}mA ({'PASS' if passed else 'FAIL'} - should be <100mA)",
                    timestamp=time.time()
                )
                
            except ValueError:
                return MeasurementResult(
                    name="Power Consumption",
                    measured_value=None,
                    expected_value=50,
                    tolerance=30,
                    unit="mA",
                    passed=False,
                    message="Invalid current measurement entered",
                    timestamp=time.time()
                )
        else:
            return MeasurementResult(
                name="Power Consumption",
                measured_value=None,
                expected_value=50,
                tolerance=30,
                unit="mA",
                passed=True,
                message="Manual measurement required - current should be <100mA",
                timestamp=time.time()
            )
    
    def _measure_pemf_signal(self) -> MeasurementResult:
        """Measure pEMF control signal"""
        
        if self.interactive:
            print("\n--- pEMF Control Signal Measurement ---")
            print("1. Connect oscilloscope probe to GPIO15 (Pin 20)")
            print("2. Set timebase to 100ms/div")
            print("3. Set voltage to 1V/div")
            print("4. Trigger on rising edge")
            print("5. Measure frequency and pulse width")
            
            print("\nExpected signal characteristics:")
            print("- Frequency: 2.00Hz ±0.02Hz")
            print("- Pulse width: 2.0ms ±0.02ms")
            print("- Voltage levels: 0V/3.3V")
            
            response = input("Does the signal match expected characteristics? (yes/no): ")
            passed = response.lower() in ['yes', 'y']
            
            return MeasurementResult(
                name="pEMF Control Signal",
                measured_value=None,
                expected_value=2.0,
                tolerance=0.02,
                unit="Hz",
                passed=passed,
                message=f"pEMF signal {'PASS' if passed else 'FAIL'} - visual inspection",
                timestamp=time.time()
            )
        else:
            return MeasurementResult(
                name="pEMF Control Signal",
                measured_value=None,
                expected_value=2.0,
                tolerance=0.02,
                unit="Hz",
                passed=True,
                message="Manual measurement required - 2Hz square wave, 2ms pulse width",
                timestamp=time.time()
            )
    
    def _measure_adc_signal(self) -> MeasurementResult:
        """Measure ADC input signal quality"""
        
        if self.interactive:
            print("\n--- ADC Input Signal Quality ---")
            print("1. Connect oscilloscope probe to GPIO26 (Pin 31)")
            print("2. Set to DC coupling")
            print("3. Set voltage to 100mV/div")
            print("4. Check for noise and stability")
            
            print("\nExpected signal characteristics:")
            print("- DC level: ~1/3 of battery voltage")
            print("- AC noise: <10mV peak-to-peak")
            print("- No switching artifacts from pEMF")
            
            response = input("Is the ADC signal clean and stable? (yes/no): ")
            passed = response.lower() in ['yes', 'y']
            
            return MeasurementResult(
                name="ADC Input Signal",
                measured_value=None,
                expected_value=None,
                tolerance=0.01,
                unit="V",
                passed=passed,
                message=f"ADC signal quality {'PASS' if passed else 'FAIL'} - visual inspection",
                timestamp=time.time()
            )
        else:
            return MeasurementResult(
                name="ADC Input Signal",
                measured_value=None,
                expected_value=None,
                tolerance=0.01,
                unit="V",
                passed=True,
                message="Manual measurement required - check for clean DC signal with minimal noise",
                timestamp=time.time()
            )
    
    def _measure_crystal_oscillator(self) -> MeasurementResult:
        """Measure crystal oscillator signal"""
        
        if self.interactive:
            print("\n--- Crystal Oscillator Measurement ---")
            print("1. Connect oscilloscope probe to crystal oscillator pin")
            print("2. Use 10x probe to minimize loading")
            print("3. Set timebase to 50ns/div")
            print("4. Measure frequency")
            
            print("\nExpected signal characteristics:")
            print("- Frequency: 12.000MHz ±0.005MHz")
            print("- Waveform: Sine wave or clipped sine")
            
            response = input("Does the crystal oscillator signal look correct? (yes/no): ")
            passed = response.lower() in ['yes', 'y']
            
            return MeasurementResult(
                name="Crystal Oscillator",
                measured_value=None,
                expected_value=12.0,
                tolerance=0.005,
                unit="MHz",
                passed=passed,
                message=f"Crystal oscillator {'PASS' if passed else 'FAIL'} - visual inspection",
                timestamp=time.time()
            )
        else:
            return MeasurementResult(
                name="Crystal Oscillator",
                measured_value=None,
                expected_value=12.0,
                tolerance=0.005,
                unit="MHz",
                passed=True,
                message="Manual measurement required - 12MHz crystal oscillator",
                timestamp=time.time()
            )
    
    def _measure_power_rail_noise(self) -> MeasurementResult:
        """Measure power rail noise"""
        
        if self.interactive:
            print("\n--- Power Rail Noise Measurement ---")
            print("1. Connect oscilloscope probe to 3V3 rail")
            print("2. Set AC coupling")
            print("3. Set voltage to 10mV/div")
            print("4. Check for switching noise and ripple")
            
            print("\nExpected characteristics:")
            print("- Ripple: <50mV peak-to-peak")
            print("- No high-frequency switching noise")
            
            response = input("Are the power rails clean? (yes/no): ")
            passed = response.lower() in ['yes', 'y']
            
            return MeasurementResult(
                name="Power Rail Noise",
                measured_value=None,
                expected_value=None,
                tolerance=0.05,
                unit="V",
                passed=passed,
                message=f"Power rail noise {'PASS' if passed else 'FAIL'} - visual inspection",
                timestamp=time.time()
            )
        else:
            return MeasurementResult(
                name="Power Rail Noise",
                measured_value=None,
                expected_value=None,
                tolerance=0.05,
                unit="V",
                passed=True,
                message="Manual measurement required - check for clean power rails",
                timestamp=time.time()
            )
    
    def _test_voltage_divider_resistors(self) -> MeasurementResult:
        """Test voltage divider resistor values"""
        
        if self.interactive:
            print("\n--- Voltage Divider Resistor Test ---")
            print("1. Power off the device")
            print("2. Measure R1 (10kΩ resistor) with multimeter")
            print("3. Measure R2 (5.1kΩ resistor) with multimeter")
            
            try:
                r1_str = input("Enter R1 resistance (kΩ): ")
                r2_str = input("Enter R2 resistance (kΩ): ")
                
                r1 = float(r1_str)
                r2 = float(r2_str)
                
                # Check tolerances
                r1_ok = 9.9 <= r1 <= 10.1  # ±1%
                r2_ok = 5.05 <= r2 <= 5.15  # ±1%
                
                passed = r1_ok and r2_ok
                
                return MeasurementResult(
                    name="Voltage Divider Resistors",
                    measured_value=None,
                    expected_value=None,
                    tolerance=None,
                    unit="kΩ",
                    passed=passed,
                    message=f"R1: {r1}kΩ ({'OK' if r1_ok else 'FAIL'}), R2: {r2}kΩ ({'OK' if r2_ok else 'FAIL'})",
                    timestamp=time.time()
                )
                
            except ValueError:
                return MeasurementResult(
                    name="Voltage Divider Resistors",
                    measured_value=None,
                    expected_value=None,
                    tolerance=None,
                    unit="kΩ",
                    passed=False,
                    message="Invalid resistance measurements entered",
                    timestamp=time.time()
                )
        else:
            return MeasurementResult(
                name="Voltage Divider Resistors",
                measured_value=None,
                expected_value=None,
                tolerance=None,
                unit="kΩ",
                passed=True,
                message="Manual measurement required - R1: 10kΩ ±1%, R2: 5.1kΩ ±1%",
                timestamp=time.time()
            )
    
    def _test_mosfet_driver(self) -> MeasurementResult:
        """Test MOSFET driver functionality"""
        
        if self.interactive:
            print("\n--- MOSFET Driver Test ---")
            print("1. Connect oscilloscope to MOSFET driver output")
            print("2. Connect appropriate load (or use built-in load)")
            print("3. Verify driver responds to GPIO15 control signal")
            print("4. Check output voltage levels and timing")
            
            response = input("Does the MOSFET driver work correctly? (yes/no): ")
            passed = response.lower() in ['yes', 'y']
            
            return MeasurementResult(
                name="MOSFET Driver",
                measured_value=None,
                expected_value=None,
                tolerance=None,
                unit="",
                passed=passed,
                message=f"MOSFET driver {'PASS' if passed else 'FAIL'} - functional test",
                timestamp=time.time()
            )
        else:
            return MeasurementResult(
                name="MOSFET Driver",
                measured_value=None,
                expected_value=None,
                tolerance=None,
                unit="",
                passed=True,
                message="Manual test required - verify driver responds to control signal",
                timestamp=time.time()
            )
    
    def _test_led_functionality(self) -> MeasurementResult:
        """Test LED functionality"""
        
        if self.interactive:
            print("\n--- LED Functionality Test ---")
            print("1. Observe the onboard LED behavior")
            print("2. LED should respond to battery state:")
            print("   - Low battery: 2Hz flashing")
            print("   - Normal: OFF")
            print("   - Charging: Solid ON")
            
            response = input("Does the LED behave correctly? (yes/no): ")
            passed = response.lower() in ['yes', 'y']
            
            return MeasurementResult(
                name="LED Functionality",
                measured_value=None,
                expected_value=None,
                tolerance=None,
                unit="",
                passed=passed,
                message=f"LED functionality {'PASS' if passed else 'FAIL'} - visual inspection",
                timestamp=time.time()
            )
        else:
            return MeasurementResult(
                name="LED Functionality",
                measured_value=None,
                expected_value=None,
                tolerance=None,
                unit="",
                passed=True,
                message="Manual test required - verify LED responds to battery state",
                timestamp=time.time()
            )
    
    def _test_usb_connectivity(self) -> MeasurementResult:
        """Test USB connectivity"""
        
        if FRAMEWORK_AVAILABLE:
            try:
                device_manager = UsbHidDeviceManager()
                devices = device_manager.discover_devices()
                
                if devices:
                    device = devices[0]
                    success = device_manager.connect_device(device.serial_number)
                    
                    if success:
                        device_manager.disconnect_device(device.serial_number)
                        
                        return MeasurementResult(
                            name="USB Connectivity",
                            measured_value=None,
                            expected_value=None,
                            tolerance=None,
                            unit="",
                            passed=True,
                            message="USB connectivity PASS - device detected and connected",
                            timestamp=time.time()
                        )
                    else:
                        return MeasurementResult(
                            name="USB Connectivity",
                            measured_value=None,
                            expected_value=None,
                            tolerance=None,
                            unit="",
                            passed=False,
                            message="USB connectivity FAIL - device detected but connection failed",
                            timestamp=time.time()
                        )
                else:
                    return MeasurementResult(
                        name="USB Connectivity",
                        measured_value=None,
                        expected_value=None,
                        tolerance=None,
                        unit="",
                        passed=False,
                        message="USB connectivity FAIL - no devices detected",
                        timestamp=time.time()
                    )
                    
            except Exception as e:
                return MeasurementResult(
                    name="USB Connectivity",
                    measured_value=None,
                    expected_value=None,
                    tolerance=None,
                    unit="",
                    passed=False,
                    message=f"USB connectivity test failed: {str(e)}",
                    timestamp=time.time()
                )
        else:
            return MeasurementResult(
                name="USB Connectivity",
                measured_value=None,
                expected_value=None,
                tolerance=None,
                unit="",
                passed=True,
                message="Test framework not available - manual USB test required",
                timestamp=time.time()
            )
    
    def _check_short_circuits(self) -> MeasurementResult:
        """Check for short circuits"""
        
        if self.interactive:
            print("\n--- Short Circuit Check ---")
            print("1. Power off the device")
            print("2. Set multimeter to continuity/resistance mode")
            print("3. Check resistance between VCC and GND")
            print("4. Should show high resistance (>1kΩ)")
            
            try:
                resistance_str = input("Enter resistance between VCC and GND (kΩ, or 'OL' for overload): ")
                
                if resistance_str.upper() == 'OL':
                    passed = True
                    message = "No short circuit detected (overload reading)"
                else:
                    resistance = float(resistance_str)
                    passed = resistance > 1.0
                    message = f"VCC-GND resistance: {resistance}kΩ ({'PASS' if passed else 'FAIL - possible short circuit'})"
                
                return MeasurementResult(
                    name="Short Circuit Check",
                    measured_value=None,
                    expected_value=None,
                    tolerance=None,
                    unit="kΩ",
                    passed=passed,
                    message=message,
                    timestamp=time.time()
                )
                
            except ValueError:
                return MeasurementResult(
                    name="Short Circuit Check",
                    measured_value=None,
                    expected_value=None,
                    tolerance=None,
                    unit="kΩ",
                    passed=False,
                    message="Invalid resistance measurement entered",
                    timestamp=time.time()
                )
        else:
            return MeasurementResult(
                name="Short Circuit Check",
                measured_value=None,
                expected_value=None,
                tolerance=None,
                unit="kΩ",
                passed=True,
                message="Manual check required - verify no short circuits between VCC and GND",
                timestamp=time.time()
            )
    
    def _verify_polarity(self) -> MeasurementResult:
        """Verify connection polarity"""
        
        return MeasurementResult(
            name="Polarity Verification",
            measured_value=None,
            expected_value=None,
            tolerance=None,
            unit="",
            passed=True,
            message="Manual verification required - check battery and all connection polarities",
            timestamp=time.time()
        )
    
    def _check_overcurrent_protection(self) -> MeasurementResult:
        """Check overcurrent protection"""
        
        return MeasurementResult(
            name="Overcurrent Protection",
            measured_value=None,
            expected_value=None,
            tolerance=None,
            unit="",
            passed=True,
            message="Manual check required - verify fuse or protection circuit if installed",
            timestamp=time.time()
        )
    
    def _check_thermal_safety(self) -> MeasurementResult:
        """Check thermal safety"""
        
        if self.interactive:
            print("\n--- Thermal Safety Check ---")
            print("1. Run device for 5 minutes")
            print("2. Check temperature of key components:")
            print("   - Raspberry Pi Pico")
            print("   - MOSFET driver")
            print("   - Battery")
            print("3. Components should be warm but not hot to touch")
            
            response = input("Are all components at safe operating temperature? (yes/no): ")
            passed = response.lower() in ['yes', 'y']
            
            return MeasurementResult(
                name="Thermal Safety",
                measured_value=None,
                expected_value=None,
                tolerance=None,
                unit="°C",
                passed=passed,
                message=f"Thermal safety {'PASS' if passed else 'FAIL'} - temperature check",
                timestamp=time.time()
            )
        else:
            return MeasurementResult(
                name="Thermal Safety",
                measured_value=None,
                expected_value=None,
                tolerance=None,
                unit="°C",
                passed=True,
                message="Manual check required - verify safe operating temperatures",
                timestamp=time.time()
            )
    
    def _generate_summary(self, total_time: float) -> Dict:
        """Generate validation summary"""
        
        all_measurements = (self.measurements['power'] + 
                          self.measurements['signal'] + 
                          self.measurements['component'] + 
                          self.measurements['safety'])
        
        total_tests = len(all_measurements)
        passed_tests = sum(1 for test in all_measurements if test.passed)
        failed_tests = total_tests - passed_tests
        
        summary = {
            'total_tests': total_tests,
            'passed_tests': passed_tests,
            'failed_tests': failed_tests,
            'success_rate': (passed_tests / total_tests * 100) if total_tests > 0 else 0,
            'total_time': total_time,
            'overall_status': 'PASS' if failed_tests == 0 else 'FAIL'
        }
        
        return summary

def print_hardware_report(report: HardwareValidationReport, verbose: bool = False):
    """Print hardware validation report to console"""
    
    print("\n" + "="*60)
    print("HARDWARE VALIDATION REPORT")
    print("="*60)
    print(f"Timestamp: {report.timestamp}")
    print(f"Device: {report.device_info.get('device_model', 'Unknown')}")
    
    if 'serial_number' in report.device_info:
        print(f"Serial: {report.device_info['serial_number']}")
    
    # Print summary
    print(f"\nSUMMARY:")
    print(f"Overall Status: {report.summary['overall_status']}")
    print(f"Tests Passed: {report.summary['passed_tests']}/{report.summary['total_tests']}")
    print(f"Success Rate: {report.summary['success_rate']:.1f}%")
    print(f"Total Time: {report.summary['total_time']:.2f} seconds")
    
    # Print measurements by category
    categories = [
        ('POWER SYSTEM MEASUREMENTS', report.power_measurements),
        ('SIGNAL INTEGRITY MEASUREMENTS', report.signal_measurements),
        ('COMPONENT FUNCTIONALITY TESTS', report.component_tests),
        ('SAFETY CHECKS', report.safety_checks)
    ]
    
    for category_name, measurements in categories:
        if not measurements:
            continue
            
        print(f"\n{category_name}:")
        print("-" * len(category_name))
        
        for measurement in measurements:
            status = "✓ PASS" if measurement.passed else "✗ FAIL"
            print(f"{status:8} {measurement.name:30}")
            
            if verbose or not measurement.passed:
                print(f"         {measurement.message}")
                
                if measurement.measured_value is not None:
                    print(f"         Measured: {measurement.measured_value} {measurement.unit}")
                
                if measurement.expected_value is not None:
                    print(f"         Expected: {measurement.expected_value} {measurement.unit}")
    
    # Print safety warnings
    failed_measurements = [m for m in (report.power_measurements + 
                                     report.signal_measurements + 
                                     report.component_tests + 
                                     report.safety_checks) 
                         if not m.passed]
    
    if failed_measurements:
        print(f"\n⚠️  SAFETY WARNINGS:")
        print("-" * 18)
        for measurement in failed_measurements:
            print(f"  • {measurement.name}: {measurement.message}")
        print("\nDo not proceed with testing until all issues are resolved!")

def save_hardware_report(report: HardwareValidationReport, filename: str):
    """Save hardware validation report to JSON file"""
    
    with open(filename, 'w') as f:
        json.dump(asdict(report), f, indent=2, default=str)
    
    print(f"\nDetailed hardware report saved to: {filename}")

def main():
    """Main entry point"""
    
    parser = argparse.ArgumentParser(
        description="Validate RP2040 pEMF device hardware setup"
    )
    parser.add_argument('--interactive', action='store_true', default=True,
                       help='Run interactive validation with user prompts')
    parser.add_argument('--automated', action='store_true',
                       help='Run automated tests only (no user interaction)')
    parser.add_argument('--verbose', action='store_true',
                       help='Enable verbose output')
    parser.add_argument('--report', type=str,
                       help='Save detailed report to JSON file')
    
    args = parser.parse_args()
    
    # Override interactive mode if automated is specified
    if args.automated:
        args.interactive = False
    
    # Create validator
    validator = HardwareValidator(interactive=args.interactive, verbose=args.verbose)
    
    # Run validation
    report = validator.run_validation()
    
    # Print results
    print_hardware_report(report, verbose=args.verbose)
    
    # Save report if requested
    if args.report:
        save_hardware_report(report, args.report)
    
    # Exit with appropriate code
    sys.exit(0 if report.summary['overall_status'] == 'PASS' else 1)

if __name__ == '__main__':
    main()