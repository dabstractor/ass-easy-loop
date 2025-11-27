# RP2040-Zero GPIO Patterns Research

## 1. pinMode() Usage - OUTPUT Configuration

### Basic Implementation
```cpp
pinMode(15, OUTPUT);        // Configure GPIO 15 as output (default 4mA drive strength)
pinMode(15, OUTPUT_2MA);    // 2mA output drive
pinMode(15, OUTPUT_4MA);    // 4mA output drive (default)
pinMode(15, OUTPUT_8MA);    // 8mA output drive
pinMode(15, OUTPUT_12MA);   // 12mA output drive
```

### What Happens Under the Hood
The arduino-pico implementation in `wiring_digital.cpp` uses the Pico SDK directly:
- Calls `gpio_init(ulPin)` to initialize the GPIO
- Calls `gpio_set_dir(ulPin, true)` to set direction to output
- Configures pad drive strength via `gpio_set_drive_strength()`

**Key Point**: The RP2040 has configurable pad strength (2mA, 4mA, 8mA, 12mA), which is important for MOSFET gate drive considerations.

---

## 2. digitalWrite() Usage - HIGH/LOW Control

### Basic Implementation
```cpp
digitalWrite(15, HIGH);     // Set GPIO 15 to logic HIGH (3.3V)
digitalWrite(15, LOW);      // Set GPIO 15 to logic LOW (0V)
```

### Implementation Details
The `__digitalWrite()` function uses `gpio_put(ulPin, ulVal == LOW ? 0 : 1)` to directly control the GPIO state.

**Critical for MOSFET Control**: Ensure pins default to LOW on startup to prevent unwanted MOSFET activation.

---

## 3. GPIO 15 Specifics for RP2040-Zero

- **Pin number**: GPIO 15
- **Peripherals**: PWM7 channel B, I2C1 SCL
- **RP2040-Zero note**: 29 GPIO pins total; GPIO 15 is readily accessible on pin headers
- **Important**: RP2040 pins are NOT 5V tolerant (max 3.3V input)
- **No Special Bootloader Function**: Unlike ESP32 GPIO 15, the RP2040 GPIO 15 has no bootloader or strapping function

---

## 4. Safe Initialization Pattern

```cpp
// Safe MOSFET initialization for RP2040
class CoilDriver {
private:
    const uint8_t pin;

public:
    CoilDriver(uint8_t mosfet_pin) : pin(mosfet_pin) {
        // Constructor: store pin only, don't touch hardware
    }

    void begin() {
        // Step 1: Set initial state to LOW BEFORE configuring as output
        gpio_init(pin);
        gpio_put(pin, 0);  // Ensure LOW before output enable

        // Step 2: Configure as output
        pinMode(pin, OUTPUT);
    }

    void setActive(bool active) {
        digitalWrite(pin, active ? HIGH : LOW);
    }

    ~CoilDriver() {
        digitalWrite(pin, LOW);  // Ensure safe state on destruction
    }
};
```

---

## 5. ISR Context Safety

**CRITICAL**: ISR callbacks run in interrupt context:
- NO `digitalWrite()`, `Serial.print()`, `delay()` in ISRs
- Just set a flag or increment a counter
- Handle actual work in main loop

---

## Sources

- [Arduino-Pico Digital I/O](https://arduino-pico.readthedocs.io/en/latest/digital.html)
- [Arduino-Pico GitHub](https://github.com/earlephilhower/arduino-pico)
- [wiring_digital.cpp Implementation](https://github.com/earlephilhower/arduino-pico/blob/master/cores/rp2040/wiring_digital.cpp)
- [RP2040 Datasheet](https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf)
