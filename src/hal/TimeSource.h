#ifndef TIME_SOURCE_H
#define TIME_SOURCE_H

#include <Arduino.h>

/**
 * @brief Interface for time abstraction to enable unit testing.
 *
 * Provides a clean abstraction over Arduino's millis() function,
 * allowing mocking of time in unit tests without requiring actual delays.
 * This interface is critical for testing time-dependent logic in the
 * pEMF pulse generation system.
 */
class ITimeSource {
public:
    /**
     * @brief Get the current time in milliseconds.
     * @return Current time in milliseconds since boot
     *
     * Pure virtual method that must be implemented by concrete classes.
     * The production implementation should wrap Arduino's millis(),
     * while test implementations can return controlled values.
     */
    virtual unsigned long millis() const = 0;

    /**
     * @brief Virtual destructor for proper polymorphic cleanup.
     */
    virtual ~ITimeSource() = default;
};

/**
 * @brief Production implementation of ITimeSource using Arduino's millis().
 *
 * This is the standard time source used in production firmware.
 * Simply delegates to Arduino's built-in millis() function.
 */
class ArduinoTimeSource : public ITimeSource {
public:
    /**
     * @brief Get the current time using Arduino's millis().
     * @return Current time in milliseconds from Arduino's internal counter
     */
    unsigned long millis() const override {
        return ::millis();
    }
};

#endif // TIME_SOURCE_H