# PlatformIO RP2040-Zero Configuration Research

## 1. Platform and Board Specifications

### Platform Name for RP2040
- **Official Platform ID:** `raspberrypi`
- **Official Registry:** [platformio/raspberrypi](https://registry.platformio.org/platforms/platformio/raspberrypi)
- **Alternative (with earlephilhower support):** `https://github.com/maxgerhardt/platform-raspberrypi.git`

### Board Definition for RP2040-Zero (Waveshare)
- **Board ID:** `waveshare_rp2040_zero` (official but may have issues)
- **Alternative approach:** Use `pico` board definition (more reliable)

---

## 2. Framework Specification

**Framework:** Arduino (via the Wiring API)

---

## 3. earlephilhower/arduino-pico Core Specification

The earlephilhower/arduino-pico core is a feature-rich community-maintained Arduino core for RP2040. To use it:

**Platform Package Configuration:**
```ini
platform = https://github.com/maxgerhardt/platform-raspberrypi.git
board_build.core = earlephilhower
```

**Optional: Specify Custom Framework Version**
```ini
platform_packages =
    framework-arduinopico@https://github.com/earlephilhower/arduino-pico.git#master
```

The platform automatically manages these packages:
- `toolchain-rp2040-earlephilhower` (GCC 14.3/Newlib 4.5)
- `framework-arduinopico` (points to Arduino-Pico GitHub repo)

---

## 4. Complete platformio.ini Examples

### Recommended Configuration for RP2040-Zero (Waveshare)

**Option A: Using Waveshare-specific board (if supported)**
```ini
[env:rp2040_zero]
platform = raspberrypi
board = waveshare_rp2040_zero
framework = arduino
monitor_speed = 115200
upload_protocol = picotool
```

**Option B: Using generic Pico board (more reliable)**
```ini
[env:rp2040_zero]
platform = raspberrypi
board = pico
framework = arduino
monitor_speed = 115200
upload_protocol = picotool
```

**Option C: Using maxgerhardt's platform with earlephilhower core (most features)**
```ini
[env:rp2040_zero]
platform = https://github.com/maxgerhardt/platform-raspberrypi.git
board = pico
framework = arduino
board_build.core = earlephilhower
board_build.f_cpu = 133000000L
monitor_speed = 115200
upload_protocol = picotool
```

### Full-Featured Configuration with Filesystem and Libraries

```ini
[env:rp2040_zero_full]
platform = https://github.com/maxgerhardt/platform-raspberrypi.git
board = pico
framework = arduino
board_build.core = earlephilhower
board_build.f_cpu = 133000000L
board_build.filesystem_size = 0.5m

; Monitor settings
monitor_speed = 115200
monitor_port = /dev/ttyACM0

; Upload settings
upload_protocol = picotool

; Libraries for onboard WS2812 RGB LED (GPIO16)
lib_deps =
    fastled/FastLED
    adafruit/Adafruit NeoPixel

; Build flags - common options
build_flags =
    -DDEBUG_RP2040_CORE
    -DPIO_FRAMEWORK_ARDUINO_ENABLE_EXCEPTIONS
    -DPIO_FRAMEWORK_ARDUINO_ENABLE_RTTI
```

---

## 5. Common Build Flags for RP2040

### Debug Flags
```ini
build_flags =
    -DDEBUG_RP2040_CORE          ; Debug Core
    -DDEBUG_RP2040_SPI           ; Debug SPI
    -DDEBUG_RP2040_WIRE          ; Debug Wire (I2C)
    -DDEBUG_RP2040_PORT=Serial   ; Specify debug output port
```

### Feature Enablement Flags
```ini
build_flags =
    -DPIO_FRAMEWORK_ARDUINO_ENABLE_EXCEPTIONS  ; Enable C++ exceptions
    -DPIO_FRAMEWORK_ARDUINO_ENABLE_RTTI        ; Enable RTTI (Run-Time Type Information)
    -DPIO_FRAMEWORK_ARDUINO_ENABLE_FREERTOS    ; Enable FreeRTOS support
    -DPIO_FRAMEWORK_ARDUINO_ENABLE_BLUETOOTH   ; Enable Bluetooth support
```

### Performance & Memory Flags
```ini
; CPU Frequency (use in board_build.f_cpu instead)
board_build.f_cpu = 133000000L                ; Default 133MHz

; Filesystem allocation
board_build.filesystem_size = 0.5m            ; 512KB filesystem
board_build.filesystem_size = 1m              ; 1MB filesystem
```

---

## 6. Upload Protocols

Available upload protocols for RP2040:
- **picotool** (default) - requires picotool utility
- **cmsis-dap** - requires external debug probe
- **jlink** - requires J-LINK probe
- **raspberrypi-swd** - requires Raspberry Pi SWD probe

### UF2 Manual Upload (when picotool fails)

If automatic upload via picotool fails, you can manually upload:

```bash
# 1. Put the board into bootloader mode (press BOOT button while plugging in)
# 2. Device appears as RPI-RP2 drive
# 3. Copy the UF2 file
cp .pio/build/rp2040_zero/firmware.uf2 /run/media/username/RPI-RP2/
# Device will automatically flash and reboot
```

---

## 7. Common Gotchas and Solutions

### Issue 1: Picotool Upload Failures
**Problem:** Picotool may fail to upload the firmware
**Solution:**
- Manually copy the UF2 file to the RPI-RP2 drive when board is in bootloader mode
- Ensure the board is in bootloader mode (BOOT button pressed while plugging in)

### Issue 2: waveshare_rp2040_zero Board Definition Issues
**Problem:** Official `waveshare_rp2040_zero` board definition may not work reliably
**Solution:**
- Use the `pico` board definition instead (compatible hardware)
- Or use `https://github.com/maxgerhardt/platform-raspberrypi.git` platform with `board_build.core = earlephilhower`

### Issue 3: Missing Board Definition
**Problem:** Your specific RP2040 variant may not be listed in PlatformIO
**Solution:**
- Use the generic `pico` board definition (all RP2040 boards share the same core)
- Override specific settings with `board_build.*` options

### Issue 4: Platform Update Issues
**Problem:** Outdated platform or framework packages
**Solution:**
```bash
# Update the platform globally
pio pkg update -g -p https://github.com/maxgerhardt/platform-raspberrypi.git
# Or delete platform packages and rebuild
rm -rf ~/.platformio/packages/platform-*
pio run -t clean
pio run
```

---

## 8. Hardware Specifications Reference

**RP2040 Microcontroller:**
- Dual-core Arm Cortex-M0+ @ 133MHz (configurable)
- 2MB Flash Memory
- 264KB RAM
- Rich peripheral set (UART, SPI, I2C, PWM, ADC, PIO, etc.)

**RP2040-Zero (Waveshare) Specifics:**
- Pico-like form factor (smaller than Pico)
- WS2812 RGB LED on GPIO16
- No SWD debug pins (debugging not possible)
- USB bootloader for firmware upload

---

## 9. Quick Reference Table

| Configuration | Use Case |
|---|---|
| `platform = raspberrypi` + `board = pico` | Standard setup, stable releases |
| `platform = https://github.com/maxgerhardt/platform-raspberrypi.git` + `board_build.core = earlephilhower` | Full earlephilhower features, latest updates |
| `upload_protocol = picotool` | Automatic USB upload (default) |
| Manual UF2 copy | When picotool fails |
| `board_build.filesystem_size = 0.5m` | Enable LittleFS filesystem (512KB) |
| `build_flags = -DPIO_FRAMEWORK_ARDUINO_ENABLE_EXCEPTIONS` | Enable C++ exception handling |
| `board_build.f_cpu = 133000000L` | Set CPU frequency (133MHz default) |

---

## Sources

- [Raspberry Pi RP2040 Platform Documentation](https://docs.platformio.org/en/latest/platforms/raspberrypi.html)
- [Raspberry Pi Pico Board Documentation](https://docs.platformio.org/en/latest/boards/raspberrypi/pico.html)
- [Arduino-Pico PlatformIO Integration Guide](https://arduino-pico.readthedocs.io/en/latest/platformio.html)
- [Arduino-Pico GitHub Repository](https://github.com/earlephilhower/arduino-pico)
- [PlatformIO Raspberry Pi Platform GitHub](https://github.com/platformio/platform-raspberrypi)
- [PlatformIO Registry - raspberrypi Platform](https://registry.platformio.org/platforms/platformio/raspberrypi)
- [Build Flags Documentation](https://docs.platformio.org/en/latest/projectconf/sections/env/options/build/build_flags.html)
- [RP2040-Zero with PlatformIO - Community FAQ](https://community.platformio.org/t/can-i-use-a-rp2040-zero-board-with-platformio-and-the-arduino-framework/25940)
