//! Battery state management module
//! 
//! This module provides the battery state enum and state machine logic
//! for monitoring battery voltage levels and determining charging status.

use core::option::Option::{self, Some, None};
use core::result::Result::{self, Ok, Err};

use hal::{
    adc::{Adc, AdcPin},
    gpio::{
        bank0::Gpio26,
        FunctionSio, Pin, PullNone, SioInput,
    },
};
use rp2040_hal as hal;

#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(any(test, not(test)), derive(Debug))]
pub enum BatteryState {
    Low,      // ADC ≤ 1425 (< 3.1V)
    Normal,   // 1425 < ADC < 1675 (3.1V - 3.6V)
    Charging, // ADC ≥ 1675 (> 3.6V)
}

impl BatteryState {
    /// Determine battery state from ADC reading with threshold comparisons
    pub fn from_adc_reading(adc_value: u16) -> Self {
        if adc_value <= 1425 {
            BatteryState::Low
        } else if adc_value < 1675 {
            BatteryState::Normal
        } else {
            BatteryState::Charging
        }
    }

    /// Get the ADC threshold values for this state
    pub fn get_thresholds(&self) -> (u16, u16) {
        match self {
            BatteryState::Low => (0, 1425),
            BatteryState::Normal => (1425, 1675),
            BatteryState::Charging => (1675, u16::MAX),
        }
    }

    /// Check if a state transition should occur based on new ADC reading
    pub fn should_transition_to(&self, adc_value: u16) -> Option<BatteryState> {
        let new_state = Self::from_adc_reading(adc_value);
        if new_state != *self {
            Some(new_state)
        } else {
            None
        }
    }
}

/// Validation function to verify battery state machine logic
/// This function performs basic validation of the state machine logic
/// and can be called during initialization to ensure correctness
#[allow(dead_code)] // TODO: Remove this directive when implementing task 9.1 (unit tests)
pub fn validate_battery_state_logic() -> bool {
    // Test Low battery state (ADC ≤ 1425)
    if BatteryState::from_adc_reading(0) != BatteryState::Low { return false; }
    if BatteryState::from_adc_reading(1000) != BatteryState::Low { return false; }
    if BatteryState::from_adc_reading(1425) != BatteryState::Low { return false; }

    // Test Normal battery state (1425 < ADC < 1675)
    if BatteryState::from_adc_reading(1426) != BatteryState::Normal { return false; }
    if BatteryState::from_adc_reading(1500) != BatteryState::Normal { return false; }
    if BatteryState::from_adc_reading(1674) != BatteryState::Normal { return false; }

    // Test Charging battery state (ADC ≥ 1675)
    if BatteryState::from_adc_reading(1675) != BatteryState::Charging { return false; }
    if BatteryState::from_adc_reading(2000) != BatteryState::Charging { return false; }
    if BatteryState::from_adc_reading(4095) != BatteryState::Charging { return false; }

    // Test threshold values
    let low_state = BatteryState::Low;
    let (low_min, low_max) = low_state.get_thresholds();
    if low_min != 0 || low_max != 1425 { return false; }

    let normal_state = BatteryState::Normal;
    let (normal_min, normal_max) = normal_state.get_thresholds();
    if normal_min != 1425 || normal_max != 1675 { return false; }

    let charging_state = BatteryState::Charging;
    let (charging_min, charging_max) = charging_state.get_thresholds();
    if charging_min != 1675 || charging_max != u16::MAX { return false; }

    // Test state transitions
    let current_state = BatteryState::Normal;
    
    // Test transition from Normal to Low
    match current_state.should_transition_to(1400) {
        Some(BatteryState::Low) => {},
        _ => return false,
    }

    // Test transition from Normal to Charging
    match current_state.should_transition_to(1700) {
        Some(BatteryState::Charging) => {},
        _ => return false,
    }

    // Test no transition when staying in same state
    match current_state.should_transition_to(1500) {
        None => {},
        _ => return false,
    }

    // Test boundary conditions
    if BatteryState::from_adc_reading(1425) != BatteryState::Low { return false; }
    if BatteryState::from_adc_reading(1426) != BatteryState::Normal { return false; }
    if BatteryState::from_adc_reading(1674) != BatteryState::Normal { return false; }
    if BatteryState::from_adc_reading(1675) != BatteryState::Charging { return false; }

    true
}

/// Error type for ADC operations
#[allow(dead_code)] // TODO: Remove this directive when implementing task 8.1 (error handling)
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AdcError {
    ReadFailed,
    InvalidReading,
}

/// ADC reading and voltage conversion functions
#[allow(dead_code)] // TODO: Remove this directive when implementing task 6.1 (LED control) and task 9.1 (unit tests)
pub struct BatteryMonitor;

impl BatteryMonitor {
    /// Read ADC value from GPIO 26 with error handling
    /// Returns the raw ADC value (0-4095) or an error
    /// 
    /// Note: The actual ADC reading implementation will be completed when implementing
    /// the battery monitoring task (task 5.1). The rp2040-hal ADC API requires specific
    /// usage patterns that will be implemented in the context of the actual RTIC task.
    #[allow(dead_code)] // TODO: Remove this directive when implementing task 6.1 (LED control) or task 8.1 (error handling)
    pub fn read_adc_value(
        _adc: &mut Adc,
        _adc_pin: &mut AdcPin<Pin<Gpio26, FunctionSio<SioInput>, PullNone>>,
    ) -> Result<u16, AdcError> {
        // TODO: Implement actual ADC reading in task 5.1 (battery_monitor_task)
        // The implementation will use the correct rp2040-hal ADC API methods
        // For now, return a placeholder value that represents a normal battery state
        Ok(1500) // This represents a normal battery state for testing purposes
    }

    /// Convert ADC reading to actual battery voltage
    /// Uses voltage divider calculation: Vbat = ADC_value * (3.3V / 4095) / voltage_divider_ratio
    /// Voltage divider ratio = R2 / (R1 + R2) = 5.1kΩ / (10kΩ + 5.1kΩ) = 0.337
    #[allow(dead_code)] // TODO: Remove this directive when implementing task 6.1 (LED control) or task 9.1 (unit tests)
    pub fn adc_to_battery_voltage(adc_value: u16) -> u32 {
        // Convert to millivolts for better precision
        // ADC voltage = adc_value * 3300mV / 4095
        // Battery voltage = ADC voltage / 0.337
        // Simplified: battery_voltage_mv = adc_value * 3300 / (4095 * 0.337)
        // Further simplified: battery_voltage_mv = adc_value * 2386 / 1000
        (adc_value as u32 * 2386) / 1000
    }

    /// Convert battery voltage (in millivolts) back to expected ADC reading
    /// This is useful for testing and validation
    #[allow(dead_code)] // TODO: Remove this directive when implementing task 9.1 (unit tests) or task 11.1 (performance validation)
    pub fn battery_voltage_to_adc(battery_voltage_mv: u32) -> u16 {
        // Reverse calculation: adc_value = battery_voltage_mv * 1000 / 2386
        let adc_value = (battery_voltage_mv * 1000) / 2386;
        if adc_value > 4095 {
            4095
        } else {
            adc_value as u16
        }
    }

    /// Determine battery state from ADC reading with error handling
    #[allow(dead_code)] // TODO: Remove this directive when implementing task 6.1 (LED control) or task 8.1 (error handling)
    pub fn get_battery_state_from_adc(
        adc: &mut Adc,
        adc_pin: &mut AdcPin<Pin<Gpio26, FunctionSio<SioInput>, PullNone>>,
    ) -> Result<BatteryState, AdcError> {
        match Self::read_adc_value(adc, adc_pin) {
            Ok(adc_value) => Ok(BatteryState::from_adc_reading(adc_value)),
            Err(error) => Err(error),
        }
    }

    /// Get battery voltage in millivolts with error handling
    #[allow(dead_code)] // TODO: Remove this directive when implementing task 6.1 (LED control) or task 11.1 (performance validation)
    pub fn get_battery_voltage_mv(
        adc: &mut Adc,
        adc_pin: &mut AdcPin<Pin<Gpio26, FunctionSio<SioInput>, PullNone>>,
    ) -> Result<u32, AdcError> {
        match Self::read_adc_value(adc, adc_pin) {
            Ok(adc_value) => Ok(Self::adc_to_battery_voltage(adc_value)),
            Err(error) => Err(error),
        }
    }
}

/// Validation function for ADC and voltage conversion logic
#[allow(dead_code)] // TODO: Remove this directive when implementing task 9.1 (unit tests) or task 11.2 (stability testing)
pub fn validate_adc_conversion_logic() -> bool {
    // Test voltage conversion calculations
    
    // Test known ADC values to expected battery voltages
    // ADC 1425 should be approximately 3100mV (3.1V)
    let voltage_1425 = BatteryMonitor::adc_to_battery_voltage(1425);
    if !(3000..=3200).contains(&voltage_1425) { return false; }
    
    // ADC 1675 should be approximately 3600mV (3.6V)  
    let voltage_1675 = BatteryMonitor::adc_to_battery_voltage(1675);
    if !(3500..=3700).contains(&voltage_1675) { return false; }
    
    // Test reverse conversion
    let adc_from_3100mv = BatteryMonitor::battery_voltage_to_adc(3100);
    if !(1400..=1450).contains(&adc_from_3100mv) { return false; }
    
    let adc_from_3600mv = BatteryMonitor::battery_voltage_to_adc(3600);
    if !(1650..=1700).contains(&adc_from_3600mv) { return false; }
    
    // Test boundary conditions
    let voltage_0 = BatteryMonitor::adc_to_battery_voltage(0);
    if voltage_0 != 0 { return false; }
    
    let voltage_4095 = BatteryMonitor::adc_to_battery_voltage(4095);
    if !(9700..=9800).contains(&voltage_4095) { return false; } // Should be ~9.77V
    
    true
}