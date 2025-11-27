# Arduino/PlatformIO Piezo Buzzer Control Patterns

A comprehensive guide for controlling piezo buzzers via GPIO using simple ON/OFF digital control with Arduino and PlatformIO frameworks.

## Table of Contents

1. [Simple ON/OFF Control](#simple-onoff-control)
2. [Initialization Patterns](#initialization-patterns)
3. [Synchronous Feedback Pattern](#synchronous-feedback-pattern)
4. [RP2040/Arduino-Pico Considerations](#rp2040arduino-pico-considerations)
5. [Code Examples](#code-examples)
6. [References](#references)

---

## Simple ON/OFF Control

### Active vs Passive Buzzers

There are two fundamental types of piezo buzzers that have very different control requirements:

#### Active Buzzers (Simple GPIO Control)

**Characteristics:**
- Have an internal oscillator that generates the tone
- Generate sound immediately when voltage is applied
- Typically fixed frequency around 2-3 kHz (manufacturer tuned)
- Identified by higher internal resistance (several kΩ range)
- Usually more expensive than passive alternatives

**GPIO Control Method:**
```cpp
digitalWrite(buzzer, HIGH);   // Turn ON
digitalWrite(buzzer, LOW);    // Turn OFF
```

**Active buzzers work perfectly with simple digitalWrite() calls** - they act like digital outputs similar to LEDs. This is the recommended approach for simple feedback patterns requiring no frequency control.

#### Passive Buzzers (Requires PWM/Tone)

**Characteristics:**
- No internal oscillator - require external AC signal
- Act like electromagnetic speakers requiring oscillating input
- Allow arbitrary frequency selection for pitch control
- Typically lower resistance (few ohms)
- Require PWM or tone() function for operation
- Do NOT work with simple digitalWrite() alone

**GPIO Control Method:**
```cpp
// NOT suitable for simple ON/OFF via digitalWrite()
// Must use tone() function:
tone(buzzer, 440);    // 440 Hz (A4 note)
noTone(buzzer);       // Stop tone
```

**Key Difference Summary:**

| Aspect | Active Buzzer | Passive Buzzer |
|--------|---------------|----------------|
| **Internal Oscillator** | Yes | No |
| **digitalWrite() Support** | Yes - Simple ON/OFF | No - produces only "pop" |
| **Frequency Control** | Fixed only | Full range via PWM/tone() |
| **Typical Use** | Simple alerts/alarms | Music/melodies/variable tones |
| **Best for GPIO GPIO Drive** | Ideal | Not recommended (needs PWM) |
| **Measurement Test** | Apply DC voltage → immediate beep | Apply DC voltage → just a click |

### Current Draw and GPIO Direct Drive

#### Typical Current Specifications

**Active Buzzers:**
- Measured range: 10-30 mA at 3.3V or 5V
- Common examples: KY-012 active buzzer module draws up to 30 mA
- Volume increases with current (voltage applied)

**Passive Buzzers/Piezo Elements:**
- Very low current devices - essentially capacitive loads
- Can draw 20-30 mA depending on voltage and frequency
- Volume primarily determined by applied voltage, not frequency

#### Arduino GPIO Current Limits

**Standard Arduino (ATmega328P-based):**
- **Absolute maximum per pin:** 40 mA
- **Recommended per pin:** 20 mA
- **Total all pins combined:** 200 mA

**Important:** While 40 mA is the absolute maximum, exceeding 20 mA per pin can reduce component lifespan and affect performance of other circuits.

#### Direct GPIO Drive Decision Tree

```
Is your buzzer current draw < 12 mA at 3.3V?
├─ YES: Direct GPIO drive is acceptable
│   ├─ Active buzzer: Use digitalWrite(pin, HIGH/LOW)
│   └─ Passive buzzer: Use tone(pin, frequency) if available
│
└─ NO: Use transistor/MOSFET driver
    └─ Common: N-channel MOSFET (e.g., 2N7000, IRF520N)
       ├─ Gate → GPIO pin
       ├─ Drain → Buzzer positive
       ├─ Source → GND
       └─ Optional: 100Ω resistor on gate for protection
```

#### Best Practice Recommendations

1. **For Simple Feedback (Recommended):** Use an active buzzer with direct GPIO drive if current < 20 mA
2. **For Loud Alerts:** Use a transistor driver to supply adequate current
3. **For Tone Control:** Use passive buzzer with tone() function via PWM-capable pin
4. **Check Datasheet:** Always verify your specific buzzer module's current requirements

---

## Initialization Patterns

### Safe Startup State Configuration

#### Recommended Initialization Sequence

The safest approach pre-sets the digital output value **before** setting the pin as OUTPUT. This prevents unexpected buzzer activation during startup:

```cpp
// RECOMMENDED: Pre-set value before pinMode
void setup() {
  digitalWrite(buzzPin, LOW);    // Pre-set to LOW (silent state)
  pinMode(buzzPin, OUTPUT);      // Now set as output - will be LOW
}
```

**Why this matters:**
- During normal Arduino startup, pins briefly float to undefined states
- Setting pinMode() alone can cause transient HIGH states
- For buzzer pins, a brief HIGH can produce an undesirable startup "click"
- Pre-setting value ensures predictable startup behavior

#### Alternative: If You Need Default HIGH

```cpp
// If your application needs default HIGH (less common)
void setup() {
  digitalWrite(buzzPin, HIGH);   // Pre-set to HIGH
  pinMode(buzzPin, OUTPUT);      // Now set as output - will be HIGH
}
```

**Use case:** Rare; only if your logic inverts the signal (active-low buzzer control).

#### Explanation of Arduino Startup Behavior

When `pinMode(pin, OUTPUT)` is called:
1. The pin is configured as an output
2. The pin defaults to LOW state
3. However, a transitional HIGH can occur briefly before completion
4. For normal Arduino AVR chips, the pre-set value ensures correct initial state

**Note on Pull-up Resistors:**
- Arduino has internal pull-up resistors for INPUT mode only
- These cannot be used for guaranteed LOW state on OUTPUT pins
- For critical applications, external pull-down resistors are needed (rarely necessary for buzzers)

### Initialization in Classes/Wrapper Patterns

```cpp
class Buzzer {
private:
  uint8_t pin;

public:
  Buzzer(uint8_t buzzPin) : pin(buzzPin) {}

  void begin() {
    // Safe initialization
    digitalWrite(pin, LOW);      // Pre-set LOW
    pinMode(pin, OUTPUT);        // Configure as output
  }

  void on() {
    digitalWrite(pin, HIGH);
  }

  void off() {
    digitalWrite(pin, LOW);
  }
};
```

---

## Synchronous Feedback Pattern

### Non-Blocking Simultaneous Control

**Problem:** Using `delay()` blocks all code execution. This prevents simultaneous control of buzzer and other outputs (LEDs, etc.).

#### Solution 1: Time-Based State Machine with millis()

This is the preferred pattern for PlatformIO/Arduino embedded systems:

```cpp
class FeedbackController {
private:
  uint8_t buzzerPin;
  uint8_t ledPin;
  unsigned long buzzerOnTime = 0;
  unsigned long ledBlinkTime = 0;
  bool buzzerActive = false;
  bool ledState = false;

public:
  FeedbackController(uint8_t bPin, uint8_t lPin)
    : buzzerPin(bPin), ledPin(lPin) {}

  void begin() {
    digitalWrite(buzzerPin, LOW);
    pinMode(buzzerPin, OUTPUT);
    digitalWrite(ledPin, LOW);
    pinMode(ledPin, OUTPUT);
  }

  // Non-blocking buzzer pulse - returns immediately
  void buzzFor(unsigned long durationMs) {
    digitalWrite(buzzerPin, HIGH);
    buzzerOnTime = millis() + durationMs;
    buzzerActive = true;
  }

  // Non-blocking LED blink
  void blinkLed(unsigned long onDurationMs, unsigned long offDurationMs) {
    // Implementation with state machine
  }

  // MUST be called regularly in loop() to maintain timing
  void update() {
    unsigned long now = millis();

    // Handle buzzer auto-off
    if (buzzerActive && (now >= buzzerOnTime)) {
      digitalWrite(buzzerPin, LOW);
      buzzerActive = false;
    }

    // Handle LED blinking
    // ... LED blink logic ...
  }
};

// In main sketch:
FeedbackController feedback(BUZZER_PIN, LED_PIN);

void setup() {
  feedback.begin();
}

void loop() {
  feedback.update();  // Called frequently - no blocking

  // Other code runs uninterrupted
  handleSensors();
  processCommands();
}
```

**Key Advantages:**
- Both buzzer and LED respond independently
- No blocking delays - main loop remains responsive
- Precise timing using `millis()` timer (1 ms resolution on most Arduino boards)
- Scales to multiple simultaneous feedback elements

#### Solution 2: Simple Simultaneous GPIO Writes

For simple synchronized activation (both on/off together):

```cpp
// Activate buzzer and LED simultaneously
void activateFeedback() {
  digitalWrite(buzzerPin, HIGH);
  digitalWrite(ledPin, HIGH);
}

void deactivateFeedback() {
  digitalWrite(buzzerPin, LOW);
  digitalWrite(ledPin, LOW);
}

// In main code:
void alertUser() {
  activateFeedback();      // Both turn on together
  delay(500);              // Brief alert
  deactivateFeedback();    // Both turn off together
}
```

**Limitations:** Uses blocking delay() - simple but blocks other code.

#### Solution 3: Using tone() Function

For active buzzers or when you need tone control:

```cpp
// Synchronous: blocks execution
void briefAlert() {
  tone(buzzerPin, 1000, 100);  // 1kHz for 100ms, then auto-stops
  digitalWrite(ledPin, HIGH);
  delay(100);
  digitalWrite(ledPin, LOW);
}

// Note: tone() parameters:
// tone(pin, frequency_Hz)           - starts tone indefinitely
// tone(pin, frequency_Hz, duration) - plays for duration_ms then stops
// noTone(pin)                       - stops tone immediately
```

#### Solution 4: Library Approach - Non-Blocking Buzzer Library

For PlatformIO, the **ezBuzzer** library provides non-blocking control:

```cpp
#include <ezBuzzer.h>

ezBuzzer buzzer(BUZZER_PIN);

void setup() {
  Serial.begin(9600);
}

void loop() {
  buzzer.update();  // Must call frequently

  // Buzzer operates in background
  if (someCondition) {
    buzzer.beep(100);  // 100ms beep
  }
}
```

**Library Details:**
- **Name:** ezBuzzer
- **Type:** Non-blocking buzzer control
- **Platform:** Available on PlatformIO registry
- **GitHub:** https://github.com/ArduinoGetStarted/buzzer
- **Key methods:**
  - `update()` - must be called in loop()
  - `beep(duration)` - single beep
  - `setPin(pin)` - set buzzer pin

### Best Practice Pattern for Synchronous Feedback

```cpp
// General pattern for your application:
class SimpleBuzzer {
private:
  uint8_t pin;

public:
  SimpleBuzzer(uint8_t buzzPin) : pin(buzzPin) {}

  void begin() {
    digitalWrite(pin, LOW);
    pinMode(pin, OUTPUT);
  }

  // Simple synchronous alert (blocks for duration)
  void alert(unsigned long durationMs) {
    digitalWrite(pin, HIGH);
    delay(durationMs);
    digitalWrite(pin, LOW);
  }

  // Direct control
  void on() { digitalWrite(pin, HIGH); }
  void off() { digitalWrite(pin, LOW); }
};

// Usage:
SimpleBuzzer buzzer(9);

void setup() {
  buzzer.begin();
}

void loop() {
  if (buttonPressed()) {
    buzzer.alert(200);  // Simple 200ms alert
  }
}
```

---

## RP2040/Arduino-Pico Considerations

### RP2040 GPIO Current Specifications

The RP2040 microcontroller (used in Raspberry Pi Pico, Arduino-Pico boards) has specific GPIO current limitations:

#### Per-Pin Drive Strength Settings

The RP2040 allows configurable GPIO drive strength:

```cpp
// RP2040-specific GPIO drive strength configuration
// Available levels: 2mA, 4mA (default), 8mA, 12mA
```

| Drive Setting | Typical Current | Use Case |
|---------------|-----------------|----------|
| 2 mA | Minimal | Very light loads only |
| 4 mA (Default) | Conservative | Safe for most applications |
| 8 mA | Moderate | Higher impedance loads |
| 12 mA | Maximum per pin | Heavy loads (not recommended for piezo) |

#### Total GPIO Current Budget

**Critical Limitation:**
- **Per single pin maximum:** ~28 mA (measured in real-world testing; spec suggests 12 mA configurable max)
- **All GPIO pins combined total:** 50 mA maximum

**Example:**
If using only a single GPIO pin for a buzzer, you can theoretically draw up to 28 mA, but this leaves no budget for other GPIO-driven devices.

#### Practical Limits for Piezo Buzzers

**For RP2040 direct GPIO drive:**

```
If buzzer draws 15+ mA at 3.3V:
├─ Single pin: Can drive, but near voltage drop limits
├─ Multiple devices: Problematic - exceeds 50 mA total budget
└─ Recommendation: Use transistor driver instead

If buzzer draws < 12 mA at 3.3V:
├─ Direct GPIO: Acceptable, leaves headroom
├─ Reliable operation: More stable voltage
└─ Recommendation: OK for direct GPIO drive
```

### RP2040 Voltage Considerations

**GPIO Logic Level:** 3.3V (not 5V tolerant)
**Typical Piezo Buzzer Requirements:** 3.3V - 5V

**Issue:** Many piezo buzzers specify 5V operation. Running them at 3.3V:
- Reduced volume (lower voltage = less displacement)
- May not reach design loudness
- Current draw is voltage-dependent

### Recommended Driver Circuit for RP2040

For buzzers requiring > 12 mA or 5V operation:

```
RP2040 GPIO (3.3V)
        │
        ├─────[100Ω]────┐
                         │
                        /
                       / N-channel MOSFET
                      / (e.g., 2N7000)
                       \
                         ├─ Drain: to Buzzer (+5V supply)
                         │
                         ├─ Source: to GND
                         │
                         └─ Gate: from GPIO via 100Ω resistor

Buzzer (+)  ─────────┬──────→ MOSFET Drain
                    │
Buzzer (-)  ─────── GND (common)

5V Supply ──→ MOSFET Drain (via buzzer)
```

**Alternative: Using a transistor (BJT)**

```
RP2040 GPIO
        │
        ├─────[10kΩ]─────┐
                          │
                         /|
                        / | NPN Transistor
                       /  | (e.g., 2N2222, BC547)
                        \ |
                         \|
                          ├─ Collector: to Buzzer (+5V)
                          │
                          ├─ Emitter: to GND
                          │
                          └─ Base: from GPIO via 10kΩ resistor
```

**Why transistor driver is preferred for RP2040:**
- Protects GPIO from high current demand
- Allows 5V buzzer operation (GPIO only supplies 3.3V)
- Enables driving multiple buzzers from single GPIO (with current limiting)
- More robust design for production use

### Example Code for RP2040 with Arduino-Pico

```cpp
// Simple buzzer wrapper for RP2040/Arduino-Pico
// Works with direct GPIO drive for low-current buzzers

#include <Arduino.h>

class RP2040Buzzer {
private:
  uint8_t pin;

public:
  RP2040Buzzer(uint8_t buzzPin) : pin(buzzPin) {}

  void begin() {
    // RP2040 safe initialization
    digitalWrite(pin, LOW);
    pinMode(pin, OUTPUT);
  }

  void on() {
    digitalWrite(pin, HIGH);
  }

  void off() {
    digitalWrite(pin, LOW);
  }

  void pulse(unsigned long durationMs) {
    on();
    delay(durationMs);
    off();
  }
};

// Usage:
RP2040Buzzer buzzer(15);  // GPIO15 (arbitrary example)

void setup() {
  buzzer.begin();
}

void loop() {
  // Your code here
  buzzer.pulse(100);
  delay(500);
}
```

---

## Code Examples

### Example 1: Minimal Active Buzzer Control

```cpp
// Absolute minimum code for active buzzer control
// Works with any Arduino-compatible board

const int BUZZER_PIN = 9;

void setup() {
  digitalWrite(BUZZER_PIN, LOW);    // Pre-set LOW
  pinMode(BUZZER_PIN, OUTPUT);      // Configure output
}

void loop() {
  digitalWrite(BUZZER_PIN, HIGH);   // Buzzer ON
  delay(500);                       // Wait 500ms
  digitalWrite(BUZZER_PIN, LOW);    // Buzzer OFF
  delay(500);                       // Wait 500ms
}
```

### Example 2: Buzzer Wrapper Class

```cpp
// Reusable buzzer class - PlatformIO compatible

#ifndef BUZZER_H
#define BUZZER_H

#include <Arduino.h>

class Buzzer {
private:
  uint8_t pin;
  bool isOn;

public:
  // Constructor
  Buzzer(uint8_t buzzPin) : pin(buzzPin), isOn(false) {}

  // Initialize buzzer pin
  void begin() {
    digitalWrite(pin, LOW);     // Safe startup
    pinMode(pin, OUTPUT);
    isOn = false;
  }

  // Turn buzzer on
  void on() {
    if (!isOn) {
      digitalWrite(pin, HIGH);
      isOn = true;
    }
  }

  // Turn buzzer off
  void off() {
    if (isOn) {
      digitalWrite(pin, LOW);
      isOn = false;
    }
  }

  // Check if currently buzzing
  bool active() const {
    return isOn;
  }

  // Simple blocking pulse
  void pulse(unsigned long durationMs) {
    on();
    delay(durationMs);
    off();
  }

  // Toggle state
  void toggle() {
    if (isOn) {
      off();
    } else {
      on();
    }
  }
};

#endif // BUZZER_H
```

**Usage in main sketch:**

```cpp
#include "Buzzer.h"

Buzzer buzzer(9);  // GPIO pin 9

void setup() {
  buzzer.begin();
}

void loop() {
  buzzer.on();
  delay(200);
  buzzer.off();
  delay(300);
}
```

### Example 3: Synchronized Buzzer + LED Feedback

```cpp
// Non-blocking simultaneous control of buzzer and LED
// Pattern suitable for real-world PlatformIO projects

#include <Arduino.h>

class FeedbackSystem {
private:
  uint8_t buzzerPin;
  uint8_t ledPin;

  // Buzzer state tracking
  bool buzzerActive;
  unsigned long buzzerEndTime;

  // LED state tracking
  bool ledState;
  unsigned long ledToggleTime;
  unsigned long ledOnDuration;
  unsigned long ledOffDuration;
  bool ledBlinking;

public:
  FeedbackSystem(uint8_t bPin, uint8_t lPin)
    : buzzerPin(bPin),
      ledPin(lPin),
      buzzerActive(false),
      ledState(false),
      ledBlinking(false) {}

  void begin() {
    // Initialize buzzer
    digitalWrite(buzzerPin, LOW);
    pinMode(buzzerPin, OUTPUT);

    // Initialize LED
    digitalWrite(ledPin, LOW);
    pinMode(ledPin, OUTPUT);
  }

  // Non-blocking: Start buzzer for specified duration
  void buzz(unsigned long durationMs) {
    digitalWrite(buzzerPin, HIGH);
    buzzerActive = true;
    buzzerEndTime = millis() + durationMs;
  }

  // Non-blocking: Start LED blinking
  void blink(unsigned long onMs, unsigned long offMs) {
    ledOnDuration = onMs;
    ledOffDuration = offMs;
    ledToggleTime = millis() + ledOnDuration;
    ledState = true;
    ledBlinking = true;
    digitalWrite(ledPin, HIGH);
  }

  // Stop blinking and turn LED off
  void ledOff() {
    ledBlinking = false;
    digitalWrite(ledPin, LOW);
    ledState = false;
  }

  // Must be called frequently (e.g., in main loop)
  void update() {
    unsigned long now = millis();

    // Handle buzzer auto-off
    if (buzzerActive && (now >= buzzerEndTime)) {
      digitalWrite(buzzerPin, LOW);
      buzzerActive = false;
    }

    // Handle LED blinking
    if (ledBlinking && (now >= ledToggleTime)) {
      if (ledState) {
        // Was ON, turn OFF
        digitalWrite(ledPin, LOW);
        ledState = false;
        ledToggleTime = now + ledOffDuration;
      } else {
        // Was OFF, turn ON
        digitalWrite(ledPin, HIGH);
        ledState = true;
        ledToggleTime = now + ledOnDuration;
      }
    }
  }

  // Convenience: Alert pattern (buzz + LED together)
  void alert(unsigned long durationMs) {
    buzz(durationMs);
    blink(durationMs / 2, durationMs / 2);
  }
};

// Example usage:
FeedbackSystem feedback(9, 13);  // Buzzer on pin 9, LED on pin 13

void setup() {
  Serial.begin(115200);
  feedback.begin();
}

void loop() {
  // MUST call update() frequently to maintain timing
  feedback.update();

  // Simulate some event
  static unsigned long lastAlert = 0;
  if (millis() - lastAlert > 3000) {
    feedback.alert(200);  // 200ms alert
    lastAlert = millis();
  }

  // Rest of your code here - this loop remains responsive
  // because we're not using delay()
}
```

### Example 4: Button-Triggered Buzzer

```cpp
// Simple button + buzzer control pattern
// Demonstrates real-world interaction pattern

#include <Arduino.h>

const int BUTTON_PIN = 2;
const int BUZZER_PIN = 9;

// Debounce variables
unsigned long lastDebounceTime = 0;
unsigned long debounceDelay = 20;  // 20ms debounce
int lastButtonState = HIGH;
int buttonState = HIGH;

void setup() {
  pinMode(BUTTON_PIN, INPUT_PULLUP);
  digitalWrite(BUZZER_PIN, LOW);
  pinMode(BUZZER_PIN, OUTPUT);
}

void loop() {
  // Read button with debouncing
  int reading = digitalRead(BUTTON_PIN);

  if (reading != lastButtonState) {
    lastDebounceTime = millis();
  }

  if ((millis() - lastDebounceTime) > debounceDelay) {
    if (reading != buttonState) {
      buttonState = reading;

      // Button state changed
      if (buttonState == LOW) {
        // Button pressed - turn buzzer on
        digitalWrite(BUZZER_PIN, HIGH);
      } else {
        // Button released - turn buzzer off
        digitalWrite(BUZZER_PIN, LOW);
      }
    }
  }

  lastButtonState = reading;
}
```

### Example 5: PlatformIO Configuration

```ini
; platformio.ini example for buzzer project
[env:arduino_uno]
platform = atmelavr
board = uno
framework = arduino
upload_port = /dev/ttyUSB0

[env:rp2040]
platform = raspberrypi
board = pico
framework = arduino
upload_port = /dev/ttyACM0

; Optional: Add ezBuzzer library
; [env:buzzer_lib]
; lib_deps =
;     ArduinoGetStarted/ezBuzzer @ ^1.0
```

---

## References

### Arduino Official Documentation
- [Arduino digitalWrite() Reference](https://www.arduino.cc/reference/en/language/functions/digital-io/digitalwrite/)
- [Arduino pinMode() Reference](https://www.arduino.cc/reference/en/language/functions/digital-io/pinmode/)
- [Arduino tone() Function](https://www.arduino.cc/reference/en/language/functions/advanced-io/tone/)

### Circuit & Control References
- [How to Use Active and Passive Buzzers on Arduino - Circuit Basics](https://www.circuitbasics.com/how-to-use-active-and-passive-buzzers-on-the-arduino/)
- [Arduino Piezo Buzzer Tutorial - Arduino Getting Started](https://arduinogetstarted.com/tutorials/arduino-piezo-buzzer)
- [Piezo Buzzer with Button - Arduino Tutorial](https://www.circuits-diy.com/piezo-buzzer-with-button-arduino-tutorial/)
- [Control Piezo Buzzer with Button - The Geek Pub](https://www.thegeekpub.com/275886/control-a-piezo-buzzer-with-a-button/)

### STM32/RP2040 Specific References
- [STM32 Buzzer Control - Deep Blue Embedded](https://deepbluembedded.com/stm32-buzzer-piezo-active-passive-buzzer-example-code-tone/)
- [Raspberry Pi Pico GPIO Maximum Current - Raspberry Pi Forums](https://forums.raspberrypi.com/viewtopic.php?t=300735)
- [Raspberry Pi Pico Digital I/O (C/C++ SDK) - Deep Blue Embedded](https://deepbluembedded.com/raspberry-pi-pico-w-digital-inputs-outputs-c-sdk-rp2040/)
- [Buzzer Music with Raspberry Pi Pico - Tom's Hardware](https://www.tomshardware.com/how-to/buzzer-music-raspberry-pi-pico)

### PlatformIO Libraries & Community
- [PlatformIO Registry - Buzzer Libraries](https://platformio.org/lib/search?query=keyword%3A%22buzzer%22)
- [ezBuzzer Library - GitHub](https://github.com/ArduinoGetStarted/buzzer)
- [ezBuzzer on PlatformIO Registry](https://registry.platformio.org/libraries/arduinogetstarted/ezBuzzer)
- [EasyBuzzer Library - GitHub](https://github.com/evert-arias/EasyBuzzer)
- [PlatformIO Community - Buzzer Discussion](https://community.platformio.org/t/new-arduino-library-to-use-piezo-buzzers-in-a-non-blocking-and-easy-way/31737)

### Stack Exchange & Forum Resources
- [How to Run LED and Buzzer Simultaneously - Arduino Stack Exchange](https://arduino.stackexchange.com/questions/70277/how-to-run-led-and-buzzer-at-the-same-time)
- [Synchronizing LED with Buzzer - Arduino Forum](https://forum.arduino.cc/t/synchronizing-led-with-buzzer/993217)
- [Arduino Piezo Buzzer FAQ - Arduino Forum](https://forum.arduino.cc/t/piezo-buzzer/4557)
- [Passive Buzzer GPIO Issues - Arduino Stack Exchange](https://arduino.stackexchange.com/questions/57718/passive-buzzer-works-with-analogwrite-but-not-with-digitalwrite-it-also-ha)

### Current & Electrical Specifications
- [Arduino Current Consumption - Arduino Stack Exchange](https://arduino.stackexchange.com/questions/66771/current-consumption-of-a-buzzer)
- [Driving Piezo Buzzer with MOSFET - Arduino Forum](https://forum.arduino.cc/t/driving-piezo-buzzer-with-n-mosfet/483406)
- [RP2040 GPIO Current Rating - Raspberry Pi Forums](https://forums.raspberrypi.com/viewtopic.php?t=300735)

### Sensor Module References
- [KY-012 Active Piezo-Buzzer - SensorKit](https://sensorkit.joy-it.net/en/sensors/ky-012)
- [KY-006 Passive Piezo-Buzzer - SensorKit](https://sensorkit.joy-it.net/en/sensors/ky-006)

---

## Quick Reference Cheatsheet

### Checklist: Choosing Between Active and Passive

- [ ] Do you need simple on/off control only?
  - Yes → Use **Active Buzzer** with `digitalWrite(pin, HIGH/LOW)`
  - No → Use **Passive Buzzer** with `tone(pin, frequency)`

- [ ] Will current draw exceed 20 mA?
  - Yes → Use **transistor driver circuit**
  - No → Safe for **direct GPIO drive**

- [ ] Is your board RP2040/Pico?
  - Yes → Check GPIO current (total 50 mA budget), consider transistor driver
  - No → Standard Arduino GPIO limits apply (40 mA absolute per pin)

### Initialization Template

```cpp
const int BUZZER_PIN = 9;

void setup() {
  digitalWrite(BUZZER_PIN, LOW);    // Pre-set LOW
  pinMode(BUZZER_PIN, OUTPUT);      // Set as output
}
```

### Simple Control Template

```cpp
// For Active Buzzer
digitalWrite(BUZZER_PIN, HIGH);  // ON
digitalWrite(BUZZER_PIN, LOW);   // OFF

// For Passive Buzzer
tone(BUZZER_PIN, 440);           // 440 Hz
noTone(BUZZER_PIN);              // Stop
```

### Simultaneous Control Pattern

```cpp
// Use millis() instead of delay() for responsive code
unsigned long lastTime = 0;
unsigned long interval = 500;

void loop() {
  if (millis() - lastTime >= interval) {
    lastTime = millis();
    digitalWrite(BUZZER_PIN, !digitalRead(BUZZER_PIN));  // Toggle
  }
}
```

---

## Document Information

- **Version:** 1.0
- **Last Updated:** November 2025
- **Applicable Platforms:** Arduino (AVR, SAMD, etc.), RP2040/Pico, STM32
- **Framework:** Arduino Framework on PlatformIO
- **Author Notes:** This document focuses on simple GPIO-based ON/OFF control. For advanced PWM/tone generation, see audio synthesis libraries.

