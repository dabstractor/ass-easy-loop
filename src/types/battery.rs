use core::sync::atomic::{AtomicBool, Ordering};
use core::default::Default;

/// Battery state enumeration with exact ADC threshold mapping
/// From PRPs/ai_docs/battery_adc_mapping.md - exact voltage thresholds
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatteryState {
    /// Low battery - requires immediate attention
    /// < 3.1V battery voltage - ADC ≤ 1425
    Low = 0,
    
    /// Normal operation range
    /// 3.1V - 3.6V battery voltage - ADC 1426-1674
    Normal = 1,
    
    /// Charging detected - voltage above normal range
    /// > 3.6V battery voltage - ADC ≥ 1675
    Charging = 2,
    
    /// Fully charged
    /// ~4.2V battery voltage - ADC ~1800
    Full = 3,
    
    /// Fault condition - immediate safety response required
    Fault = 4,
}

impl BatteryState {
    /// Convert ADC reading to battery state using exact thresholds
    pub fn from_adc_reading(adc_value: u16) -> Self {
        match adc_value {
            0..=LOW_BATTERY_ADC_THRESHOLD => BatteryState::Low,
            LOW_BATTERY_ADC_THRESHOLD_PLUS_ONE..=CHARGING_ADC_THRESHOLD_MINUS_ONE => BatteryState::Normal,
            CHARGING_ADC_THRESHOLD..=OVERVOLTAGE_ADC_THRESHOLD => BatteryState::Charging,
            OVERVOLTAGE_ADC_THRESHOLD_PLUS_ONE.. => BatteryState::Full,
        }
    }
}

/// Safety flags for critical battery monitoring - thread-safe atomic operations
#[derive(Debug)]
pub struct SafetyFlags {
    /// Battery voltage > 4.2V - immediate charging disable required
    pub over_voltage: AtomicBool,
    
    /// Battery voltage < 3.0V - immediate load disconnect required
    pub under_voltage: AtomicBool,
    
    /// Charge current > 1A - current limiting required
    pub over_current: AtomicBool,
    
    /// Battery temperature > 50°C - thermal protection required
    pub over_temperature: AtomicBool,
    
    /// Emergency stop triggered - immediate system shutdown
    pub emergency_stop: AtomicBool,
}

impl SafetyFlags {
    /// Create new safety flags with all flags cleared (safe state)
    pub fn new() -> Self {
        Self {
            over_voltage: AtomicBool::new(false),
            under_voltage: AtomicBool::new(false),
            over_current: AtomicBool::new(false),
            over_temperature: AtomicBool::new(false),
            emergency_stop: AtomicBool::new(false),
        }
    }
    
    /// Set emergency stop flag - thread-safe atomic operation
    pub fn set_emergency_stop(&self, value: bool) {
        self.emergency_stop.store(value, Ordering::SeqCst);
    }
    
    /// Check if system is in safe state (no active safety flags)
    pub fn is_safe(&self) -> bool {
        !self.emergency_stop.load(Ordering::SeqCst) &&
        !self.over_voltage.load(Ordering::SeqCst) &&
        !self.under_voltage.load(Ordering::SeqCst) &&
        !self.over_current.load(Ordering::SeqCst) &&
        !self.over_temperature.load(Ordering::SeqCst)
    }
    
    /// Get packed safety flags as byte for logging
    pub fn get_packed_flags(&self) -> u8 {
        let mut flags = 0u8;
        if self.emergency_stop.load(Ordering::SeqCst) { flags |= 0x01; }
        if self.over_voltage.load(Ordering::SeqCst) { flags |= 0x02; }
        if self.under_voltage.load(Ordering::SeqCst) { flags |= 0x04; }
        if self.over_current.load(Ordering::SeqCst) { flags |= 0x08; }
        if self.over_temperature.load(Ordering::SeqCst) { flags |= 0x10; }
        flags
    }
}

impl Default for SafetyFlags {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete battery reading structure for logging and monitoring
#[derive(Clone, Copy, Debug)]
pub struct BatteryReading {
    /// Timestamp when reading was taken (milliseconds)
    pub timestamp_ms: u32,
    
    /// Raw ADC reading (0-4095, 12-bit resolution)
    pub adc_value: u16,
    
    /// Calculated battery voltage in millivolts
    pub voltage_mv: u16,
    
    /// Current battery state based on voltage thresholds
    pub state: BatteryState,
    
    /// Charging circuit active (detected via voltage > 3.6V)
    pub is_charging: bool,
    
    /// Packed safety flags byte (for 64-byte HID report efficiency)
    pub safety_flags: u8,
}

impl BatteryReading {
    /// Create new battery reading from ADC value and safety flags
    pub fn new(timestamp_ms: u32, adc_value: u16, safety_flags: &SafetyFlags) -> Self {
        let voltage_mv = convert_adc_to_voltage_mv(adc_value);
        let state = BatteryState::from_adc_reading(adc_value);
        let is_charging = adc_value >= CHARGING_ADC_THRESHOLD;
        
        Self {
            timestamp_ms,
            adc_value,
            voltage_mv,
            state,
            is_charging,
            safety_flags: safety_flags.get_packed_flags(),
        }
    }
}

// Voltage threshold constants from PRPs/ai_docs/battery_adc_mapping.md
// CRITICAL: These values are safety-critical and must not be modified
// without full validation testing

/// Low battery threshold - ADC value for 3.1V battery voltage
/// Below this threshold triggers low battery warning
pub const LOW_BATTERY_ADC_THRESHOLD: u16 = 1425;

/// Helper constant for range matching (LOW_BATTERY_ADC_THRESHOLD + 1)
const LOW_BATTERY_ADC_THRESHOLD_PLUS_ONE: u16 = LOW_BATTERY_ADC_THRESHOLD + 1;

/// Charging detection threshold - ADC value for 3.6V battery voltage  
/// Above this threshold indicates charging circuit is active
pub const CHARGING_ADC_THRESHOLD: u16 = 1675;

/// Helper constant for range matching (CHARGING_ADC_THRESHOLD - 1)
const CHARGING_ADC_THRESHOLD_MINUS_ONE: u16 = CHARGING_ADC_THRESHOLD - 1;

/// Over-voltage safety threshold - ADC value for 4.2V battery voltage
/// Above this threshold triggers emergency charging disable
pub const OVERVOLTAGE_ADC_THRESHOLD: u16 = 1800;

/// Helper constant for range matching (OVERVOLTAGE_ADC_THRESHOLD + 1)
const OVERVOLTAGE_ADC_THRESHOLD_PLUS_ONE: u16 = OVERVOLTAGE_ADC_THRESHOLD + 1;

/// Under-voltage safety threshold - ADC value for 3.0V battery voltage
/// Below this threshold triggers emergency load disconnect
pub const UNDERVOLTAGE_ADC_THRESHOLD: u16 = 1200;

/// Voltage scale factor for 10kΩ:5.1kΩ voltage divider
/// Scale Factor = 5.1kΩ / (10kΩ + 5.1kΩ) = 0.337
/// From PRPs/ai_docs/battery_adc_mapping.md
pub const VOLTAGE_SCALE_FACTOR: f32 = 0.337;

/// ADC reference voltage in millivolts
pub const ADC_REFERENCE_VOLTAGE_MV: u16 = 3300;

/// ADC resolution (12-bit = 4096 levels, 0-4095 range)
pub const ADC_RESOLUTION: u16 = 4095;

/// Convert ADC reading to battery voltage in millivolts
/// Using exact conversion formula from PRPs/ai_docs/battery_adc_mapping.md:
/// Battery Voltage = (ADC Reading * 3.3V / 4095) / 0.337
pub fn convert_adc_to_voltage_mv(adc_value: u16) -> u16 {
    // Calculate ADC pin voltage in millivolts
    let adc_voltage_mv = (adc_value as f32 * ADC_REFERENCE_VOLTAGE_MV as f32) / ADC_RESOLUTION as f32;
    
    // Undo voltage divider scaling to get actual battery voltage
    let battery_voltage_mv = adc_voltage_mv / VOLTAGE_SCALE_FACTOR;
    
    battery_voltage_mv as u16
}

/// Convert battery voltage in millivolts to expected ADC reading
/// Inverse of convert_adc_to_voltage_mv for validation and testing
pub fn convert_voltage_mv_to_adc(voltage_mv: u16) -> u16 {
    // Apply voltage divider scaling
    let adc_voltage_mv = voltage_mv as f32 * VOLTAGE_SCALE_FACTOR;
    
    // Convert to ADC reading
    let adc_value = (adc_voltage_mv * ADC_RESOLUTION as f32) / ADC_REFERENCE_VOLTAGE_MV as f32;
    
    adc_value as u16
}
