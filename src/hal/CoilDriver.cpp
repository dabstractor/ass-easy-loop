#include "CoilDriver.h"

CoilDriver::CoilDriver(uint8_t pin)
    : _pin(pin) {
    // Constructor: store pin only, do NOT touch hardware
    // Hardware initialization deferred to begin()
}

void CoilDriver::begin() {
    // Configure pin as OUTPUT
    pinMode(_pin, OUTPUT);

    // Immediately set LOW for safety
    // This ensures coil is OFF before any other code runs
    digitalWrite(_pin, LOW);
}

void CoilDriver::setActive(bool active) {
    digitalWrite(_pin, active ? HIGH : LOW);
}

CoilDriver::~CoilDriver() {
    // Force safe state on destruction
    // Ensures coil is OFF when object goes out of scope
    digitalWrite(_pin, LOW);
}