use crate::types::battery::BatteryState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemError {
    ConfigurationInvalid,
    FlashOperationFailed,
    BootloaderError,
}

/// Battery-specific error conditions requiring immediate safety response
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatteryError {
    /// ADC peripheral failed to read battery voltage
    /// Critical: Cannot monitor battery safety without ADC
    AdcFailed,
    
    /// Battery voltage exceeded safe charging limit (>4.2V)
    /// Critical: Risk of battery damage, fire, or explosion
    OverVoltage { 
        adc_value: u16, 
        voltage_mv: u16,
        current_state: BatteryState 
    },
    
    /// Battery voltage below critical discharge limit (<3.0V)
    /// Critical: Risk of deep discharge damage and inability to recharge
    UnderVoltage { 
        adc_value: u16, 
        voltage_mv: u16,
        current_state: BatteryState 
    },
    
    /// Safety monitoring system failed to respond within timeout
    /// Critical: Safety system compromised, immediate shutdown required
    SafetyTimeout {
        timeout_ms: u32,
        last_known_state: BatteryState
    },
    
    /// Over-current condition detected (>1A charge current)
    /// Critical: Risk of battery/charging circuit damage
    OverCurrent {
        measured_current_ma: u16,
        duration_ms: u32
    },
    
    /// Battery temperature exceeded safe operating range
    /// Critical: Thermal runaway risk
    OverTemperature {
        temperature_c: i16,
        current_state: BatteryState
    },
    
    /// Charging circuit hardware malfunction detected
    /// Critical: Unsafe charging conditions
    ChargingCircuitFault {
        fault_code: u8,
        description: &'static str
    },
    
    /// Battery state machine entered invalid transition
    /// Critical: Logic error in safety-critical state management
    InvalidStateTransition {
        from_state: BatteryState,
        to_state: BatteryState,
        trigger_adc: u16
    },
}

impl BatteryError {
    /// Check if error requires immediate emergency shutdown
    pub fn requires_emergency_shutdown(&self) -> bool {
        match self {
            BatteryError::OverVoltage { .. } => true,
            BatteryError::UnderVoltage { .. } => true,
            BatteryError::SafetyTimeout { .. } => true,
            BatteryError::OverCurrent { .. } => true,
            BatteryError::OverTemperature { .. } => true,
            BatteryError::ChargingCircuitFault { .. } => true,
            BatteryError::AdcFailed => true, // Cannot monitor = unsafe
            BatteryError::InvalidStateTransition { .. } => false, // Log but continue
        }
    }
    
    /// Get error severity level for logging
    pub fn severity_level(&self) -> u8 {
        match self {
            BatteryError::OverVoltage { .. } => 4, // Critical
            BatteryError::UnderVoltage { .. } => 4, // Critical
            BatteryError::SafetyTimeout { .. } => 4, // Critical
            BatteryError::OverCurrent { .. } => 3,  // High
            BatteryError::OverTemperature { .. } => 4, // Critical
            BatteryError::ChargingCircuitFault { .. } => 3, // High
            BatteryError::AdcFailed => 4, // Critical
            BatteryError::InvalidStateTransition { .. } => 2, // Medium
        }
    }
    
    /// Get human-readable error description for logging
    pub fn description(&self) -> &'static str {
        match self {
            BatteryError::AdcFailed => "ADC failed to read battery voltage",
            BatteryError::OverVoltage { .. } => "Battery over-voltage detected",
            BatteryError::UnderVoltage { .. } => "Battery under-voltage detected", 
            BatteryError::SafetyTimeout { .. } => "Safety monitoring timeout",
            BatteryError::OverCurrent { .. } => "Charging over-current detected",
            BatteryError::OverTemperature { .. } => "Battery over-temperature",
            BatteryError::ChargingCircuitFault { .. } => "Charging circuit fault",
            BatteryError::InvalidStateTransition { .. } => "Invalid battery state transition",
        }
    }
}
