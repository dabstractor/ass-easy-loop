#ifndef CONFIGURATION_H
#define CONFIGURATION_H

#include <Arduino.h>

/**
 * @brief Centralized configuration for Ass-Easy-Loop pEMF therapy device.
 *
 * This file contains all user-configurable parameters in one place.
 * Modify these values to customize device behavior without touching the core logic.
 */

namespace Config {
    /**
     * Total period for one complete pulse cycle in milliseconds.
     * PRODUCTION: 100 (10Hz therapeutic frequency)
     * TEST MODE: 5000 (0.2Hz for manual testing)
     */
    static constexpr unsigned long PERIOD_MS = 100;

    /**
     * Duration of pulse ON time in milliseconds.
     * PRODUCTION: 2 (2% duty cycle for safety)
     * TEST MODE: 500 (longer pulses for visual testing)
     */
    static constexpr unsigned long ON_DURATION_MS = 2;

    /**
     * Duration of pulse OFF time in milliseconds.
     * Calculated as PERIOD_MS - ON_DURATION_MS
     */
    static constexpr unsigned long OFF_DURATION_MS = PERIOD_MS - ON_DURATION_MS;

    // ================================================
    // NEOPIXEL FEEDBACK SETTINGS
    // ================================================

    /**
     * Global brightness scaling for NeoPixel LED.
     * Range: 0.0 (off) to 1.0 (full brightness)
     * Recommended: 0.05 to 0.2 for indoor use
     */
    static constexpr float NEOPIXEL_BRIGHTNESS = 0.08f;

    /**
     * Speed of pastel color cycling during normal operation.
     * Range: 0.00001 (very slow) to 0.01 (very fast)
     * Recommended: 0.00002 for gentle cycling
     */
    static constexpr float HUE_INCREMENT = 0.00003f;

    // ================================================
    // WAVEFORM TIMING SETTINGS
    // ================================================


    // ================================================
    // PRESET CONFIGURATIONS
    // ================================================

    /**
     * Uncomment ONE of the following preset configurations:
     */

    // PRODUCTION CONFIGURATION (Therapeutic 10Hz, 2% duty cycle)
    // Note: This is the default, no changes needed for production use

    // TEST CONFIGURATION (Slow 0.2Hz, visible pulses for testing)
    // To enable: Uncomment the following block and comment out the production values above
    /*
    static constexpr unsigned long PERIOD_MS = 5000;
    static constexpr unsigned long ON_DURATION_MS = 500;
    static constexpr unsigned long OFF_DURATION_MS = PERIOD_MS - ON_DURATION_MS;
    */

    // ================================================
    // ADVANCED SETTINGS
    // ================================================

    /**
     * NeoPixel color when device is plugged into power (Safety Mode).
     * RGB values (0-255), will be scaled by NEOPIXEL_BRIGHTNESS
     * Set to 0,0,0 to disable LED while plugged in.
     */
    static constexpr uint8_t CHARGING_COLOR_R = 0;
    static constexpr uint8_t CHARGING_COLOR_G = 0;
    static constexpr uint8_t CHARGING_COLOR_B = 0;

    /**
     * Pastel color generation parameters
     * Saturation: 0.0 (white) to 1.0 (vibrant)
     * Value: 0.0 (black) to 1.0 (bright)
     */
    static constexpr float PASTEL_SATURATION = 0.5f;
    static constexpr float PASTEL_VALUE = 1.0f;

    /**
     * Blink pattern timing settings in milliseconds
     */
    static constexpr unsigned long BLINK_FAST_ON_MS = 100;
    static constexpr unsigned long BLINK_FAST_OFF_MS = 100;
    static constexpr unsigned long BLINK_SLOW_ON_MS = 250;
    static constexpr unsigned long BLINK_SLOW_OFF_MS = 250;

} // namespace Config

#endif // CONFIGURATION_H
