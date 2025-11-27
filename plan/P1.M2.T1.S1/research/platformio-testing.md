# PlatformIO Testing Patterns Research

## 1. PlatformIO Native Testing

PlatformIO's native testing allows running tests on desktop without hardware.

### Configuration (platformio.ini)
```ini
[env:native]
platform = native
test_framework = unity
lib_deps = fabiobatsilva/ArduinoFake
build_src_filter = +<*> -<.git/> -<.pio/> -<src/main.cpp>
```

### Running Tests
```bash
pio test -e native          # Run tests on desktop
pio test                    # Run all test environments
```

---

## 2. Unity Test Framework

Unity is PlatformIO's built-in, lightweight C testing framework.

### Basic Test Structure
```cpp
#include <unity.h>

void setUp(void) {
    // Called before each test
}

void tearDown(void) {
    // Called after each test
}

void test_function_returns_correct_value(void) {
    TEST_ASSERT_EQUAL(expected, actual);
    TEST_ASSERT_TRUE(condition);
}

int main(void) {
    UNITY_BEGIN();
    RUN_TEST(test_function_returns_correct_value);
    return UNITY_END();
}
```

### Project Structure
```
project/
├── platformio.ini
├── src/
│   └── main.cpp
├── lib/
│   └── hal/
│       ├── CoilDriver.h
│       └── CoilDriver.cpp
└── test/
    └── test_coildriver/
        └── test_main.cpp
```

---

## 3. Mocking Arduino Functions with ArduinoFake

### Installation
Add to `platformio.ini`:
```ini
[env:native]
platform = native
lib_deps = fabiobatsilva/ArduinoFake
```

### Mock Examples
```cpp
#include <ArduinoFake.h>
using namespace fakeit;

void setUp(void) {
    ArduinoFakesReset();  // Reset mocks before each test
}

void test_digitalWrite(void) {
    When(Method(ArduinoFake(), digitalWrite)).AlwaysReturn();

    digitalWrite(15, HIGH);

    Verify(Method(ArduinoFake(), digitalWrite)
        .Using(15, HIGH)).Once();
}

void test_pinMode(void) {
    When(Method(ArduinoFake(), pinMode)).AlwaysReturn();

    pinMode(15, OUTPUT);

    Verify(Method(ArduinoFake(), pinMode)
        .Using(15, OUTPUT)).Once();
}
```

---

## 4. HAL Testing Pattern

### CoilDriver Test Example
```cpp
#include <unity.h>
#include <ArduinoFake.h>
#include "hal/CoilDriver.h"

using namespace fakeit;

void setUp(void) {
    ArduinoFakesReset();
}

void tearDown(void) {}

void test_begin_sets_pin_output_and_low(void) {
    When(Method(ArduinoFake(), pinMode)).AlwaysReturn();
    When(Method(ArduinoFake(), digitalWrite)).AlwaysReturn();

    CoilDriver driver(15);
    driver.begin();

    Verify(Method(ArduinoFake(), pinMode).Using(15, OUTPUT)).Once();
    Verify(Method(ArduinoFake(), digitalWrite).Using(15, LOW)).Once();
}

void test_setActive_true_writes_high(void) {
    When(Method(ArduinoFake(), pinMode)).AlwaysReturn();
    When(Method(ArduinoFake(), digitalWrite)).AlwaysReturn();

    CoilDriver driver(15);
    driver.begin();
    driver.setActive(true);

    Verify(Method(ArduinoFake(), digitalWrite).Using(15, HIGH)).Once();
}

void test_setActive_false_writes_low(void) {
    When(Method(ArduinoFake(), pinMode)).AlwaysReturn();
    When(Method(ArduinoFake(), digitalWrite)).AlwaysReturn();

    CoilDriver driver(15);
    driver.begin();
    driver.setActive(false);

    // Should have two LOW writes: one from begin(), one from setActive(false)
    Verify(Method(ArduinoFake(), digitalWrite).Using(15, LOW)).AtLeast(2);
}

int main(void) {
    UNITY_BEGIN();
    RUN_TEST(test_begin_sets_pin_output_and_low);
    RUN_TEST(test_setActive_true_writes_high);
    RUN_TEST(test_setActive_false_writes_low);
    return UNITY_END();
}
```

---

## 5. Hardware-Less Testing Feasibility

| Feature | Native (Desktop) | Comments |
|---------|-----------------|----------|
| Pure Logic/Algorithms | Excellent | Instant feedback |
| GPIO/Pin Control | With ArduinoFake | Can mock on desktop |
| Timing (delay) | Can mock | Mock delays for fast testing |
| State Machines | Excellent | Perfect for desktop |

**Verdict for GPIO Wrappers: YES, hardware-less testing is feasible.**

---

## Sources

- [PlatformIO Unit Testing Documentation](https://docs.platformio.org/en/latest/advanced/unit-testing/index.html)
- [Unity Test Framework](https://docs.platformio.org/en/latest/advanced/unit-testing/frameworks/unity.html)
- [ArduinoFake GitHub](https://github.com/FabioBatSilva/ArduinoFake)
- [Embedded C/C++ Unit Testing with Mocks](https://interrupt.memfault.com/blog/unit-test-mocking)
