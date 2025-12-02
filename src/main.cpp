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
#include "hal/TimeSource.h"
#include "logic/WaveformController.h"
#include "logic/SessionManager.h"

// HAL layer - hardware abstractions
ArduinoTimeSource timeSource;
CoilDriver coilDriver(15);             // GPIO 15 - MOSFET control
FeedbackDriver feedbackDriver(16);     // GPIO 16 - NeoPixel

// Logic layer - business logic with injected dependencies
WaveformController waveformController(coilDriver, feedbackDriver, timeSource);
SessionManager sessionManager(waveformController, timeSource);

void setup() {
    // Enable USB Serial for bootloader backdoor (PRD 5.3)
    Serial.begin(115200);

    // Initialize HAL components
    coilDriver.begin();
    feedbackDriver.begin();

    // Initialize and start session
    waveformController.begin();
    sessionManager.start();

    Serial.println("pEMF Session Started - 15 minute limit");
}

void loop() {
    sessionManager.update();
}
