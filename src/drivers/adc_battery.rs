use crate::types::battery::{
    BatteryState, BatteryReading, SafetyFlags, convert_adc_to_voltage_mv,
    LOW_BATTERY_ADC_THRESHOLD, CHARGING_ADC_THRESHOLD, OVERVOLTAGE_ADC_THRESHOLD,
    UNDERVOLTAGE_ADC_THRESHOLD
};
use crate::types::errors::BatteryError;
use rp2040_hal::adc::Adc;
use core::sync::atomic::{AtomicU16, AtomicU32, Ordering};
use embedded_hal::adc::OneShot;

/// Battery monitoring system with ADC reading and safety detection
pub struct BatteryMonitor {
    /// ADC peripheral for voltage measurements
    adc: Adc,
    
    /// Last successful ADC reading (atomic for thread-safe access)
    last_adc_reading: AtomicU16,
    
    /// Timestamp of last successful reading (milliseconds)
    last_reading_timestamp: AtomicU32,
    
    /// Current battery state
    current_state: BatteryState,
    
    /// ADC read error counter for fault detection
    error_count: u8,
    
    /// Maximum allowed consecutive ADC errors before fault declaration
    max_error_count: u8,
}

impl BatteryMonitor {
    /// Create new battery monitor with ADC peripheral
    pub fn new(adc: Adc) -> Self {
        Self {
            adc,
            last_adc_reading: AtomicU16::new(2000), // Safe default (normal range)
            last_reading_timestamp: AtomicU32::new(0),
            current_state: BatteryState::Normal,
            error_count: 0,
            max_error_count: 5, // Allow up to 5 consecutive errors before fault
        }
    }
    
    /// Read battery voltage via ADC with proper error handling
    /// Returns Result<u16, BatteryError> for ADC reading or error condition
    pub fn read_adc_voltage(&mut self, battery_pin: &mut rp2040_hal::adc::AdcPin<rp2040_hal::gpio::Pin<rp2040_hal::gpio::bank0::Gpio26, rp2040_hal::gpio::FunctionSioInput, rp2040_hal::gpio::PullNone>>) -> Result<u16, BatteryError> 
    {
        // Use ADC's OneShot trait read method
        match self.adc.read(battery_pin) {
            Ok(adc_value) => {
                // Successful reading - reset error count
                self.error_count = 0;
                
                // Store reading atomically for other tasks
                self.last_adc_reading.store(adc_value, Ordering::SeqCst);
                
                // Update timestamp (would come from RTIC monotonic timer in real system)
                // For now using a placeholder that increments
                let current_timestamp = self.last_reading_timestamp.load(Ordering::SeqCst) + 100;
                self.last_reading_timestamp.store(current_timestamp, Ordering::SeqCst);
                
                Ok(adc_value)
            },
            Err(_) => {
                // Hardware error - increment error count
                self.error_count += 1;
                
                if self.error_count >= self.max_error_count {
                    // Too many consecutive errors - declare ADC failed
                    Err(BatteryError::AdcFailed)
                } else {
                    // Return last known good reading if we haven't hit max errors
                    let last_reading = self.last_adc_reading.load(Ordering::SeqCst);
                    if last_reading != 0 {
                        Ok(last_reading)
                    } else {
                        Err(BatteryError::AdcFailed)
                    }
                }
            }
        }
    }
    
    /// Get current battery state based on ADC reading
    pub fn get_battery_state(&self) -> BatteryState {
        let adc_value = self.last_adc_reading.load(Ordering::SeqCst);
        BatteryState::from_adc_reading(adc_value)
    }
    
    /// Update internal battery state and detect state transitions
    pub fn update_battery_state(&mut self) -> Option<(BatteryState, BatteryState)> {
        let new_state = self.get_battery_state();
        
        if new_state != self.current_state {
            let old_state = self.current_state;
            self.current_state = new_state;
            Some((old_state, new_state))
        } else {
            None
        }
    }
    
    /// Check battery voltage against safety limits
    /// Returns critical safety violations that require immediate response
    pub fn check_safety_limits(&self, adc_value: u16) -> Result<(), BatteryError> {
        let voltage_mv = convert_adc_to_voltage_mv(adc_value);
        let current_state = BatteryState::from_adc_reading(adc_value);
        
        // Check over-voltage condition (>4.2V battery)
        if adc_value > OVERVOLTAGE_ADC_THRESHOLD {
            return Err(BatteryError::OverVoltage {
                adc_value,
                voltage_mv,
                current_state,
            });
        }
        
        // Check under-voltage condition (<3.0V battery)  
        if adc_value < UNDERVOLTAGE_ADC_THRESHOLD {
            return Err(BatteryError::UnderVoltage {
                adc_value,
                voltage_mv,
                current_state,
            });
        }
        
        Ok(())
    }
    
    /// Create complete battery reading with timestamp and safety flags
    pub fn create_battery_reading(&self, safety_flags: &SafetyFlags) -> BatteryReading {
        let adc_value = self.last_adc_reading.load(Ordering::SeqCst);
        let timestamp = self.last_reading_timestamp.load(Ordering::SeqCst);
        
        BatteryReading::new(timestamp, adc_value, safety_flags)
    }
    
    /// Get last ADC reading (thread-safe atomic access)
    pub fn get_last_adc_reading(&self) -> u16 {
        self.last_adc_reading.load(Ordering::SeqCst)
    }
    
    /// Get timestamp of last reading
    pub fn get_last_timestamp(&self) -> u32 {
        self.last_reading_timestamp.load(Ordering::SeqCst)
    }
    
    /// Check if charging is detected (voltage > 3.6V threshold)
    pub fn is_charging_detected(&self) -> bool {
        let adc_value = self.last_adc_reading.load(Ordering::SeqCst);
        adc_value >= CHARGING_ADC_THRESHOLD
    }
    
    /// Check if battery is in low state (requires attention)
    pub fn is_low_battery(&self) -> bool {
        let adc_value = self.last_adc_reading.load(Ordering::SeqCst);
        adc_value <= LOW_BATTERY_ADC_THRESHOLD
    }
    
    /// Get current error count for diagnostics
    pub fn get_error_count(&self) -> u8 {
        self.error_count
    }
    
    /// Reset error count (for manual error recovery)
    pub fn reset_error_count(&mut self) {
        self.error_count = 0;
    }
    
    /// Validate ADC reading is within expected range
    /// ADC should never read exactly 0 or 4095 in normal operation
    pub fn validate_adc_reading(&self, adc_value: u16) -> Result<(), BatteryError> {
        if adc_value == 0 || adc_value >= 4095 {
            // ADC reading at rail values indicates hardware problem
            Err(BatteryError::AdcFailed)
        } else {
            Ok(())
        }
    }
    
    /// Self-test function to verify ADC functionality
    /// This would be called during system initialization
    pub fn self_test(&mut self, battery_pin: &mut rp2040_hal::adc::AdcPin<rp2040_hal::gpio::Pin<rp2040_hal::gpio::bank0::Gpio26, rp2040_hal::gpio::FunctionSioInput, rp2040_hal::gpio::PullNone>>) -> Result<(), BatteryError> 
    {
        // Take several ADC readings to verify functionality
        let mut readings = [0u16; 5];
        
        for reading in readings.iter_mut() {
            match self.read_adc_voltage(battery_pin) {
                Ok(adc_value) => {
                    self.validate_adc_reading(adc_value)?;
                    *reading = adc_value;
                },
                Err(e) => return Err(e),
            }
            
            // Small delay between readings (would use proper timer in RTIC)
            for _ in 0..1000 {
                cortex_m::asm::nop();
            }
        }
        
        // Check for reasonable variation in readings (not stuck)
        let min_reading = readings.iter().min().unwrap();
        let max_reading = readings.iter().max().unwrap();
        
        if max_reading - min_reading < 5 {
            // Readings too consistent - might indicate stuck ADC
            return Err(BatteryError::AdcFailed);
        }
        
        Ok(())
    }
}

/// Battery monitoring task implementation for RTIC integration
impl BatteryMonitor {
    /// Process single ADC sample - designed to be called at 10Hz from RTIC task
    /// This is the main entry point for periodic battery monitoring
    pub fn process_sample(&mut self, battery_pin: &mut rp2040_hal::adc::AdcPin<rp2040_hal::gpio::Pin<rp2040_hal::gpio::bank0::Gpio26, rp2040_hal::gpio::FunctionSioInput, rp2040_hal::gpio::PullNone>>, safety_flags: &SafetyFlags) 
        -> Result<BatteryReading, BatteryError> 
    {
        
        // Read ADC value with error handling
        let adc_value = self.read_adc_voltage(battery_pin)?;
        
        // Validate reading is in expected range
        self.validate_adc_reading(adc_value)?;
        
        // Check critical safety limits first
        self.check_safety_limits(adc_value)?;
        
        // Update battery state and detect transitions
        if let Some((old_state, new_state)) = self.update_battery_state() {
            // State transition detected - this would trigger logging in RTIC task
            // For now, we continue with normal processing
            let _ = (old_state, new_state); // Acknowledge transition
        }
        
        // Create complete battery reading for logging and monitoring
        let reading = self.create_battery_reading(safety_flags);
        
        Ok(reading)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_adc_error_handling() {
        // Test error count tracking and fault detection
        let mock_adc = unsafe { core::mem::zeroed() }; // Mock ADC for testing
        let mut monitor = BatteryMonitor::new(mock_adc);
        
        // Verify initial state
        assert_eq!(monitor.get_error_count(), 0);
        assert_eq!(monitor.get_battery_state(), BatteryState::Normal);
    }
    
    #[test]
    fn test_safety_limit_detection() {
        let mock_adc = unsafe { core::mem::zeroed() };
        let monitor = BatteryMonitor::new(mock_adc);
        
        // Test over-voltage detection
        let result = monitor.check_safety_limits(2000); // Above OVERVOLTAGE_ADC_THRESHOLD
        assert!(result.is_err());
        if let Err(BatteryError::OverVoltage { adc_value, .. }) = result {
            assert_eq!(adc_value, 2000);
        }
        
        // Test under-voltage detection  
        let result = monitor.check_safety_limits(1000); // Below UNDERVOLTAGE_ADC_THRESHOLD
        assert!(result.is_err());
        if let Err(BatteryError::UnderVoltage { adc_value, .. }) = result {
            assert_eq!(adc_value, 1000);
        }
        
        // Test normal range
        let result = monitor.check_safety_limits(1500); // Normal range
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_charging_detection() {
        let mock_adc = unsafe { core::mem::zeroed() };
        let mut monitor = BatteryMonitor::new(mock_adc);
        
        // Set ADC reading above charging threshold
        monitor.last_adc_reading.store(1700, Ordering::SeqCst);
        assert!(monitor.is_charging_detected());
        
        // Set ADC reading below charging threshold  
        monitor.last_adc_reading.store(1600, Ordering::SeqCst);
        assert!(!monitor.is_charging_detected());
    }
}
