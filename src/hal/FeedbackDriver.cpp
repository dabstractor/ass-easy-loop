#include "FeedbackDriver.h"
#include "hardware/gpio.h"

FeedbackDriver::FeedbackDriver(const ChargeMonitor& chargeMonitor, uint8_t neoPixelPin)
    : _neoPixelPin(neoPixelPin),
      _chargeMonitor(chargeMonitor),
      _pixel(LED_COUNT, neoPixelPin, NEO_GRB + NEO_KHZ800),
      _hueOffset(0.0f) {
    // Constructor: store pins and create NeoPixel object
    // Hardware initialization deferred to begin()
}

void FeedbackDriver::begin() {
    // Initialize NeoPixel
    _pixel.begin();
    // We don't use setBrightness here because we do manual scaling
    // _pixel.setBrightness(255);
    _pixel.clear();
    _pixel.show();  // Apply OFF state to LED
}

void FeedbackDriver::turnOff() {
    // Comprehensive hardware cleanup for bootloader entry

    // First, try normal NeoPixel clear
    _pixel.clear();
    _pixel.show();

    // Add delay to ensure data transmission completes
    delay(10);

    // Force the pin low using direct GPIO control
    gpio_put(_neoPixelPin, 0);
    gpio_set_dir(_neoPixelPin, GPIO_IN);

    // Additional delay to ensure cleanup is complete
    delay(50);
}

void FeedbackDriver::update() {
    uint8_t r = 0, g = 0, b = 0;

    if (_chargeMonitor.isCharging()) {
        // Charging state: Continuous charging color (Red)
        // We want this to be solid so user knows it's charging
        r = Config::CHARGING_COLOR_R;
        g = Config::CHARGING_COLOR_G;
        b = Config::CHARGING_COLOR_B;
    } else {
        // Running state: continuous Pastel RGB cycle
        // Increment hue
        _hueOffset += Config::HUE_INCREMENT;
        if (_hueOffset >= 1.0f) {
            _hueOffset -= 1.0f;
        }

        getPastelColor(_hueOffset, r, g, b);
    }

    setScaledColor(r, g, b);
}

void FeedbackDriver::setScaledColor(uint8_t r, uint8_t g, uint8_t b) {
    // Apply global brightness scaling from configuration
    uint8_t scaledR = (uint8_t)(r * Config::NEOPIXEL_BRIGHTNESS);
    uint8_t scaledG = (uint8_t)(g * Config::NEOPIXEL_BRIGHTNESS);
    uint8_t scaledB = (uint8_t)(b * Config::NEOPIXEL_BRIGHTNESS);

    _pixel.setPixelColor(0, scaledR, scaledG, scaledB);
    _pixel.show();
}

void FeedbackDriver::getPastelColor(float hue, uint8_t& r, uint8_t& g, uint8_t& b) {
    // Simple HSV to RGB conversion logic specialized for pastels
    // Use configured pastel parameters
    const float s = Config::PASTEL_SATURATION;
    const float v = Config::PASTEL_VALUE;

    int i = (int)(hue * 6);
    float f = hue * 6 - i;
    float p = v * (1 - s);
    float q = v * (1 - f * s);
    float t = v * (1 - (1 - f) * s);

    float fr, fg, fb;

    switch (i % 6) {
        case 0: fr = v; fg = t; fb = p; break;
        case 1: fr = q; fg = v; fb = p; break;
        case 2: fr = p; fg = v; fb = t; break;
        case 3: fr = p; fg = q; fb = v; break;
        case 4: fr = t; fg = p; fb = v; break;
        case 5: fr = v; fg = p; fb = q; break;
        default: fr = v; fg = t; fb = p; break; // Should not happen
    }

    r = (uint8_t)(fr * 255);
    g = (uint8_t)(fg * 255);
    b = (uint8_t)(fb * 255);
}

FeedbackDriver::~FeedbackDriver() {
    // Use comprehensive cleanup for safety
    turnOff();
}
