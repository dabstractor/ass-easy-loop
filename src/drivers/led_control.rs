use embedded_hal::digital::v2::OutputPin;

pub struct LedControl<P: OutputPin> {
    #[allow(dead_code)]
    pin: P,
}

impl<P: OutputPin> LedControl<P> {
    pub fn new(pin: P) -> Self {
        Self { pin }
    }

    pub fn set_on(&mut self) {
        // Implementation to be added
    }

    pub fn set_off(&mut self) {
        // Implementation to be added
    }
}
