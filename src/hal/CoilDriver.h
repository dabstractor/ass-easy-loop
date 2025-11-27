#ifndef COIL_DRIVER_H
#define COIL_DRIVER_H

#include <Arduino.h>

/**
 * @brief Safe wrapper for MOSFET-controlled magnetic coil.
 *
 * Controls GPIO pin connected to IRF520 MOSFET driver module.
 * Implements fail-safe patterns:
 * - begin() sets OUTPUT mode and immediately writes LOW
 * - Destructor forces LOW state for safety
 * - Non-copyable (hardware resource protection)
 *
 * @note Flyback diode protection is handled in hardware (PRD Section 2.1)
 */
class CoilDriver {
public:
    /**
     * @brief Construct CoilDriver with specified GPIO pin.
     * @param pin GPIO pin number connected to MOSFET TRIG/SIG (default: 15)
     * @note Does NOT configure hardware - call begin() in setup()
     */
    explicit CoilDriver(uint8_t pin = 15);

    /**
     * @brief Initialize GPIO pin as OUTPUT and set LOW (safe state).
     * @note Must be called in setup() before any setActive() calls
     */
    void begin();

    /**
     * @brief Control coil activation state.
     * @param active true = energize coil (HIGH), false = de-energize (LOW)
     */
    void setActive(bool active);

    /**
     * @brief Force coil to safe state (LOW) on destruction.
     */
    ~CoilDriver();

    // Delete copy operations - hardware resource is unique
    CoilDriver(const CoilDriver&) = delete;
    CoilDriver& operator=(const CoilDriver&) = delete;

private:
    const uint8_t _pin;  ///< GPIO pin number (immutable)
};

#endif // COIL_DRIVER_H