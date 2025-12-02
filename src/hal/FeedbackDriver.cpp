#include "FeedbackDriver.h"

FeedbackDriver::FeedbackDriver(uint8_t neoPixelPin)
    : _neoPixelPin(neoPixelPin),
      _pixel(LED_COUNT, neoPixelPin, NEO_GRB + NEO_KHZ800) {
    // Constructor: store pin and create NeoPixel object
    // Hardware initialization deferred to begin()
}

void FeedbackDriver::begin() {
    // Initialize NeoPixel
    _pixel.begin();
    _pixel.setBrightness(BRIGHTNESS);
    _pixel.clear();
    _pixel.show();  // Apply OFF state to LED
}

void FeedbackDriver::indicateActive(bool isActive) {
    if (isActive) {
        // Active state: LED Green
        _pixel.setPixelColor(0, GREEN_R, GREEN_G, GREEN_B);
    } else {
        // Inactive state: LED OFF
        _pixel.setPixelColor(0, 0, 0, 0);
    }
    _pixel.show();  // Push color change to LED
}

FeedbackDriver::~FeedbackDriver() {
    // Force safe state on destruction
    _pixel.clear();
    _pixel.show();
}
