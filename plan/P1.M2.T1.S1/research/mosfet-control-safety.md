# MOSFET Control Safety Patterns Research

## 1. IRF520 MOSFET Driver Module

### Key Characteristics
- **Voltage Range**: 0-24V DC loads, compatible with both 3.3V and 5V control signals
- **Current Capacity**: Up to 5A maximum
- **Control Signal**: Digital HIGH/LOW on a single GPIO pin
- **Gate Protection**: Maximum +/-20V on gate

### Control Pattern for IRF520
```cpp
class MOSFETDriver {
private:
    uint8_t controlPin;

public:
    MOSFETDriver(uint8_t pin) : controlPin(pin) {}

    void begin() {
        pinMode(controlPin, OUTPUT);
        digitalWrite(controlPin, LOW);  // Immediate safe state
    }

    void setActive(bool active) {
        digitalWrite(controlPin, active ? HIGH : LOW);
    }

    ~MOSFETDriver() {
        digitalWrite(controlPin, LOW);  // Force safe state on destruction
    }
};
```

---

## 2. Inductive Load Control - Flyback Protection

### The Problem: Back EMF Spikes
When switching inductive loads (coils, solenoids), turning OFF the MOSFET causes the collapsing magnetic field to generate back EMF (reverse voltage). Without protection:
- Measured spikes can reach -300V across the switch
- This can instantly destroy the MOSFET

### Flyback Diode Protection (Per PRD Section 2.1)
**Hardware requirement from PRD**: 1N4007 or 1N5408 flyback diode

The diode provides a safe discharge path, limiting reverse voltage to ~-1.4V.

**Circuit (from PRD Section 2.2):**
```
MOSFET Output + --> Coil Start AND Diode Cathode (Stripe)
MOSFET Output - --> Coil End AND Diode Anode (Black)
```

---

## 3. Fail-Safe GPIO Patterns

### Core Principles
1. **No Floating Pins**: Always define explicit state (HIGH or LOW)
2. **Initialization Order**: Constructor receives pin number, `begin()` performs hardware setup
3. **Destructor Safety**: Destructor must guarantee safe state (noexcept)
4. **Non-copyable Resources**: Hardware pins are unique - delete copy/move constructors

### RAII Pattern for Safe GPIO Control
```cpp
class SafeCoilDriver {
private:
    uint8_t controlPin;
    bool initialized;

    // Non-copyable hardware resource
    SafeCoilDriver(const SafeCoilDriver&) = delete;
    SafeCoilDriver& operator=(const SafeCoilDriver&) = delete;

public:
    // Constructor: Store pin, don't touch hardware
    explicit SafeCoilDriver(uint8_t pin)
        : controlPin(pin), initialized(false) {}

    // begin(): Initialize hardware, force safe state
    void begin() {
        if (initialized) return;
        pinMode(controlPin, OUTPUT);
        digitalWrite(controlPin, LOW);  // Safe state
        initialized = true;
    }

    // setActive(): User control
    void setActive(bool active) {
        if (!initialized) return;  // Guard against pre-begin use
        digitalWrite(controlPin, active ? HIGH : LOW);
    }

    // Destructor: Guarantee safe shutdown
    ~SafeCoilDriver() {
        digitalWrite(controlPin, LOW);  // Force LOW for safety
    }
};
```

---

## 4. Key Safety Considerations

### Critical Issues

1. **Floating Pins on Reset**
   - When Arduino resets, GPIO pins become inputs by default
   - Floating input pins can cause spurious switching
   - **Solution**: Destructor must force LOW, and first operation in `begin()` is `pinMode(OUTPUT)` then immediate `digitalWrite(LOW)`

2. **Constructor/Destructor Timing**
   - Constructor runs at object declaration (early)
   - **Problem**: Hardware operations in constructor may happen before system is stable
   - **Solution**: Split initialization into constructor (data only) and begin() (hardware setup)

3. **Destructor Exceptions**
   - C++11+ makes all destructors noexcept by default
   - **Critical**: Never throw from destructors used with hardware
   - Safe operations: `digitalWrite()`, simple assignments

4. **External Pull-Down Resistor** (Hardware recommendation)
   - 100K-1M ohm resistor from GPIO pin to GND (parallel with gate)
   - Ensures gate defaults to LOW on startup before firmware runs
   - **Correct wiring:**
     ```
     GPIO 15 ────────┬──── MOSFET Gate
                     │
                   [100K]
                     │
                    GND
     ```
   - **WARNING:** Do NOT put this resistor in series between GPIO and MOSFET gate.
     A series resistor forms an RC filter with gate capacitance and will kill the drive signal.

---

## Sources

- [Electropeak - IRF520 MOSFET Driver Module](https://electropeak.com/learn/interfacing-irf520-mosfet-driver-module-switch-button-hcmodu0083-with-arduino/)
- [Wikipedia - Flyback Diode](https://en.wikipedia.org/wiki/Flyback_diode)
- [Stratify Labs - RAII in Embedded C++](https://blog.stratifylabs.dev/device/2020-12-22-RAII-Everywhere-in-Cpp/)
- [cppreference.com - C++ RAII](https://en.cppreference.com/w/cpp/language/raii.html)
