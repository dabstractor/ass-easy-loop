#include "ChargeMonitor.h"

ChargeMonitor::ChargeMonitor(int pin) : _pin(pin) {
}

void ChargeMonitor::begin() {
    // Use INPUT_PULLDOWN so floating pin (unplugged) reads LOW
    // When plugged in, the voltage divider will pull it HIGH
    pinMode(_pin, INPUT_PULLDOWN);
}

bool ChargeMonitor::isCharging() const {
    // Active High logic: HIGH means Plugged In (Charging/Powered)
    return digitalRead(_pin) == HIGH;
}
