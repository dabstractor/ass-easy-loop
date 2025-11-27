#include "SessionManager.h"

SessionManager::SessionManager(WaveformController& waveformController,
                               const ITimeSource& timeSource)
    : _waveformController(waveformController)
    , _timeSource(timeSource)
    , _startTime(0)
    , _isRunning(false) {
}

void SessionManager::start() {
    _startTime = _timeSource.millis();
    _isRunning = true;
}

bool SessionManager::update() {
    if (!_isRunning) {
        return false;
    }

    const unsigned long currentTime = _timeSource.millis();
    const unsigned long elapsedTime = currentTime - _startTime;

    if (elapsedTime > MAX_SESSION_DURATION_MS) {
        terminateSession();
        return false;
    }

    _waveformController.update();
    return true;
}

bool SessionManager::isActive() const {
    if (!_isRunning) {
        return false;
    }

    const unsigned long currentTime = _timeSource.millis();
    const unsigned long elapsedTime = currentTime - _startTime;
    return elapsedTime <= MAX_SESSION_DURATION_MS;
}

unsigned long SessionManager::getRemainingTime() const {
    if (!_isRunning) {
        return 0;
    }

    const unsigned long currentTime = _timeSource.millis();
    const unsigned long elapsedTime = currentTime - _startTime;

    if (elapsedTime >= MAX_SESSION_DURATION_MS) {
        return 0;
    }

    return MAX_SESSION_DURATION_MS - elapsedTime;
}

void SessionManager::terminateSession() {
    _isRunning = false;
    // Ensure CoilDriver is OFF as per safety requirements
    _waveformController.forceInactive();
    idleLoop();
}

void SessionManager::idleLoop() const {
    while (true) {
        delay(1000);
    }
}