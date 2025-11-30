# Device Validation

## Quick Test (Recommended)

**What you'll need:**
- A refrigerator magnet (any common household magnet)

**Steps:**
1. Press the reset button to start the device
2. Confirm the green LED is flashing rapidly (10 times per second)
3. Hold the magnet loosely in your hand
4. Slowly bring the magnet close to the loop
5. You should feel the magnet vibrating in your hand

If you feel the vibration and see the LED flashing, the device is working correctly.

---

## Troubleshooting

### No LED flashing
- Check battery connections
- Verify batteries are charged
- Press reset button firmly

### LED flashes but no magnet vibration
- Check MOSFET connections
- Verify flyback diode orientation (stripe/cathode to positive side)
- Ensure there is NO resistor in series between GPIO 15 and the MOSFET
- Verify coil is connected to MOSFET output

### Device gets hot
- Stop use immediately
- Check for short circuits
- Verify flyback diode is installed
- Verify coil resistance is around 4 ohms
