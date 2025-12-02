#ifndef FEEDBACK_DRIVER_H
#define FEEDBACK_DRIVER_H

#include <Arduino.h>
#include <Adafruit_NeoPixel.h>
#include "ChargeMonitor.h"
#include "../config/Configuration.h"

/**
 * @brief Safe wrapper for synchronous Audio/Visual feedback.
 *
 * Controls onboard WS2812 NeoPixel LED for
 * therapeutic session feedback at 10Hz.
 *
 * Implements fail-safe patterns:
 * - begin() initializes output to OFF state
 * - Destructor forces output to OFF
 * - Non-copyable (hardware resource protection)
 *
 * @note PRD Section 2.1 (NeoPixel GPIO 16)
 */
class FeedbackDriver {
public:
    /**
     * @brief Construct FeedbackDriver with specified GPIO pins.
     * @param chargeMonitor Reference to ChargeMonitor for charging state detection
     * @param neoPixelPin GPIO pin for WS2812 LED (default: 16)
     * @note Does NOT configure hardware - call begin() in setup()
     */
    explicit FeedbackDriver(const ChargeMonitor& chargeMonitor, uint8_t neoPixelPin = 16);

    /**
     * @brief Initialize NeoPixel to safe OFF state.
     * @note Must be called in setup() before any indicateActive() calls
     */
    void begin();

    /**
     * @brief Control feedback state.
     * Updated to handle software PWM for safety and visual persistence.
     */
    void update();

    /**
     * @brief Force LED to OFF state immediately.
     * Used for bootloader entry and emergency shutdown.
     */
    void turnOff();

    /**
     * @brief Enable or disable the NeoPixel output.
     * @param enabled true to enable visual feedback, false to disable
     * @note When disabled, LED stays off regardless of update() calls
     */
    void setEnabled(bool enabled);

    /**
     * @brief Check if NeoPixel output is enabled.
     * @return true if enabled, false if disabled
     */
    bool isEnabled() const;

    /**
     * @brief Toggle the NeoPixel enabled state.
     * @return New enabled state after toggling
     */
    bool toggleEnabled();

    /**
     * @brief Force all outputs to safe state (OFF) on destruction.
     */
    ~FeedbackDriver();

    // Delete copy operations - hardware resource is unique
    FeedbackDriver(const FeedbackDriver&) = delete;
    FeedbackDriver& operator=(const FeedbackDriver&) = delete;

private:
    const uint8_t _neoPixelPin;        ///< GPIO pin for NeoPixel (immutable)
    const ChargeMonitor& _chargeMonitor; ///< Reference to ChargeMonitor
    Adafruit_NeoPixel _pixel;          ///< NeoPixel driver instance

    static constexpr uint8_t LED_COUNT = 1;           ///< Single onboard LED

    // Color cycling state
    float _hueOffset = 0.0f;

    // Enable/disable state
    bool _enabled = true;              ///< Whether NeoPixel output is enabled

    /**
     * @brief Helper to set pixel color with brightness scaling
     * @param r Red component (0-255)
     * @param g Green component (0-255)
     * @param b Blue component (0-255)
     */
    void setScaledColor(uint8_t r, uint8_t g, uint8_t b);

    /**
     * @brief Generate pastel color from hue
     * @param hue Hue value (0.0 to 1.0)
     * @param r Output Red
     * @param g Output Green
     * @param b Output Blue
     */
    void getPastelColor(float hue, uint8_t& r, uint8_t& g, uint8_t& b);
};

#endif // FEEDBACK_DRIVER_H
