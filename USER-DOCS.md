(For Documentation Agent and End-User Reference)

User Manual: Ass-Easy-Loop Dog Relief Device
1. Device Description

This is a high-power, pulsed electromagnetic field (pEMF) loop designed for veterinary therapy. It pulses a magnetic field at 10Hz to reduce inflammation and pain.

2. Assembly & Fabrication
2.1 The Coil (The Hard Part)

Material: 22AWG Enameled Copper Wire.

Jigs (Provided in Repo):

Coil_Winder_Jig.stl: Use this to wind the wire.

Silicone_Mold_Jig.stl: Use this to encase the finished coil.

Winding Specs:

Diameter: 7.5 inches.

Width: 12 coils wide.

Depth: 11 layers deep.

Total Length: ~252 feet.

Finishing: Encase in silicone using the mold to prevent thermal burns and damage.

2.2 The Electronics (The Brains)

Controller: RP2040-Zero.

Power: 2x 18300 Batteries (1600mAh total).

Switching: IRF520 MOSFET.

Charging: TP4056 or Vape Charging Board.

Safety Warning: You MUST install the Diode across the output terminals.

Stripe End 
→
→
 Positive (+)

Black End 
→
→
 Negative (-)

Failure to do this will destroy the electronics instantly.

3. Operation
3.1 Turning It On

Press the small RESET button on the RP2040-Zero controller.

The device will immediately begin pulsing.

3.2 Indicators

Running: The light on the controller will flash Green rapidly, and you will hear a faint buzzing/clicking sound (10 times a second).

Finished: After 15 minutes, the device will stop buzzing and the light will turn off.

Charging: Check the lights on the Charging Board (TP4056).

Red: Charging.

Blue/Green: Fully Charged.

Note: Do not use the device on the dog while it is plugged into the wall.

4. Manual Validation (Do This Before Use)

Before strapping this to your dog, verify it is working correctly.

Test A: The "Compass" Test

Place a standard hiking compass inside the loop.

Turn the device on.

Result: The compass needle should vibrate or oscillate visibly at the frequency of the sound.

Test B: The "Magnet" Test

Hold a strong magnet (rare earth) near the silicone ring.

Turn the device on.

Result: You should feel a physical "kick" or "thump" against your hand synchronized with the buzzing sound.

Test C: The Heat Check

Run the device for 2 minutes.

Place your hand on the MOSFET (the driver chip) and the Coil ring.

Result: They should be warm, but not too hot to touch. If they are burning hot, unplug the battery immediately.

5. Battery & Charging

Capacity: 1600mAh.

Runtime: The device can perform approximately 90+ sessions on a single charge.

Recommendation: Charge weekly to keep the voltage optimal.
