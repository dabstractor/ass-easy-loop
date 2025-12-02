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
FeedbackDriver feedbackDriver(chargeMonitor, 16);     // GPIO 16 - NeoPixel
ButtonController buttonController(timeSource, 26);    // GPIO 26 - Control button

// Logic layer - business logic with injected dependencies
WaveformController waveformController(coilDriver, feedbackDriver, timeSource);
SessionManager sessionManager(waveformController, timeSource);

void setup() {
    // Enable USB Serial for bootloader backdoor (PRD 5.3)
    Serial.begin(115200);

    // Initialize HAL components
    coilDriver.begin();
    chargeMonitor.begin();
    feedbackDriver.begin();
    buttonController.begin();

    // Start with NeoPixel disabled (must be off when pEMF not running)
    feedbackDriver.setEnabled(false);

    Serial.println("pEMF Device Ready - Press button to start");
}

void loop() {
    // Safety check: Disable pEMF if charging
    if (chargeMonitor.isCharging()) {
        if (sessionManager.isActive()) {
            sessionManager.stop();
            feedbackDriver.setEnabled(false);
            Serial.println("Session stopped - device charging");
        }
        // Update feedback to show charging state (if enabled, shows charging color)
        feedbackDriver.update();
        return;
    }

    sessionManager.update();
}
