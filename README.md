# Ass-Easy Loop

A Raspberry Pi Pico-based system that combines precise pulsed Electromagnetic Field (pEMF) generation with intelligent battery monitoring and visual status indication.

## Project Overview

This DIY project creates a dual-function embedded device that:

1. **Generates precise pEMF waveforms** - Produces a continuous 2Hz square wave with exact 2ms HIGH / 498ms LOW timing for driving electromagnetic field devices
2. **Monitors battery status** - Continuously samples battery voltage and provides visual feedback through LED indicators
3. **Operates in real-time** - Uses the RTIC framework to ensure timing-critical operations are never compromised

### 🔧 Quick Build & Flash Command
```bash
cargo run-embedded
```
**Must have RP2040/Pico in bootloader mode (BOOTSEL + USB)**
[📖 Full Build Instructions Below](#-quick-start-building-and-flashing-to-hardware)

### What is pEMF Therapy?

Pulsed Electromagnetic Field (pEMF) therapy uses electromagnetic fields to potentially stimulate cellular repair and improve circulation. This device generates precise electromagnetic pulses that can be used with external coils or electromagnetic applicators. The 2Hz frequency with specific pulse timing is commonly used in therapeutic applications.

**Important**: This is a DIY educational project. Any therapeutic claims are not medically validated. Consult healthcare professionals before using for any health-related purposes.

### Battery Monitoring System

The integrated battery monitoring system provides:
- Real-time voltage monitoring of LiPo batteries (3.0V - 4.2V range)
- Automatic state detection (Low/Normal/Charging)
- Visual feedback through onboard LED patterns
- Protection against over-discharge

## Technical Specifications

| Parameter | Specification | Notes |
|-----------|---------------|-------|
| **Microcontroller** | Raspberry Pi Pico (RP2040) | ARM Cortex-M0+ dual-core |
| **Operating Voltage** | 3.3V | Regulated from USB or battery |
| **External Crystal** | 12MHz | For precise timing |
| **pEMF Output** | GPIO 15 | MOSFET driver control |
| **Battery Input** | GPIO 26 (ADC) | Through voltage divider |
| **Status LED** | GPIO 25 | Onboard LED |

### pEMF Driver Specifications

| Parameter | Value | Tolerance |
|-----------|-------|-----------|
| **Frequency** | 2.0 Hz | ±1% |
| **Pulse Width HIGH** | 2.0 ms | ±1% |
| **Pulse Width LOW** | 498.0 ms | ±1% |
| **Output Type** | Digital (3.3V logic) | MOSFET driver compatible |
| **Priority** | Highest | Real-time guaranteed |

### Battery Monitor Specifications

| Parameter | Value | Notes |
|-----------|-------|-------|
| **Sampling Rate** | 10 Hz | 100ms intervals |
| **ADC Resolution** | 12-bit | 0-4095 range |
| **Battery Range** | 3.0V - 4.2V | LiPo compatible |
| **Low Battery Threshold** | 3.1V | ADC ≤ 1425 |
| **Normal Range** | 3.1V - 3.6V | ADC 1425-1675 |
| **Charging Threshold** | 3.6V+ | ADC ≥ 1675 |
| **Update Latency** | <200ms | State change response |

### LED Status Indicators

| Battery State | LED Pattern | Description |
|---------------|-------------|-------------|
| **Low** | 2Hz Flash | 250ms ON, 250ms OFF |
| **Normal** | OFF | Solid OFF |
| **Charging** | Solid ON | Continuous ON |
| **State Change** | <500ms | Response time |

## Hardware Requirements

### Core Components

| Component | Specification | Quantity | Purpose |
|-----------|---------------|----------|---------|
| **Raspberry Pi Pico** | RP2040-based | 1 | Main microcontroller |
| **MOSFET Driver Module** | Logic-level, 3.3V compatible | 1 | pEMF pulse amplification |
| **Resistors** | 10kΩ, 5.1kΩ (1% tolerance) | 1 each | Voltage divider |
| **Capacitors** | 100nF ceramic | 2-3 | Power filtering |
| **LiPo Battery** | 3.7V, 500mAh+ | 1 | Power source |
| **Connector** | JST-PH 2-pin | 1 | Battery connection |

### Optional Components

| Component | Purpose | Notes |
|-----------|---------|-------|
| **Breadboard/PCB** | Prototyping | For permanent assembly |
| **Enclosure** | Protection | 3D printed or commercial |
| **External Coil** | pEMF generation | User-designed electromagnetic coil |
| **Charging Module** | Battery charging | TP4056 or similar |

### Tools Required

- Soldering iron and solder
- Multimeter
- Breadboard or PCB
- Wire strippers
- Computer with USB port
- Probe-rs compatible debugger (optional)

## Pin Assignments

| GPIO Pin | Function | Direction | Configuration |
|----------|----------|-----------|---------------|
| **GPIO 15** | MOSFET Control | Output | Push-pull, pEMF driver |
| **GPIO 25** | Status LED | Output | Push-pull, onboard LED |
| **GPIO 26** | Battery Monitor | Input | ADC, floating input |

### Voltage Divider Circuit

The battery monitoring uses a voltage divider to scale the 3.7V LiPo battery to the ADC's 3.3V range:

```
Battery (+) ----[10kΩ]----+----[5.1kΩ]---- GND
                          |
                     GPIO 26 (ADC)
```

**Scaling Factor**: 0.337 (5.1kΩ / 15.1kΩ)
- 3.0V battery → 1.01V ADC → 1260 counts
- 3.7V battery → 1.25V ADC → 1556 counts  
- 4.2V battery → 1.42V ADC → 1769 counts

## Software Architecture

### Real-Time Task System (RTIC 2.0)

The software uses a priority-based task system:

1. **Highest Priority**: pEMF pulse generation (hardware timer driven)
2. **Medium Priority**: Battery monitoring (100ms periodic)
3. **Low Priority**: LED control (event-driven)

### Key Features

- **Memory Safe**: Rust's ownership system prevents common embedded bugs
- **Real-Time Guarantees**: RTIC ensures timing-critical tasks are never delayed
- **Resource Sharing**: Compile-time verified shared resource access
- **No Dynamic Allocation**: Stack-only memory usage for predictable behavior

## Safety Considerations

⚠️ **Important Safety Information**

### Electrical Safety
- Always disconnect power when making connections
- Verify polarity before connecting battery
- Use appropriate fuse protection for battery circuits
- Ensure MOSFET driver is rated for your load current

### pEMF Safety
- Start with low power levels and short exposure times
- Do not use near pacemakers or other medical implants
- Avoid prolonged exposure without medical supervision
- This device is for educational/experimental use only

### Battery Safety
- Use only quality LiPo batteries with protection circuits
- Never leave charging unattended
- Store batteries at proper voltage levels (3.7V-3.8V)
- Dispose of damaged batteries properly

## Project Structure

The project is organized into the following directories:

```
├── src/                    # Rust source code
├── tests/                  # Rust integration tests
├── docs/                   # All documentation
│   ├── setup/             # Setup and installation guides
│   ├── hardware/          # Hardware documentation and wiring guides
│   ├── api/               # API documentation and usage examples
│   ├── troubleshooting/   # Troubleshooting guides
│   ├── development/       # Development environment documentation
│   ├── BOOTLOADER_FLASHING_GUIDE.md  # Firmware flashing guide
│   └── USB_HID_INTEGRATION_TESTS.md  # USB HID integration tests
├── scripts/               # Executable scripts organized by function
│   ├── validation/        # Hardware validation scripts (includes validate_* executables)
│   ├── bootloader/        # Bootloader-related scripts
│   ├── testing/           # Test execution scripts
│   └── utilities/         # General utility scripts
├── test_framework/        # Comprehensive test framework
├── artifacts/             # Generated files and outputs
│   ├── test_results/      # Test output files
│   ├── firmware/          # Generated firmware files
│   ├── logs/              # Log files
│   ├── bootloader_debugging_summary.md  # Bootloader debugging info
│   ├── bootloader_entry_fix.patch       # Bootloader patches
│   └── bootloader_fix.rs                # Bootloader fixes
└── validation_scripts/    # Setup validation scripts
```

## Build and Flash Instructions

### 🚀 Quick Start: Building and Flashing to Hardware

**IMPORTANT**: This application runs on embedded ARM hardware (Raspberry Pi Pico RP2040). You must use the embedded-specific commands to build and flash to hardware.

#### Primary Command (Recommended)
```bash
cargo run-embedded
```

This command will:
- Build the application for the correct embedded target (`thumbv6m-none-eabi`)
- Compile only embedded dependencies (no std dependencies)
- Flash the compiled binary directly to the connected RP2040/Pico device
- Automatically detect and upload via UF2 bootloader mode

#### Alternative Manual Command
If the alias doesn't work, use this full command:
```bash
cargo run --release --target thumbv6m-none-eabi --features embedded --no-default-features
```

#### Hardware Setup for Flashing
1. **Connect your Raspberry Pi Pico** to your computer via USB
2. **Enter Bootloader Mode** by holding the BOOTSEL button while plugging in the USB cable
3. **Wait for RPI-RP2 drive** to appear on your computer
4. **Run the build/flash command** above
5. **Success**: The binary will transfer and the device will automatically reboot

#### Expected Output
```
Found pico uf2 disk /run/media/your_username/RPI-RP2
Transferring program to pico
512 B / 769.00 KB [==>----------------------------] 0.67 % 156.00 MB/s 0s
...
769.00 KB / 769.00 KB [=============================] 100.00 % 205.00 MB/s 0s
```

#### Troubleshooting Flashing
| Issue | Solution |
|-------|----------|
| "Unable to find mounted pico" | Ensure device is in bootloader mode (BOOTSEL + USB) |
| "can't find crate for `std`" | You're using wrong command - use `cargo run-embedded` |
| Linking errors with `cc` failed | Add `--target thumbv6m-none-eabi --no-default-features` |
| USB connection issues | Try different USB cable or port |
| RPI-RP2 drive not appearing | Re-enter bootloader mode |

#### Common Commands Reference
| Purpose | Command |
|---------|---------|
| **Build only** (no flash) | `cargo build-embedded` |
| **Release build** (optimized) | `cargo run-embedded` (defaults to release) |
| **Debug build** (with debugging) | Manual: `cargo run --target thumbv6m-none-eabi --features embedded --no-default-features` |

⚠️ **DO NOT USE** `cargo run --release --features embedded` - This will fail with linking errors because it tries to build for your host computer, not the embedded target.

---

## Getting Started

1. **Hardware Assembly** - See [docs/hardware/WIRING_GUIDE.md](docs/hardware/WIRING_GUIDE.md) for detailed wiring instructions
2. **Software Setup** - See [docs/setup/SOFTWARE_SETUP.md](docs/setup/SOFTWARE_SETUP.md) for development environment setup
3. **USB HID Logging** - See [docs/setup/USB_HID_LOGGING_SETUP_GUIDE.md](docs/setup/USB_HID_LOGGING_SETUP_GUIDE.md) for comprehensive logging setup
4. **Usage Guide** - See [docs/api/USB_HID_USAGE_EXAMPLES.md](docs/api/USB_HID_USAGE_EXAMPLES.md) for operation examples

## Documentation Index

For a complete overview of all available documentation, see [DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md).

## Project Status

This project implements a complete real-time embedded system with:
- ✅ Hardware abstraction layer
- ✅ Battery state management
- 🔄 pEMF pulse generation (in progress)
- 🔄 LED control system (in progress)
- 🔄 RTIC task coordination (in progress)

## Contributing

This is an open-source educational project. Contributions are welcome for:
- Hardware design improvements
- Software optimizations
- Documentation enhancements
- Testing and validation

## License

This project is provided as-is for educational purposes. Use at your own risk.

---

**Next Steps**: Continue with the detailed wiring diagrams and assembly instructions in the following sections.
#
# Usage Guide and Safety Information

### Device Operation

#### Initial Setup and Testing

1. **Pre-operation Checklist**:
   - [ ] Verify all wiring connections match the wiring diagram
   - [ ] Check battery voltage (should be 3.0V - 4.2V)
   - [ ] Ensure MOSFET driver is properly connected
   - [ ] Confirm electromagnetic load is within driver specifications
   - [ ] Test device without load first

2. **Power-On Sequence**:
   - Connect battery to device
   - Observe LED status indicator
   - Verify pEMF output with oscilloscope (if available)
   - Connect electromagnetic load only after verification

#### LED Status Indicators

The onboard LED provides real-time feedback about battery and system status:

| LED Pattern | Battery State | Voltage Range | Action Required |
|-------------|---------------|---------------|-----------------|
| **2Hz Flash** | Low Battery | < 3.1V | Charge battery immediately |
| **Solid OFF** | Normal | 3.1V - 3.6V | Normal operation |
| **Solid ON** | Charging | > 3.6V | Battery charging or fully charged |

**LED Response Times**:
- State changes are detected within 200ms
- LED pattern updates within 500ms
- Consistent patterns indicate proper operation

#### pEMF Output Characteristics

**Normal Operation Indicators**:
- Continuous 2Hz square wave on GPIO 15
- 2ms HIGH pulse, 498ms LOW period
- Timing accuracy within ±1% (±10ms per cycle)
- No interruption during battery monitoring or LED updates

**Verification Methods**:
- **Oscilloscope**: Connect to GPIO 15 to verify timing
- **Multimeter**: Should show ~1.65V DC average (50% duty cycle at 3.3V)
- **Audio**: 2Hz clicking from electromagnetic load (if audible)

### Battery Management

#### Supported Battery Types

**Recommended**: 3.7V Lithium Polymer (LiPo) batteries
- Capacity: 500mAh minimum for stable operation
- Discharge rate: 1C or higher
- Protection circuit: Built-in protection recommended

**Voltage Ranges**:
- **Minimum Operating**: 3.0V (device will indicate low battery)
- **Normal Operating**: 3.1V - 3.6V
- **Charging/Full**: 3.6V - 4.2V
- **Maximum Safe**: 4.2V (do not exceed)

#### Charging Guidelines

1. **Charging Methods**:
   - **USB Charging**: Connect USB cable while battery is connected
   - **External Charger**: Use dedicated LiPo charger (recommended)
   - **Charging Module**: TP4056 or similar integrated charging circuit

2. **Charging Safety**:
   - Never leave charging unattended
   - Monitor battery temperature during charging
   - Stop charging if battery becomes warm (>40°C)
   - Use appropriate charging current (typically 0.5C to 1C)

3. **Charging Indicators**:
   - LED solid ON indicates charging voltage detected
   - Use external charging indicator for charge completion
   - Verify voltage with multimeter if uncertain

#### Battery Maintenance

**Regular Monitoring**:
- Check battery voltage weekly during active use
- Inspect battery for physical damage or swelling
- Clean battery contacts monthly

**Storage Guidelines**:
- Store at 3.7V - 3.8V for long-term storage
- Avoid storing fully charged (4.2V) or fully discharged (<3.0V)
- Store in cool, dry location away from heat sources
- Check voltage monthly during storage

**Replacement Indicators**:
- Rapid voltage drop under load
- Reduced operating time between charges
- Physical swelling or damage to battery case
- Inability to hold charge above 3.6V

### Performance Validation

#### Timing Accuracy Testing

**Equipment Needed**:
- Digital oscilloscope or logic analyzer
- Multimeter with frequency measurement
- Stopwatch for long-term verification

**Test Procedures**:

1. **Pulse Width Verification**:
   ```
   Test: Measure HIGH pulse width
   Expected: 2.0ms ± 0.02ms (1% tolerance)
   Method: Oscilloscope on GPIO 15, measure pulse width
   ```

2. **Frequency Verification**:
   ```
   Test: Measure pulse frequency
   Expected: 2.00Hz ± 0.02Hz
   Method: Frequency counter or oscilloscope FFT
   ```

3. **Long-term Stability**:
   ```
   Test: 24-hour continuous operation
   Expected: <0.1% frequency drift
   Method: Log timestamps of pulse edges
   ```

#### Battery Monitoring Accuracy

**Calibration Procedure**:

1. **Voltage Reference Test**:
   - Measure actual battery voltage with precision multimeter
   - Compare with device ADC reading (through serial output if available)
   - Calculate calibration factor if needed

2. **Threshold Testing**:
   ```
   Test Low Threshold (3.1V):
   - Gradually discharge battery to 3.1V
   - Verify LED changes to 2Hz flash pattern
   - Note actual voltage when transition occurs
   
   Test Charging Threshold (3.6V):
   - Gradually charge battery to 3.6V  
   - Verify LED changes to solid ON
   - Note actual voltage when transition occurs
   ```

3. **Response Time Testing**:
   - Rapidly change battery voltage (using variable power supply)
   - Measure time from voltage change to LED pattern change
   - Should be <500ms for LED updates

### Safety Information

#### ⚠️ Electrical Safety

**Before Each Use**:
- Inspect all connections for damage or corrosion
- Verify no loose wires or exposed conductors
- Check battery condition and voltage
- Ensure proper grounding of all metal components

**During Operation**:
- Never touch exposed electrical connections while powered
- Keep device away from water and moisture
- Monitor device temperature - discontinue use if overheating
- Be prepared to disconnect power quickly in emergency

**Power Supply Safety**:
- Use only specified battery types (3.7V LiPo)
- Never exceed 4.2V input voltage
- Install fuse protection in battery circuit (recommended)
- Use batteries with built-in protection circuits

#### ⚠️ pEMF Safety

**Electromagnetic Field Exposure**:
- Start with lowest power settings and short exposure times
- Gradually increase intensity only as needed
- Limit continuous exposure time (suggest <30 minutes per session)
- Allow cooling periods between sessions

**Medical Contraindications**:
- **DO NOT USE** if you have a pacemaker or other implanted medical device
- **DO NOT USE** during pregnancy without medical consultation
- **DO NOT USE** on or near the head/brain area
- **DISCONTINUE USE** if you experience any adverse effects

**Electromagnetic Compatibility**:
- May interfere with sensitive electronic equipment
- Keep away from computers, phones, and medical devices during operation
- Use in well-ventilated area away from other electronics
- Consider EMI shielding for permanent installations

#### ⚠️ Battery Safety

**Lithium Battery Hazards**:
- Risk of fire or explosion if damaged, overcharged, or overheated
- Toxic gases may be released if battery is damaged
- Never puncture, crush, or disassemble battery
- Dispose of damaged batteries at appropriate recycling centers

**Charging Safety**:
- Use only appropriate LiPo charging methods
- Never charge unattended or overnight
- Charge in fire-safe location away from flammable materials
- Stop charging immediately if battery becomes hot or swells

**Emergency Procedures**:
- **Battery Fire**: Use Class D fire extinguisher or sand, never water
- **Battery Swelling**: Disconnect immediately, handle with care, dispose properly
- **Electrical Shock**: Disconnect power, seek medical attention if needed

### Maintenance and Troubleshooting

#### Regular Maintenance Schedule

**Weekly** (during active use):
- Check battery voltage and charge level
- Inspect LED operation and patterns
- Verify pEMF output with test equipment
- Clean device exterior and connections

**Monthly**:
- Deep clean all connections with isopropyl alcohol
- Check all solder joints and wire connections
- Test emergency shutdown procedures
- Update firmware if new versions available

**Quarterly**:
- Perform full performance validation tests
- Replace any worn or damaged components
- Review and update safety procedures
- Document any performance changes

#### Common Issues and Solutions

**Issue**: LED not responding to battery changes
- **Cause**: Faulty voltage divider or ADC connection
- **Solution**: Check resistor values and GPIO 26 connection
- **Prevention**: Use 1% tolerance resistors, secure connections

**Issue**: Inconsistent pEMF timing
- **Cause**: Clock configuration or software timing issues
- **Solution**: Verify 12MHz crystal, check software configuration
- **Prevention**: Use quality crystal oscillator, avoid EMI sources

**Issue**: Rapid battery drain
- **Cause**: Excessive load current or battery degradation
- **Solution**: Check load specifications, test battery capacity
- **Prevention**: Use appropriate load ratings, maintain battery properly

**Issue**: Device resets or stops operating
- **Cause**: Power supply instability or software crash
- **Solution**: Check power connections, reflash firmware
- **Prevention**: Use stable power supply, add decoupling capacitors

#### Performance Monitoring

**Key Performance Indicators**:
- pEMF timing accuracy (should remain within ±1%)
- Battery monitoring response time (<500ms)
- LED pattern consistency
- Overall system stability (no unexpected resets)

**Logging and Documentation**:
- Keep log of operating hours and performance
- Document any modifications or repairs
- Record battery replacement dates and performance
- Note any environmental factors affecting operation

### Advanced Usage

#### Custom Timing Configuration

For users comfortable with software modification:

1. **Frequency Adjustment**:
   - Modify `PULSE_HIGH_MS` and `PULSE_LOW_MS` constants
   - Maintain total period for desired frequency
   - Recompile and flash updated firmware

2. **Battery Threshold Customization**:
   - Adjust `LOW_BATTERY_THRESHOLD` and `CHARGING_THRESHOLD`
   - Calibrate for specific battery chemistry
   - Test thoroughly before regular use

#### Integration with External Systems

**Control Interfaces**:
- GPIO pins available for external control signals
- Serial communication possible with additional code
- I2C/SPI interfaces available for sensor integration

**Monitoring Interfaces**:
- ADC readings can be output via serial
- GPIO pins can provide status signals
- Real-time data logging possible with modifications

### Regulatory and Compliance

#### FCC/CE Compliance

This device may require regulatory approval for commercial use:
- Electromagnetic emissions testing may be required
- Medical device regulations may apply for therapeutic use
- Consult local regulations before commercial distribution

#### Open Source Licensing

This project is provided under open source license:
- Hardware designs may be modified and redistributed
- Software source code is available for modification
- Commercial use may require additional licensing

---

**Support and Community**

For technical support, updates, and community discussion:
- Check project documentation for latest information
- Report issues through appropriate channels
- Contribute improvements back to the community
- Share safety experiences and best practices

**Disclaimer**: This device is provided for educational and experimental purposes. Users assume all responsibility for safe operation and compliance with local regulations. No medical claims are made or implied.