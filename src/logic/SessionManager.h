#ifndef SESSION_MANAGER_H
#define SESSION_MANAGER_H

#include "../hal/TimeSource.h"
#include "WaveformController.h"

/**
 * @brief Session timeout manager for therapeutic pEMF safety constraints.
 *
 * Enforces a maximum 15-minute session duration as a safety mechanism.
 * Tracks session start time and automatically terminates therapy sessions
 * when the time limit is exceeded, ensuring the CoilDriver is disabled
 * and entering an idle state to prevent continued operation.
 *
 * Uses dependency injection pattern for testability:
 * - WaveformController for therapy generation control
 * - ITimeSource for time abstraction
 */
class SessionManager {
public:
    /**
     * @brief Maximum session duration in milliseconds (15 minutes).
     */
    static constexpr unsigned long MAX_SESSION_DURATION_MS = 900000UL;

    /**
     * @brief Construct SessionManager with required dependencies.
     * @param waveformController Reference to WaveformController instance
     * @param timeSource Reference to ITimeSource instance
     */
    SessionManager(WaveformController& waveformController,
                   const ITimeSource& timeSource);

    /**
     * @brief Start a new therapy session.
     *
     * Records the start time and enables session tracking.
     * Call this when beginning a therapy session.
     */
    void start();

    /**
     * @brief Update session state - call in main loop.
     *
     * Checks if the current session has exceeded the 15-minute limit.
     * If within limit, delegates to WaveformController::update().
     * If limit exceeded, ensures CoilDriver is OFF and enters idle state.
     *
     * @return true if session is active and within time limit, false if terminated
     */
    bool update();

    /**
     * @brief Check if session is currently active.
     * @return true if session is running and within time limit
     */
    bool isActive() const;

    /**
     * @brief Get remaining time in session.
     * @return Remaining milliseconds, or 0 if no active session
     */
    unsigned long getRemainingTime() const;

private:
    WaveformController& _waveformController;  ///< Reference to therapy controller
    const ITimeSource& _timeSource;           ///< Reference to time abstraction

    unsigned long _startTime;                  ///< Timestamp when session started
    bool _isRunning;                          ///< Session running state

    /**
     * @brief Force termination of the current session.
     *
     * Ensures CoilDriver is OFF and enters idle state.
     * Called when time limit is exceeded.
     */
    void terminateSession();

    /**
     * @brief Enter idle state after session termination.
     *
     * Enters infinite loop to prevent further operation.
     */
    void idleLoop() const;
};

#endif // SESSION_MANAGER_H