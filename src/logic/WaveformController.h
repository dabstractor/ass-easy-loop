#ifndef WAVEFORM_CONTROLLER_H
#define WAVEFORM_CONTROLLER_H

#include "../hal/CoilDriver.h"
#include "../hal/FeedbackDriver.h"
#include "../hal/TimeSource.h"

/**
 * @brief Controller for generating 10Hz therapeutic pEMF waveform.
 *
 * Manages the timing logic for producing a 10Hz square wave signal
 * with 2ms ON and 98ms OFF periods (100ms total cycle). Coordinates
 * coil activation with synchronized audio/visual feedback.
 *
 * Uses dependency injection pattern for testability:
 * - CoilDriver for magnetic coil control
 * - FeedbackDriver for user feedback
 * - ITimeSource for time abstraction
 */
class WaveformController {
public:
    /**
     * @brief Construct WaveformController with required dependencies.
     * @param coilDriver Reference to CoilDriver instance
     * @param feedbackDriver Reference to FeedbackDriver instance
     * @param timeSource Reference to ITimeSource instance
     */
    WaveformController(CoilDriver& coilDriver,
                      FeedbackDriver& feedbackDriver,
                      const ITimeSource& timeSource);

    /**
     * @brief Initialize controller state.
     * @note Call once in setup() before starting update() loop
     */
    void begin();

    /**
     * @brief Update controller state - call in main loop.
     *
     * Non-blocking implementation that checks elapsed time and
     * updates coil/feedback state according to 10Hz timing:
     * - ON for 2ms (2% duty cycle)
     * - OFF for 98ms
     * - Total period: 100ms (10Hz)
     */
    void update();

private:
    static constexpr unsigned long PERIOD_MS = 100;    ///< 10Hz = 100ms period
    static constexpr unsigned long ON_DURATION_MS = 2; ///< 2ms ON time
    static constexpr unsigned long OFF_DURATION_MS = 98; ///< 98ms OFF time

    CoilDriver& _coilDriver;              ///< Reference to coil control
    FeedbackDriver& _feedbackDriver;      ///< Reference to feedback control
    const ITimeSource& _timeSource;       ///< Reference to time abstraction

    unsigned long _cycleStartTime;         ///< Timestamp when current cycle started
    bool _isActive;                        ///< Current output state (true=ON, false=OFF)
    bool _isRunning;                       ///< Controller running state

    /**
     * @brief Start a new timing cycle.
     */
    void startCycle();

    /**
     * @brief Set outputs to active state (coil ON, feedback ON).
     */
    void setActiveState();

    /**
     * @brief Set outputs to inactive state (coil OFF, feedback OFF).
     */
    void setInactiveState();
};

#endif // WAVEFORM_CONTROLLER_H