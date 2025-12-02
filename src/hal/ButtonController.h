#ifndef BUTTON_CONTROLLER_H
#define BUTTON_CONTROLLER_H

#include <Arduino.h>
#include "../hal/TimeSource.h"

/**
 * @brief Button input controller with debouncing and gesture detection.
 *
 * Handles button press detection including:
 * - Single press detection
 * - Double press detection (two quick presses)
 * - Long hold detection (3 seconds)
 *
 * Uses internal pull-up resistor - button should connect GPIO to GND.
 */
class ButtonController {
public:
    /**
     * @brief Button event types detected by the controller.
     */
    enum class Event {
        NONE,           ///< No event
        SINGLE_PRESS,   ///< Single press and release
        DOUBLE_PRESS,   ///< Two presses within the double-press window
        LONG_HOLD       ///< Button held for 3+ seconds
    };

    /**
     * @brief Construct ButtonController with specified GPIO pin.
     * @param buttonPin GPIO pin for button input (default: 26)
     * @param timeSource Reference to time source for timing
     */
    ButtonController(const ITimeSource& timeSource, uint8_t buttonPin = 26);

    /**
     * @brief Initialize button GPIO with internal pull-up.
     * @note Must be called in setup()
     */
    void begin();

    /**
     * @brief Update button state and detect gestures.
     * @return Event type if a gesture was detected, NONE otherwise
     * @note Call this in the main loop
     */
    Event update();

    /**
     * @brief Check if button is currently pressed.
     * @return true if button is currently held down
     */
    bool isPressed() const;

private:
    const uint8_t _buttonPin;
    const ITimeSource& _timeSource;

    // Debounce settings
    static constexpr unsigned long DEBOUNCE_MS = 50;

    // Timing thresholds
    static constexpr unsigned long LONG_HOLD_MS = 3000;      ///< 3 seconds for long hold
    static constexpr unsigned long DOUBLE_PRESS_WINDOW_MS = 400; ///< Window for double press

    // Button state tracking
    bool _lastRawState;           ///< Last raw reading
    bool _debouncedState;         ///< Debounced button state (true = pressed)
    bool _lastDebouncedState;     ///< Previous debounced state for edge detection
    unsigned long _lastDebounceTime;

    // Gesture detection state
    unsigned long _pressStartTime;     ///< When button was first pressed
    unsigned long _lastReleaseTime;    ///< When button was last released
    uint8_t _pressCount;               ///< Count of presses in double-press window
    bool _longHoldFired;               ///< Has long hold event been fired for this press?
    bool _waitingForDoublePress;       ///< Currently in double-press detection window

    /**
     * @brief Read and debounce raw button state.
     * @return true if debounced state changed
     */
    bool debounceButton();
};

#endif // BUTTON_CONTROLLER_H
