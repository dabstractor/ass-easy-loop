#ifndef FEEDBACK_DRIVER_H
#define FEEDBACK_DRIVER_H

#include <Arduino.h>
#include <Adafruit_NeoPixel.h>

/**
 * @brief Safe wrapper for synchronous Audio/Visual feedback.
 *
 * Controls onboard WS2812 NeoPixel LED and piezo buzzer for
 * therapeutic session feedback at 10Hz.
 *
 * Implements fail-safe patterns:
 * - begin() initializes both outputs to OFF state
 * - Destructor forces both outputs to OFF
 * - Non-copyable (hardware resource protection)
 *
 * @note PRD Section 2.1 (NeoPixel GPIO 16) & 2.2 (Buzzer GPIO 14)
 */
class FeedbackDriver {
public:
    /**
     * @brief Construct FeedbackDriver with specified GPIO pins.
     * @param buzzerPin GPIO pin for piezo buzzer (default: 14)
     * @param neoPixelPin GPIO pin for WS2812 LED (default: 16)
     * @note Does NOT configure hardware - call begin() in setup()
     */
    explicit FeedbackDriver(uint8_t buzzerPin = 14, uint8_t neoPixelPin = 16);

    /**
     * @brief Initialize buzzer and NeoPixel to safe OFF state.
     * @note Must be called in setup() before any indicateActive() calls
     */
    void begin();

    /**
     * @brief Control synchronized feedback state.
     * @param isActive true = Buzzer HIGH + LED Green, false = both OFF
     */
    void indicateActive(bool isActive);

    /**
     * @brief Force all outputs to safe state (OFF) on destruction.
     */
    ~FeedbackDriver();

    // Delete copy operations - hardware resource is unique
    FeedbackDriver(const FeedbackDriver&) = delete;
    FeedbackDriver& operator=(const FeedbackDriver&) = delete;

private:
    const uint8_t _buzzerPin;          ///< GPIO pin for buzzer (immutable)
    const uint8_t _neoPixelPin;        ///< GPIO pin for NeoPixel (immutable)
    Adafruit_NeoPixel _pixel;          ///< NeoPixel driver instance

    static constexpr uint8_t LED_COUNT = 1;           ///< Single onboard LED
    static constexpr uint8_t BRIGHTNESS = 30;         ///< ~12% brightness
    static constexpr uint8_t GREEN_R = 0;             ///< Green color - Red
    static constexpr uint8_t GREEN_G = 255;           ///< Green color - Green
    static constexpr uint8_t GREEN_B = 0;             ///< Green color - Blue
};

#endif // FEEDBACK_DRIVER_H
