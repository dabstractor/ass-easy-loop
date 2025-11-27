# PRP-002: CoilDriver HAL Implementation

## Metadata
| Field | Value |
|-------|-------|
| PRP ID | 002 |
| Task Reference | P1.M2.T1.S1 |
| Story Points | 1 |
| Status | Ready for Implementation |
| Confidence Score | 9/10 |

---

## Goal

### Feature Goal
Create a safe, testable Hardware Abstraction Layer (HAL) class for controlling the MOSFET-driven magnetic coil on GPIO 15, implementing fail-safe patterns that ensure the coil is never accidentally activated.

### Deliverable
Two files:
- `src/hal/CoilDriver.h` - Header with class declaration
- `src/hal/CoilDriver.cpp` - Implementation

### Success Definition
- `CoilDriver` class compiles without errors
- `begin()` method sets GPIO 15 as OUTPUT and immediately writes LOW
- `setActive(true)` writes HIGH, `setActive(false)` writes LOW
- Destructor forces GPIO LOW for safety
- Class is non-copyable (hardware resource protection)
- `pio run` compiles successfully with the new files

---

## Context

### PRD Requirements (Section 2.2 & 4.1)
```yaml
gpio_assignment: GPIO 15 (MOSFET TRIG/SIG)
mosfet_driver: IRF520 MOSFET Driver Module
coil_specs:
  resistance: ~4 Ohms
  wire: 22AWG Enameled Copper
  geometry: 7.5" diameter, 12 coils wide, 11 layers deep
flyback_protection: 1N4007 or 1N5408 diode (hardware, not firmware)
waveform_duty:
  pulse_on: 2ms (GPIO 15 HIGH)
  pulse_off: 98ms (GPIO 15 LOW)
  frequency: 10Hz
safety_requirement: "Force all outputs LOW" on stop (PRD Section 4.2)
```

### Hardware Context
```yaml
controller: RP2040-Zero (Waveshare)
gpio_voltage: 3.3V logic (NOT 5V tolerant)
irp520_compatibility: Works with 3.3V control signals
peak_coil_current: 0.925A (3.7V / 4 Ohms)
duty_cycle: 2% (safe for continuous operation)
```

### Research References
```yaml
local_research:
  - plan/P1.M2.T1.S1/research/rp2040-gpio-patterns.md
  - plan/P1.M2.T1.S1/research/mosfet-control-safety.md
  - plan/P1.M2.T1.S1/research/platformio-testing.md

external_docs:
  - url: https://arduino-pico.readthedocs.io/en/latest/digital.html
    section: "Digital I/O - pinMode, digitalWrite"
  - url: https://github.com/earlephilhower/arduino-pico/blob/master/cores/rp2040/wiring_digital.cpp
    section: "Source implementation of GPIO functions"
  - url: https://electropeak.com/learn/interfacing-irf520-mosfet-driver-module-switch-button-hcmodu0083-with-arduino/
    section: "IRF520 control patterns"

library_registry:
  - note: "No external libraries required - uses Arduino core GPIO functions"
```

### Key Technical Decisions
```yaml
initialization_pattern:
  selected: "Deferred initialization (constructor + begin())"
  rationale: "Constructor runs before setup(), hardware may not be stable"
  alternative: "Constructor-only init (unsafe for embedded)"

safety_approach:
  selected: "RAII pattern with destructor forcing LOW"
  rationale: "Guarantees safe state on scope exit or program termination"
  gotcha: "Destructor is noexcept by default in C++11+"

copyability:
  selected: "Delete copy constructor and assignment operator"
  rationale: "GPIO pin is unique hardware resource, prevent double-control"

pin_storage:
  selected: "const uint8_t member variable"
  rationale: "Pin number is immutable after construction"
```

---

## Implementation Tasks

### Task 1: Create HAL directory structure
**Action:** Create directory

**Directory:** `src/hal/`

**Command:**
```bash
mkdir -p src/hal
```

---

### Task 2: Create CoilDriver.h header file
**Action:** Create new file

**File:** `src/hal/CoilDriver.h`

**Content:**
```cpp
#ifndef COIL_DRIVER_H
#define COIL_DRIVER_H

#include <Arduino.h>

/**
 * @brief Safe wrapper for MOSFET-controlled magnetic coil.
 *
 * Controls GPIO pin connected to IRF520 MOSFET driver module.
 * Implements fail-safe patterns:
 * - begin() sets OUTPUT mode and immediately writes LOW
 * - Destructor forces LOW state for safety
 * - Non-copyable (hardware resource protection)
 *
 * @note Flyback diode protection is handled in hardware (PRD Section 2.1)
 */
class CoilDriver {
public:
    /**
     * @brief Construct CoilDriver with specified GPIO pin.
     * @param pin GPIO pin number connected to MOSFET TRIG/SIG (default: 15)
     * @note Does NOT configure hardware - call begin() in setup()
     */
    explicit CoilDriver(uint8_t pin = 15);

    /**
     * @brief Initialize GPIO pin as OUTPUT and set LOW (safe state).
     * @note Must be called in setup() before any setActive() calls
     */
    void begin();

    /**
     * @brief Control coil activation state.
     * @param active true = energize coil (HIGH), false = de-energize (LOW)
     */
    void setActive(bool active);

    /**
     * @brief Force coil to safe state (LOW) on destruction.
     */
    ~CoilDriver();

    // Delete copy operations - hardware resource is unique
    CoilDriver(const CoilDriver&) = delete;
    CoilDriver& operator=(const CoilDriver&) = delete;

private:
    const uint8_t _pin;  ///< GPIO pin number (immutable)
};

#endif // COIL_DRIVER_H
```

**Placement:** `src/hal/CoilDriver.h`

**Naming Convention:**
- Class name: PascalCase (`CoilDriver`)
- Private members: underscore prefix (`_pin`)
- Header guard: `COIL_DRIVER_H`

---

### Task 3: Create CoilDriver.cpp implementation file
**Action:** Create new file

**File:** `src/hal/CoilDriver.cpp`

**Content:**
```cpp
#include "CoilDriver.h"

CoilDriver::CoilDriver(uint8_t pin)
    : _pin(pin) {
    // Constructor: store pin only, do NOT touch hardware
    // Hardware initialization deferred to begin()
}

void CoilDriver::begin() {
    // Configure pin as OUTPUT
    pinMode(_pin, OUTPUT);

    // Immediately set LOW for safety
    // This ensures coil is OFF before any other code runs
    digitalWrite(_pin, LOW);
}

void CoilDriver::setActive(bool active) {
    digitalWrite(_pin, active ? HIGH : LOW);
}

CoilDriver::~CoilDriver() {
    // Force safe state on destruction
    // Ensures coil is OFF when object goes out of scope
    digitalWrite(_pin, LOW);
}
```

**Placement:** `src/hal/CoilDriver.cpp`

---

### Task 4: Update main.cpp to include CoilDriver (verification only)
**Action:** Verify compilation by adding include

**File:** `src/main.cpp`

**Changes:** Add include at top of file (after `#include <Arduino.h>`):
```cpp
#include "hal/CoilDriver.h"
```

**Note:** Do NOT instantiate or use CoilDriver yet - that happens in P1.M4.T1.S1. This task only verifies the class compiles.

---

## Validation Gates

### Gate 1: Directory Structure Verification
```bash
ls -la src/hal/
```
**Expected:** Directory exists with `CoilDriver.h` and `CoilDriver.cpp`

### Gate 2: Compilation Test
```bash
pio run
```
**Expected:**
```
Building .pio/build/rp2040_zero/firmware.uf2
=============== [SUCCESS] ===============
```

### Gate 3: Include Verification
```bash
grep -r "CoilDriver" src/
```
**Expected:** Shows include in `main.cpp` and class definition in `hal/` files

### Gate 4: Header Guard Check
```bash
head -5 src/hal/CoilDriver.h
```
**Expected:** Shows `#ifndef COIL_DRIVER_H` / `#define COIL_DRIVER_H`

---

## Final Validation Checklist

- [ ] `src/hal/` directory exists
- [ ] `src/hal/CoilDriver.h` exists with class declaration
- [ ] `src/hal/CoilDriver.cpp` exists with implementation
- [ ] Header guard present (`COIL_DRIVER_H`)
- [ ] Constructor takes `uint8_t pin` parameter with default value 15
- [ ] `begin()` method calls `pinMode()` then `digitalWrite(LOW)`
- [ ] `setActive(bool)` method calls `digitalWrite()` with HIGH/LOW
- [ ] Destructor calls `digitalWrite(_pin, LOW)`
- [ ] Copy constructor and assignment operator are deleted
- [ ] `src/main.cpp` includes `"hal/CoilDriver.h"`
- [ ] `pio run` compiles successfully

---

## Gotchas and Troubleshooting

### Gotcha 1: Include Path Issues
**Issue:** Compiler cannot find `"hal/CoilDriver.h"`
**Solution:** PlatformIO automatically includes `src/` in the include path. Use relative path from `src/`:
```cpp
#include "hal/CoilDriver.h"  // Correct
#include "CoilDriver.h"      // Wrong - not in src/ root
```

### Gotcha 2: Destructor Called Before setup()
**Issue:** Global CoilDriver object destructor runs at shutdown, not during normal operation
**Solution:** This is expected behavior. The destructor provides safety guarantee, not runtime control. For normal operation, use `setActive(false)`.

### Gotcha 3: begin() Not Called
**Issue:** `setActive()` called before `begin()` - GPIO may be in undefined state
**Solution:** Always call `begin()` in `setup()` function. Consider adding a guard:
```cpp
void setActive(bool active) {
    // Could add: if (!_initialized) return;
    digitalWrite(_pin, active ? HIGH : LOW);
}
```
**Note:** Guard omitted for simplicity per task scope. SessionManager (P1.M3.T2.S1) will enforce initialization order.

### Gotcha 4: 3.3V Logic Level
**Issue:** RP2040 outputs 3.3V, but IRF520 may prefer 5V gate drive
**Solution:** IRF520 module typically has level-shifting circuitry. 3.3V is sufficient for the module's control input. If direct MOSFET drive, consider IRL520 (logic-level variant).

---

## Dependencies

### Upstream Dependencies
- **P1.M1.T1.S1** (Complete): PlatformIO configuration - provides build system and Arduino framework

### Downstream Dependencies
- **P1.M2.T2.S1**: FeedbackDriver HAL - similar HAL pattern
- **P1.M3.T1.S1**: WaveformController - will use CoilDriver for 10Hz pulse generation
- **P1.M4.T1.S1**: main.cpp integration - will instantiate and initialize CoilDriver

---

## Notes for Implementation Agent

1. **Scope Boundary:** This task creates ONLY the CoilDriver class files. Do NOT modify `main.cpp` beyond adding the include statement for compilation verification.

2. **No Unit Tests Yet:** Unit testing setup (with ArduinoFake) is a separate task. This task focuses on the production code.

3. **Pattern Consistency:** This HAL pattern (constructor + begin() + destructor safety) should be followed by FeedbackDriver in P1.M2.T2.S1.

4. **Default Pin Value:** The default parameter `pin = 15` matches PRD Section 2.2 GPIO assignment. This allows `CoilDriver driver;` without explicit pin in main.cpp.

5. **No Over-Engineering:** Keep the implementation minimal:
   - No state tracking boolean
   - No guard in setActive()
   - No logging/debug output
   - No PWM support (not needed for 10Hz on/off)

6. **Research Available:** Comprehensive research docs in `plan/P1.M2.T1.S1/research/` if clarification needed on GPIO patterns, MOSFET safety, or testing approaches.
