# earlephilhower/arduino-pico Core Research

## 1. Specifying the Core in PlatformIO

### Current Recommended Method (Modern)

The latest approach uses the maxgerhardt platform repository with core selection:

```ini
[env:pico]
platform = https://github.com/maxgerhardt/platform-raspberrypi.git
board = pico
framework = arduino
board_build.core = earlephilhower
```

**Important Note:** The `board_build.core = earlephilhower` directive is only required for boards that support multiple cores (standard Pico and Nano RP2040 Connect). For Arduino-Pico-exclusive boards like `rpipico`, `rpipico2`, or `adafruit_feather`, this line is not needed.

### Alternative Method: Direct Framework Package

For a specific version or branch, you can specify the framework package directly:

```ini
platform_packages = framework-arduinopico@https://github.com/earlephilhower/arduino-pico.git#master
```

You can replace `#master` with:
- `#branchname` - for specific branches
- `#commithash` - for specific commits
- Leave empty to use the default branch

You can also use pseudo-protocols:
- `file://` - for local copies on disk
- `symlink://` - for symlinked versions with modifications

**Deprecated Method (Do Not Use):**
Previous documentation recommended manual framework/toolchain injection via `maxgerhardt/framework-arduinopico` and `maxgerhardt/toolchain-pico`. This is deprecated and should be removed. Instead, update the platform:

```bash
pio pkg update -g -p https://github.com/maxgerhardt/platform-raspberrypi.git
```

---

## 2. USB Serial Support Configuration and Build Flags

### Two USB Stack Options

The core provides two distinct USB implementations:

**A) Pico SDK USB (Default)**
- No special build flags needed
- Automatically includes `Serial` USB port
- Supports automatic IDE reset-to-upload
- Works out of the box with `Serial.begin()` (baud rate is ignored for USB)

**B) Adafruit TinyUSB Stack**
- Add build flag: `-DUSE_TINYUSB`
- Requires explicit `Serial.begin(115200)` in `setup()`
- Provides advanced USB features
- More flexibility for custom USB configurations

### Build Flags for USB Configuration

```ini
; Use Adafruit TinyUSB
build_flags = -DUSE_TINYUSB

; Disable USB entirely
build_flags = -DPIO_FRAMEWORK_ARDUINO_NO_USB

; Custom TinyUSB configuration with multiple CDC ports
build_flags =
    -DUSE_TINYUSB
    -DCFG_TUSB_CONFIG_FILE=\"custom_tusb_config.h\"
    -Iinclude/
```

### Custom USB Descriptors (VID/PID)

```ini
board_build.arduino.earlephilhower.usb_manufacturer = Custom Manufacturer
board_build.arduino.earlephilhower.usb_product = Ultra Cool Product
board_build.arduino.earlephilhower.usb_vid = 0xABCD
board_build.arduino.earlephilhower.usb_pid = 0x1337
```

### Dynamic USB Configuration in Code

When using the Pico SDK USB stack, include the USB header for runtime configuration:

```cpp
#include <USB.h>

// Before modifying, disconnect:
USB.disconnect();

// Modify descriptors
USB.setVIDPID(0xABCD, 0x1337);
USB.setManufacturer("My Company");
USB.setProduct("My Device");
USB.setSerialNumber("ABC123");

// Reconnect
USB.connect();
```

**Critical:** Never call `Serial.end()` if you need the IDE auto-reset feature to work properly.

---

## 3. Magic 1200 Baud Reset Functionality

### How It Works

The USB Serial port implements the Arduino standard for bootloader entry:

- Opening the USB Serial connection at **1200 baud** triggers a reset into **BOOTSEL (bootloader) mode**
- This follows Arduino's established convention used on boards like Arduino Leonardo
- Implemented via the `PICO_STDIO_USB_ENABLE_RESET_VIA_BAUD_RATE` configuration (enabled by default)
- The reset happens automatically when uploading via the Arduino IDE or PlatformIO

### The Process

1. IDE/tool opens USB serial at 1200 baud
2. RP2040 detects the baud rate change
3. Device resets into BOOTSEL mass storage mode
4. Serial port disappears temporarily
5. UF2 bootloader appears as mass storage device
6. Firmware upload proceeds

### Programmatic Bootloader Control

**Method 1: Double-Reset Bootloader**
```cpp
void setup() {
  rp2040.enableDoubleResetBootloader();
  // Now double-tap the reset button to enter bootloader mode
}
```

**Method 2: Forced Bootloader Reboot**
```cpp
// Include required headers
extern "C" {
  #include "pico.h"
  #include "pico/time.h"
  #include "pico/bootrom.h"
}

void forceBootloader() {
  reset_usb_boot(1 << PICO_DEFAULT_LED_PIN, 0);
}

// Or use the helper method
void anotherBootloader() {
  rp2040.rebootToBootloader();
}
```

### Alternative Reset Methods

- **Watchdog Reset:** `watchdog_reboot()` for software reset
- **Hardware Reset:** Physical reset button (with double-tap for bootloader)
- **Check Reset Reason:** `rp2040.getResetReason()` returns enumeration of reset cause

---

## 4. Required Serial.begin() Configuration

### Pico SDK USB Stack (Default)

```cpp
void setup() {
  // Baud rate is ignored for USB serial
  Serial.begin(9600); // Any baud rate works - USB doesn't use it

  // USB serial is ready to use immediately
  Serial.println("Hello from Pico!");
}
```

**Characteristics:**
- `Serial.begin()` call is optional but recommended
- Baud rate parameter is completely ignored
- `Serial.end()` will break IDE auto-reset feature
- USB serial appears on host immediately upon startup

### Adafruit TinyUSB Stack

```cpp
void setup() {
  // TinyUSB REQUIRES this call to enable USB serial
  Serial.begin(115200);

  // Without this call:
  // - No USB serial will be available
  // - Auto-reset-to-upload will NOT work
  // - Manual BOOTSEL + plug-in required for uploads

  Serial.println("TinyUSB enabled!");
}
```

**Characteristics:**
- `Serial.begin(115200)` is mandatory
- 115200 is the standard rate (actual USB rate doesn't matter)
- Enables both USB serial AND auto-reset functionality
- Without this call, manual bootloader entry required

### Hardware UART Serial Ports

The RP2040 provides two hardware UARTs accessible as Serial1 and Serial2:

```cpp
void setup() {
  // Configure Serial1 (UART0) with custom pins
  Serial1.setRX(5);   // GPIO5
  Serial1.setTX(4);   // GPIO4
  Serial1.begin(9600); // Actual baud rate matters here

  // Configure Serial2 (UART1) with custom pins
  Serial2.setRX(9);   // GPIO9
  Serial2.setTX(8);   // GPIO8
  Serial2.begin(115200);

  // USB Serial (regardless of stack)
  Serial.begin(115200); // Still ignored for baud
}
```

**Configuration Methods:**
- `setRX(pin)` / `setTX(pin)` - must be called before `begin()`
- `setFIFOSize(size)` - adjust receive buffer (default 32 bytes)
- `setPollingMode(bool)` - switch from interrupt to polling
- `setInvertRX()` / `setInvertTX()` - signal inversion support
- `ignoreFlowControl(bool)` - disable DTR verification
- `dtr()` / `rts()` - check line states

---

## 5. Complete platformio.ini Examples

### Basic Configuration (Pico with Default USB)

```ini
[env:pico]
platform = https://github.com/maxgerhardt/platform-raspberrypi.git
board = pico
framework = arduino
board_build.core = earlephilhower
board_build.filesystem_size = 1m
monitor_speed = 115200
```

### With TinyUSB Stack

```ini
[env:pico_tinyusb]
platform = https://github.com/maxgerhardt/platform-raspberrypi.git
board = pico
framework = arduino
board_build.core = earlephilhower
build_flags = -DUSE_TINYUSB
monitor_speed = 115200
```

### With Debugging Support

```ini
[env:pico_debug]
platform = https://github.com/maxgerhardt/platform-raspberrypi.git
board = pico
framework = arduino
board_build.core = earlephilhower
debug_tool = picoprobe
upload_protocol = cmsis-dap
build_flags =
    -DDEBUG_RP2040_CORE
    -DDEBUG_RP2040_PORT=Serial2
monitor_speed = 115200
```

---

## 6. Other Important Build Flags

```ini
; Enable C++ exceptions
build_flags = -DPIO_FRAMEWORK_ARDUINO_ENABLE_EXCEPTIONS

; Enable RTTI (Run-Time Type Information)
build_flags = -DPIO_FRAMEWORK_ARDUINO_ENABLE_RTTI

; Enable IPv6 networking
build_flags = -DPIO_FRAMEWORK_ARDUINO_ENABLE_IPV6

; Enable FreeRTOS kernel
build_flags = -DPIO_FRAMEWORK_ARDUINO_ENABLE_FREERTOS

; Enable Bluetooth stack
build_flags = -DPIO_FRAMEWORK_ARDUINO_ENABLE_BLUETOOTH

; Debug flags
build_flags = -DDEBUG_RP2040_CORE        ; Core debugging
build_flags = -DDEBUG_RP2040_SPI         ; SPI debugging
build_flags = -DDEBUG_RP2040_WIRE        ; I2C debugging
build_flags = -DDEBUG_RP2040_PORT=Serial ; Debug output port
```

---

## Sources

- [Using this core with PlatformIO — Arduino-Pico 5.4.3 documentation](https://arduino-pico.readthedocs.io/en/latest/platformio.html)
- [arduino-pico/docs/platformio.rst at master · earlephilhower/arduino-pico](https://github.com/earlephilhower/arduino-pico/blob/master/docs/platformio.rst)
- [USB (Arduino and Adafruit_TinyUSB) — Arduino-Pico 5.4.3 documentation](https://arduino-pico.readthedocs.io/en/latest/usb.html)
- [Serial Ports (USB and UART) — Arduino-Pico 5.4.3 documentation](https://arduino-pico.readthedocs.io/en/latest/serial.html)
- [RP2040 Helper Class — Arduino-Pico 5.4.3 documentation](https://arduino-pico.readthedocs.io/en/latest/rp2040.html)
- [arduino-pico/docs/usb.rst at master · earlephilhower/arduino-pico](https://github.com/earlephilhower/arduino-pico/blob/master/docs/usb.rst)
- [maxgerhardt/pio-pico-core-earlephilhower-test/platformio.ini](https://github.com/maxgerhardt/pio-pico-core-earlephilhower-test/blob/main/platformio.ini)
- [Arduino-Pico GitHub Repository](https://github.com/earlephilhower/arduino-pico)
