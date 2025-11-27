# Ass-Easy-Loop DIY pEMF Therapy Device

A complete guide to building a pulsed electromagnetic field (pEMF) therapy device for veterinary use. This device delivers 10Hz magnetic pulses to help reduce inflammation and pain in animals.

## What This Device Does

The Ass-Easy-Loop creates a pulsing magnetic field at 10 cycles per second (10Hz). Each magnetic pulse helps reduce swelling and pain in tissues. A full treatment session lasts exactly 15 minutes and then shuts off automatically.

This is designed specifically for veterinary therapy - helping dogs and other animals recover from injuries, reduce inflammation, and manage pain.

## Safety First

This device involves:
- High current electrical circuits
- Strong magnetic fields
- Lithium batteries that can be dangerous if mishandled

**Important:** You MUST install the protection diode exactly as shown in the instructions. Failure to do this will destroy the electronics instantly.

## Parts You Need

### The Coil (Magnetic Loop)
- 22AWG enameled copper wire - approximately 252 feet (0.5 lbs)
- Silicone mold material for potting the finished coil

### The Electronics
- RP2040-Zero microcontroller board
- IRF520 MOSFET driver module
- TP4056 battery charging board
- 2x 18300 lithium batteries (3.7V, 800mAh each)
- 1N4007 or 1N5408 protection diode
- Piezo buzzer for audio feedback
- Small push button for reset

### Tools and Supplies
- Wire stripper and cutter
- Soldering iron and solder
- Multimeter for testing
- 3D printed jigs (files provided)

## Building the Coil

The coil is the most important part and requires careful construction.

### Winding the Coil

Use the provided Coil_Winder_Jig.stl to wind your wire:

1. **Dimensions:** 7.5 inches diameter
2. **Pattern:** 12 coils wide, 11 layers deep
3. **Total wire:** About 252 feet
4. **Technique:** Wind neatly and tightly, keeping each turn flat against the previous

### Potting the Coil

After winding, you must protect the coil:

1. Use the Silicone_Mold_Jig.stl to create a mold
2. Place the wound coil in the mold
3. Pour silicone around the coil
4. Let it cure completely
5. The silicone prevents burns and protects the wire

## Wiring the Electronics

Follow this exact wiring diagram:

### Battery Connections
- Connect both 18300 batteries in parallel to the TP4056
- Red wires go to positive (+) terminals
- Black wires go to negative (-) terminals

### Power Distribution
- TP4056 output positive goes to both RP2040 5V pin AND MOSFET VCC
- TP4056 output negative goes to both RP2040 GND pin AND MOSFET GND

### Signal Connections
- RP2040 GPIO 15 connects to MOSFET trigger/signal pin
- RP2040 GPIO 14 connects to piezo buzzer positive
- Piezo buzzer negative connects to ground

### Output Protection (Critical)

**You MUST install this diode correctly:**

```
MOSFET Output (+) -----> Coil Start -----> Diode Stripe End (Cathode)
MOSFET Output (-) -----> Coil End   -----> Diode Black End (Anode)
```

The diode goes across the coil terminals - stripe end to positive coil connection, black end to negative. If you get this backwards, you will destroy the MOSFET instantly.

## Setting Up the Software

The device uses PlatformIO for programming:

1. Install PlatformIO on your computer
2. Open this project folder in PlatformIO
3. Connect the RP2040-Zero to your computer
4. Upload the firmware using: `pio run -t upload`

## Testing Before Use

Before using this on any animal, perform these tests:

### Test 1: Compass Test
1. Place a compass inside the silicone loop
2. Turn on the device (press the reset button)
3. You should see the compass needle vibrate rapidly
4. This confirms the magnetic field is working

### Test 2: Magnet Test
1. Hold a strong magnet near the silicone ring
2. Turn on the device
3. You should feel a physical "kick" or "thump" against the magnet
4. This confirms the pulse strength

### Test 3: Heat Test
1. Run the device for 2 minutes
2. Touch the MOSFET and coil
3. They should be warm but not painfully hot
4. If burning hot, disconnect the battery immediately

## How to Use

### Starting a Treatment
1. Press the small reset button on the RP2040-Zero
2. The device starts immediately
3. Green light flashes rapidly and you hear buzzing
4. Place the loop around the treatment area on your animal

### During Treatment
- The green light flashes 10 times per second
- You hear a soft buzzing/clicking sound
- Each buzz is one magnetic pulse
- Treatment automatically stops after 15 minutes

### When Treatment Ends
- Buzzing stops
- Green light turns off
- Device is safe to remove

### Charging
1. Connect the TP4056 to USB power
2. Red light means charging
3. Blue/green light means fully charged
4. **Never use the device while charging**

## Battery Life

With 1600mAh total capacity:
- Each 15-minute session uses significant power
- You get about 9 sessions per charge
- At 3-4 sessions daily, charge every 2-3 days

### Battery Capacity Guide
| Battery Size | Sessions | Days of Use (3-4 daily) |
|--------------|----------|-------------------------|
| 200mAh       | 1        | 0.3 days               |
| 250mAh       | 1.5      | 0.5 days               |
| 500mAh       | 3        | 1 day                  |
| 750mAh       | 4.5      | 1.5 days               |
| 1000mAh      | 6        | 2 days                 |
| 1600mAh      | 9        | 2.5-3 days             |

## Treatment Guidelines

### When to Use
- After injuries or surgery
- For chronic inflammation
- Pain management
- As recommended by your veterinarian

### Treatment Schedule
- 15-minute sessions
- 3-4 times per day maximum
- Wait at least 2 hours between sessions
- Continue for as long as recommended by your vet

### Placement
- Place the loop directly around the affected area
- Ensure good contact with the body
- Remove any metal objects from the area first

## Troubleshooting

### Device Won't Turn On
- Check battery connections
- Verify batteries are charged
- Press reset button firmly

### No Magnetic Field
- Check MOSFET connections
- Verify the diode is installed correctly
- Test with compass method described above

### Device Gets Too Hot
- Stop use immediately
- Check for short circuits
- Verify coil resistance is around 4 ohms
- Ensure MOSFET is properly sized

### Battery Life Too Short
- Check for power leaks
- Verify battery health
- Consider using larger capacity batteries

## Technical Specifications

- **Frequency:** 10Hz (10 pulses per second)
- **Pulse Duration:** 2 milliseconds
- **Session Length:** 15 minutes automatic
- **Coil Resistance:** ~4 ohms
- **Peak Current:** ~0.925 amps
- **Battery:** 1600mAh 3.7V lithium
- **Size:** 7.5 inch diameter loop

## Important Notes

- This is a DIY medical device - build carefully
- Consult with your veterinarian before use
- Not intended to replace professional veterinary care
- Keep away from pacemakers and electronic implants
- Store away from water and extreme temperatures
- Check all connections before each use

## Getting Help

If you have issues during building:
1. Review all wiring connections
2. Verify component orientations
3. Test with a multimeter
4. Check the project documentation for updates

Remember: Safety first for both you and the animals you're helping.