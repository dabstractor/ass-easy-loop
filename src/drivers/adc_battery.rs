use rp2040_hal::adc::Adc;
use crate::types::battery::BatteryState;

pub struct BatteryMonitor {
    adc: Adc,
}

impl BatteryMonitor {
    pub fn new(adc: Adc) -> Self {
        Self { adc }
    }

    pub fn read_state(&mut self) -> BatteryState {
        // Implementation to be added
        BatteryState::Normal
    }
}