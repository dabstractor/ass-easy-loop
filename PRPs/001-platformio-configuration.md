# PRP-001: PlatformIO Configuration for RP2040-Zero

## Metadata
| Field | Value |
|-------|-------|
| PRP ID | 001 |
| Task Reference | P1.M1.T1.S1 |
| Story Points | 1 |
| Status | Ready for Implementation |
| Confidence Score | 9/10 |

---

## Goal

### Feature Goal
Create a complete PlatformIO configuration file (`platformio.ini`) that enables building and uploading firmware to an RP2040-Zero microcontroller using the Arduino framework with the earlephilhower/arduino-pico core.

### Deliverable
A single file: `platformio.ini` at project root.

### Success Definition
- Running `pio run` compiles without errors
- Running `pio run -t upload` successfully flashes the RP2040-Zero
- USB Serial communication works at 115200 baud via magic 1200 baud reset
- Adafruit NeoPixel library is available for the onboard WS2812 LED (GPIO 16)

---

## Context

### PRD Requirements (Section 5.1 - Toolchain)
```yaml
platform: PlatformIO
board_definition: rp2040 (Generic)
software_core: earlephilhower/arduino-pico
library: Adafruit NeoPixel
build_command: pio run -t upload
bootloader_mechanism: Magic 1200 Baud Reset
serial_requirement: Serial.begin(115200) in setup()
```

### Hardware Context (From PRD Section 2)
```yaml
controller: RP2040-Zero (Generic/Waveshare)
onboard_led: WS2812 RGB LED on GPIO 16
gpio_assignments:
  mosfet_signal: GPIO 15
  piezo_buzzer: GPIO 14
  neopixel_led: GPIO 16
```

### Research References
```yaml
platformio_docs:
  - url: https://docs.platformio.org/en/latest/platforms/raspberrypi.html
    section: "Raspberry Pi RP2040 Platform"
  - url: https://arduino-pico.readthedocs.io/en/latest/platformio.html
    section: "Using this core with PlatformIO"

local_research:
  - ./research/platformio-rp2040-config.md
  - ./research/arduino-pico-core.md
  - ./research/adafruit-neopixel.md
  - ./research/usb-serial-config.md

library_registry:
  - url: https://registry.platformio.org/libraries/adafruit/Adafruit%20NeoPixel
    note: "Library ID 28, requires version >= 1.10.2 for RP2040 PIO support"
```

### Key Technical Decisions
```yaml
platform_choice:
  selected: "https://github.com/maxgerhardt/platform-raspberrypi.git"
  rationale: "Provides full earlephilhower core support with latest updates"
  alternative: "raspberrypi (official but less feature-complete)"

board_choice:
  selected: "pico"
  rationale: "Generic pico board is most reliable; RP2040-Zero is hardware-compatible"
  gotcha: "waveshare_rp2040_zero board definition may have issues"

core_selection:
  directive: "board_build.core = earlephilhower"
  note: "Required when using maxgerhardt platform with boards that support multiple cores"

usb_stack:
  selected: "Pico SDK USB (default)"
  rationale: "Automatic USB Serial support, no extra flags needed"
  alternative: "-DUSE_TINYUSB for advanced USB features"
```

---

## Implementation Tasks

### Task 1: Create platformio.ini file
**Action:** Create new file at project root

**File:** `platformio.ini`

**Content:**
```ini
; PlatformIO Configuration for Ass-Easy-Loop pEMF Driver
; Target: RP2040-Zero (Waveshare) with earlephilhower/arduino-pico core

[env:rp2040_zero]
; Platform: Use maxgerhardt's fork for full earlephilhower support
platform = https://github.com/maxgerhardt/platform-raspberrypi.git

; Board: Generic Pico (RP2040-Zero is hardware-compatible)
board = pico

; Framework: Arduino
framework = arduino

; Core Selection: earlephilhower/arduino-pico
board_build.core = earlephilhower

; CPU Frequency: 133MHz (default for RP2040)
board_build.f_cpu = 133000000L

; Upload Protocol: picotool (uses magic 1200 baud reset)
upload_protocol = picotool

; Serial Monitor Speed: Must match Serial.begin() in code
monitor_speed = 115200

; Library Dependencies
lib_deps =
    adafruit/Adafruit NeoPixel@^1.12.0
```

**Placement:** Project root directory (`/home/dustin/projects/ass-ez-loop/platformio.ini`)

**Naming Convention:** Standard PlatformIO filename (must be exact)

---

## Validation Gates

### Gate 1: Configuration Syntax Validation
```bash
# Verify platformio.ini is valid
pio project config
```
**Expected:** No errors, displays parsed configuration

### Gate 2: Build Compilation
```bash
# Compile without uploading (no hardware required)
pio run
```
**Expected:** Successful compilation with output like:
```
Building .pio/build/rp2040_zero/firmware.uf2
=============== [SUCCESS] ===============
```

### Gate 3: Library Resolution
```bash
# Verify library is downloaded and available
pio pkg list
```
**Expected:** Shows `Adafruit NeoPixel` in installed packages

### Gate 4: Upload Test (requires hardware)
```bash
# Flash to connected RP2040-Zero
pio run -t upload
```
**Expected:** Successful upload via picotool or UF2 mass storage

---

## Final Validation Checklist

- [ ] `platformio.ini` exists at project root
- [ ] File contains `platform = https://github.com/maxgerhardt/platform-raspberrypi.git`
- [ ] File contains `board = pico`
- [ ] File contains `framework = arduino`
- [ ] File contains `board_build.core = earlephilhower`
- [ ] File contains `lib_deps` with Adafruit NeoPixel
- [ ] File contains `monitor_speed = 115200`
- [ ] `pio project config` runs without errors
- [ ] `pio run` compiles successfully
- [ ] `pio pkg list` shows Adafruit NeoPixel library

---

## Gotchas and Troubleshooting

### Gotcha 1: First Build Downloads
**Issue:** First `pio run` will download platform, toolchain, and framework (~500MB)
**Solution:** Allow time for download; subsequent builds are fast

### Gotcha 2: waveshare_rp2040_zero Board Issues
**Issue:** Using `board = waveshare_rp2040_zero` may cause issues
**Solution:** Use `board = pico` instead (hardware-compatible)

### Gotcha 3: Upload Failures
**Issue:** `picotool` upload may fail if device not in bootloader mode
**Solution:**
1. Press BOOTSEL button while plugging in USB
2. Device appears as "RPI-RP2" drive
3. Copy `.pio/build/rp2040_zero/firmware.uf2` to drive
4. Device auto-reboots

### Gotcha 4: Platform Update
**Issue:** Outdated platform packages
**Solution:**
```bash
pio pkg update -g -p https://github.com/maxgerhardt/platform-raspberrypi.git
```

### Gotcha 5: Linux USB Permissions
**Issue:** Permission denied on /dev/ttyACM0
**Solution:** Add user to `dialout` group:
```bash
sudo usermod -a -G dialout $USER
# Log out and back in
```

---

## Dependencies

### Upstream Dependencies
- None (this is the first subtask)

### Downstream Dependencies
- All subsequent firmware implementation tasks depend on this configuration
- P1.M1.T1.S2 and beyond require working build system

---

## Notes for Implementation Agent

1. **Single File Creation:** This task creates only `platformio.ini` - no other files needed
2. **No Code Yet:** Do not create any `.cpp` or `.h` files - those come in later subtasks
3. **Validation Focus:** After creating the file, run validation gates to confirm success
4. **Research Available:** Comprehensive research docs in `./research/` directory if clarification needed
