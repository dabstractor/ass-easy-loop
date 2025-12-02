/**
 * @file main.cpp
 * @brief Entry point for Ass-Easy-Loop pEMF therapy firmware.
 *
 * Wires together all HAL and Logic components using dependency injection
 * to create a complete 10Hz pEMF therapy system with button control.
 *
 * Button controls (GPIO26):
 * - Single press when stopped: Start session
 * - 3-second hold when running: Stop session
 * - Single press when running: Toggle NeoPixel on/off
 * - Double press when running: Extend time by 5 minutes (max 45 min total)
 */

#include <Arduino.h>
#include "hal/CoilDriver.h"
#include "hal/FeedbackDriver.h"
#include "hal/ChargeMonitor.h"
#include "hal/ButtonController.h"
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

    // Process button events
    ButtonController::Event event = buttonController.update();

    if (sessionManager.isActive()) {
        // Session is running - handle running-state events
        switch (event) {
            case ButtonController::Event::LONG_HOLD:
                // Stop the session
                sessionManager.stop();
                feedbackDriver.setEnabled(false);  // NeoPixel off when pEMF not running
                Serial.println("Session stopped by user (long hold)");
                break;

            case ButtonController::Event::SINGLE_PRESS:
                // Toggle NeoPixel
                if (feedbackDriver.toggleEnabled()) {
                    Serial.println("NeoPixel enabled");
                } else {
                    Serial.println("NeoPixel disabled");
                }
                break;

            case ButtonController::Event::DOUBLE_PRESS:
                // Extend time by 5 minutes
                if (sessionManager.extendTime()) {
                    unsigned long totalMin = sessionManager.getSessionDuration() / 60000UL;
                    Serial.print("Time extended - total session: ");
                    Serial.print(totalMin);
                    Serial.println(" minutes");
                } else {
                    Serial.println("Cannot extend - already at max (45 min)");
                }
                break;

            default:
                break;
        }

        // Update session (handles timing and waveform generation)
        sessionManager.update();

    } else {
        // Session is NOT running - handle stopped-state events
        if (event == ButtonController::Event::SINGLE_PRESS) {
            // Start a new session
            sessionManager.start();
            feedbackDriver.setEnabled(true);  // Enable NeoPixel when session starts
            Serial.println("pEMF Session Started - 15 minute default");
        }

        // Keep NeoPixel off when not running (but still call update for consistency)
        feedbackDriver.update();
    }
}
