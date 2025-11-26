# Phase: Ass-Easy-Loop V1.0 Firmware Implementation
![Planned](https://img.shields.io/badge/Planned-lightgrey) > `P1`

> Implementation of 10Hz pEMF driver firmware on RP2040-Zero with safety constraints and power management logic.

## Milestone: Environment & Infrastructure
![Planned](https://img.shields.io/badge/Planned-lightgrey) > `P1.M1`

> Establish build system, toolchain dependencies, and core configuration.

### Task: Project Configuration
![Planned](https://img.shields.io/badge/Planned-lightgrey) > `P1.M1.T1`

> Configure PlatformIO for RP2040-Zero and external libraries.

#### Subtask: PlatformIO Configuration
![Planned](https://img.shields.io/badge/Planned-lightgrey) > `P1.M1.T1.S1` | **Story Points**: 1 | **Dependencies**: 


**Context & Scope:**
```text
CONTRACT DEFINITION:
1. INPUT: PRD Section 5.1 (Toolchain).
2. LOGIC: Create `platformio.ini`. Define environment for `rp2040`. Add `earlephilhower/arduino-pico` core. Add `Adafruit NeoPixel` library dependency. Define build flags for USB Serial support.
3. OUTPUT: A valid `platformio.ini` file ready for `pio run`.
```

## Milestone: Hardware Abstraction Layer (HAL)
![Planned](https://img.shields.io/badge/Planned-lightgrey) > `P1.M2`

> Encapsulate direct hardware access (GPIO, Coil, LED) into testable classes.

### Task: Coil Driver HAL
![Planned](https://img.shields.io/badge/Planned-lightgrey) > `P1.M2.T1`

> Safe wrapper for MOSFET control.

#### Subtask: Implement CoilDriver Class
![Planned](https://img.shields.io/badge/Planned-lightgrey) > `P1.M2.T1.S1` | **Story Points**: 1 | **Dependencies**: P1.M1.T1.S1


**Context & Scope:**
```text
CONTRACT DEFINITION:
1. INPUT: Pin definition (GPIO 15) from PRD Section 2.2.
2. LOGIC: Create class `CoilDriver`. Constructor accepts pin number. `begin()` sets pinMode OUTPUT and immediately writes LOW (Safety). `setActive(bool state)` writes HIGH/LOW. Destructor forces LOW.
3. OUTPUT: `src/hal/CoilDriver.h` and `.cpp`.
```

### Task: Feedback Driver HAL
![Planned](https://img.shields.io/badge/Planned-lightgrey) > `P1.M2.T2`

> Wrapper for synchronous Audio/Visual feedback.

#### Subtask: Implement FeedbackDriver Class
![Planned](https://img.shields.io/badge/Planned-lightgrey) > `P1.M2.T2.S1` | **Story Points**: 2 | **Dependencies**: P1.M1.T1.S1


**Context & Scope:**
```text
CONTRACT DEFINITION:
1. INPUT: PRD Section 2.1 (NeoPixel) & 2.2 (Buzzer GPIO 14).
2. LOGIC: Create class `FeedbackDriver`. Encapsulate `Adafruit_NeoPixel` object. Constructor accepts buzzer pin and NeoPixel pin. `begin()` initializes NeoPixel and sets buzzer pin OUTPUT. Method `indicateActive(bool isActive)`: If true, set buzzer HIGH and NeoPixel Green (Low Brightness). If false, buzzer LOW and NeoPixel OFF.
3. OUTPUT: `src/hal/FeedbackDriver.h` and `.cpp`.
```

### Task: Time Source Interface
![Planned](https://img.shields.io/badge/Planned-lightgrey) > `P1.M2.T3`

> Abstraction for time to enable unit testing without `delay()`.

#### Subtask: Implement ITimeSource
![Planned](https://img.shields.io/badge/Planned-lightgrey) > `P1.M2.T3.S1` | **Story Points**: 1 | **Dependencies**: 


**Context & Scope:**
```text
CONTRACT DEFINITION:
1. INPUT: None.
2. LOGIC: Create interface `ITimeSource` with method `unsigned long millis()`. Create concrete class `ArduinoTimeSource` implementing it using Arduino's `millis()`. This allows mocking time in logic tests.
3. OUTPUT: `src/hal/TimeSource.h`.
```

## Milestone: Core Application Logic
![Planned](https://img.shields.io/badge/Planned-lightgrey) > `P1.M3`

> Implementation of the therapeutic waveform and session management.

### Task: Waveform Controller
![Planned](https://img.shields.io/badge/Planned-lightgrey) > `P1.M3.T1`

> Logic for the 10Hz signal generation.

#### Subtask: Implement WaveformController
![Planned](https://img.shields.io/badge/Planned-lightgrey) > `P1.M3.T1.S1` | **Story Points**: 2 | **Dependencies**: P1.M2.T1.S1, P1.M2.T2.S1, P1.M2.T3.S1


**Context & Scope:**
```text
CONTRACT DEFINITION:
1. INPUT: `CoilDriver` (P1.M2.T1.S1), `FeedbackDriver` (P1.M2.T2.S1), `ITimeSource` (P1.M2.T3.S1).
2. LOGIC: Create class `WaveformController`. Method `update()` called in loop. Logic: 10Hz cycle = 100ms period. ON for 2ms, OFF for 98ms. Use `ITimeSource` to track elapsed time non-blocking. Call `CoilDriver` and `FeedbackDriver` based on state.
3. OUTPUT: `src/logic/WaveformController.h` and `.cpp`.
```

### Task: Session Manager
![Planned](https://img.shields.io/badge/Planned-lightgrey) > `P1.M3.T2`

> Global timer enforcement (15 minutes).

#### Subtask: Implement SessionManager
![Planned](https://img.shields.io/badge/Planned-lightgrey) > `P1.M3.T2.S1` | **Story Points**: 2 | **Dependencies**: P1.M3.T1.S1, P1.M2.T3.S1


**Context & Scope:**
```text
CONTRACT DEFINITION:
1. INPUT: `WaveformController` (P1.M3.T1.S1), `ITimeSource`.
2. LOGIC: Class `SessionManager`. `start()` records start time. `update()` checks if `current_time - start_time > 15 minutes` (900,000ms). If within limit, call `WaveformController.update()`. If limit exceeded, ensure `CoilDriver` is OFF and enter infinite idle loop/state.
3. OUTPUT: `src/logic/SessionManager.h` and `.cpp`.
```

## Milestone: Integration & Entry Point
![Planned](https://img.shields.io/badge/Planned-lightgrey) > `P1.M4`

> Wiring of dependencies and Main Sketch setup.

### Task: Main Sketch
![Planned](https://img.shields.io/badge/Planned-lightgrey) > `P1.M4.T1`

> Dependency injection and runtime loop.

#### Subtask: Implement main.cpp
![Planned](https://img.shields.io/badge/Planned-lightgrey) > `P1.M4.T1.S1` | **Story Points**: 1 | **Dependencies**: P1.M3.T2.S1


**Context & Scope:**
```text
CONTRACT DEFINITION:
1. INPUT: All previous HAL and Logic classes.
2. LOGIC: In `setup()`: Init `Serial` (115200 for bootloader per PRD 5.3). Instantiate `CoilDriver` (GPIO 15), `FeedbackDriver` (GPIO 14, NeoPixel), `ArduinoTimeSource`. Inject into `WaveformController`, then into `SessionManager`. Call `SessionManager.start()`. In `loop()`: Call `SessionManager.update()`.
3. OUTPUT: `src/main.cpp`.
```
