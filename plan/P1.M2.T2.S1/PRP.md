# PRP-P1.M2.T2.S1: FeedbackDriver HAL Implementation

## Metadata
| Field | Value |
|-------|-------|
| PRP ID | P1.M2.T2.S1 |
| Task Reference | P1.M2.T2.S1 |
| Story Points | 2 |
| Status | Ready for Implementation |
| Confidence Score | 9/10 |

---

## Goal

### Feature Goal
Create a `FeedbackDriver` HAL class that encapsulates the onboard WS2812 NeoPixel LED and piezo buzzer, providing synchronized audio/visual feedback for the 10Hz pEMF therapeutic waveform.

### Deliverable
- `src/hal/FeedbackDriver.h` - Class declaration with Doxygen documentation
- `src/hal/FeedbackDriver.cpp` - Implementation file

### Success Definition
1. Class compiles without errors alongside existing CoilDriver
2. `begin()` initializes both NeoPixel and buzzer to safe (OFF) states
3. `indicateActive(true)` turns buzzer HIGH and LED green (low brightness)
4. `indicateActive(false)` turns buzzer LOW and LED OFF
5. Build passes: `pio run -e rp2040_zero`

---

## Context

### PRD Requirements
```yaml
neopixel:
  type: WS2812 RGB LED (onboard RP2040-Zero)
  gpio_pin: 16
  library: Adafruit NeoPixel
  active_state: Green, Low Brightness
  inactive_state: OFF

buzzer:
  type: Piezo Buzzer (Active or Passive - GPIO driven)
  gpio_pin: 14
  active_state: HIGH
  inactive_state: LOW

waveform_timing:
  frequency: 10 Hz
  period: 100 ms
  pulse_active: 2 ms (LED Green + Buzzer HIGH)
  pulse_rest: 98 ms (LED OFF + Buzzer LOW)

therapeutic_session:
  duration: 15 minutes
  led_flash_rate: 10 Hz (Green)
```

### Research References
```yaml
external_docs:
  - url: "https://learn.adafruit.com/adafruit-neopixel-uberguide/arduino-library-use"
    section: "Arduino Library Use - setPixelColor, setBrightness, show"
  - url: "https://adafruit.github.io/Adafruit_NeoPixel/html/class_adafruit___neo_pixel.html"
    section: "API Class Reference"
  - url: "https://arduino-pico.readthedocs.io/en/latest/digital.html"
    section: "RP2040 Digital I/O"

local_research:
  - ./research/NEOPIXEL_API_REFERENCE.md
  - ./research/BUZZER_CONTROL_PATTERNS.md
  - ../P1.M1.T1.S1/research/adafruit-neopixel.md
  - ../P1.M2.T1.S1/research/rp2040-gpio-patterns.md

library_registry:
  - url: "https://registry.platformio.org/libraries/adafruit/Adafruit%20NeoPixel"
    note: "Already in platformio.ini: adafruit/Adafruit NeoPixel@^1.12.0"
```

### Key Technical Decisions
```yaml
led_gpio_pin:
  selected: "GPIO 16"
  rationale: "RP2040-Zero onboard WS2812 LED is hardwired to GPIO 16"
  alternative: "None - hardware fixed"

buzzer_gpio_pin:
  selected: "GPIO 14"
  rationale: "PRD Section 2.2 specifies GPIO 14 for buzzer"
  alternative: "None - per PRD specification"

neopixel_color_active:
  selected: "Green (0, 255, 0) at Low Brightness"
  rationale: "PRD Section 4.1 specifies Green for active state"
  alternative: "Could use packed color constant"

brightness_level:
  selected: "30 out of 255 (~12%)"
  rationale: "Research indicates onboard LEDs are very bright; 30 is comfortable"
  alternative: "50 (still visible, slightly brighter)"

class_pattern:
  selected: "Match CoilDriver pattern (begin/setActive style -> indicateActive)"
  rationale: "Consistency with existing HAL codebase"
  alternative: "Separate on()/off() methods"
```

---

## Implementation Tasks

### Task 1: Create FeedbackDriver Header File
**Action:** Create

**File:** `src/hal/FeedbackDriver.h`

**Content:**
```cpp
#ifndef FEEDBACK_DRIVER_H
#define FEEDBACK_DRIVER_H

#include <Arduino.h>
#include <Adafruit_NeoPixel.h>

/**
 * @brief Safe wrapper for synchronous Audio/Visual feedback.
 *
 * Controls onboard WS2812 NeoPixel LED and piezo buzzer for
 * therapeutic session feedback at 10Hz.
 *
 * Implements fail-safe patterns:
 * - begin() initializes both outputs to OFF state
 * - Destructor forces both outputs to OFF
 * - Non-copyable (hardware resource protection)
 *
 * @note PRD Section 2.1 (NeoPixel GPIO 16) & 2.2 (Buzzer GPIO 14)
 */
class FeedbackDriver {
public:
    /**
     * @brief Construct FeedbackDriver with specified GPIO pins.
     * @param buzzerPin GPIO pin for piezo buzzer (default: 14)
     * @param neoPixelPin GPIO pin for WS2812 LED (default: 16)
     * @note Does NOT configure hardware - call begin() in setup()
     */
    explicit FeedbackDriver(uint8_t buzzerPin = 14, uint8_t neoPixelPin = 16);

    /**
     * @brief Initialize buzzer and NeoPixel to safe OFF state.
     * @note Must be called in setup() before any indicateActive() calls
     */
    void begin();

    /**
     * @brief Control synchronized feedback state.
     * @param isActive true = Buzzer HIGH + LED Green, false = both OFF
     */
    void indicateActive(bool isActive);

    /**
     * @brief Force all outputs to safe state (OFF) on destruction.
     */
    ~FeedbackDriver();

    // Delete copy operations - hardware resource is unique
    FeedbackDriver(const FeedbackDriver&) = delete;
    FeedbackDriver& operator=(const FeedbackDriver&) = delete;

private:
    const uint8_t _buzzerPin;          ///< GPIO pin for buzzer (immutable)
    const uint8_t _neoPixelPin;        ///< GPIO pin for NeoPixel (immutable)
    Adafruit_NeoPixel _pixel;          ///< NeoPixel driver instance

    static constexpr uint8_t LED_COUNT = 1;           ///< Single onboard LED
    static constexpr uint8_t BRIGHTNESS = 30;         ///< ~12% brightness
    static constexpr uint8_t GREEN_R = 0;             ///< Green color - Red
    static constexpr uint8_t GREEN_G = 255;           ///< Green color - Green
    static constexpr uint8_t GREEN_B = 0;             ///< Green color - Blue
};

#endif // FEEDBACK_DRIVER_H
```

**Placement:** New file in `src/hal/` directory alongside `CoilDriver.h`

**Naming Convention:** PascalCase class name, snake_case for private members with underscore prefix

---

### Task 2: Create FeedbackDriver Implementation File
**Action:** Create

**File:** `src/hal/FeedbackDriver.cpp`

**Content:**
```cpp
#include "FeedbackDriver.h"

FeedbackDriver::FeedbackDriver(uint8_t buzzerPin, uint8_t neoPixelPin)
    : _buzzerPin(buzzerPin),
      _neoPixelPin(neoPixelPin),
      _pixel(LED_COUNT, neoPixelPin, NEO_GRB + NEO_KHZ800) {
    // Constructor: store pins and create NeoPixel object
    // Hardware initialization deferred to begin()
}

void FeedbackDriver::begin() {
    // Initialize buzzer pin: pre-set LOW before configuring as OUTPUT
    // This prevents any transient HIGH state during startup
    digitalWrite(_buzzerPin, LOW);
    pinMode(_buzzerPin, OUTPUT);

    // Initialize NeoPixel
    _pixel.begin();
    _pixel.setBrightness(BRIGHTNESS);
    _pixel.clear();
    _pixel.show();  // Apply OFF state to LED
}

void FeedbackDriver::indicateActive(bool isActive) {
    if (isActive) {
        // Active state: Buzzer ON, LED Green
        digitalWrite(_buzzerPin, HIGH);
        _pixel.setPixelColor(0, GREEN_R, GREEN_G, GREEN_B);
    } else {
        // Inactive state: Buzzer OFF, LED OFF
        digitalWrite(_buzzerPin, LOW);
        _pixel.setPixelColor(0, 0, 0, 0);
    }
    _pixel.show();  // Push color change to LED
}

FeedbackDriver::~FeedbackDriver() {
    // Force safe state on destruction
    digitalWrite(_buzzerPin, LOW);
    _pixel.clear();
    _pixel.show();
}
```

**Placement:** New file in `src/hal/` directory alongside `CoilDriver.cpp`

**Naming Convention:** Match header file name

---

## Validation Gates

### Gate 1: Compilation Check
```bash
pio run -e rp2040_zero
```
**Expected:** Build completes with `SUCCESS` status, no errors. Warnings about unused variables are acceptable but should be noted.

### Gate 2: File Structure Verification
```bash
ls -la src/hal/
```
**Expected:** Should show:
- `CoilDriver.h`
- `CoilDriver.cpp`
- `FeedbackDriver.h`
- `FeedbackDriver.cpp`

### Gate 3: Include Verification
```bash
grep -l "FeedbackDriver" src/hal/*.h src/hal/*.cpp
```
**Expected:** Returns both `FeedbackDriver.h` and `FeedbackDriver.cpp`

---

## Final Validation Checklist

- [ ] `FeedbackDriver.h` exists at `src/hal/FeedbackDriver.h`
- [ ] `FeedbackDriver.cpp` exists at `src/hal/FeedbackDriver.cpp`
- [ ] Header includes `<Arduino.h>` and `<Adafruit_NeoPixel.h>`
- [ ] Class has `begin()` method that initializes both outputs
- [ ] Class has `indicateActive(bool)` method
- [ ] Constructor accepts `buzzerPin` (default 14) and `neoPixelPin` (default 16)
- [ ] Destructor forces both outputs to OFF state
- [ ] Class is non-copyable (deleted copy constructor and assignment)
- [ ] `pio run -e rp2040_zero` compiles successfully
- [ ] Code follows existing CoilDriver naming/style conventions

---

## Gotchas and Troubleshooting

### Gotcha 1: NeoPixel Library Include Path
**Issue:** `Adafruit_NeoPixel.h` not found during compilation
**Solution:** Verify `lib_deps` in `platformio.ini` contains `adafruit/Adafruit NeoPixel@^1.12.0` (already present)

### Gotcha 2: show() Must Be Called
**Issue:** LED color doesn't change after `setPixelColor()`
**Solution:** Always call `_pixel.show()` after setting colors - this transmits data to the LED

### Gotcha 3: Brightness is Lossy
**Issue:** Colors look wrong after multiple brightness changes
**Solution:** Call `setBrightness()` only once in `begin()`, not during animation. This PRP design does this correctly.

### Gotcha 4: Buzzer Startup Click
**Issue:** Buzzer makes brief noise at startup
**Solution:** Pre-set `digitalWrite(pin, LOW)` BEFORE `pinMode(pin, OUTPUT)`. This is implemented in the `begin()` method.

### Gotcha 5: GPIO 16 is Hardwired
**Issue:** LED doesn't work on different pin
**Solution:** RP2040-Zero onboard LED is physically connected to GPIO 16. This cannot be changed.

---

## Dependencies

### Upstream Dependencies
- **P1.M1.T1.S1** (PlatformIO Configuration): Must be complete for build to work
- **platformio.ini**: Must have `adafruit/Adafruit NeoPixel@^1.12.0` in `lib_deps` (verified present)

### Downstream Dependencies
- **P1.M2.T3** (Timer/Scheduler): Will use `FeedbackDriver::indicateActive()` in 10Hz loop
- **P1.M3** (Session Controller): Will instantiate and manage FeedbackDriver lifecycle

---

## Notes for Implementation Agent

1. **Follow CoilDriver Pattern Exactly**: The existing `src/hal/CoilDriver.h` and `.cpp` files are the gold standard for HAL class structure in this project. Mirror the documentation style, constructor pattern, and safety mechanisms.

2. **NeoPixel Object Lifetime**: The `Adafruit_NeoPixel` object must be a class member (not a pointer) to ensure proper lifecycle. The constructor initializer list creates it with the correct parameters.

3. **Color Format**: Use `setPixelColor(0, R, G, B)` with three parameters, not the packed 32-bit format. This is clearer and matches PRD color specifications.

4. **Don't Over-Engineer**: This is a simple wrapper. Don't add:
   - Animation features
   - Multiple LED support
   - Tone generation
   - State tracking beyond what's needed

5. **Test After Implementation**: After creating files, verify compilation immediately with `pio run -e rp2040_zero`. Fix any errors before marking complete.

6. **Reference Files to Read Before Starting**:
   - `src/hal/CoilDriver.h` - Pattern to follow
   - `src/hal/CoilDriver.cpp` - Implementation style
   - `platformio.ini` - Verify library dependencies
   - `PRD.md` - Sections 2.1, 2.2, 4.1 for hardware specs
