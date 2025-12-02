#include "SessionManager.h"

SessionManager::SessionManager(WaveformController& waveformController,
                               const ITimeSource& timeSource)
    : _waveformController(waveformController)
    , _timeSource(timeSource)
    , _startTime(0)
    , _sessionDuration(DEFAULT_SESSION_DURATION_MS)
    , _isRunning(false) {
}

void SessionManager::start() {
    _startTime = _timeSource.millis();
    _sessionDuration = DEFAULT_SESSION_DURATION_MS;  // Reset to default on new session
    _isRunning = true;
    _waveformController.begin();  // Initialize waveform controller
}

bool SessionManager::update() {
    if (!_isRunning) {
        return false;
    }

    const unsigned long currentTime = _timeSource.millis();
    const unsigned long elapsedTime = currentTime - _startTime;

    if (elapsedTime > _sessionDuration) {
        terminateSession();
        return false;
    }

    _waveformController.update();
    return true;
}

void SessionManager::stop() {
    if (_isRunning) {
        _isRunning = false;
        _waveformController.forceInactive();
        // Note: Unlike terminateSession(), we do NOT enter idle loop
        // This allows the session to be restarted
    }
}

bool SessionManager::isActive() const {
    if (!_isRunning) {
        return false;
    }

    const unsigned long currentTime = _timeSource.millis();
    const unsigned long elapsedTime = currentTime - _startTime;
    return elapsedTime <= _sessionDuration;
}

unsigned long SessionManager::getRemainingTime() const {
    if (!_isRunning) {
        return 0;
    }

    const unsigned long currentTime = _timeSource.millis();
    const unsigned long elapsedTime = currentTime - _startTime;

    if (elapsedTime >= _sessionDuration) {
        return 0;
    }

    return _sessionDuration - elapsedTime;
}

bool SessionManager::extendTime() {
    if (!_isRunning) {
        return false;
    }

    // Calculate new duration
    unsigned long newDuration = _sessionDuration + TIME_EXTENSION_MS;

    // Check if we're already at max
    if (_sessionDuration >= MAX_SESSION_DURATION_MS) {
        return false;
    }

    // Cap at max duration
    if (newDuration > MAX_SESSION_DURATION_MS) {
        newDuration = MAX_SESSION_DURATION_MS;
    }

    _sessionDuration = newDuration;
    return true;
}

unsigned long SessionManager::getSessionDuration() const {
    return _sessionDuration;
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