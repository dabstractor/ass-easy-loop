use crate::types::battery::{SafetyFlags, BatteryState, OVERVOLTAGE_ADC_THRESHOLD, UNDERVOLTAGE_ADC_THRESHOLD};
use crate::types::errors::BatteryError;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// Safety monitoring system for battery protection
/// CRITICAL: This module handles life-safety functions and must respond within 100ms
/// All functions are designed for use in highest-priority RTIC tasks (Priority 4)
pub struct SafetyMonitor {
    /// Safety system armed flag - false disables all safety actions
    safety_armed: AtomicBool,
    
    /// Last safety check timestamp for timeout detection
    last_safety_check_ms: AtomicU32,
    
    /// Safety timeout threshold in milliseconds
    safety_timeout_ms: u32,
    
    /// Emergency stop flag - once set, requires manual reset
    emergency_stop_triggered: AtomicBool,
    
    /// Fault recovery counter - tracks automatic recovery attempts
    recovery_attempts: u8,
    
    /// Maximum automatic recovery attempts before manual intervention required
    max_recovery_attempts: u8,
}

impl SafetyMonitor {
    /// Create new safety monitor with default safety parameters
    /// CRITICAL: Safety system starts in ARMED state - ready for immediate response
    pub fn new() -> Self {
        Self {
            safety_armed: AtomicBool::new(true), // Start armed for safety
            last_safety_check_ms: AtomicU32::new(0),
            safety_timeout_ms: 100, // 100ms maximum between safety checks
            emergency_stop_triggered: AtomicBool::new(false),
            recovery_attempts: 0,
            max_recovery_attempts: 3,
        }
    }
    
    /// CRITICAL SAFETY FUNCTION: Immediately disable charging circuit
    /// This function MUST complete within 10ms to meet safety requirements
    /// Called from highest priority RTIC task (Priority 4)
    pub fn emergency_disable_charging(&mut self) -> Result<(), BatteryError> {
        // Set emergency stop flag immediately
        self.emergency_stop_triggered.store(true, Ordering::SeqCst);
        
        // In a complete implementation, this would control hardware charging circuit
        // For now, just set the emergency stop flag which other systems can monitor
        
        // Log this critical event immediately
        // In real implementation, this would use high-priority logging
        
        Ok(())
    }
    
    /// CRITICAL SAFETY FUNCTION: Check all battery safety limits
    /// This function is called from Priority 3 RTIC task for continuous monitoring  
    /// Must complete within 50ms to meet overall 100ms safety response requirement
    pub fn check_safety_limits(&mut self, adc_value: u16, timestamp_ms: u32, safety_flags: &SafetyFlags) 
        -> Result<(), BatteryError> {
        
        // Update last safety check timestamp
        self.last_safety_check_ms.store(timestamp_ms, Ordering::SeqCst);
        
        // Skip checks if safety system is disarmed (for testing only)
        if !self.safety_armed.load(Ordering::SeqCst) {
            return Ok(());
        }
        
        // Check if emergency stop is already triggered
        if self.emergency_stop_triggered.load(Ordering::SeqCst) {
            return Err(BatteryError::SafetyTimeout { 
                timeout_ms: 0, 
                last_known_state: BatteryState::Fault 
            });
        }
        
        // CRITICAL: Over-voltage protection (>4.2V)
        if adc_value > OVERVOLTAGE_ADC_THRESHOLD {
            let voltage_mv = ((adc_value as f32 * 3300.0) / 4095.0 / 0.337) as u16;
            return Err(BatteryError::OverVoltage {
                adc_value,
                voltage_mv,
                current_state: BatteryState::from_adc_reading(adc_value),
            });
        }
        
        // CRITICAL: Under-voltage protection (<3.0V) 
        if adc_value < UNDERVOLTAGE_ADC_THRESHOLD {
            let voltage_mv = ((adc_value as f32 * 3300.0) / 4095.0 / 0.337) as u16;
            return Err(BatteryError::UnderVoltage {
                adc_value, 
                voltage_mv,
                current_state: BatteryState::from_adc_reading(adc_value),
            });
        }
        
        // Check safety flags for other critical conditions
        if safety_flags.over_current.load(Ordering::SeqCst) {
            return Err(BatteryError::OverCurrent {
                measured_current_ma: 1100, // Example - real implementation would measure
                duration_ms: 0,
            });
        }
        
        if safety_flags.over_temperature.load(Ordering::SeqCst) {
            return Err(BatteryError::OverTemperature {
                temperature_c: 60, // Example - real implementation would measure  
                current_state: BatteryState::from_adc_reading(adc_value),
            });
        }
        
        Ok(())
    }
    
    /// Check for safety system timeout - no recent safety checks
    /// Called periodically to ensure safety monitoring is active
    pub fn check_safety_timeout(&self, current_time_ms: u32) -> Result<(), BatteryError> {
        let last_check = self.last_safety_check_ms.load(Ordering::SeqCst);
        
        if current_time_ms.saturating_sub(last_check) > self.safety_timeout_ms {
            return Err(BatteryError::SafetyTimeout {
                timeout_ms: current_time_ms.saturating_sub(last_check),
                last_known_state: BatteryState::Fault,
            });
        }
        
        Ok(())
    }
    
    /// Enable charging circuit (when safe conditions are met)
    /// Only allowed when no safety violations are active
    pub fn enable_charging(&mut self, safety_flags: &SafetyFlags) -> Result<(), BatteryError> {
        // Check if emergency stop is active
        if self.emergency_stop_triggered.load(Ordering::SeqCst) {
            return Err(BatteryError::ChargingCircuitFault {
                fault_code: 0x02,
                description: "Cannot enable charging: emergency stop active"
            });
        }
        
        // Verify all safety conditions are met
        if !safety_flags.is_safe() {
            return Err(BatteryError::ChargingCircuitFault {
                fault_code: 0x03,
                description: "Cannot enable charging: safety violation active"
            });
        }
        
        // In a complete implementation, this would control hardware charging circuit
        // For now, just clear the emergency stop if conditions are safe
        
        Ok(())
    }
    
    /// Attempt automatic recovery from safety fault
    /// Only allowed for specific non-critical faults and limited attempts
    pub fn attempt_recovery(&mut self, error: &BatteryError) -> Result<(), BatteryError> {
        // Only allow recovery for specific fault types
        match error {
            BatteryError::InvalidStateTransition { .. } => {
                // State machine errors can be recovered automatically
            },
            BatteryError::AdcFailed => {
                // ADC errors might be transient - allow limited recovery attempts
                if self.recovery_attempts >= self.max_recovery_attempts {
                    return Err(BatteryError::SafetyTimeout {
                        timeout_ms: 0,
                        last_known_state: BatteryState::Fault
                    });
                }
            },
            _ => {
                // Critical safety violations require manual recovery
                return Err(error.clone());
            }
        }
        
        self.recovery_attempts += 1;
        
        // Reset specific safety flags for recovery
        // Real implementation would reset appropriate hardware
        
        Ok(())
    }
    
    /// Manual safety system reset - requires operator intervention
    /// CRITICAL: Only call this when physical safety has been verified
    pub fn manual_reset(&mut self, safety_flags: &SafetyFlags) -> Result<(), BatteryError> {
        // Verify all safety conditions are actually clear
        if !safety_flags.is_safe() {
            return Err(BatteryError::ChargingCircuitFault {
                fault_code: 0x05,
                description: "Cannot reset: safety violations still active"
            });
        }
        
        // Reset emergency stop flag
        self.emergency_stop_triggered.store(false, Ordering::SeqCst);
        
        // Reset recovery attempt counter
        self.recovery_attempts = 0;
        
        // Re-arm safety system
        self.safety_armed.store(true, Ordering::SeqCst);
        
        Ok(())
    }
    
    /// Disarm safety system (DANGEROUS - for testing only)
    /// This function should NEVER be called in production code
    #[cfg(feature = "testing")]
    pub fn disarm_for_testing(&mut self) {
        self.safety_armed.store(false, Ordering::SeqCst);
    }
    
    /// Re-arm safety system after testing
    #[cfg(feature = "testing")]
    pub fn rearm_after_testing(&mut self) {
        self.safety_armed.store(true, Ordering::SeqCst);
    }
    
    /// Get current safety system status for diagnostics
    pub fn get_safety_status(&self) -> SafetyStatus {
        SafetyStatus {
            armed: self.safety_armed.load(Ordering::SeqCst),
            emergency_stop_active: self.emergency_stop_triggered.load(Ordering::SeqCst),
            last_check_ms: self.last_safety_check_ms.load(Ordering::SeqCst),
            recovery_attempts: self.recovery_attempts,
            max_recovery_attempts: self.max_recovery_attempts,
        }
    }
    
    /// Validate safety system configuration
    /// Called during system initialization to verify safety system is ready
    pub fn self_test(&mut self) -> Result<(), BatteryError> {
        // Verify safety system is properly armed
        if !self.safety_armed.load(Ordering::SeqCst) {
            return Err(BatteryError::SafetyTimeout {
                timeout_ms: 0,
                last_known_state: BatteryState::Fault
            });
        }
        
        // Test emergency stop flag functionality
        let original_state = self.emergency_stop_triggered.load(Ordering::SeqCst);
        
        // Test setting the flag
        self.emergency_stop_triggered.store(true, Ordering::SeqCst);
        if !self.emergency_stop_triggered.load(Ordering::SeqCst) {
            return Err(BatteryError::SafetyTimeout {
                timeout_ms: 0,
                last_known_state: BatteryState::Fault
            });
        }
        
        // Restore original state
        self.emergency_stop_triggered.store(original_state, Ordering::SeqCst);
        
        Ok(())
    }
}

/// Safety system status structure for diagnostics and monitoring
#[derive(Debug, Clone, Copy)]
pub struct SafetyStatus {
    pub armed: bool,
    pub emergency_stop_active: bool,
    pub last_check_ms: u32,
    pub recovery_attempts: u8,
    pub max_recovery_attempts: u8,
}

impl SafetyStatus {
    /// Check if safety system is fully operational
    pub fn is_operational(&self) -> bool {
        self.armed && !self.emergency_stop_active
    }
    
    /// Check if manual intervention is required
    pub fn requires_manual_intervention(&self) -> bool {
        self.emergency_stop_active || self.recovery_attempts >= self.max_recovery_attempts
    }
}

/// RTIC Task Integration Functions
/// These functions are specifically designed for use in RTIC tasks with appropriate priorities
impl SafetyMonitor {
    /// Emergency shutdown handler for Priority 4 RTIC task
    /// CRITICAL: This must complete within 10ms
    pub fn handle_emergency_shutdown(&mut self, error: BatteryError) -> Result<(), BatteryError> {
        // Immediately disable charging - this is the critical safety action
        self.emergency_disable_charging()?;
        
        // Set appropriate safety flags based on error type
        // This would be done in cooperation with shared safety flags
        
        // Emergency shutdown complete - error should be logged by calling task
        Err(error) // Propagate error to trigger logging and further response
    }
    
    /// Continuous safety monitoring for Priority 3 RTIC task  
    /// Called every 10ms (100Hz) to ensure rapid fault detection
    pub fn continuous_safety_check(&mut self, adc_value: u16, timestamp_ms: u32, 
                                   safety_flags: &SafetyFlags) -> Result<(), BatteryError> {
        
        // Check for safety timeout first
        self.check_safety_timeout(timestamp_ms)?;
        
        // Perform comprehensive safety limit checks
        self.check_safety_limits(adc_value, timestamp_ms, safety_flags)?;
        
        Ok(())
    }
    
    /// Periodic safety system health check for lower priority task
    /// Called every 1 second to verify overall system health
    pub fn periodic_health_check(&self, current_time_ms: u32) -> Result<SafetyStatus, BatteryError> {
        // Check for safety timeout
        self.check_safety_timeout(current_time_ms)?;
        
        // Return current status for monitoring
        Ok(self.get_safety_status())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::battery::SafetyFlags;
    
    #[test]
    fn test_safety_monitor_creation() {
        let monitor = SafetyMonitor::new();
        let status = monitor.get_safety_status();
        
        assert!(status.armed);
        assert!(!status.emergency_stop_active);
        assert_eq!(status.recovery_attempts, 0);
    }
    
    #[test]  
    fn test_safety_limit_detection() {
        let mut monitor = SafetyMonitor::new();
        let safety_flags = SafetyFlags::new();
        
        // Test over-voltage detection
        let result = monitor.check_safety_limits(2000, 1000, &safety_flags);
        assert!(result.is_err());
        
        // Test under-voltage detection
        let result = monitor.check_safety_limits(1000, 1000, &safety_flags);
        assert!(result.is_err());
        
        // Test normal voltage range
        let result = monitor.check_safety_limits(1500, 1000, &safety_flags);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_safety_timeout_detection() {
        let monitor = SafetyMonitor::new();
        
        // Set last check to old timestamp
        monitor.last_safety_check_ms.store(1000, Ordering::SeqCst);
        
        // Check timeout with current time 200ms later (exceeds 100ms limit)
        let result = monitor.check_safety_timeout(1200);
        assert!(result.is_err());
        
        // Check no timeout with current time 50ms later (within limit)
        monitor.last_safety_check_ms.store(1000, Ordering::SeqCst);
        let result = monitor.check_safety_timeout(1050);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_emergency_stop_functionality() {
        let mut monitor = SafetyMonitor::new();
        
        // Verify not in emergency stop initially
        assert!(!monitor.emergency_stop_triggered.load(Ordering::SeqCst));
        
        // Trigger emergency stop
        let _ = monitor.emergency_disable_charging();
        
        // Verify emergency stop is active
        assert!(monitor.emergency_stop_triggered.load(Ordering::SeqCst));
        
        let status = monitor.get_safety_status();
        assert!(!status.is_operational());
        assert!(status.requires_manual_intervention());
    }
}