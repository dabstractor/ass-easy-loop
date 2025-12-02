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

    // Update feedback driver for continuous pastel cycling
    // We do this regardless of pulse state
    _feedbackDriver.update();

    const unsigned long currentTime = _timeSource.millis();
    const unsigned long elapsedInCycle = currentTime - _cycleStartTime;

    if (_isActive) {
        // Currently in ON state - check if it's time to turn OFF
        if (elapsedInCycle >= Config::ON_DURATION_MS) {
            setInactiveState();
        }
    } else {
        // Currently in OFF state - check if it's time to turn ON
        if (elapsedInCycle >= Config::PERIOD_MS) {
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
    // Feedback is now handled continuously in update()
}

void WaveformController::setInactiveState() {
    _isActive = false;
    _coilDriver.setActive(false);
    // Feedback is now handled continuously in update()
}

void WaveformController::forceInactive() {
    _isRunning = false;
    _isActive = false;
    _coilDriver.setActive(false);
    // We might want to turn off the LED here, or keep it running if charging
    // But for now, let's stick to the existing pattern but we can't call indicateActive anymore
    // Let's just destruct/reconstruct if we really want to turn it off, or maybe add a stop() to FeedbackDriver
    // For now, let's just leave it be as the user asked for "While the device is running"
}
