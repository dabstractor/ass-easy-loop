#include "WaveformController.h"

WaveformController::WaveformController(CoilDriver& coilDriver,
                                      FeedbackDriver& feedbackDriver,
                                      const ITimeSource& timeSource)
    : _coilDriver(coilDriver)
    , _feedbackDriver(feedbackDriver)
    , _timeSource(timeSource)
    , _cycleStartTime(0)
    , _isActive(false)
    , _isRunning(false) {
}

void WaveformController::begin() {
    _isRunning = true;
    _isActive = false;
    _cycleStartTime = _timeSource.millis();

    // Start with inactive state for safety
    setInactiveState();
}

void WaveformController::update() {
    if (!_isRunning) {
        return;
    }

    const unsigned long currentTime = _timeSource.millis();
    const unsigned long elapsedInCycle = currentTime - _cycleStartTime;

    if (_isActive) {
        // Currently in ON state - check if it's time to turn OFF
        if (elapsedInCycle >= ON_DURATION_MS) {
            setInactiveState();
        }
    } else {
        // Currently in OFF state - check if it's time to turn ON
        if (elapsedInCycle >= PERIOD_MS) {
            // Start new cycle
            startCycle();
            setActiveState();
        }
    }
}

void WaveformController::startCycle() {
    _cycleStartTime = _timeSource.millis();
}

void WaveformController::setActiveState() {
    _isActive = true;
    _coilDriver.setActive(true);
    _feedbackDriver.indicateActive(true);
}

void WaveformController::setInactiveState() {
    _isActive = false;
    _coilDriver.setActive(false);
    _feedbackDriver.indicateActive(false);
}

void WaveformController::forceInactive() {
    _isRunning = false;
    _isActive = false;
    _coilDriver.setActive(false);
    _feedbackDriver.indicateActive(false);
}