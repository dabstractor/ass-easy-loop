(For Planning and Implementation Agents)

Project Requirements Document (PRD)
1. Overview

Project: Ass-Easy-Loop (10Hz pEMF Driver)
Target: Veterinary Therapeutic Device.
Core Design: A simplified "Super Loop" firmware architecture running on an RP2040-Zero, powered by a 1600mAh Li-Ion battery bank, driving a high-current magnetic coil via an IRF520 MOSFET.
Stack: C++ / Arduino Framework (PlatformIO).

2. Hardware Specifications
2.1 Bill of Materials (BOM)

Controller: RP2040-Zero (Generic/Waveshare).

Requirement: Must feature onboard WS2812 RGB LED.

Power Core:

Batteries: 2x 18300 Li-Ion (3.7V, 800mAh, 2.96Wh each). Wired in Parallel.

Total Capacity: 1600mAh / 5.92Wh.

Management: TP4056 Module (or harvested Vape Charging Board equivalent).

Requirement: Must handle 1A charging and provide Over-Discharge protection.

Driver: IRF520 MOSFET Driver Module.

Coil: ~4 Ohm Loop (Custom Geometry).

Wire: 22AWG Enameled Copper.

Length: ~252ft (approx 0.5 lbs).

Geometry: 7.5" Diameter loop. Wound 12 coils wide, 11 layers deep.

Feedback:

Audio: Piezo Buzzer (Passive or Active).

Visual: Onboard RP2040-Zero RGB LED.

Protection: Flyback Diode (1N4007 or 1N5408).

2.2 Wiring Definitions

Power Hub (TP4056/Vape Board):

B+ / B- 
→
→
 Battery Bank (Permanent connection).

OUT+ (or B+) 
→
→
 RP2040 5V pin AND MOSFET VCC.

OUT- (or B-) 
→
→
 RP2040 GND pin AND MOSFET GND.

Signal Path:

RP2040 GPIO 15
→
→
 MOSFET TRIG/SIG.

GPIO 15 Pull-Down (Optional but Recommended):
100K-1M Ohm resistor from GPIO 15 to GND.
Purpose: Ensures MOSFET stays OFF during boot before firmware initializes.
WARNING: Do NOT put resistor in series between GPIO and MOSFET - this kills the signal.

RP2040 GPIO 14
→
→
 Piezo Buzzer Positive.

Output Path:

MOSFET Output + 
→
→
 Coil Start AND Diode Cathode (Stripe).

MOSFET Output - 
→
→
 Coil End AND Diode Anode (Black).

3. Runtime & Electrical Analysis
3.1 Power Consumption Math

Voltage: 3.7V (Nominal).

Coil Resistance: ~4.0 
Ω
Ω
.

Peak Current (Pulse): 
𝐼
=
𝑉
/
𝑅
=
3.7
/
4.0
=
0.925
I=V/R=3.7/4.0=0.925
 Amps.

Duty Cycle: 2% (2ms ON / 98ms OFF).

Average Coil Current: 
0.925
𝐴
×
0.02
=
18.5
0.925A×0.02=18.5
 mA.

System Overhead (RP2040 + LED): ~50 mA.

Total Average Draw: ~68.5 mA.

3.2 Battery Life Projection

Total Capacity: 1600 mAh.

Calculated Runtime: 
1600
𝑚
𝐴
ℎ
/
68.5
𝑚
𝐴
≈
23.3
1600mAh/68.5mA≈23.3
 Hours.

Session Capacity: 
23.3
𝐻
𝑜
𝑢
𝑟
𝑠
/
0.25
𝐻
𝑜
𝑢
𝑟
𝑠
(
15
𝑚
𝑖
𝑛
𝑠
)
=
93
23.3Hours/0.25Hours(15mins)=93
 Sessions.

Note: Real-world efficiency losses may reduce this, but it significantly exceeds the 45-minute baseline of the previous prototype.

4. Functional Requirements
4.1 Therapeutic Waveform

Frequency: 10 Hz.

Period: 100 ms.

Pulse (Active): 2 ms.

GPIO 15: HIGH.

GPIO 14: HIGH (Tone).

Onboard LED: Green (Low Brightness).

Rest (Inactive): 98 ms.

GPIO 15: LOW.

GPIO 14: LOW.

Onboard LED: OFF.

4.2 Operating Logic

Boot: Initialize USB Serial, Pin Modes, and Start Timer.

Run: Loop 10Hz waveform for exactly 15 Minutes.

Stop: Force all outputs LOW. Enter blocking loop.

Reset: Hardware Reset button required to restart.

4.3 Indicators

Run State: Onboard LED flashes Green at 10Hz.

Charge State: Handled by the TP4056/Vape Board hardware LEDs (Red/Blue). The RP2040 is powered down during charging.

5. Developer Surface
5.1 Toolchain

Platform: PlatformIO.

Board Definition: rp2040 (Generic).

Software Core: earlephilhower/arduino-pico (This is the required software driver for the chip).

Library: Adafruit NeoPixel (Required to drive the RP2040-Zero's specific onboard LED).

5.2 Build Command

Command: pio run -t upload

Behavior: Compiles, Auto-detects Bootloader, Flashes.

5.3 Bootloader Backdoor

Mechanism: Magic 1200 Baud Reset.

Implementation: Serial.begin(115200) must be called in setup() to enable the Core's background listener.
