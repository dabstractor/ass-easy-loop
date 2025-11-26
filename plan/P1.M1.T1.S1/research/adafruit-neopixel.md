# Adafruit NeoPixel Library Research

## 1. Library Identification

**Library Name/ID in PlatformIO Registry:**
- **Library Name:** `adafruit/Adafruit NeoPixel`
- **Library ID:** 28
- **Registry URL:** https://registry.platformio.org/libraries/adafruit/Adafruit%20NeoPixel
- **GitHub Repository:** https://github.com/adafruit/Adafruit_NeoPixel

**Minimum Version:** Version 1.10.2 or higher (required for RP2040 support)

---

## 2. lib_deps Syntax for platformio.ini

**Standard syntax:**
```ini
[env:rp2040-zero]
lib_deps =
    adafruit/Adafruit NeoPixel@^1.10.4
```

**Alternative version specifications:**
```ini
# Using latest compatible version in 1.10.x range
lib_deps = adafruit/Adafruit NeoPixel@^1.10.2

# Using latest compatible version in 1.12.x range
lib_deps = adafruit/Adafruit NeoPixel@^1.12.0

# Using direct GitHub repository
lib_deps = https://github.com/adafruit/Adafruit_NeoPixel.git
```

---

## 3. RP2040 and earlephilhower/arduino-pico Compatibility

**Confirmed Compatibility:**
- The Adafruit NeoPixel library **fully supports RP2040** with the earlephilhower/arduino-pico core since version 1.10.2
- The library uses the RP2040's **PIO (Programmable I/O) capability** to handle the timing-sensitive WS2812 protocol
- This PIO-based approach makes the library **clock-speed independent**, a key advantage for reliable NeoPixel operation

**How It Works:**
- Instead of bit-banging the WS2812 serial protocol on the CPU, the library leverages RP2040's two PIO peripherals (each with 4 state machines)
- The PIO reads data buffers and clocks out the correct bitstream with perfect accuracy
- No external timing libraries required

**Configuration Example:**
```cpp
#include <Adafruit_NeoPixel.h>

// For single NeoPixel on GPIO 16 (RP2040-Zero)
Adafruit_NeoPixel strip(1, 16, NEO_GRB + NEO_KHZ800);

void setup() {
  strip.begin();
  strip.show();
}

void loop() {
  strip.setPixelColor(0, 255, 0, 0);  // Red
  strip.show();
  delay(500);
}
```

**Known PIO Considerations:**
- If using other libraries that rely on PIO (like SerialPIO), the NeoPixelConnect library (an alternative) can cause conflicts
- Adafruit NeoPixel library is tested to work **simultaneously with SerialPIO** without issues
- Avoid using NeoPixelConnect if serial communication via PIO is required

---

## 4. RP2040-Zero Onboard WS2812 RGB LED Confirmation

**Hardware Specifications:**
- **Board:** Waveshare RP2040-Zero
- **LED Type:** WS2812 (NeoPixel) RGB LED
- **GPIO Pin:** GPIO 16 (confirmed)
- **LED Count:** 1 (single onboard LED)
- **Configuration:** The WS2812 LED is the primary LED on this board (no traditional LED)

**Arduino Code for RP2040-Zero Onboard LED:**
```cpp
#define LED_PIN 16
#define LED_COUNT 1

Adafruit_NeoPixel strip(LED_COUNT, LED_PIN, NEO_GRB + NEO_KHZ800);

void setup() {
  strip.begin();
  strip.setBrightness(50);  // Recommended to limit brightness (max 255)
  strip.show();
}

void loop() {
  // Set color and show
  strip.setPixelColor(0, 255, 0, 0);  // Red
  strip.show();
  delay(1000);

  strip.setPixelColor(0, 0, 255, 0);  // Green
  strip.show();
  delay(1000);

  strip.setPixelColor(0, 0, 0, 255);  // Blue
  strip.show();
  delay(1000);
}
```

**Important Note:** The WS2812 on the RP2040-Zero draws power directly from a GPIO pin, so it's recommended to limit maximum brightness to around 50 (out of 255) to prevent excessive current draw, though even at brightness 8 the LED appears quite bright.

---

## 5. Complete platformio.ini Example for RP2040-Zero

```ini
[env:rp2040-zero]
platform = https://github.com/maxgerhardt/platform-raspberrypi.git
board = pico
framework = arduino
board_build.core = earlephilhower
upload_protocol = picotool
lib_deps =
    adafruit/Adafruit NeoPixel@^1.12.0
monitor_speed = 115200
```

---

## 6. Alternative Libraries

If you encounter issues with Adafruit NeoPixel:

- **NeoPixelConnect** - PIO-based library specifically for Arduino Nano RP2040 Connect and Raspberry Pi Pico (use only if SerialPIO not needed)
- **Adafruit DMA NeoPixel Library** - For boards supporting DMA transfers
- **pico-ws2812** - C/C++ SDK library for lower-level control

---

## Sources

- [Adafruit NeoPixel PlatformIO Registry](https://registry.platformio.org/libraries/adafruit/Adafruit%20NeoPixel)
- [Adafruit NeoPixel Arduino Library Installation Guide](https://learn.adafruit.com/adafruit-neopixel-uberguide/arduino-library-installation)
- [Adafruit Feather RP2040 NeoPixel Documentation](https://learn.adafruit.com/adafruit-feather-rp2040-pico/built-in-neopixel-led)
- [Arduino Forum - RP2040-Zero NeoPixel Discussion](https://forum.arduino.cc/t/adafruit_neopixel-library-adapted-for-raspberry-pi-pico-rp2040/704414)
- [earlephilhower/arduino-pico GitHub Discussions](https://github.com/earlephilhower/arduino-pico/discussions/756)
- [NeoPixelConnect GitHub Repository](https://github.com/MrYsLab/NeoPixelConnect)
- [TinyGo RP2040-Zero Documentation](https://tinygo.org/docs/reference/microcontrollers/machine/waveshare-rp2040-zero/)
