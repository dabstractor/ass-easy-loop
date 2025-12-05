#include "ChargeMonitor.h"

ChargeMonitor::ChargeMonitor(int pin) : _pin(pin) {
}

void ChargeMonitor::begin() {
    // Voltage divider from USB VIN: 10k from VIN to pin, 10k from pin to GND
    // Plugged in: ~2.5V on pin → reads HIGH
    // Unplugged: 0V (pulled to GND through bottom resistor) → reads LOW
    pinMode(_pin, INPUT_PULLDOWN);
}

bool ChargeMonitor::isPluggedIn() const {
    // Active HIGH: voltage divider outputs ~2.5V when USB connected
    return digitalRead(_pin) == HIGH;
}

bool ChargeMonitor::isCharging() const {
    // Legacy alias - actual function is USB power detection, not charge state
    return isPluggedIn();
}
