#ifndef CHARGE_MONITOR_H
#define CHARGE_MONITOR_H

#include <Arduino.h>

class ChargeMonitor {
public:
    explicit ChargeMonitor(int pin = 14);
    void begin();

    /**
     * @brief Checks if the device is plugged into USB power.
     * Uses voltage divider on VIN to detect USB connection.
     * @return true if USB connected, false if running on battery
     */
    bool isPluggedIn() const;

    /**
     * @brief Legacy alias for isPluggedIn().
     * @deprecated Use isPluggedIn() for clarity
     */
    bool isCharging() const;

private:
    int _pin;
};

#endif // CHARGE_MONITOR_H
