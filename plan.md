# Implementation Plan: P1.M4.T1.S1 - Implement main.cpp

## Subtask Context
**ID**: P1.M4.T1.S1
**Title**: Implement main.cpp
**Story Points**: 1
**Dependencies**: P1.M3.T2.S1 (SessionManager - completed)

## Contract Definition
1. **INPUT**: All previous HAL and Logic classes
2. **LOGIC**:
   - In `setup()`: Init Serial (115200 for bootloader per PRD 5.3). Instantiate CoilDriver (GPIO 15), FeedbackDriver (GPIO 14, NeoPixel), ArduinoTimeSource. Inject into WaveformController, then into SessionManager. Call SessionManager.start()
   - In `loop()`: Call SessionManager.update()
3. **OUTPUT**: `src/main.cpp`

---

## Implementation Steps

### Step 1: Update includes
Replace minimal includes with all required headers:
- `<Arduino.h>` (already present)
- `"hal/CoilDriver.h"` (already present)
- `"hal/FeedbackDriver.h"` (new)
- `"hal/TimeSource.h"` (new)
- `"logic/WaveformController.h"` (new)
- `"logic/SessionManager.h"` (new)

### Step 2: Declare global instances
Create global instances in correct dependency order (since C++ requires objects to exist before they can be referenced):

```cpp
// HAL layer - hardware abstractions
ArduinoTimeSource timeSource;
CoilDriver coilDriver(15);           // GPIO 15 - MOSFET control
FeedbackDriver feedbackDriver(14, 16); // GPIO 14 - Buzzer, GPIO 16 - NeoPixel

// Logic layer - business logic with injected dependencies
WaveformController waveformController(coilDriver, feedbackDriver, timeSource);
SessionManager sessionManager(waveformController, timeSource);
```

**Design Notes**:
- `ArduinoTimeSource` declared first (no dependencies)
- `CoilDriver` and `FeedbackDriver` declared next (depend only on TimeSource for some methods)
- `WaveformController` depends on all three HAL objects
- `SessionManager` depends on WaveformController and TimeSource
- All use default GPIO pins per PRD Section 2.2

### Step 3: Implement setup() function
Initialize all components in correct order:

```cpp
void setup() {
    // Enable USB Serial for bootloader backdoor (PRD 5.3)
    Serial.begin(115200);

    // Initialize HAL components
    coilDriver.begin();
    feedbackDriver.begin();

    // Initialize and start session
    waveformController.begin();
    sessionManager.start();

    // Debug output confirming initialization
    Serial.println("pEMF Session Started - 15 minute limit");
}
```

**Key Requirements**:
- `Serial.begin(115200)` MUST be called for magic 1200 baud reset bootloader mechanism
- HAL `begin()` calls initialize GPIO to safe states
- `waveformController.begin()` prepares timing state
- `sessionManager.start()` records start time and enables session tracking

### Step 4: Implement loop() function
Minimal non-blocking loop:

```cpp
void loop() {
    sessionManager.update();
}
```

**Behavior**:
- `sessionManager.update()` handles entire session lifecycle
- Delegates to `waveformController.update()` while active
- Automatically terminates and enters idle loop when 15 minutes exceeded
- No additional logic needed in main loop

---

## File Changes Summary

| File | Action | Description |
|------|--------|-------------|
| `src/main.cpp` | EDIT | Complete rewrite with dependency injection wiring |

---

## Validation Criteria

1. **Compilation**: Code compiles without errors via `pio run`
2. **Includes**: All required headers included
3. **GPIO Pins**: Correct pins used (15, 14, 16 per PRD)
4. **Serial**: 115200 baud for bootloader backdoor
5. **Dependency Order**: Objects instantiated in valid dependency order
6. **Initialization Order**: `begin()` calls before `start()`
7. **Loop Simplicity**: Single `sessionManager.update()` call

---

## Dependencies Verified

| Component | Location | Status |
|-----------|----------|--------|
| CoilDriver | `src/hal/CoilDriver.h/cpp` | Available |
| FeedbackDriver | `src/hal/FeedbackDriver.h/cpp` | Available |
| ArduinoTimeSource | `src/hal/TimeSource.h` | Available |
| WaveformController | `src/logic/WaveformController.h/cpp` | Available |
| SessionManager | `src/logic/SessionManager.h/cpp` | Available |

---

## Risk Assessment

**Low Risk** - This is a simple wiring task:
- All component classes are already implemented and tested
- GPIO pin assignments are defined in PRD
- Dependency injection pattern is established
- No new logic required, only instantiation and method calls
