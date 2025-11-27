#include "FeedbackDriver.h"

FeedbackDriver::FeedbackDriver(uint8_t buzzerPin, uint8_t neoPixelPin)
    : _buzzerPin(buzzerPin),
      _neoPixelPin(neoPixelPin),
      _pixel(LED_COUNT, neoPixelPin, NEO_GRB + NEO_KHZ800) {
    // Constructor: store pins and create NeoPixel object
    // Hardware initialization deferred to begin()
}

void FeedbackDriver::begin() {
    // Initialize buzzer pin: pre-set LOW before configuring as OUTPUT
    // This prevents any transient HIGH state during startup
    digitalWrite(_buzzerPin, LOW);
    pinMode(_buzzerPin, OUTPUT);

    // Initialize NeoPixel
    _pixel.begin();
    _pixel.setBrightness(BRIGHTNESS);
    _pixel.clear();
    _pixel.show();  // Apply OFF state to LED
}

void FeedbackDriver::indicateActive(bool isActive) {
    if (isActive) {
        // Active state: Buzzer ON, LED Green
        digitalWrite(_buzzerPin, HIGH);
        _pixel.setPixelColor(0, GREEN_R, GREEN_G, GREEN_B);
    } else {
        // Inactive state: Buzzer OFF, LED OFF
        digitalWrite(_buzzerPin, LOW);
        _pixel.setPixelColor(0, 0, 0, 0);
    }
    _pixel.show();  // Push color change to LED
}

FeedbackDriver::~FeedbackDriver() {
    // Force safe state on destruction
    digitalWrite(_buzzerPin, LOW);
    _pixel.clear();
    _pixel.show();
}
