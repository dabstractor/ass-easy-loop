# Wiring Guide and Assembly Instructions

This document provides detailed wiring diagrams, component connections, and step-by-step assembly instructions for the pEMF Dual-Function Device.

## Overview

The device consists of three main circuit sections:
1. **Power and Control** - Raspberry Pi Pico with power management
2. **Battery Monitoring** - Voltage divider circuit for ADC input
3. **pEMF Driver** - MOSFET driver module for electromagnetic field generation

## Complete Wiring Diagram

```
                    pEMF Dual-Function Device Wiring Diagram
                    
    Battery Pack                 Raspberry Pi Pico                    MOSFET Driver
    ┌─────────────┐             ┌─────────────────────┐              ┌─────────────┐
    │             │             │                     │              │             │
    │     3.7V    │             │  ┌─────────────┐    │              │    VCC      │
    │   LiPo      │             │  │             │    │              │     │       │
    │   Battery   │             │  │   RP2040    │    │              │    ┌▼┐      │
    │             │             │  │             │    │              │    │ │      │
    │             │             │  │             │    │              │    │M│      │
    │    [+]──────┼─────────────┼──┤ VSYS        │    │              │    │O│      │
    │             │             │  │             │    │              │    │S│      │
    │             │             │  │             │    │              │    │F│      │
    │             │             │  │        GPIO15├───┼──────────────┼────┤E│      │
    │             │             │  │             │    │              │    │T│      │
    │             │             │  │             │    │              │    └┬┘      │
    │             │             │  │        GPIO25├─────── LED       │     │       │
    │             │             │  │             │    │              │    GND      │
    │             │             │  │             │    │              │     │       │
    │             │             │  │        GPIO26├───┼──┐           │     │       │
    │             │             │  │             │    │  │           └─────┼───────┘
    │             │             │  │             │    │  │                 │
    │             │             │  │         GND ├────┼──┼─────────────────┼───┐
    │             │             │  └─────────────┘    │  │                 │   │
    │    [-]──────┼─────────────┼──────────────────────┼──┼─────────────────┘   │
    │             │             │                     │  │                     │
    └─────────────┘             └─────────────────────┘  │                     │
                                                         │                     │
                                    Voltage Divider     │                     │
                                    ┌─────────────────┐  │                     │
                                    │                 │  │                     │
                                    │   10kΩ Resistor │  │                     │
                                    │   ┌─────────┐   │  │                     │
                                    │   │         │   │  │                     │
                    Battery [+] ────┼───┤  10kΩ   ├───┼──┘                     │
                                    │   │         │   │                        │
                                    │   └────┬────┘   │                        │
                                    │        │        │                        │
                                    │        ├────────┼─── To GPIO26 (ADC)     │
                                    │        │        │                        │
                                    │   ┌────▼────┐   │                        │
                                    │   │         │   │                        │
                                    │   │  5.1kΩ  ├───┼────────────────────────┘
                                    │   │         │   │
                                    │   └─────────┘   │
                                    │                 │
                                    └─────────────────┘
```

## Detailed Pin Connections

### Raspberry Pi Pico Connections

| Pico Pin | Pin Name | Function | Connection |
|----------|----------|----------|------------|
| **1** | GP0 | - | Not used |
| **2** | GP1 | - | Not used |
| **3** | GND | Ground | Common ground |
| **4-19** | GP2-GP14 | - | Not used |
| **20** | GP15 | pEMF Control | To MOSFET driver input |
| **21** | GP16-GP24 | - | Not used |
| **25** | GP25 | LED Control | Onboard LED (built-in) |
| **31** | GP26 | Battery ADC | To voltage divider output |
| **36** | 3V3 | Power Out | Optional: MOSFET driver VCC |
| **38** | GND | Ground | Common ground |
| **39** | VSYS | Power In | Battery positive |
| **40** | VBUS | USB Power | USB 5V (when connected) |

### Battery Connection Details

```
Battery Connector (JST-PH 2-pin recommended):
┌─────────────────────────────────────┐
│  Red Wire    │  Black Wire          │
│  (Positive)  │  (Negative/Ground)   │
│      │       │       │              │
│      ▼       │       ▼              │
│   VSYS       │     GND              │
│   (Pin 39)   │   (Pin 38)           │
└─────────────────────────────────────┘
```

### Voltage Divider Circuit

**Purpose**: Scale 3.7V battery voltage to safe ADC range (0-3.3V)

**Components**:
- R1: 10kΩ resistor (1% tolerance recommended)
- R2: 5.1kΩ resistor (1% tolerance recommended)

**Circuit**:
```
Battery Positive ──┬── 10kΩ (R1) ──┬── 5.1kΩ (R2) ── Ground
                   │               │
                   │               └── To GPIO26 (ADC Input)
                   │
                   └── To VSYS (Pin 39)
```

**Voltage Calculations**:
- Divider Ratio: R2/(R1+R2) = 5.1kΩ/15.1kΩ = 0.337
- 3.0V battery → 1.01V ADC
- 3.7V battery → 1.25V ADC  
- 4.2V battery → 1.42V ADC

### MOSFET Driver Module Connections

**Typical N-Channel MOSFET Driver Module**:

| Driver Pin | Function | Connection |
|------------|----------|------------|
| **VCC** | Power Input | 3.3V from Pico (Pin 36) or Battery+ |
| **GND** | Ground | Common ground (Pin 38) |
| **IN** | Logic Input | GPIO15 (Pin 20) |
| **OUT+** | Load Positive | To electromagnetic coil positive |
| **OUT-** | Load Negative | To electromagnetic coil negative |

**Alternative: Discrete MOSFET Circuit**:
```
GPIO15 ──┬── 1kΩ ──┬── Gate (MOSFET)
         │         │
         └── 10kΩ ──┴── Source (MOSFET) ── GND
                    
Drain (MOSFET) ── Load Positive
Load Negative ── Battery Positive
```

## Step-by-Step Assembly Instructions

### Phase 1: Prepare Components

1. **Gather all components** from the hardware requirements list
2. **Test the Raspberry Pi Pico**:
   - Connect via USB
   - Verify it appears as a USB device
   - Test with a simple blink program if possible

3. **Prepare the breadboard or PCB**:
   - Clean the surface
   - Plan component placement
   - Mark pin locations if using PCB

### Phase 2: Power Connections

⚠️ **Safety First**: Disconnect all power sources before making connections

4. **Install the Raspberry Pi Pico**:
   - Insert into breadboard or solder to PCB
   - Ensure all pins are properly seated
   - Double-check orientation (USB connector should be accessible)

5. **Create ground bus**:
   - Connect GND (Pin 38) to breadboard ground rail
   - This will be your common ground reference

6. **Install battery connector**:
   - Solder JST-PH connector or wire leads
   - **Red wire** → VSYS (Pin 39)
   - **Black wire** → GND (Pin 38)
   - **Double-check polarity** before connecting battery

### Phase 3: Voltage Divider Circuit

7. **Install voltage divider resistors**:
   ```
   Step 7a: Place 10kΩ resistor
   - One end to battery positive (VSYS)
   - Other end to junction point
   
   Step 7b: Place 5.1kΩ resistor  
   - One end to junction point
   - Other end to ground
   
   Step 7c: Connect junction to GPIO26
   - Wire from resistor junction to Pin 31 (GP26)
   ```

8. **Add filtering capacitor** (optional but recommended):
   - 100nF ceramic capacitor
   - Connect between GPIO26 and ground
   - Reduces ADC noise

### Phase 4: MOSFET Driver Connection

9. **Install MOSFET driver module**:
   - Mount driver module on breadboard/PCB
   - Ensure adequate spacing for heat dissipation

10. **Connect driver power**:
    - **VCC** → 3.3V (Pin 36) or Battery+ (VSYS)
    - **GND** → Common ground

11. **Connect control signal**:
    - **IN** (driver input) → GPIO15 (Pin 20)
    - Use short, direct connection to minimize noise

### Phase 5: Load Connections

12. **Connect electromagnetic load**:
    - **OUT+** → Coil positive terminal
    - **OUT-** → Coil negative terminal
    - Ensure load current is within driver specifications

13. **Add protection components** (recommended):
    - Flyback diode across inductive loads
    - Fuse in series with battery positive
    - TVS diode for transient protection

### Phase 6: Testing and Verification

14. **Visual inspection**:
    - Check all connections against wiring diagram
    - Verify no short circuits between power rails
    - Ensure proper component orientation

15. **Continuity testing**:
    - Use multimeter to verify connections
    - Test for shorts between VCC and GND
    - Verify GPIO connections

16. **Power-on test**:
    - Connect battery (start with partially charged battery)
    - Verify Pico powers up (LED should be controllable)
    - Check voltage levels with multimeter

## Component Placement Guidelines

### Breadboard Layout

```
    A  B  C  D  E     F  G  H  I  J
 1  [     Power Rails     ] [  GND  ]
 2  
 3     [10kΩ]
 4        │
 5     [Junction]──── GPIO26
 6        │
 7     [5.1kΩ]
 8        │
 9     [ GND ]
10  
11  [  Raspberry Pi Pico  ]
12  [                     ]
13  [                     ]
14  [     GPIO15 ────────────── MOSFET Driver
15  [                     ]
16  [                     ]
17  [                     ]
18  [                     ]
19  [                     ]
20  [                     ]
```

### PCB Layout Considerations

- **Keep power traces wide** (minimum 0.5mm for 1A current)
- **Separate analog and digital grounds** if possible
- **Place voltage divider close to ADC pin** to minimize noise
- **Add test points** for debugging and calibration
- **Include mounting holes** for enclosure attachment

## Troubleshooting Common Issues

### Power Problems

**Symptom**: Device doesn't power on
- **Check**: Battery voltage (should be >3.0V)
- **Check**: Battery connector polarity
- **Check**: Fuse continuity (if installed)
- **Solution**: Verify VSYS connection to Pin 39

**Symptom**: Inconsistent operation
- **Check**: Battery capacity and charge level
- **Check**: Power supply noise
- **Solution**: Add decoupling capacitors near Pico

### ADC Reading Issues

**Symptom**: Incorrect battery voltage readings
- **Check**: Voltage divider resistor values
- **Check**: ADC reference voltage (should be 3.3V)
- **Check**: Connection to GPIO26
- **Solution**: Calibrate using known battery voltages

**Symptom**: Noisy ADC readings
- **Check**: Grounding and shielding
- **Check**: Capacitor placement
- **Solution**: Add filtering capacitor, improve ground connections

### MOSFET Driver Problems

**Symptom**: No output from MOSFET driver
- **Check**: Driver power supply (VCC and GND)
- **Check**: Input signal from GPIO15
- **Check**: Driver enable pins (if present)
- **Solution**: Verify driver module specifications

**Symptom**: Weak or distorted output
- **Check**: Driver current rating vs. load requirements
- **Check**: Heat dissipation and thermal protection
- **Solution**: Upgrade to higher current driver or add heat sink

### Timing Issues

**Symptom**: Incorrect pulse timing
- **Check**: Crystal oscillator (12MHz external)
- **Check**: Software configuration
- **Solution**: Verify clock configuration in software

### Connection Verification Checklist

Before first power-on, verify:

- [ ] Battery polarity correct (Red=+, Black=-)
- [ ] No shorts between VCC and GND
- [ ] Voltage divider resistor values correct
- [ ] GPIO15 connected to MOSFET driver input
- [ ] GPIO26 connected to voltage divider output
- [ ] All ground connections secure
- [ ] MOSFET driver power connections correct
- [ ] Load connections secure and properly rated

## Safety Reminders

⚠️ **Before Each Use**:
- Inspect all connections for damage
- Verify battery voltage and condition
- Check for loose connections
- Ensure proper ventilation for heat dissipation

⚠️ **During Operation**:
- Monitor device temperature
- Watch for unusual LED patterns
- Listen for abnormal sounds from electromagnetic loads
- Be prepared to disconnect power quickly if needed

---

**Next**: Continue with software setup and flashing instructions in section 10.3.