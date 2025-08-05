# Hardware Setup Documentation

This document provides comprehensive hardware setup instructions, wiring diagrams, and assembly guidelines for the RP2040 pEMF/Battery Monitoring Device with automated testing capabilities.

## Table of Contents

1. [Hardware Overview](#hardware-overview)
2. [Component Specifications](#component-specifications)
3. [Detailed Wiring Diagrams](#detailed-wiring-diagrams)
4. [Assembly Instructions](#assembly-instructions)
5. [Testing and Validation](#testing-and-validation)
6. [Safety Guidelines](#safety-guidelines)
7. [Troubleshooting](#troubleshooting)

## Hardware Overview

### System Architecture

The device consists of four main subsystems:

1. **Microcontroller Unit (MCU)**: Raspberry Pi Pico (RP2040)
2. **Power Management**: Battery monitoring and power distribution
3. **pEMF Generation**: MOSFET driver and electromagnetic field output
4. **Communication**: USB HID interface for automated testing

### Block Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           RP2040 pEMF Device                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐    ┌─────────────────────┐    ┌─────────────────────────┐  │
│  │   Battery   │    │                     │    │                         │  │
│  │   3.7V      │────┤    Raspberry Pi     │    │     MOSFET Driver       │  │
│  │   LiPo      │    │       Pico          │    │       Module            │  │
│  │             │    │     (RP2040)        │────┤                         │  │
│  └─────────────┘    │                     │    │                         │  │
│         │            │  GPIO15 ────────────┼────┤ Control Input           │  │
│         │            │  GPIO25 ── LED      │    │                         │  │
│         │            │  GPIO26 ── ADC      │    └─────────────────────────┘  │
│         │            │                     │               │                │
│         │            └─────────────────────┘               │                │
│         │                       │                         │                │
│         │            ┌─────────────────────┐               │                │
│         └────────────┤   Voltage Divider   │               │                │
│                      │   10kΩ + 5.1kΩ     │               │                │
│                      └─────────────────────┘               │                │
│                                                            │                │
│                      ┌─────────────────────────────────────┼──────────────┐ │
│                      │           External Load             │              │ │
│                      │     (Electromagnetic Coil)         │              │ │
│                      └─────────────────────────────────────┼──────────────┘ │
│                                                            │                │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Component Specifications

### Essential Components

| Component | Specification | Quantity | Purpose | Notes |
|-----------|---------------|----------|---------|-------|
| **Raspberry Pi Pico** | RP2040, 264KB RAM, 2MB Flash | 1 | Main microcontroller | Must be genuine or compatible |
| **LiPo Battery** | 3.7V, 500mAh minimum | 1 | Power source | With protection circuit |
| **MOSFET Driver** | Logic level, 3.3V compatible | 1 | pEMF pulse amplification | IRF520 or similar |
| **Resistor (R1)** | 10kΩ, 1% tolerance, 1/4W | 1 | Voltage divider (high side) | Metal film preferred |
| **Resistor (R2)** | 5.1kΩ, 1% tolerance, 1/4W | 1 | Voltage divider (low side) | Metal film preferred |
| **Capacitor (C1)** | 100nF, ceramic, 50V | 1 | ADC input filtering | X7R dielectric |
| **Capacitor (C2)** | 10µF, electrolytic, 16V | 1 | Power supply decoupling | Low ESR |
| **JST Connector** | JST-PH 2-pin, 2mm pitch | 1 | Battery connection | With matching cable |
| **USB Cable** | USB-A to Micro-USB | 1 | Programming and communication | Data capable |

### Optional Components

| Component | Specification | Purpose | Notes |
|-----------|---------------|---------|-------|
| **Breadboard** | Half-size, 400 tie points | Prototyping | For initial testing |
| **PCB** | Custom or universal | Permanent assembly | Recommended for final build |
| **Enclosure** | Plastic, 80x60x25mm | Protection | 3D printed or commercial |
| **Heat Sink** | TO-220 compatible | MOSFET cooling | If high current loads |
| **Fuse** | 1A, fast-blow | Protection | In series with battery |
| **Switch** | SPST, 1A rating | Power control | Optional power switch |

### Tools Required

| Tool | Purpose | Notes |
|------|---------|-------|
| **Soldering Iron** | 25-40W, temperature controlled | For permanent connections |
| **Solder** | 60/40 or 63/37, 0.6-0.8mm | Rosin core |
| **Wire Strippers** | 22-30 AWG | For connection wires |
| **Multimeter** | DC voltage, continuity | For testing and validation |
| **Oscilloscope** | 50MHz minimum | For timing verification (optional) |
| **Logic Analyzer** | 8+ channels | For protocol debugging (optional) |

## Detailed Wiring Diagrams

### Complete System Wiring

```
                    RP2040 pEMF Device - Complete Wiring Diagram
                    
    ┌─────────────────────────────────────────────────────────────────────────────┐
    │                              Power Section                                   │
    └─────────────────────────────────────────────────────────────────────────────┘
    
    Battery Pack (3.7V LiPo)           Raspberry Pi Pico
    ┌─────────────────────┐           ┌─────────────────────────────────────┐
    │                     │           │                                     │
    │  ┌─────────────┐    │           │                                     │
    │  │     3.7V    │    │           │                                     │
    │  │   LiPo      │    │           │                                     │
    │  │   Battery   │    │           │                                     │
    │  │             │    │           │                                     │
    │  │             │    │           │                                     │
    │  │    [+]──────┼────┼───────────┼─── VSYS (Pin 39)                   │
    │  │             │    │           │                                     │
    │  │             │    │           │                                     │
    │  │    [-]──────┼────┼───────────┼─── GND (Pin 38)                    │
    │  │             │    │           │                                     │
    │  └─────────────┘    │           │                                     │
    │                     │           │                                     │
    └─────────────────────┘           └─────────────────────────────────────┘
    
    ┌─────────────────────────────────────────────────────────────────────────────┐
    │                           Voltage Divider Section                           │
    └─────────────────────────────────────────────────────────────────────────────┘
    
                                      Voltage Divider Circuit
                                    ┌─────────────────────────┐
                                    │                         │
                                    │      R1 = 10kΩ         │
                                    │    ┌─────────────┐      │
    Battery [+] ────────────────────┼────┤             ├──────┼──┐
                                    │    │    10kΩ     │      │  │
                                    │    │             │      │  │
                                    │    └──────┬──────┘      │  │
                                    │           │             │  │
                                    │           │             │  │
                                    │           ├─────────────┼──┼─── To GPIO26 (Pin 31)
                                    │           │             │  │
                                    │      R2 = 5.1kΩ        │  │
                                    │    ┌──────▼──────┐      │  │
                                    │    │             │      │  │
                                    │    │    5.1kΩ    ├──────┼──┼─── To GND
                                    │    │             │      │  │
                                    │    └─────────────┘      │  │
                                    │                         │  │
                                    └─────────────────────────┘  │
                                                                 │
                                    ┌─────────────────────────────┼──┐
                                    │     C1 = 100nF              │  │
                                    │   ┌─────────────┐           │  │
                                    │   │             │           │  │
                                    │   │    100nF    ├───────────┼──┘
                                    │   │             │           │
                                    │   └─────────────┘           │
                                    │                             │
                                    └─────────────────────────────┘
    
    ┌─────────────────────────────────────────────────────────────────────────────┐
    │                           MOSFET Driver Section                             │
    └─────────────────────────────────────────────────────────────────────────────┘
    
    Raspberry Pi Pico                    MOSFET Driver Module
    ┌─────────────────────┐             ┌─────────────────────────┐
    │                     │             │                         │
    │                     │             │                         │
    │  GPIO15 (Pin 20)────┼─────────────┼─── IN (Control Input)   │
    │                     │             │                         │
    │  3V3 (Pin 36)───────┼─────────────┼─── VCC (Power)          │
    │                     │             │                         │
    │  GND (Pin 38)───────┼─────────────┼─── GND (Ground)         │
    │                     │             │                         │
    └─────────────────────┘             │                         │
                                        │  OUT+ ──────────────────┼─── To Load [+]
                                        │                         │
                                        │  OUT- ──────────────────┼─── To Load [-]
                                        │                         │
                                        └─────────────────────────┘
    
    ┌─────────────────────────────────────────────────────────────────────────────┐
    │                              Load Section                                   │
    └─────────────────────────────────────────────────────────────────────────────┘
    
    MOSFET Driver Output                 Electromagnetic Load
    ┌─────────────────────┐             ┌─────────────────────────┐
    │                     │             │                         │
    │  OUT+ ──────────────┼─────────────┼─── Coil [+]             │
    │                     │             │                         │
    │                     │             │     ┌─────────────┐     │
    │                     │             │     │             │     │
    │                     │             │     │ Electromagnetic │     │
    │                     │             │     │     Coil     │     │
    │                     │             │     │             │     │
    │                     │             │     └─────────────┘     │
    │                     │             │                         │
    │  OUT- ──────────────┼─────────────┼─── Coil [-]             │
    │                     │             │                         │
    └─────────────────────┘             └─────────────────────────┘
```

### Pin Assignment Details

#### Raspberry Pi Pico Pinout

```
                    Raspberry Pi Pico - Pin Assignments
                    
                         ┌─────────────────┐
                         │      USB        │
                         └─────────────────┘
                                  │
    ┌────────────────────────────────────────────────────────────┐
    │                                                            │
    │  GP0  │ 1                                        40 │ VBUS │ ── USB 5V (when connected)
    │  GP1  │ 2                                        39 │ VSYS │ ── Battery Positive
    │  GND  │ 3                                        38 │ GND  │ ── Ground/Battery Negative
    │  GP2  │ 4                                        37 │ 3V3E │ ── 3.3V Enable
    │  GP3  │ 5                                        36 │ 3V3  │ ── 3.3V Output
    │  GP4  │ 6                                        35 │ ADC_VREF │
    │  GP5  │ 7                                        34 │ GP28 │
    │  GND  │ 8                                        33 │ GND  │
    │  GP6  │ 9                                        32 │ GP27 │
    │  GP7  │ 10                                       31 │ GP26 │ ── Battery ADC Input
    │  GP8  │ 11                                       30 │ RUN  │
    │  GP9  │ 12                                       29 │ GP22 │
    │  GND  │ 13                                       28 │ GND  │
    │  GP10 │ 14                                       27 │ GP21 │
    │  GP11 │ 15                                       26 │ GP20 │
    │  GP12 │ 16                                       25 │ GP19 │
    │  GP13 │ 17                                       24 │ GP18 │
    │  GND  │ 18                                       23 │ GND  │
    │  GP14 │ 19                                       22 │ GP17 │
    │  GP15 │ 20 ── pEMF Control Output               21 │ GP16 │
    │                                                            │
    └────────────────────────────────────────────────────────────┘
                                  │
                         ┌─────────────────┐
                         │   Debug Port    │
                         │   (SWD)         │
                         └─────────────────┘
```

#### Critical Pin Functions

| Pin Number | GPIO | Function | Connection | Notes |
|------------|------|----------|------------|-------|
| **20** | GP15 | pEMF Control | MOSFET Driver IN | High priority output |
| **25** | GP25 | Status LED | Onboard LED | Built-in connection |
| **31** | GP26 | Battery ADC | Voltage Divider | ADC0 input |
| **36** | 3V3 | Power Output | MOSFET Driver VCC | 3.3V regulated |
| **38** | GND | Ground | Common Ground | Multiple connections |
| **39** | VSYS | Power Input | Battery Positive | 3.0V - 5.5V range |

### Voltage Divider Design

#### Circuit Analysis

The voltage divider scales the 3.7V battery voltage to the ADC's 3.3V input range:

```
Voltage Divider Calculation:

Input Voltage Range: 3.0V - 4.2V (LiPo battery)
ADC Input Range: 0V - 3.3V (RP2040 ADC)
Required Scaling Factor: 3.3V / 4.2V = 0.786

Selected Resistor Values:
R1 = 10kΩ (high side)
R2 = 5.1kΩ (low side)

Actual Scaling Factor:
K = R2 / (R1 + R2) = 5.1kΩ / (10kΩ + 5.1kΩ) = 5.1 / 15.1 = 0.337

Voltage Mapping:
3.0V battery → 3.0V × 0.337 = 1.01V ADC
3.7V battery → 3.7V × 0.337 = 1.25V ADC
4.2V battery → 4.2V × 0.337 = 1.42V ADC

ADC Count Mapping (12-bit ADC, 3.3V reference):
1.01V → (1.01 / 3.3) × 4095 = 1253 counts
1.25V → (1.25 / 3.3) × 4095 = 1553 counts
1.42V → (1.42 / 3.3) × 4095 = 1761 counts
```

#### Component Tolerances

| Parameter | Nominal | Tolerance | Impact |
|-----------|---------|-----------|--------|
| **R1 Resistance** | 10kΩ | ±1% | ±0.1kΩ variation |
| **R2 Resistance** | 5.1kΩ | ±1% | ±0.051kΩ variation |
| **Scaling Factor** | 0.337 | ±2% | ±0.007 variation |
| **ADC Reference** | 3.3V | ±3% | ±0.1V variation |
| **Total Accuracy** | - | ±5% | ±0.2V battery reading |

## Assembly Instructions

### Phase 1: Component Preparation

#### Step 1: Component Verification

1. **Verify all components** against the parts list
2. **Test the Raspberry Pi Pico**:
   ```bash
   # Connect via USB and verify detection
   lsusb | grep -i "2e8a"
   # Should show: Bus XXX Device XXX: ID 2e8a:000a Raspberry Pi RP2 Boot
   ```
3. **Measure resistor values** with multimeter:
   - R1 should read 9.9kΩ - 10.1kΩ
   - R2 should read 5.05kΩ - 5.15kΩ
4. **Test battery voltage**: Should read 3.0V - 4.2V

#### Step 2: Workspace Preparation

1. **Set up clean workspace** with good lighting
2. **Prepare soldering station** at 350°C (662°F)
3. **Organize components** in labeled containers
4. **Have multimeter ready** for continuity testing

### Phase 2: Power System Assembly

#### Step 3: Install Raspberry Pi Pico

1. **Mount Pico on breadboard or PCB**:
   - Ensure USB connector is accessible
   - Verify all pins are properly seated
   - Check for bent or damaged pins

2. **Create ground bus connections**:
   ```
   Connect Pin 38 (GND) to breadboard ground rail
   Connect Pin 3 (GND) to same ground rail
   Connect Pin 13 (GND) to same ground rail
   Connect Pin 18 (GND) to same ground rail
   Connect Pin 23 (GND) to same ground rail
   Connect Pin 28 (GND) to same ground rail
   Connect Pin 33 (GND) to same ground rail
   ```

#### Step 4: Install Battery Connection

1. **Prepare battery connector**:
   - Strip wire ends 5mm
   - Tin wire ends with solder
   - Use heat shrink tubing for insulation

2. **Connect battery wires**:
   ```
   Red wire (Battery +) → VSYS (Pin 39)
   Black wire (Battery -) → GND (Pin 38)
   ```

3. **Add power supply decoupling**:
   - Install 10µF capacitor between VSYS and GND
   - Place capacitor close to Pico power pins

### Phase 3: Voltage Divider Assembly

#### Step 5: Install Voltage Divider Resistors

1. **Install R1 (10kΩ resistor)**:
   ```
   One end → Battery positive (VSYS connection point)
   Other end → Junction point (to be connected to R2 and GPIO26)
   ```

2. **Install R2 (5.1kΩ resistor)**:
   ```
   One end → Junction point (connected to R1)
   Other end → Ground rail
   ```

3. **Connect ADC input**:
   ```
   Junction point → GPIO26 (Pin 31)
   Use short, direct connection to minimize noise
   ```

4. **Add ADC filtering capacitor**:
   ```
   100nF ceramic capacitor between GPIO26 and GND
   Place capacitor close to GPIO26 pin
   ```

#### Step 6: Verify Voltage Divider Operation

1. **Connect battery** (ensure correct polarity)
2. **Measure voltages**:
   ```bash
   # With multimeter, measure:
   Battery voltage: Should be 3.0V - 4.2V
   VSYS voltage: Should equal battery voltage
   GPIO26 voltage: Should be ~1/3 of battery voltage
   ```
3. **Calculate scaling factor**:
   ```
   Measured scaling = GPIO26 voltage / Battery voltage
   Should be approximately 0.337 ± 0.02
   ```

### Phase 4: MOSFET Driver Installation

#### Step 7: Install MOSFET Driver Module

1. **Mount driver module**:
   - Ensure adequate spacing for heat dissipation
   - Orient for easy connection access
   - Secure mounting to prevent movement

2. **Connect driver power**:
   ```
   Driver VCC → 3V3 (Pin 36) or VSYS (Pin 39)
   Driver GND → Ground rail
   ```

3. **Connect control signal**:
   ```
   Driver IN → GPIO15 (Pin 20)
   Use short, direct connection
   Add 100Ω series resistor if needed for current limiting
   ```

#### Step 8: Install Load Connections

1. **Prepare load terminals**:
   - Use appropriate wire gauge for load current
   - Install proper connectors for easy disconnection
   - Add strain relief to prevent wire damage

2. **Connect electromagnetic load**:
   ```
   Driver OUT+ → Load positive terminal
   Driver OUT- → Load negative terminal
   ```

3. **Add protection components** (recommended):
   ```
   Flyback diode across inductive loads (cathode to OUT+)
   Fuse in series with battery positive (1A fast-blow)
   TVS diode for transient protection
   ```

### Phase 5: Final Assembly and Testing

#### Step 9: Complete Assembly Verification

1. **Visual inspection checklist**:
   - [ ] All connections match wiring diagram
   - [ ] No short circuits between power rails
   - [ ] Proper component orientation
   - [ ] Secure mechanical connections
   - [ ] No exposed conductors

2. **Continuity testing**:
   ```bash
   # Use multimeter continuity mode to verify:
   Battery + to VSYS: Should have continuity
   Battery - to all GND pins: Should have continuity
   GPIO15 to driver IN: Should have continuity
   GPIO26 to voltage divider junction: Should have continuity
   No continuity between VCC and GND anywhere
   ```

#### Step 10: Power-On Testing

1. **Initial power-on sequence**:
   ```bash
   # Step 1: Connect battery (device should power on)
   # Step 2: Verify LED operation (should respond to battery state)
   # Step 3: Connect USB cable
   # Step 4: Verify USB enumeration
   lsusb | grep -i "2e8a"
   ```

2. **Voltage verification**:
   ```bash
   # Measure with multimeter:
   VSYS: Should equal battery voltage
   3V3: Should be 3.3V ± 0.1V
   GPIO26: Should be ~1/3 of battery voltage
   Driver VCC: Should be 3.3V or battery voltage
   ```

3. **Basic functionality test**:
   ```bash
   # Flash test firmware and verify:
   # - LED responds to battery state changes
   # - GPIO15 outputs 2Hz square wave
   # - USB HID communication works
   ```

## Testing and Validation

### Electrical Testing

#### Power System Validation

1. **Battery monitoring accuracy**:
   ```bash
   # Test procedure:
   # 1. Measure actual battery voltage with precision multimeter
   # 2. Read ADC value from device
   # 3. Calculate accuracy: |measured - expected| / expected
   # 4. Accuracy should be within ±5%
   ```

2. **Power consumption measurement**:
   ```bash
   # Measure current consumption:
   # Normal operation: <50mA average
   # pEMF pulse active: <200mA peak
   # Sleep mode (if implemented): <1mA
   ```

#### Signal Integrity Testing

1. **pEMF timing verification**:
   ```bash
   # Use oscilloscope on GPIO15:
   # Frequency: 2.00Hz ± 0.02Hz
   # Pulse width: 2.0ms ± 0.02ms
   # Rise time: <1µs
   # Fall time: <1µs
   ```

2. **ADC noise measurement**:
   ```bash
   # Measure GPIO26 with oscilloscope:
   # DC level: As calculated from voltage divider
   # AC noise: <10mV peak-to-peak
   # No switching noise from pEMF operation
   ```

### Functional Testing

#### Automated Test Execution

1. **Run comprehensive test suite**:
   ```bash
   cd test_framework
   python comprehensive_test_runner.py --hardware-validation
   ```

2. **Verify test results**:
   ```bash
   # All tests should pass:
   # - USB communication test
   # - pEMF timing validation
   # - Battery ADC calibration
   # - LED functionality test
   # - System stress test
   ```

#### Long-term Stability Testing

1. **24-hour continuous operation**:
   ```bash
   python test_framework/comprehensive_test_runner.py --stability-test --duration 86400
   ```

2. **Performance monitoring**:
   ```bash
   # Monitor for:
   # - Timing drift: <0.1% over 24 hours
   # - Temperature stability: No thermal shutdown
   # - Memory leaks: No memory usage increase
   # - Communication errors: <0.1% error rate
   ```

## Safety Guidelines

### Electrical Safety

#### Before Assembly

- [ ] **Verify component ratings** match specifications
- [ ] **Check battery condition** - no swelling or damage
- [ ] **Ensure proper workspace** - ESD protection, good lighting
- [ ] **Have safety equipment** - fire extinguisher, first aid kit

#### During Assembly

- [ ] **Disconnect power** before making connections
- [ ] **Verify polarity** before connecting battery
- [ ] **Use proper soldering technique** - avoid overheating
- [ ] **Test connections** before applying power

#### After Assembly

- [ ] **Inspect for shorts** before first power-on
- [ ] **Monitor temperature** during initial testing
- [ ] **Have disconnect method** readily available
- [ ] **Test in safe environment** away from flammable materials

### pEMF Safety

#### Electromagnetic Field Exposure

- **Start with low power** and short exposure times
- **Avoid prolonged exposure** without medical consultation
- **Keep away from sensitive electronics** during operation
- **Do not use near pacemakers** or other medical implants

#### Load Safety

- **Verify load ratings** match driver specifications
- **Use appropriate wire gauge** for load current
- **Install protection devices** (fuses, diodes)
- **Monitor for overheating** during operation

### Battery Safety

#### Lithium Battery Precautions

- **Use only quality batteries** with protection circuits
- **Never exceed 4.2V** charging voltage
- **Monitor for swelling** or damage
- **Charge in safe location** away from flammable materials
- **Dispose properly** at recycling centers

#### Emergency Procedures

- **Battery fire**: Use Class D extinguisher or sand, never water
- **Electrical shock**: Disconnect power immediately, seek medical attention
- **Overheating**: Disconnect power, allow cooling, investigate cause
- **Smoke or unusual odors**: Disconnect power, evacuate area if necessary

## Troubleshooting

### Common Hardware Issues

#### Power Problems

**Symptom**: Device doesn't power on
```bash
# Troubleshooting steps:
1. Check battery voltage (should be >3.0V)
2. Verify battery connector polarity
3. Check fuse continuity (if installed)
4. Measure VSYS voltage at Pin 39
5. Check for short circuits
```

**Symptom**: Inconsistent operation
```bash
# Possible causes and solutions:
1. Low battery capacity → Replace or charge battery
2. Poor connections → Re-solder connections
3. Electromagnetic interference → Add shielding
4. Thermal issues → Improve ventilation
```

#### ADC Reading Issues

**Symptom**: Incorrect battery voltage readings
```bash
# Troubleshooting steps:
1. Verify resistor values with multimeter
2. Check voltage divider connections
3. Measure actual voltages at each point
4. Recalibrate software scaling factors
```

**Symptom**: Noisy or unstable ADC readings
```bash
# Solutions:
1. Add or replace filtering capacitor
2. Improve ground connections
3. Separate analog and digital grounds
4. Shield ADC input from switching noise
```

#### MOSFET Driver Problems

**Symptom**: No output from driver
```bash
# Check list:
1. Verify driver power supply (VCC and GND)
2. Check control signal from GPIO15
3. Verify driver enable pins (if present)
4. Test with known good driver module
```

**Symptom**: Weak or distorted output
```bash
# Possible solutions:
1. Check driver current rating vs load requirements
2. Verify adequate power supply capacity
3. Add heat sink if thermal limiting
4. Check for loose connections
```

### Signal Integrity Issues

#### Timing Problems

**Symptom**: Incorrect pEMF timing
```bash
# Investigation steps:
1. Verify crystal oscillator (12MHz)
2. Check software timing configuration
3. Measure actual timing with oscilloscope
4. Look for interrupt latency issues
```

**Symptom**: Timing jitter or instability
```bash
# Solutions:
1. Improve power supply decoupling
2. Reduce electromagnetic interference
3. Check RTIC task priorities
4. Verify crystal oscillator stability
```

#### Communication Issues

**Symptom**: USB HID communication failures
```bash
# Troubleshooting:
1. Verify USB cable is data-capable
2. Check USB enumeration in device manager
3. Test with different USB ports
4. Verify HID descriptor configuration
```

**Symptom**: Intermittent communication
```bash
# Solutions:
1. Check for loose USB connections
2. Verify adequate power supply
3. Test with shorter USB cable
4. Check for electromagnetic interference
```

### Diagnostic Tools and Techniques

#### Multimeter Testing

```bash
# Essential measurements:
DC Voltages:
- Battery: 3.0V - 4.2V
- VSYS: Equal to battery
- 3V3: 3.3V ± 0.1V
- GPIO26: ~1/3 of battery voltage

Continuity:
- All ground connections
- Signal paths
- Power distribution

Resistance:
- R1: 9.9kΩ - 10.1kΩ
- R2: 5.05kΩ - 5.15kΩ
- No shorts between VCC and GND
```

#### Oscilloscope Analysis

```bash
# Key signals to monitor:
GPIO15 (pEMF Control):
- Frequency: 2.00Hz ± 0.02Hz
- Pulse width: 2.0ms ± 0.02ms
- Voltage levels: 0V/3.3V
- Rise/fall times: <1µs

GPIO26 (ADC Input):
- DC level: As calculated
- AC noise: <10mV p-p
- No switching artifacts

Power Rails:
- 3V3: Clean DC with <50mV ripple
- VSYS: Stable, no dropout during pulses
```

#### Software Debugging

```bash
# Debug information collection:
# Enable debug logging in firmware
# Monitor USB HID communication
# Use test framework diagnostic modes
# Collect timing statistics
```

### Getting Additional Help

#### Documentation References

- Refer to component datasheets for detailed specifications
- Check RP2040 datasheet for electrical characteristics
- Review RTIC documentation for timing behavior
- Consult USB HID specification for communication issues

#### Community Resources

- Post detailed problem descriptions with:
  - Complete hardware configuration
  - Measured voltages and signals
  - Error messages and symptoms
  - Steps already attempted
- Include photos of assembly for visual inspection
- Provide oscilloscope traces for timing issues

---

This hardware setup documentation provides comprehensive guidance for successful assembly and validation of the RP2040 pEMF/Battery Monitoring Device. Follow all safety guidelines and take time to verify each step before proceeding.