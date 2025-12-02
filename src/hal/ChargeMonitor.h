#ifndef CHARGE_MONITOR_H
#define CHARGE_MONITOR_H

#include <Arduino.h>

class ChargeMonitor {
public:
    explicit ChargeMonitor(int pin = 14);
    void begin();

    /**
     * @brief Checks if the device is plugged into power (USB).
     * @return true if plugged in (Input Voltage detected), false if portable
     */
    bool isCharging() const;

private:
    int _pin;
};

#endif // CHARGE_MONITOR_H
