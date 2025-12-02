Image Generation Prompt: Ass-Easy-Loop pEMF Wiring Diagram

IMAGE FORMAT: SQUARE (1:1 aspect ratio)

---
STYLE

Futuristic electrofunk technical wiring diagram. Dark blue/purple cyberpunk background. Neon glow on wires. Clean routing. Square format.

---
CRITICAL INSTRUCTION

You are drawing THREE CIRCUIT BOARD MODULES and ONE SEPARATE DIODE COMPONENT. Each module is a small PCB with specific physical features. DO NOT draw bare electronic components (transistors, chips). Draw the MODULES as they appear in real
life - small rectangular circuit boards with labeled terminals.

---
COMPONENT A: TP4056 USB CHARGING MODULE

Physical Description

- Shape: Small BLUE rectangular PCB, approximately 26mm × 17mm
- Color: BLUE circuit board with white silkscreen text
- USB connector: USB-C port on the RIGHT EDGE of the board
- LEDs: Two small SMD LEDs on the board surface (one red, one blue)

Terminal Layout

The TP4056 has exactly 6 through-hole solder pad terminals arranged in a ROW along the LEFT EDGE of the board (opposite the USB connector).

Looking at the board with USB-C on the right:

         LEFT EDGE                              RIGHT EDGE
    (terminals here)                          (USB-C here)
           │                                        │
           ▼                                        ▼
    ┌──────────────────────────────────────────────────┐
    │  ●    ●    ●    ●    ●    ●                      │
    │ OUT+ OUT-  B+   B-   +    -          [USB-C]     │
    │  (1)  (2)  (3)  (4)  (5)  (6)                    │
    │                                                  │
    │              [RED LED] [BLUE LED]                │
    │                                                  │
    └──────────────────────────────────────────────────┘

The 6 Terminals (Left to Right)

| Position | Label | Name            | Function                               | Wire To                                 | Color       |
|----------|-------|-----------------|----------------------------------------|-----------------------------------------|-------------|
| 1        | OUT+  | Charge Output + | External charge indicator LED positive | OPTIONAL external LED                   | Dashed red  |
| 2        | OUT-  | Charge Output - | External charge indicator LED negative | OPTIONAL external LED                   | Dashed blue |
| 3        | B+    | Battery +       | Battery positive input                 | Both 18300 battery (+) terminals        | RED         |
| 4        | B-    | Battery -       | Battery negative input                 | Both 18300 battery (-) terminals        | BLACK       |
| 5        | +     | Load +          | Main power output positive             | RP2040 5V AND IRF520 VCC AND IRF520 VIN | RED         |
| 6        | -     | Load -          | Main power output negative             | RP2040 GND AND IRF520 GND (both)        | BLUE        |

Wiring Rules for TP4056

- Terminal 3 (B+): ONE red wire going UP to battery positive
- Terminal 4 (B-): ONE black wire going UP to battery negative
- Terminal 5 (+): ONE red wire that BRANCHES to feed three destinations (RP2040 5V, IRF520 VIN, IRF520 VCC)
- Terminal 6 (-): ONE blue wire that BRANCHES to feed three destinations (RP2040 GND, IRF520 GND screw, IRF520 GND pin)
- Terminals 1 & 2: Dashed lines to "OPTIONAL: External Charge LED"

---
COMPONENT B: IRF520 MOSFET DRIVER MODULE

Physical Description

- Shape: Small RED rectangular PCB, approximately 32mm × 24mm
- Color: RED circuit board with white silkscreen text reading "MOS Module"
- Key feature: Large black IRF520 MOSFET transistor component mounted on the board
- Screw terminals: TWO blue plastic screw terminal blocks on the board

THIS IS A MODULE, NOT A BARE TRANSISTOR

Do NOT draw a bare 3-pin transistor. Draw the complete red PCB module with screw terminals.

Terminal Layout

The IRF520 module has exactly 7 terminals total:
- 4 screw terminals (in two 2-position blocks)
- 3 through-hole pins

Physical layout of the IRF520 module:

    ┌─────────────────────────────────────────────────────────┐
    │                                                         │
    │   ┌─────────────┐              ┌─────────────┐          │
    │   │ ■VIN   ■GND │              │ ■V+    ■V-  │          │
    │   │ (screw     )│              │(screw      )│          │
    │   │(terminals  )│              │(terminals  )│          │
    │   └─────────────┘              └─────────────┘          │
    │    POWER INPUT                  COIL OUTPUT             │
    │    SCREW BLOCK                  SCREW BLOCK             │
    │                                                         │
    │              ┌─────────────────┐                        │
    │              │                 │                        │
    │              │    [IRF520]     │                        │
    │              │   (black chip)  │                        │
    │              │                 │                        │
    │              └─────────────────┘                        │
    │                                                         │
    │                 "MOS Module"                            │
    │                                                         │
    │              ●          ●          ●                    │
    │             SIG        VCC        GND                   │
    │           (pin 5)    (pin 6)    (pin 7)                │
    │                                                         │
    │           (ACTIVE)   (ACTIVE)   (ACTIVE)               │
    │                                                         │
    └─────────────────────────────────────────────────────────┘

    BOTTOM PINS ORDER:  SIG (left) — VCC (middle) — GND (right)

The 7 Terminals

SCREW TERMINALS (top of board):

| #   | Label | Block       | Position in Block | Function      | Wire To               | Color       |
|-----|-------|-------------|-------------------|---------------|-----------------------|-------------|
| 1   | VIN   | Left block  | Left screw        | Power input + | TP4056 terminal 5 (+) | RED         |
| 2   | GND   | Left block  | Right screw       | Power input - | TP4056 terminal 6 (-) | BLUE        |
| 3   | V+    | Right block | Left screw        | Coil output + | COIL+ (via diode)     | RED thick   |
| 4   | V-    | Right block | Right screw       | Coil output - | COIL- (via diode)     | BLACK thick |

THROUGH-HOLE PINS (bottom of board):

| #   | Label | Position   | Function      | Wire To               | Color |
|-----|-------|------------|---------------|-----------------------|-------|
| 5   | SIG   | LEFT pin   | Signal input  | RP2040 GPIO 15        | GREEN |
| 6   | VCC   | MIDDLE pin | Logic power + | TP4056 terminal 5 (+) | RED   |
| 7   | GND   | RIGHT pin  | Logic ground  | TP4056 terminal 6 (-) | BLUE  |

CRITICAL: Bottom Pin Order

LEFT = SIG, MIDDLE = VCC, RIGHT = GND

This is NOT negotiable. SIG must be on the left. GND must be on the right.

---
COMPONENT C: RP2040-ZERO MICROCONTROLLER

Physical Description

- Shape: Small BLUE rectangular PCB, approximately 23mm × 18mm
- Color: BLUE circuit board with gold castellated edge pads
- USB connector: USB-C port on one short edge
- Buttons: Two small tactile buttons labeled BOOT and RESET
- Text: "RP2040-Zero" printed on the board

Terminal Layout

The RP2040-Zero has pins around all edges. We only care about 4 specific pins, all on the SAME EDGE of the board.

Orient the board so the USB-C points UP. The LEFT edge has these pins (top to bottom):

              [USB-C]
         ┌───────┴───────┐
         │               │
    5V ──●               │
   GND ──●               │
   3V3 ──●               │
         ●               │
         ●               │
         ●               │
         ●               │
GPIO 15──●               │
         │               │
         │ [BOOT][RESET] │
         │               │
         └───────────────┘

The 4 Relevant Terminals (all on LEFT edge)

| Pin     | Position         | Function       | Wire To                    | Color |
|---------|------------------|----------------|----------------------------|-------|
| 5V      | Top of left edge | Power input    | TP4056 terminal 5 (+)      | RED   |
| GND     | 2nd from top     | Ground         | TP4056 terminal 6 (-)      | BLUE  |
| 3V3     | 3rd from top     | 3.3V reference | NOT CONNECTED (label only) | —     |
| GPIO 15 | Lower left edge  | Signal output  | IRF520 SIG pin             | GREEN |

---
COMPONENT D: FLYBACK PROTECTION DIODE (Separate Component)

Physical Description

- Shape: Small black cylinder, approximately 5mm long × 2.5mm diameter
- Markings: Silver or white STRIPE band on one end (this is the CATHODE)
- Type: 1N5408 or 1N4007
- THIS IS NOT ON ANY BOARD - it is a separate discrete component with wire leads

Placement

The diode is wired EXTERNALLY between the IRF520 output terminals and the coil. It creates a parallel path.

    IRF520 V+ terminal ────┬────────────────┬──── COIL (+)
                           │                │
                           │   ┌────────┐   │
                           │   │ DIODE  │   │
                           └───┤ ▓▓▓▓▓▓ ├───┘
                               │ ▲      │
                               │STRIPE  │
                               │(cathode│
                               │ to +)  │
                           ┌───┤        ├───┐
                           │   └────────┘   │
                           │                │
    IRF520 V- terminal ────┴────────────────┴──── COIL (-)

Wiring

- Diode CATHODE (stripe end) connects to the V+ / COIL+ side
- Diode ANODE (plain end) connects to the V- / COIL- side
- The diode and coil share the same two connection points (parallel)

---
COMPONENT E: 18300 LITHIUM BATTERIES (×2)

Physical Description

- Two red cylindrical cells
- Labeled "18300 LITHIUM BATTERY (3.7V, 800mAh)"
- Each has + and - terminals

Wiring

- Connected in PARALLEL (both + together, both - together)
- Combined (+) → TP4056 terminal 3 (B+)
- Combined (-) → TP4056 terminal 4 (B-)

---
COMPONENT F: SILICONE ENCASED COIL

Physical Description

- Large toroidal (donut) ring shape
- Pink/magenta color
- Translucent silicone with copper windings visible inside
- 7.5 inches diameter (make it LARGE in the image)
- Label: "SILICONE ENCASED COIL (22AWG, ~4Ω, 252ft, 7.5" diameter)"

Terminals

- COIL+ receives wire from diode/V+ junction
- COIL- receives wire from diode/V- junction

---
COMPONENT G: OPTIONAL 10KΩ PULL-DOWN RESISTOR

Physical Description

- Small axial resistor
- Show with DASHED lines (indicating optional)

Wiring

- One leg to GPIO 15 signal line
- Other leg to GND
- Label: "OPTIONAL: 10KΩ Pull-Down (GPIO 15 to GND) - Keeps MOSFET OFF during boot"

---
COMPLETE WIRING TABLE

| From           | Terminal          | To             | Terminal          | Wire Color  | Notes               |
|----------------|-------------------|----------------|-------------------|-------------|---------------------|
| Battery 1 (+)  | +                 | Battery 2 (+)  | +                 | Red         | Parallel connection |
| Battery 1 (-)  | -                 | Battery 2 (-)  | -                 | Black       | Parallel connection |
| Batteries      | + (joined)        | TP4056         | B+ (terminal 3)   | Red         |                     |
| Batteries      | - (joined)        | TP4056         | B- (terminal 4)   | Black       |                     |
| TP4056         | + (terminal 5)    | RP2040-Zero    | 5V                | Red         | Branch 1 of 3       |
| TP4056         | + (terminal 5)    | IRF520         | VIN (screw)       | Red         | Branch 2 of 3       |
| TP4056         | + (terminal 5)    | IRF520         | VCC (pin, middle) | Red         | Branch 3 of 3       |
| TP4056         | - (terminal 6)    | RP2040-Zero    | GND               | Blue        | Branch 1 of 3       |
| TP4056         | - (terminal 6)    | IRF520         | GND (screw)       | Blue        | Branch 2 of 3       |
| TP4056         | - (terminal 6)    | IRF520         | GND (pin, right)  | Blue        | Branch 3 of 3       |
| RP2040-Zero    | GPIO 15           | IRF520         | SIG (pin, left)   | Green       | Signal              |
| IRF520         | V+ (screw)        | Diode junction | + side            | Red thick   |                     |
| IRF520         | V- (screw)        | Diode junction | - side            | Black thick |                     |
| Diode junction | + side            | Coil           | COIL+             | Red thick   |                     |
| Diode junction | - side            | Coil           | COIL-             | Black thick |                     |
| TP4056         | OUT+ (terminal 1) | External LED   | +                 | Dashed      | Optional            |
| TP4056         | OUT- (terminal 2) | External LED   | -                 | Dashed      | Optional            |

---
LAYOUT POSITIONS

┌────────────────────────────────────────────────────────────────┐
│                                                                │
│              [BATTERIES - TOP CENTER]                          │
│                    ║                                           │
│                    ▼                                           │
│              [TP4056 - UPPER CENTER]                           │
│         (6 terminals on left edge, USB-C on right)             │
│                    ║                                           │
│         ┌──────────╬──────────┐                                │
│         ▼                     ▼                                │
│   [IRF520 MODULE]      [RP2040-ZERO]                          │
│   LEFT SIDE            RIGHT SIDE                              │
│   (Red PCB)            (Blue PCB)                              │
│   (4 screws + 3 pins)  (USB-C pointing right)                  │
│         ║                                                      │
│    [DIODE]                                                     │
│    (separate component)                                        │
│         ║                                                      │
│         ▼                                                      │
│   [COIL - BOTTOM LEFT/CENTER]         [ROXANNE - BOTTOM RIGHT]│
│   (Large pink ring)                   (Dog with coil on chest) │
│                                                                │
└────────────────────────────────────────────────────────────────┘

---
MASCOT: ROXANNE THE DOG

Position: Bottom-right corner

Appearance:
- NOT a German Shepherd pattern (no black saddle)
- Body: Tan/brown colored fur on back and body
- Face: Black fur that has TURNED GREY WITH AGE
  - Grey/silver muzzle
  - Greying around eyes and eyebrows
  - Shows wisdom and age
- Ears: TRIANGULAR, STANDING STRAIGHT UP (erect pointed ears, not floppy)
- Expression: Calm, wise, gentle

Accessories:
- Futuristic goggles on head (cyan/teal glowing lenses)
- Cyber-collar with pink LED accents

Pose:
- Sitting or standing proudly
- Large 7.5" pEMF coil around her CHEST/TORSO (not neck)
- Coil should look proportionally large on her body

---
TEXT ELEMENTS

Title (top): "ASS-EASY-LOOP DIY pEMF THERAPY DEVICE"

Specs box (corner):
KEY SPECS:
• Frequency: 10Hz
• Pulse Duration: 2ms
• Session: 15 min (auto)
• Peak Current: ~925mA
• Battery: ~9 sessions

Wire legend (corner):
WIRE COLORS:
━━ Red = Power (+)
━━ Blue = Ground (-)
━━ Green = Signal
┄┄ Dashed = Optional

Near TP4056 LEDs: "CHARGING: RED=Charging, BLUE=Full"

Near diode (ONE callout only):
⚠️ CRITICAL: Diode MUST be parallel
across coil output. Stripe to positive.
Incorrect installation destroys MOSFET.

Near OUT+/OUT-: "OPTIONAL: External Charge LED"

---
ABSOLUTE REQUIREMENTS CHECKLIST

- TP4056 is a BLUE PCB MODULE with 6 terminals in a row, USB-C on opposite edge
- TP4056 terminals left-to-right: OUT+, OUT-, B+, B-, +, -
- IRF520 is a RED PCB MODULE labeled "MOS Module" (NOT a bare transistor)
- IRF520 has TWO screw terminal blocks (VIN/GND and V+/V-)
- IRF520 has THREE pins at bottom: SIG (LEFT), VCC (MIDDLE), GND (RIGHT)
- Diode is a SEPARATE cylindrical component, NOT on any board
- Diode bridges V+/V- to COIL+/COIL- in parallel
- RP2040-Zero shows 5V, GND, 3V3, GPIO15 on left edge
- GPIO 15 connects to SIG (the LEFT pin on IRF520)
- Power branches at TP4056, not mid-wire
- Roxanne has tan body, grey-black aged face, TRIANGULAR ERECT EARS
- Coil on Roxanne's CHEST, not neck
- Square aspect ratio

---
Generate this SQUARE image with exact terminal accuracy as specified.

