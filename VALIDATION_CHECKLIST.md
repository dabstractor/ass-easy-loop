# CoilDriver Real-World Validation Checklist

## Quick Validation (No Special Equipment)

### 1. LED Test - Easiest
**What you'll need:**
- LED (any color)
- 220Ω to 470Ω resistor
- Jumper wires

**Setup:**
```
GPIO 15 ──┐
           │
         [LED]
           │
         [220Ω]
           │
          GND
```

**Expected behavior:**
- LED OFF on power-up (safety check)
- LED blinks at 1Hz in loop()
- LED responds to serial commands in validation sketch

---

### 2. Multimeter Test - Confirms Voltage Levels
**What you'll need:**
- Digital multimeter
- Test probes

**Setup:**
- Red probe on GPIO 15 pin
- Black probe on any GND pin
- Set to DC voltage range (0-20V)

**Expected readings:**
- 0.0V when coil.setActive(false)
- ~3.3V when coil.setActive(true)
- Should transition cleanly between states

---

## Full System Validation (With MOSFET & Coil)

### 3. Current Measurement Test
**What you'll need:**
- IRF520 MOSFET driver module
- Coil (as specified in PRD Section 2.2)
- 1N4007 or 1N5408 flyback diode (CRITICAL!)
- Multimeter with current mode OR current probe

**Safety warning:** NEVER run MOSFET without flyback diode! Back-EMF can destroy the MOSFET.

**Expected current:**
- 0mA when OFF
- ~925mA when ON (3.7V / 4Ω)
- Should ramp up smoothly, no spikes when turning OFF (thanks to flyback diode)

### 4. Oscilloscope Test (If Available)
**What to check:**
- Clean 3.3V high level
- 0V low level
- No ringing or bounce on edges
- Fast transitions (should be <1µs rise/fall time)

---

## Critical Failure Signs (STOP if you see these)

### 🛑 IMMEDIATE STOP SIGNALS:
1. **LED always on after power-up** - GPIO not defaulting LOW
2. **Smoke or hot MOSFET** - Missing flyback diode or wiring error
3. **Erratic behavior** - Floating pin, need pull-down resistor
4. **Back-EMF spikes on scope** - Flyback diode missing or backwards

### ⚠️ WARNING SIGNS:
1. **LED dimly lit when should be OFF** - Floating pin
2. **Slow voltage transitions** - Overloaded GPIO pin
3. **MOSFET gets warm** - Gate not fully driven

---

## Validation Commands

### To run validation:
```bash
# Backup current main.cpp
cp src/main.cpp src/main.cpp.bak

# Use validation sketch
cp src/validate_coildriver.cpp src/main.cpp

# Upload and test
pio run -t upload
pio device monitor

# Restore original when done
cp src/main.cpp.bak src/main.cpp
```

### Expected serial output:
```
=== CoilDriver Validation Test ===
1. LED Test: Connect LED+220Ω between GPIO 15 and GND
2. Multimeter Test: Probe GPIO 15 for voltage changes
3. MOSFET Test: Connect IRF520 module + coil + flyback diode
4. Safety Check: Coil should be OFF on startup

Test 1: Initializing CoilDriver...
✓ begin() called - coil should be OFF (LOW)
Test 2: Setting coil ACTIVE...
✓ setActive(true) - LED should be ON, voltage ~3.3V
Test 3: Setting coil INACTIVE...
✓ setActive(false) - LED should be OFF, voltage 0V
Test 4: Rapid cycling (simulating 10Hz)...

LED ON - GPIO 15 HIGH
LED OFF - GPIO 15 LOW
LED ON - GPIO 15 HIGH
...
```

---

## When to Proceed

✅ **SAFE TO PROCEED if:**
- LED test passes (blinks correctly)
- Multimeter shows 0V and 3.3V cleanly
- No smoke or excessive heat
- GPIO starts LOW on power-up

❌ **DO NOT PROCEED if:**
- LED stays on after power reset
- You see any smoke or smell burning
- MOSFET gets hot
- Readings are erratic or unexpected

---

## Next Steps After Validation

Once validation passes:
1. You can confidently proceed to P1.M2.T2.S1 (FeedbackDriver HAL)
2. Your hardware setup is correct for future tasks
3. You have a known-good reference for troubleshooting

Remember: It's better to spend 10 minutes validating now than hours debugging mysterious issues later!