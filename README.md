# ass-easy-loop

Firmware for the ass-easy-loop pEMF device.

# RP2040 Ass-Easy Loop pEMF Device - Wiring Guide

Based on the working ~/projects/ass-easy-loop implementation.

## Overview

This device generates configurable pEMF (pulsed electromagnetic field) patterns for research. Simple 3-component design:
- **RP2040** (Raspberry Pi Pico) - main controller
- **MOSFET Driver Module** - electromagnet control
- **Voltage Divider** - battery monitoring

## Required Components

| Component | Specification | Quantity | Purpose |
|-----------|---------------|----------|---------|
| **Raspberry Pi Pico** | RP2040, 264KB RAM, 2MB Flash | 1 | Main microcontroller |
| **LiPo Battery** | 3.7V, 500mAh minimum | 1 | Power source |
| **MOSFET Driver Module** | Logic level, 3.3V compatible | 1 | pEMF pulse amplification |
| **Resistor (R1)** | 10kΩ, 1% tolerance, 1/4W | 1 | Voltage divider (high side) |
| **Resistor (R2)** | 5.1kΩ, 1% tolerance, 1/4W | 1 | Voltage divider (low side) |
| **Capacitor (C1)** | 100nF, ceramic, 50V | 1 | ADC input filtering |
| **JST Connector** | JST-PH 2-pin, 2mm pitch | 1 | Battery connection |

## Complete Wiring Diagram

```
                    pEMF Device Wiring Diagram

    Battery Pack                 Raspberry Pi Pico                    MOSFET Driver
    ┌─────────────┐             ┌─────────────────────┐              ┌─────────────┐
    │             │             │                     │              │             │
    │     3.7V    │             │                     │              │    VCC      │
    │   LiPo      │             │   RP2040            │              │     │       │
    │   Battery   │             │                     │              │    ┌▼┐      │
    │             │             │                     │              │    │ │      │
    │    [+]──────┼─────────────┼──┤ VSYS (Pin 39)    │              │    │M│      │
    │             │             │                     │              │    │O│      │
    │             │             │                     │              │    │S│      │
    │             │             │                     │              │    │F│      │
    │             │             │  GPIO15 (Pin 20)────┼──────────────┼────┤E│      │
    │             │             │                     │              │    │T│      │
    │             │             │                     │              │    └┬┘      │
    │             │             │  GPIO25 (Pin 25)─── LED            │     │       │
    │             │             │                     │              │    GND      │
    │             │             │                     │              │     │       │
    │             │             │  GPIO26 (Pin 31)────┼──┐           │     │       │
    │             │             │                     │  │           └─────┼───────┘
    │             │             │                     │  │                 │
    │             │             │  GND (Pin 38)───────┼──┼─────────────────┼───┐
    │             │             │                     │  │                 │   │
    │    [-]──────┼─────────────┼─────────────────────┼──┼─────────────────┘   │
    │             │             │                     │  │                     │
    └─────────────┘             └─────────────────────┘  │                     │
                                                         │                     │
                                    Voltage Divider     │                     │
                                    ┌─────────────────┐  │                     │
                                    │   10kΩ Resistor │  │                     │
                    Battery [+] ────┼───┤  10kΩ   ├───┼──┘                     │
                                    │   └────┬────┘   │                        │
                                    │        │        │                        │
                                    │        ├────────┼─── To GPIO26 (ADC)     │
                                    │        │        │                        │
                                    │   ┌────▼────┐   │                        │
                                    │   │  5.1kΩ  ├───┼────────────────────────┘
                                    │   └─────────┘   │
                                    └─────────────────┘
```

## Pin Connections

### Raspberry Pi Pico Connections

| Pico Pin | Pin Name | Function | Connection |
|----------|----------|----------|------------|
| **20** | GP15 | pEMF Control | To MOSFET driver input |
| **25** | GP25 | LED Control | Onboard LED (built-in) |
| **31** | GP26 | Battery ADC | To voltage divider output |
| **36** | 3V3 | Power Out | Optional: MOSFET driver VCC |
| **38** | GND | Ground | Common ground |
| **39** | VSYS | Power In | Battery positive |

### Voltage Divider Circuit

**Purpose**: Scale 3.7V battery voltage to safe ADC range (0-3.3V)

**Circuit**:
```
Battery Positive ──┬── 10kΩ (R1) ──┬── 5.1kΩ (R2) ── Ground
                   │               │
                   └── To VSYS     └── To GPIO26 (ADC Input)
```

**Voltage Calculations**:
- Divider Ratio: R2/(R1+R2) = 5.1kΩ/15.1kΩ = 0.337
- 3.0V battery → 1.01V ADC
- 3.7V battery → 1.25V ADC
- 4.2V battery → 1.42V ADC

### MOSFET Driver Module Connections

| Driver Pin | Function | Connection |
|------------|----------|------------|
| **VCC** | Power Input | 3.3V from Pico (Pin 36) or Battery+ |
| **GND** | Ground | Common ground (Pin 38) |
| **IN** | Logic Input | GPIO15 (Pin 20) |
| **OUT+** | Load Positive | To electromagnetic coil positive |
| **OUT-** | Load Negative | To electromagnetic coil negative |

## Step-by-Step Assembly

### Phase 1: Power Connections

1. **Install the Raspberry Pi Pico**:
   - Insert into breadboard or solder to PCB
   - Ensure USB connector is accessible

2. **Connect battery**:
   ```
   Red wire (Battery +)   → VSYS (Pin 39)
   Black wire (Battery -) → GND (Pin 38)
   ```
   **⚠️ CRITICAL: Double-check polarity before connecting battery**

### Phase 2: Voltage Divider Circuit

3. **Install voltage divider resistors**:
   ```
   Step 3a: Place 10kΩ resistor
   - One end to battery positive (VSYS)
   - Other end to junction point

   Step 3b: Place 5.1kΩ resistor
   - One end to junction point
   - Other end to ground

   Step 3c: Connect junction to GPIO26
   - Wire from resistor junction to Pin 31 (GP26)
   ```

4. **Add filtering capacitor**:
   - 100nF ceramic capacitor between GPIO26 and ground

### Phase 3: MOSFET Driver Connection

5. **Install MOSFET driver module**:
   - Mount driver module on breadboard/PCB

6. **Connect driver**:
   ```
   VCC → 3.3V (Pin 36)
   GND → Common ground (Pin 38)
   IN  → GPIO15 (Pin 20)
   ```

7. **Connect electromagnetic load**:
   ```
   OUT+ → Coil positive terminal
   OUT- → Coil negative terminal
   ```

## Testing and Verification

### Power-On Test

1. **Connect battery** (start with partially charged battery)
2. **Verify Pico powers up** (LED should be controllable)
3. **Check voltage levels**:
   ```
   VSYS: Should equal battery voltage
   3V3: Should be 3.3V ± 0.1V
   GPIO26: Should be ~1/3 of battery voltage
   ```

### Functional Testing

```bash
# Flash firmware and test
cargo run

# Verify USB enumeration
lsusb | grep fade

# Test bootloader entry
python host_tools/bootloader_entry.py

# Monitor battery status
python host_tools/battery_monitor.py
```

## Safety Considerations

### Before Assembly
- [ ] Verify component ratings match specifications
- [ ] Check battery condition - no swelling or damage
- [ ] Ensure proper workspace with ESD protection

### During Assembly
- [ ] Disconnect power before making connections
- [ ] Verify polarity before connecting battery
- [ ] Test connections before applying power

### After Assembly
- [ ] Inspect for shorts before first power-on
- [ ] Monitor temperature during initial testing
- [ ] Test in safe environment

## Troubleshooting

### Power Problems

**Device doesn't power on:**
- Check battery voltage (should be >3.0V)
- Verify battery connector polarity
- Check VSYS connection to Pin 39

### ADC Reading Issues

**Incorrect battery voltage readings:**
- Verify resistor values with multimeter
- Check voltage divider connections
- Measure actual voltages at each point

### MOSFET Driver Problems

**No output from driver:**
- Verify driver power supply (VCC and GND)
- Check control signal from GPIO15
- Test with known good driver module

