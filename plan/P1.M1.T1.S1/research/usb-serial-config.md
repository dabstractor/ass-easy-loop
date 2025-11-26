# USB Serial Configuration for RP2040 Research

## 1. Build Flags for USB CDC Serial

### Default USB Stack Configuration
By default, arduino-pico uses the **Pico SDK USB stack** which automatically includes USB serial (CDC) support without requiring special flags. Serial is automatically available as a USB virtual serial port.

### Optional USB Stack Selection

**To use Adafruit TinyUSB instead:**
```ini
[env:pico]
platform = raspberrypi
board = rpipico
framework = arduino
build_flags = -DUSE_TINYUSB
```

**To disable USB entirely:**
```ini
build_flags = -DPIO_FRAMEWORK_ARDUINO_NO_USB
```

### Custom USB Identifiers (VID/PID)

Configure USB vendor/product IDs and device names:
```ini
[env:pico]
board_build.arduino.earlephilhower.usb_vid = 0xABCD
board_build.arduino.earlephilhower.usb_pid = 0x1337
board_build.arduino.earlephilhower.usb_manufacturer = "Custom Manufacturer"
board_build.arduino.earlephilhower.usb_product = "Ultra Cool Product"
```

### Debug Output Port Configuration

Route debug output to specific serial ports:
```ini
build_flags =
  -DDEBUG_RP2040_PORT=Serial
  -DDEBUG_RP2040_CORE
  -DDEBUG_RP2040_SPI
  -DDEBUG_RP2040_WIRE
```

---

## 2. Serial.begin(115200) Configuration

### Key Points
- `Serial.begin()` accepts a baud rate parameter, but **this rate is ignored** because it operates over USB
- The baud rate setting is only recognized by PlatformIO's serial monitor via the `monitor_speed` setting

### Minimal Setup Code Example

```cpp
void setup() {
  // Initialize USB serial
  Serial.begin(115200);

  // Wait for USB enumeration before using Serial
  while (!Serial) {
    delay(10);  // Required for USB to be ready
  }

  Serial.println("RP2040 USB Serial Ready!");
}

void loop() {
  Serial.printf("Core temperature: %2.1fC\n", analogReadTemp());
  delay(1000);
}
```

### Complete platformio.ini Configuration

```ini
[env:rpipico]
platform = raspberrypi
board = rpipico
framework = arduino
upload_protocol = picotool
monitor_port = /dev/ttyACM0          ; Linux/Mac
; monitor_port = COM3               ; Windows (adjust as needed)
monitor_speed = 115200
```

### Platform-Specific Port Examples
- **Linux**: `/dev/ttyACM0`, `/dev/ttyUSB0`
- **Windows**: `COM3`, `COM4` (varies by system)
- **macOS**: `/dev/cu.usbmodem14231` (auto-detected by PlatformIO)

### TinyUSB Stack Requirement

If using TinyUSB (`-DUSE_TINYUSB`), **you MUST explicitly call** `Serial.begin(115200)`:

```cpp
#include <Adafruit_TinyUSB.h>

void setup() {
  Serial.begin(115200);  // CRITICAL for TinyUSB
  while (!Serial);

  Serial.println("TinyUSB Serial Ready");
}
```

Without this call with TinyUSB, automatic IDE uploads won't work without manually pressing BOOTSEL.

---

## 3. Magic 1200 Baud Reset (Bootloader Entry)

### How It Works

The RP2040 implements a **software-based bootloader trigger** following Arduino conventions:

1. When a USB client (like PlatformIO) opens the virtual serial port at **1200 baud**
2. **AND** sets DTR (Data Terminal Ready) to LOW
3. The RP2040 firmware detects this combination
4. It performs a **software reset** into **BOOTSEL mass storage mode**
5. The device appears as an MSD (Mass Storage Device)
6. The compiled `.uf2` file is written to the MSD
7. The RP2040 automatically reboots into user code

### Technical Details

- **Feature**: `PICO_STDIO_USB_ENABLE_RESET_VIA_BAUD_RATE`
- **Default**: Enabled by default
- **Applies To**: USB Serial port only (Serial in code)
- **Hardware Requirement**: Uses the USB virtual serial lines, not hardware pins

### Four Ways to Enter BOOTSEL Mode

1. **Manual**: Press and hold BOOTSEL button while plugging in power
2. **Double-tap Reset**: Double-tap the RESET pin quickly
3. **USB Reset Command**: Programmatic USB reset commands
4. **1200 Baud Touch** (Magic 1200): Opening serial at 1200 baud with DTR

### PlatformIO Upload Behavior

PlatformIO automatically:
1. Detects when the Pico is not in bootloader mode
2. Opens USB serial at 1200 baud to trigger the reset
3. Waits for BOOTSEL MSD to appear
4. Writes the `.uf2` file
5. Waits for automatic reboot into user code

---

## 4. Monitor and Upload Configuration

### Complete platformio.ini Example

```ini
[env:rpipico]
platform = raspberrypi
board = rpipico
framework = arduino

; Upload Configuration
upload_protocol = picotool           ; Default for RP2040
; upload_protocol = jlink            ; Alternative: JLINK programmer
; upload_protocol = cmsis-dap        ; Alternative: CMSIS-DAP debugger
; upload_protocol = raspberrypi-swd  ; Alternative: Raspberry Pi SWD

; Monitor (Serial Monitor) Configuration
monitor_port = /dev/ttyACM0
monitor_speed = 115200
monitor_filters = direct            ; No extra filtering

; Optional: Set upload port if different from monitor
; upload_port = /dev/ttyACM0

; Build Configuration
board_build.mcu = rp2040
board_build.f_cpu = 133000000L      ; Default frequency
```

### Upload Protocol Options

| Protocol | Use Case | Notes |
|----------|----------|-------|
| **picotool** | Default, USB reset | Uses 1200 baud reset, most reliable |
| **jlink** | External JLINK debugger | Requires JLINK hardware |
| **cmsis-dap** | CMSIS-DAP compatible debuggers | For advanced debugging |
| **raspberrypi-swd** | Raspberry Pi SWD programmer | For dedicated hardware programmers |

### Platform-Specific Configuration

**Linux Example:**
```ini
[env:rpipico_linux]
upload_port = /dev/ttyACM0
monitor_port = /dev/ttyACM0
monitor_speed = 115200
```

**Windows Example:**
```ini
[env:rpipico_windows]
upload_port = COM3
monitor_port = COM3
monitor_speed = 115200
```

**macOS Example:**
```ini
[env:rpipico_macos]
upload_port = /dev/cu.usbmodem14231
monitor_port = /dev/cu.usbmodem14231
monitor_speed = 115200
```

### Manual Upload If Automatic Fails

If PlatformIO upload fails to trigger BOOTSEL automatically:

1. Manually press BOOTSEL button while plugging in USB
2. Device appears as "RPI-RP2" mass storage device
3. Copy `.pio/build/rpipico/firmware.uf2` to the MSD
4. Device automatically reboots

### Checking Available Ports

List all available serial ports:
```bash
pio device list
```

---

## 5. Hardware UART Ports (Serial1, Serial2)

For reference, here's how to configure the hardware UART ports separately from USB:

```cpp
void setup() {
  // USB Serial
  Serial.begin(115200);
  while (!Serial);

  // Hardware UART 0 (pins 1/2 by default, or set custom pins)
  Serial1.setRX(16);  // GPIO 16
  Serial1.setTX(17);  // GPIO 17
  Serial1.begin(9600);

  // Hardware UART 1 (pins 8/9 by default, or set custom pins)
  Serial2.setRX(12);  // GPIO 12
  Serial2.setTX(13);  // GPIO 13
  Serial2.begin(115200);
}
```

---

## 6. Troubleshooting Common Issues

### Issue: Serial Monitor Shows No Output

**Causes & Solutions:**
1. Missing `while (!Serial);` in setup() - Add it to wait for USB enumeration
2. Wrong `monitor_speed` - Match the `Serial.begin()` parameter (115200 typical)
3. Wrong `monitor_port` - Use `pio device list` to find correct port
4. TinyUSB without `Serial.begin()` - Ensure `Serial.begin(115200)` is called

### Issue: Upload Fails, Device Shows as "RPI-RP2" Mass Storage

**Solution:**
The device is already in bootloader mode. Either:
1. Manually copy `.pio/build/rpipico/firmware.uf2` to the device, OR
2. Power cycle the device (unplug USB)

### Issue: Windows Shows "Board CDC" Instead of COM Port

**Solution:**
Install proper CDC drivers using Zadig or update Windows drivers. PlatformIO should detect it automatically.

---

## Summary: Quick Reference Configuration

**Minimal working platformio.ini:**
```ini
[env:rpipico]
platform = raspberrypi
board = rpipico
framework = arduino
monitor_speed = 115200
```

**Minimal working code:**
```cpp
void setup() {
  Serial.begin(115200);
  while (!Serial);
  Serial.println("Hello from RP2040!");
}

void loop() {
  delay(1000);
}
```

---

## Sources

**Official Arduino-Pico Documentation:**
- [Using Arduino-Pico with PlatformIO](https://arduino-pico.readthedocs.io/en/latest/platformio.html)
- [Serial Ports Configuration](https://arduino-pico.readthedocs.io/en/latest/serial.html)
- [USB Configuration Guide](https://arduino-pico.readthedocs.io/en/latest/usb.html)

**PlatformIO Official Documentation:**
- [Raspberry Pi RP2040 Platform](https://docs.platformio.org/en/latest/platforms/raspberrypi.html)
- [Serial Monitor Configuration](https://docs.platformio.org/en/latest/core/userguide/device/cmd_monitor.html)
- [Upload Port Configuration](https://docs.platformio.org/en/latest/projectconf/sections/env/options/upload/upload_port.html)

**GitHub References:**
- [PlatformIO Build Script (platformio-build.py)](https://github.com/earlephilhower/arduino-pico/blob/master/tools/platformio-build.py)
- [Serial Documentation](https://github.com/earlephilhower/arduino-pico/blob/master/docs/serial.rst)
- [Arduino-Pico Discussions & Issues](https://github.com/earlephilhower/arduino-pico/discussions)
