# Adafruit NeoPixel Library API Reference

Complete reference guide for the Adafruit NeoPixel Arduino library with practical examples and technical specifications.

**Research Date:** November 27, 2025
**Official Resources:**
- [Adafruit NeoPixel Überguide](https://learn.adafruit.com/adafruit-neopixel-uberguide)
- [GitHub Repository](https://github.com/adafruit/Adafruit_NeoPixel)
- [API Class Reference](https://adafruit.github.io/Adafruit_NeoPixel/html/class_adafruit___neo_pixel.html)

---

## 1. Initialization Pattern

### Constructor

```cpp
Adafruit_NeoPixel(uint16_t numPixels, int16_t pin = 6, neoPixelType type = NEO_GRB + NEO_KHZ800)
```

**Parameters:**
- `numPixels`: Number of pixels in the NeoPixel strip
- `pin`: Arduino pin number (defaults to pin 6)
- `type`: Combination of color order and communication frequency flags (defaults to `NEO_GRB + NEO_KHZ800`)

**Example:**
```cpp
#include <Adafruit_NeoPixel.h>

#define LED_COUNT 16
#define LED_PIN 6

// Create NeoPixel strip object
Adafruit_NeoPixel strip(LED_COUNT, LED_PIN, NEO_GRB + NEO_KHZ800);

void setup() {
  strip.begin();           // Initialize the library
  strip.show();            // Initialize all pixels to off
}
```

### Type Flags Explanation

The third constructor parameter combines two flags:

**Color Order Flags:**
- `NEO_GRB` - Green, Red, Blue (most common NeoPixel products - WS2812, WS2812B)
- `NEO_RGB` - Red, Green, Blue (legacy FLORA pixels, WS2811 drivers)
- `NEO_RGBW` - Red, Green, Blue, White (RGBW NeoPixel products with white LED channel)

**Frequency Flags:**
- `NEO_KHZ800` - 800 KHz data transmission (default for most modern NeoPixels - WS2812)
- `NEO_KHZ400` - 400 KHz data transmission (classic v1 FLORA pixels)

**Example Combinations:**
```cpp
// Most common modern NeoPixels
Adafruit_NeoPixel pixels(10, 6, NEO_GRB + NEO_KHZ800);

// Legacy FLORA pixels
Adafruit_NeoPixel pixels(10, 6, NEO_RGB + NEO_KHZ400);

// RGBW NeoPixel strips
Adafruit_NeoPixel pixels(10, 6, NEO_RGBW + NEO_KHZ800);
```

### begin() Method

```cpp
bool begin(void)
```

**Purpose:** Configures the NeoPixel pin for output mode and prepares the library for transmission.

**Returns:** `false` if unable to claim required resources (rare), `true` on success.

**When to Call:** Must be called in `setup()` before any other NeoPixel operations.

**Typical Usage:**
```cpp
void setup() {
  strip.begin();    // Configure pin and initialize library
  strip.show();     // Push data to pixels (initializes all to off)
}
```

---

## 2. Color Setting Methods

### setPixelColor() - RGB Parameters

```cpp
void setPixelColor(uint16_t index, uint8_t r, uint8_t g, uint8_t b)
```

**Parameters:**
- `index`: Pixel position (0 to numPixels-1)
- `r`, `g`, `b`: Red, Green, Blue brightness (0-255 each)

**Behavior:** Sets the color of a single pixel. For RGBW pixels, white is automatically set to 0.

**Example:**
```cpp
strip.setPixelColor(0, 255, 0, 0);    // Pixel 0: bright red
strip.setPixelColor(1, 0, 255, 0);    // Pixel 1: bright green
strip.setPixelColor(2, 0, 0, 255);    // Pixel 2: bright blue
strip.show();                          // Apply changes to LEDs
```

### setPixelColor() - RGBW Parameters (For RGBW Strips)

```cpp
void setPixelColor(uint16_t index, uint8_t r, uint8_t g, uint8_t b, uint8_t w)
```

**Parameters:**
- `index`: Pixel position (0 to numPixels-1)
- `r`, `g`, `b`: Red, Green, Blue brightness (0-255 each)
- `w`: White brightness (0-255, ignored for RGB-only strips)

**Behavior:** Sets the color of a single pixel including white channel for RGBW strips.

**Example:**
```cpp
// RGBW strip - pure white light
strip.setPixelColor(0, 0, 0, 0, 255);    // White channel only
strip.show();

// RGBW strip - warm white (red + green + white)
strip.setPixelColor(1, 100, 100, 0, 100);
strip.show();
```

### setPixelColor() - Packed 32-bit Color

```cpp
void setPixelColor(uint16_t index, uint32_t color)
```

**Parameters:**
- `index`: Pixel position (0 to numPixels-1)
- `color`: Packed 32-bit RGB or WRGB color value

**Behavior:** Sets pixel color using a pre-packed color value. Most significant byte is white (RGBW) or 0 (RGB), followed by red, green, and blue.

**Example:**
```cpp
// Create packed colors first
uint32_t magenta = strip.Color(255, 0, 255);
uint32_t cyan = strip.Color(0, 255, 255);

// Use packed colors
strip.setPixelColor(0, magenta);
strip.setPixelColor(1, cyan);
strip.show();
```

**Note:** This is preferred when reusing the same colors repeatedly, as the color value is pre-calculated.

### Color() - Static Method for Creating Packed Colors

```cpp
static uint32_t Color(uint8_t r, uint8_t g, uint8_t b)
static uint32_t Color(uint8_t r, uint8_t g, uint8_t b, uint8_t w)  // RGBW version
```

**Parameters:**
- `r`, `g`, `b`: Red, Green, Blue components (0-255 each)
- `w`: White component (0-255, RGBW version only)

**Returns:** 32-bit packed color value suitable for `setPixelColor(index, color)`

**Behavior:** Converts separate RGB or RGBW components into a single 32-bit packed color. This is a **static method**, so it can be called on the class itself.

**Example:**
```cpp
// Define colors during setup
void setup() {
  strip.begin();

  // Using strip object instance
  uint32_t red = strip.Color(255, 0, 0);

  // Using static method on class
  uint32_t green = Adafruit_NeoPixel::Color(0, 255, 0);

  strip.setPixelColor(0, red);
  strip.setPixelColor(1, green);
  strip.show();
}

// Common color definitions
#define COLOR_RED      strip.Color(255, 0, 0)
#define COLOR_GREEN    strip.Color(0, 255, 0)
#define COLOR_BLUE     strip.Color(0, 0, 255)
#define COLOR_WHITE    strip.Color(255, 255, 255)
#define COLOR_BLACK    strip.Color(0, 0, 0)

// Pre-calculate in setup() for efficiency
void setup() {
  strip.begin();
  uint32_t colors[4] = {
    strip.Color(255, 0, 0),      // Red
    strip.Color(0, 255, 0),      // Green
    strip.Color(0, 0, 255),      // Blue
    strip.Color(0, 0, 0)         // Off
  };
}
```

**Packed Color Format (Internal):**
```
Bit Layout (32-bit uint32_t):
For RGB:   [00000000][RRRRRRRR][GGGGGGGG][BBBBBBBB]
For RGBW:  [WWWWWWWW][RRRRRRRR][GGGGGGGG][BBBBBBBB]
```

---

## 3. Display Update Methods

### show() - Transmit Pixel Data

```cpp
void show(void)
```

**Purpose:** Transmits pixel color data from RAM to the physical NeoPixel strip.

**Behavior:**
- Sends all pixel data accumulated by `setPixelColor()` calls
- **Temporarily disables interrupts** while transmitting (~30 microseconds per RGB pixel)
- Blocking function - code pauses until transmission completes
- NeoPixels require ~300 microseconds of silent time after last bit before new data can be sent

**When to Call:** After all `setPixelColor()` calls for a complete animation frame.

**Timing Considerations:**
- For a 60-pixel strip: ~1.8ms to transmit
- For 300 pixels: ~9ms to transmit
- 300 microseconds quiet time automatically provided between `show()` calls

**Example - Efficient Animation:**
```cpp
void loop() {
  // Set all pixel colors FIRST
  for(int i = 0; i < strip.numPixels(); i++) {
    strip.setPixelColor(i, strip.Color(255, 0, 0));  // Red
  }

  // Send data to LEDs ONCE per frame
  strip.show();

  delay(500);

  // Set new colors
  for(int i = 0; i < strip.numPixels(); i++) {
    strip.setPixelColor(i, strip.Color(0, 255, 0));  // Green
  }

  // Send updated data
  strip.show();

  delay(500);
}
```

**Example - Inefficient Animation (Do Not Use):**
```cpp
void loop() {
  for(int i = 0; i < strip.numPixels(); i++) {
    strip.setPixelColor(i, strip.Color(255, 0, 0));
    strip.show();  // BAD: Transmits after EVERY pixel!
  }
}
```

### clear() - Set All Pixels to Off

```cpp
void clear(void)
```

**Purpose:** Fills the entire NeoPixel strip with black (all pixels off).

**Behavior:**
- Uses `memset()` to clear the internal pixel buffer
- **Only modifies RAM**, no data sent to LEDs
- Must be followed by `show()` to actually turn off LEDs
- More efficient than looping through all pixels with `setPixelColor()`

**When to Call:** Before setting new colors to avoid visual artifacts from previous frames.

**Important:** `clear()` alone does NOT turn off the LEDs. You MUST call `show()` after `clear()`.

**Example - Clearing the Strip:**
```cpp
void setup() {
  strip.begin();
  strip.clear();    // Clear buffer
  strip.show();     // Actually turn off all LEDs
}

void loop() {
  // Method 1: Clear and set new colors
  strip.clear();    // Clear all pixels from previous frame

  for(int i = 0; i < strip.numPixels(); i++) {
    strip.setPixelColor(i, strip.Color(random(256), random(256), random(256)));
  }

  strip.show();     // Apply all changes at once
  delay(100);
}
```

**Example - Common Mistake (Does NOT work):**
```cpp
// WRONG - LEDs don't turn off!
strip.clear();
// LEDs still on because show() was never called

// CORRECT
strip.clear();
strip.show();      // Now LEDs turn off
```

---

## 4. Brightness Control

### setBrightness() Method

```cpp
void setBrightness(uint8_t brightness)
```

**Parameters:**
- `brightness`: Global brightness level (0-255)
  - 0 = All LEDs completely off
  - 127 = ~50% brightness
  - 255 = Maximum brightness

**Returns:** None (void)

**Behavior:**
- Adjusts overall brightness **without changing individual RGB values** you've set
- Uses pulse-width modulation (PWM) internally (~400-1000 Hz depending on NeoPixel generation)
- Changes take effect only **after the next `show()` call**
- Performs "lossy" color multiplication in RAM

**Critical Caveat - Lossy Operation:**
The library uses pre-multiplication, meaning repeated brightness changes cause color degradation:
```cpp
// WARNING: Color loss with repeated changes!
strip.setPixelColor(0, 255, 100, 50);
strip.setBrightness(200);
strip.show();
// Colors now pre-multiplied and stored in RAM

strip.setBrightness(100);  // Changes already-modified values
strip.show();              // Color may not match original
```

**Recommendation:** Call `setBrightness()` only **once in `setup()`**, not during animation.

**When to Call:**
1. In `setup()` - set initial brightness
2. NOT during `loop()` animations (if you need dynamic brightness, redraw pixels)

**Example - One-Time Brightness Setup:**
```cpp
void setup() {
  strip.begin();

  // Set brightness to 30% of maximum for onboard LEDs
  strip.setBrightness(76);  // ~30% of 255

  // Now set all colors
  for(int i = 0; i < strip.numPixels(); i++) {
    strip.setPixelColor(i, 255, 255, 255);  // White
  }

  strip.show();  // Apply brightness and colors
}

void loop() {
  // Don't change brightness here
  delay(1000);
}
```

**Example - Proper Dynamic Brightness:**
```cpp
void setBrightnessLevel(uint8_t level) {
  // Completely redraw all pixels at new brightness
  strip.setBrightness(level);

  // Re-set all pixel colors with new brightness applied
  for(int i = 0; i < strip.numPixels(); i++) {
    strip.setPixelColor(i, 255, 255, 255);  // Reapply colors
  }

  strip.show();
}
```

### Brightness and Power Consumption

**Current Draw Per Pixel (Approximate):**
- **Maximum brightness:** ~60 mA per pixel (at full white: R=255, G=255, B=255)
- **Practical rule of thumb:** 20 mA per pixel (typical mixed colors and animations)
- **Mini NeoPixels:** ~35 mA max

**How Brightness Affects Current:**
Brightness changes affect power consumption proportionally:
- 50% brightness ≈ 50% current
- 25% brightness ≈ 25% current
- 10% brightness ≈ 10% current

**Power Supply Calculation:**
```
Amps needed = (NumPixels × mA per pixel) / 1000

Conservative estimate:
Amps = (NumPixels × 20 mA) / 1000

Maximum safety margin:
Amps = (NumPixels × 60 mA) / 1000

Example: 60 pixels
Conservative: (60 × 20) / 1000 = 1.2 Amps minimum
Maximum:     (60 × 60) / 1000 = 3.6 Amps maximum
```

### Recommended Brightness for Onboard LEDs

**For onboard NeoPixels (like Circuit Playground):**
- **Default/Starting Point:** 30 out of 255 (roughly 12%)
- **Comfortable viewing indoors:** 40-80 out of 255
- **Maximum (full brightness):** 255 out of 255 (can be painfully bright)

**Example Settings:**
```cpp
// Very dim - good for nighttime operation
strip.setBrightness(15);   // ~6%

// Comfortable indoor brightness
strip.setBrightness(76);   // ~30%

// Bright but not overwhelming
strip.setBrightness(150);  // ~59%

// Full brightness
strip.setBrightness(255);  // 100% (use cautiously)
```

**Power Consumption with Brightness:**
At different brightness levels for a single white pixel (full RGB):
- Brightness 25 (10%): ~6 mA
- Brightness 51 (20%): ~12 mA
- Brightness 76 (30%): ~18 mA
- Brightness 128 (50%): ~30 mA
- Brightness 255 (100%): ~60 mA

---

## 5. Turning LEDs OFF

### Recommended Method: clear() + show()

**Most Efficient and Recommended:**
```cpp
void turnOffAllPixels() {
  strip.clear();   // Clear internal buffer
  strip.show();    // Transmit to LEDs
}
```

**Why This is Best:**
- `clear()` uses `memset()` to zero all bytes at once (very fast)
- Simple and clear intent
- More efficient than looping through pixels

### Alternative Method: setPixelColor(index, 0, 0, 0) + show()

**For Individual Pixels:**
```cpp
void turnOffPixel(uint16_t index) {
  strip.setPixelColor(index, 0, 0, 0);  // Set to black
  strip.show();
}

// For all pixels individually (not recommended)
void turnOffAllPixelsAlternative() {
  for(int i = 0; i < strip.numPixels(); i++) {
    strip.setPixelColor(i, 0, 0, 0);
  }
  strip.show();
}
```

**Why Avoid for Bulk Turning Off:**
- Slower than `clear()`
- Requires looping through every pixel
- Same result as `clear()`

### NOT Recommended: setPixelColor(index, 0, 0, 0, 0)

```cpp
// NOT RECOMMENDED - Don't do this!
strip.setPixelColor(index, 0, 0, 0, 0);  // Four zeros
```

**Why:**
- Unnecessary - the fourth parameter (white) is only used for RGBW strips
- Confusing to read
- No benefit over `setPixelColor(index, 0, 0, 0)`

### Power Consumption When Off

Even with all pixels off (0, 0, 0), there is still minimal power draw:
- **Idle current per WS2812 chip:** <1 mA (not specified in datasheet)
- Practical measurements: ~0.5-0.9 mA per pixel
- **Much less significant** than when lit

**Complete Power-Down Example:**
```cpp
void setup() {
  strip.begin();
}

void loop() {
  // Light up pixels
  for(int i = 0; i < strip.numPixels(); i++) {
    strip.setPixelColor(i, 255, 255, 255);  // White
  }
  strip.show();
  delay(2000);

  // Turn off completely
  strip.clear();
  strip.show();
  delay(2000);
}
```

---

## 6. Complete Example Programs

### Basic Lighting Pattern

```cpp
#include <Adafruit_NeoPixel.h>

#define LED_PIN 6
#define LED_COUNT 16

Adafruit_NeoPixel strip(LED_COUNT, LED_PIN, NEO_GRB + NEO_KHZ800);

// Define colors
#define RED      strip.Color(255, 0, 0)
#define GREEN    strip.Color(0, 255, 0)
#define BLUE     strip.Color(0, 0, 255)
#define YELLOW   strip.Color(255, 255, 0)
#define MAGENTA  strip.Color(255, 0, 255)
#define CYAN     strip.Color(0, 255, 255)
#define WHITE    strip.Color(255, 255, 255)
#define OFF      strip.Color(0, 0, 0)

void setup() {
  strip.begin();
  strip.setBrightness(76);  // Set to 30% brightness
  strip.clear();
  strip.show();
}

void loop() {
  // Light up all pixels in sequence
  for(int i = 0; i < strip.numPixels(); i++) {
    strip.clear();
    strip.setPixelColor(i, RED);
    strip.show();
    delay(100);
  }

  // Breathe effect
  for(int brightness = 0; brightness < 256; brightness++) {
    strip.setBrightness(brightness);
    strip.setPixelColor(0, WHITE);
    strip.show();
    delay(5);
  }

  // Turn off
  strip.clear();
  strip.show();
  delay(500);
}
```

### Dynamic Brightness Control

```cpp
#include <Adafruit_NeoPixel.h>

#define LED_PIN 6
#define LED_COUNT 16

Adafruit_NeoPixel strip(LED_COUNT, LED_PIN, NEO_GRB + NEO_KHZ800);

void setup() {
  strip.begin();
  strip.clear();
  strip.show();
}

void setBrightnessLevel(uint8_t level) {
  // Clear and redraw at new brightness level
  strip.clear();
  strip.setBrightness(level);

  // Fill strip with white at new brightness
  for(int i = 0; i < strip.numPixels(); i++) {
    strip.setPixelColor(i, 255, 255, 255);
  }

  strip.show();
}

void loop() {
  // Fade in
  for(uint8_t b = 0; b < 255; b += 5) {
    setBrightnessLevel(b);
    delay(30);
  }

  delay(2000);

  // Fade out
  for(uint8_t b = 255; b > 0; b -= 5) {
    setBrightnessLevel(b);
    delay(30);
  }

  setBrightnessLevel(0);  // Off
  delay(2000);
}
```

### RGB Animation with Pre-packed Colors

```cpp
#include <Adafruit_NeoPixel.h>

#define LED_PIN 6
#define LED_COUNT 32

Adafruit_NeoPixel strip(LED_COUNT, LED_PIN, NEO_GRB + NEO_KHZ800);

// Pre-calculated color palette
uint32_t colors[8];

void setup() {
  strip.begin();
  strip.setBrightness(150);

  // Create color palette during setup
  colors[0] = strip.Color(255, 0, 0);      // Red
  colors[1] = strip.Color(255, 127, 0);    // Orange
  colors[2] = strip.Color(255, 255, 0);    // Yellow
  colors[3] = strip.Color(0, 255, 0);      // Green
  colors[4] = strip.Color(0, 0, 255);      // Blue
  colors[5] = strip.Color(75, 0, 130);     // Indigo
  colors[6] = strip.Color(148, 0, 211);    // Violet
  colors[7] = strip.Color(255, 255, 255);  // White
}

void loop() {
  // Chase effect with colors
  for(int colorIdx = 0; colorIdx < 8; colorIdx++) {
    for(int i = 0; i < strip.numPixels(); i++) {
      strip.clear();
      strip.setPixelColor(i, colors[colorIdx]);
      strip.show();
      delay(50);
    }
  }
}
```

---

## 7. Summary Table

| Operation | Method | Before show()? | After show()? | Notes |
|-----------|--------|---|---|---|
| Set one pixel | `setPixelColor(idx, r, g, b)` | Modifies RAM | Transmits data | Required for each change |
| Set one pixel (packed) | `setPixelColor(idx, color)` | Modifies RAM | Transmits data | Use with Color() |
| Create packed color | `Color(r, g, b)` | Returns uint32_t | - | Can be called anytime |
| Set brightness | `setBrightness(val)` | Modifies RAM | Takes effect | Only call once in setup() |
| Clear all pixels | `clear()` | Modifies RAM | Required | Most efficient off method |
| Send data to LEDs | `show()` | - | Transmits | Always needed after changes |
| Turn off pixel | `setPixelColor(idx, 0, 0, 0)` | Modifies RAM | Transmit after | For individual pixels |
| Turn off all | `clear()` then `show()` | Both needed | - | Best practice |

---

## 8. Quick Reference Checklist

**Initialization (setup):**
- [ ] Create `Adafruit_NeoPixel` object with correct pin and LED count
- [ ] Call `strip.begin()`
- [ ] Call `strip.show()` to initialize LEDs to off
- [ ] Set brightness with `strip.setBrightness()` (if needed)

**Animation Loop:**
- [ ] Call `strip.clear()` at start of frame (if needed)
- [ ] Set pixel colors with `strip.setPixelColor()`
- [ ] Call `strip.show()` exactly ONCE per frame
- [ ] Never repeatedly call `setBrightness()` during animation

**Turning Off:**
- [ ] Use `strip.clear()` + `strip.show()` for all pixels
- [ ] Or `strip.setPixelColor(idx, 0, 0, 0)` + `strip.show()` for individual pixels

---

## 9. References

### Official Adafruit Documentation
- [Adafruit NeoPixel Überguide - Arduino Library Use](https://learn.adafruit.com/adafruit-neopixel-uberguide/arduino-library-use)
- [Adafruit NeoPixel Überguide - Powering NeoPixels](https://learn.adafruit.com/adafruit-neopixel-uberguide/powering-neopixels)
- [NeoPixel API Class Reference](https://adafruit.github.io/Adafruit_NeoPixel/html/class_adafruit___neo_pixel.html)

### GitHub Resources
- [Adafruit_NeoPixel Repository](https://github.com/adafruit/Adafruit_NeoPixel)
- [Source Code: Adafruit_NeoPixel.h](https://github.com/adafruit/Adafruit_NeoPixel/blob/master/Adafruit_NeoPixel.h)
- [Source Code: Adafruit_NeoPixel.cpp](https://github.com/adafruit/Adafruit_NeoPixel/blob/master/Adafruit_NeoPixel.cpp)

### Related Learning Guides
- [Sipping Power With NeoPixels](https://learn.adafruit.com/sipping-power-with-neopixels?view=all)
- [CircuitPython NeoPixel](https://learn.adafruit.com/circuitpython-essentials/circuitpython-neopixel)
- [NeoPixel Brightness Control Examples](https://learn.adafruit.com/adafruit-proximity-trinkey/neopixel-brightness)

### Arduino Library Manager
- Library: **Adafruit NeoPixel**
- Install via Arduino IDE: Sketch → Include Library → Manage Libraries (search "NeoPixel")

---

**Last Updated:** November 27, 2025
**Library Version Tested:** Latest stable from Arduino Library Manager
