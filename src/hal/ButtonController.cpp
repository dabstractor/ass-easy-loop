#include "ButtonController.h"

ButtonController::ButtonController(const ITimeSource& timeSource, uint8_t buttonPin)
    : _buttonPin(buttonPin)
    , _timeSource(timeSource)
    , _lastRawState(false)
    , _debouncedState(false)
    , _lastDebouncedState(false)
    , _lastDebounceTime(0)
    , _pressStartTime(0)
    , _lastReleaseTime(0)
    , _pressCount(0)
    , _longHoldFired(false)
    , _waitingForDoublePress(false) {
}

void ButtonController::begin() {
    pinMode(_buttonPin, INPUT_PULLUP);
    // Initialize state to current reading
    _lastRawState = !digitalRead(_buttonPin);  // Inverted: LOW = pressed
    _debouncedState = _lastRawState;
    _lastDebouncedState = _debouncedState;
    _lastDebounceTime = _timeSource.millis();
}

bool ButtonController::debounceButton() {
    // Read current state (inverted: button pulls LOW when pressed)
    bool rawState = !digitalRead(_buttonPin);
    unsigned long currentTime = _timeSource.millis();

    // If raw state changed, reset debounce timer
    if (rawState != _lastRawState) {
        _lastDebounceTime = currentTime;

        // Capture press start time on FIRST raw press (before debounce)
        // This ensures long-hold timing starts from physical button contact
        if (rawState && !_debouncedState) {
            _pressStartTime = currentTime;
            _longHoldFired = false;
        }

        _lastRawState = rawState;
    }

    // If stable for debounce period, update debounced state
    if ((currentTime - _lastDebounceTime) >= DEBOUNCE_MS) {
        if (rawState != _debouncedState) {
            _lastDebouncedState = _debouncedState;
            _debouncedState = rawState;
            return true;  // State changed
        }
    }

    return false;  // No change
}

ButtonController::Event ButtonController::update() {
    unsigned long currentTime = _timeSource.millis();
    Event event = Event::NONE;

    // Check for state change
    bool stateChanged = debounceButton();

    // Handle button press (rising edge)
    // Note: _pressStartTime is already set in debounceButton() on first raw press
    if (stateChanged && _debouncedState && !_lastDebouncedState) {
        _pressCount++;
    }

    // Handle button release (falling edge)
    if (stateChanged && !_debouncedState && _lastDebouncedState) {
        unsigned long pressDuration = currentTime - _pressStartTime;

        // Only count as a press if it wasn't a long hold
        if (!_longHoldFired && pressDuration < LONG_HOLD_MS) {
            _lastReleaseTime = currentTime;

            if (_pressCount >= 2) {
                // Double press detected
                event = Event::DOUBLE_PRESS;
                _pressCount = 0;
                _waitingForDoublePress = false;
            } else {
                // First press - wait for potential second press
                _waitingForDoublePress = true;
            }
        } else {
            // Was a long hold, reset press count
            _pressCount = 0;
            _waitingForDoublePress = false;
        }
    }

    // Check for long hold while button is still pressed
    if (_debouncedState && !_longHoldFired) {
        unsigned long pressDuration = currentTime - _pressStartTime;
        if (pressDuration >= LONG_HOLD_MS) {
            _longHoldFired = true;
            _pressCount = 0;
            _waitingForDoublePress = false;
            event = Event::LONG_HOLD;
        }
    }

    // Check if double-press window expired (single press confirmed)
    if (_waitingForDoublePress && !_debouncedState) {
        unsigned long timeSinceRelease = currentTime - _lastReleaseTime;
        if (timeSinceRelease >= DOUBLE_PRESS_WINDOW_MS) {
            _waitingForDoublePress = false;
            _pressCount = 0;
            event = Event::SINGLE_PRESS;
        }
    }

    // Update last state for next iteration
    _lastDebouncedState = _debouncedState;

    return event;
}

bool ButtonController::isPressed() const {
    return _debouncedState;
}
