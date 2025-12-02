/**
 * @file main.cpp
 * @brief Entry point for Ass-Easy-Loop pEMF therapy firmware.
 *
 * Wires together all HAL and Logic components using dependency injection
 * to create a complete 10Hz pEMF therapy system with 15-minute timeout.
 */

#include <Arduino.h>
#include "hal/CoilDriver.h"
#include "hal/FeedbackDriver.h"
#include "hal/ChargeMonitor.h"
#include "hal/TimeSource.h"
#include "logic/WaveformController.h"
#include "logic/SessionManager.h"

// HAL layer - hardware abstractions
ArduinoTimeSource timeSource;
CoilDriver coilDriver(15);             // GPIO 15 - MOSFET control
ChargeMonitor chargeMonitor(14);       // GPIO 14 - TP4056 Charging Status (Low = Charging)
FeedbackDriver feedbackDriver(chargeMonitor, 16);     // GPIO 16 - NeoPixel (needs ChargeMonitor now)

// Logic layer - business logic with injected dependencies
WaveformController waveformController(coilDriver, feedbackDriver, timeSource);
SessionManager sessionManager(waveformController, timeSource);

void setup() {
    // Enable USB Serial for bootloader backdoor (PRD 5.3)
    Serial.begin(115200);

    // Initialize HAL components
    coilDriver.begin();
    // We need chargeMonitor initialized before feedbackDriver uses it (though it just stores ref)
    // But for safety let's init it first
    chargeMonitor.begin();
    feedbackDriver.begin();

    // Initialize and start session
    waveformController.begin();
    sessionManager.start();

    Serial.println("pEMF Session Started - 15 minute limit");
}

void loop() {
    // Safety check: Disable pEMF if charging
    if (chargeMonitor.isCharging()) {
        waveformController.forceInactive();
        // Update feedback to show charging state (red)
        feedbackDriver.update();
        return;
    }

    sessionManager.update();
}
