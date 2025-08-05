# Software Setup and Flashing Instructions

This guide covers setting up the Rust embedded development environment, building the project, and flashing the firmware to your Raspberry Pi Pico.

## Prerequisites

### System Requirements

- **Operating System**: Windows, macOS, or Linux
- **USB Port**: For connecting Raspberry Pi Pico
- **Internet Connection**: For downloading tools and dependencies

### Hardware Requirements

- Raspberry Pi Pico (RP2040-based)
- USB-A to Micro-USB cable
- Optional: Picoprobe or other SWD debugger for advanced debugging

## Development Environment Setup

### Step 1: Install Rust

1. **Install Rust using rustup**:
   ```bash
   # Download and install rustup
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Follow the on-screen instructions, then restart your terminal
   source ~/.cargo/env
   ```

2. **Verify Rust installation**:
   ```bash
   rustc --version
   cargo --version
   ```

3. **Add ARM Cortex-M target**:
   ```bash
   rustup target add thumbv6m-none-eabi
   ```

### Step 2: Install Required Tools

1. **Install elf2uf2-rs** (for UF2 flashing):
   ```bash
   cargo install elf2uf2-rs
   ```

2. **Install probe-rs** (optional, for debugging):
   ```bash
   # Install probe-rs for advanced debugging
   cargo install probe-rs --features cli
   
   # Verify installation
   probe-rs --version
   ```

3. **Platform-specific tools**:

   **Linux**:
   ```bash
   # Install required packages
   sudo apt update
   sudo apt install build-essential pkg-config libudev-dev
   
   # Add user to dialout group for USB access
   sudo usermod -a -G dialout $USER
   # Log out and back in for group changes to take effect
   ```

   **macOS**:
   ```bash
   # Install Xcode command line tools
   xcode-select --install
   
   # Install Homebrew (if not already installed)
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
   ```

   **Windows**:
   - Install Visual Studio Build Tools or Visual Studio Community
   - Ensure "C++ build tools" workload is selected

### Step 3: Clone and Setup Project

1. **Clone the repository** (or download source):
   ```bash
   git clone <repository-url>
   cd ass-easy-loop
   ```

2. **Verify project structure**:
   ```
   ass-easy-loop/
   ├── Cargo.toml          # Project dependencies
   ├── .cargo/
   │   └── config.toml     # Target configuration
   ├── build.rs            # Build script
   ├── memory.x            # Memory layout
   ├── src/
   │   ├── main.rs         # Main application
   │   ├── lib.rs          # Library code
   │   └── battery.rs      # Battery management
   └── README.md
   ```

3. **Test build environment**:
   ```bash
   # This should compile without errors
   cargo check
   ```

## Project Configuration Details

### Cargo.toml Configuration

The project uses these key dependencies:

```toml
[dependencies]
cortex-m = { version = "0.7.7", features = ["critical-section-single-core"] }
cortex-m-rt = "0.7.3"                    # Runtime for Cortex-M
embedded-hal = "1.0.0"                   # Hardware abstraction layer
rp2040-hal = { version = "0.11.0", features = ["rt", "rtic-monotonic"] }
rtic = { version = "2.2.0", features = ["thumbv6-backend"] }
rtic-monotonics = { version = "2.1.0", features = ["rp2040"] }
portable-atomic = { version = "1.11", features = ["unsafe-assume-single-core"] }
panic-halt = "1.0.0"                     # Panic handler
```

### Target Configuration (.cargo/config.toml)

```toml
[target.thumbv6m-none-eabi]
runner = "elf2uf2-rs -d"                 # Automatic UF2 flashing
rustflags = [
  "-C", "link-arg=-Tlink.x",             # Linker script
  "-C", "link-arg=--nmagic",             # Linker optimization
]

[build]
target = "thumbv6m-none-eabi"            # Default target
```

### Memory Layout (memory.x)

The RP2040 memory configuration:

```
MEMORY {
    BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100      # Boot2 bootloader
    FLASH : ORIGIN = 0x10000100, LENGTH = 2048K - 0x100  # Application flash
    RAM   : ORIGIN = 0x20000000, LENGTH = 264K       # SRAM
}
```

## Building the Project

### Standard Build Process

1. **Clean build** (recommended for first build):
   ```bash
   cargo clean
   cargo build --release
   ```

2. **Development build** (faster, larger binary):
   ```bash
   cargo build
   ```

3. **Check for errors** (fast, no binary output):
   ```bash
   cargo check
   ```

### Build Optimization

For production use, always use release mode:
```bash
cargo build --release
```

**Release vs Debug differences**:
- **Debug**: Faster compilation, larger binary, includes debug symbols
- **Release**: Slower compilation, smaller binary, optimized for performance

### Troubleshooting Build Issues

**Common Error**: "linker `rust-lld` not found"
```bash
# Solution: Reinstall Rust with complete toolchain
rustup toolchain install stable
rustup default stable
```

**Common Error**: "can't find crate for `core`"
```bash
# Solution: Ensure target is installed
rustup target add thumbv6m-none-eabi
```

**Common Error**: Atomic operation compilation errors
- This is resolved by the `portable-atomic` dependency with `unsafe-assume-single-core` feature

## Flashing Methods

### Method 1: UF2 Bootloader (Recommended)

This is the easiest method for beginners:

1. **Enter bootloader mode**:
   - Hold the BOOTSEL button on the Pico
   - Connect USB cable to computer
   - Release BOOTSEL button
   - Pico should appear as "RPI-RP2" USB drive

2. **Flash using cargo run**:
   ```bash
   cargo run --release
   ```
   
   This automatically:
   - Builds the project
   - Converts ELF to UF2 format
   - Copies UF2 file to Pico
   - Resets and starts the application

3. **Manual UF2 flashing**:
   ```bash
   # Build and convert to UF2
   cargo build --release
   elf2uf2-rs target/thumbv6m-none-eabi/release/ass-easy-loop
   
   # Copy the generated .uf2 file to the RPI-RP2 drive
   cp ass-easy-loop.uf2 /path/to/RPI-RP2/
   ```

### Method 2: SWD with Probe-rs (Advanced)

For debugging and development:

1. **Connect SWD debugger**:
   - Connect Picoprobe or other SWD debugger
   - Wire SWD pins (SWDIO, SWCLK, GND)

2. **Flash with probe-rs**:
   ```bash
   # List available probes
   probe-rs list
   
   # Flash the binary
   probe-rs run --chip RP2040 target/thumbv6m-none-eabi/release/ass-easy-loop
   ```

3. **Debug with probe-rs**:
   ```bash
   # Start GDB server
   probe-rs gdb --chip RP2040 target/thumbv6m-none-eabi/release/ass-easy-loop
   
   # In another terminal, connect with GDB
   arm-none-eabi-gdb target/thumbv6m-none-eabi/release/ass-easy-loop
   (gdb) target remote :1337
   (gdb) load
   (gdb) continue
   ```

## Debugging and Monitoring

### Serial Output

The project uses `panic-halt` for error handling. For debugging output:

1. **Add defmt logging** (optional):
   ```toml
   # Add to Cargo.toml
   defmt = "0.3"
   defmt-rtt = "0.4"
   ```

2. **Monitor RTT output**:
   ```bash
   probe-rs attach --chip RP2040
   ```

### LED Status Monitoring

The onboard LED provides status information:
- **Flashing 2Hz**: Low battery
- **Solid ON**: Charging
- **OFF**: Normal operation

### Oscilloscope Monitoring

For precise timing verification:
- **GPIO 15**: Monitor pEMF pulse output (2Hz, 2ms pulse width)
- **GPIO 26**: Monitor battery voltage (through voltage divider)

## Customizing Timing Parameters

### Modifying pEMF Timing

The pEMF timing is defined in the source code. To customize:

1. **Edit timing constants** in `src/main.rs`:
   ```rust
   // Current: 2Hz (500ms period)
   const PULSE_HIGH_MS: u32 = 2;      // 2ms HIGH
   const PULSE_LOW_MS: u32 = 498;     // 498ms LOW
   
   // Example: 1Hz (1000ms period)
   const PULSE_HIGH_MS: u32 = 5;      // 5ms HIGH  
   const PULSE_LOW_MS: u32 = 995;     // 995ms LOW
   ```

2. **Rebuild and flash**:
   ```bash
   cargo run --release
   ```

### Modifying Battery Thresholds

Battery monitoring thresholds in `src/battery.rs`:

```rust
// Current thresholds (ADC values)
const LOW_BATTERY_THRESHOLD: u16 = 1425;    // ~3.1V
const CHARGING_THRESHOLD: u16 = 1675;       // ~3.6V

// Modify as needed for your battery chemistry
```

### Modifying Sampling Rates

Task scheduling in `src/main.rs`:

```rust
// Battery monitoring rate (currently 100ms = 10Hz)
const BATTERY_MONITOR_INTERVAL_MS: u32 = 100;

// LED update rate (currently 250ms for flashing)
const LED_FLASH_INTERVAL_MS: u32 = 250;
```

## Performance Optimization

### Release Build Optimizations

Add to `Cargo.toml` for maximum optimization:

```toml
[profile.release]
opt-level = "s"          # Optimize for size
lto = true              # Link-time optimization
codegen-units = 1       # Single codegen unit
panic = "abort"         # Smaller panic handler
strip = true            # Strip debug symbols
```

### Memory Usage Optimization

Monitor memory usage:
```bash
# Check binary size
cargo size --release

# Detailed memory analysis
cargo bloat --release
```

## Continuous Integration

### GitHub Actions Example

Create `.github/workflows/ci.yml`:

```yaml
name: CI
on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: thumbv6m-none-eabi
        override: true
    - name: Build
      run: cargo build --release
    - name: Test
      run: cargo test --lib
```

## Troubleshooting Common Issues

### Flashing Problems

**Issue**: "No RPI-RP2 drive found"
- **Solution**: Ensure Pico is in bootloader mode (hold BOOTSEL while connecting)

**Issue**: "Permission denied" on Linux
- **Solution**: Add user to dialout group, or use sudo

**Issue**: "Device not found" with probe-rs
- **Solution**: Check SWD connections, verify probe-rs installation

### Runtime Problems

**Issue**: Device doesn't start after flashing
- **Check**: Power supply voltage (should be 3.0V-5.5V)
- **Check**: Battery connection polarity
- **Solution**: Try reflashing with debug build

**Issue**: Incorrect timing behavior
- **Check**: Crystal oscillator (12MHz external)
- **Check**: Clock configuration in code
- **Solution**: Verify hardware connections

### Development Environment Issues

**Issue**: "cargo: command not found"
- **Solution**: Restart terminal after Rust installation, or source ~/.cargo/env

**Issue**: Build fails with linking errors
- **Solution**: Ensure all required system packages are installed

## Next Steps

After successful flashing:

1. **Verify basic operation** - LED should respond to battery state
2. **Test pEMF output** - Use oscilloscope to verify 2Hz square wave on GPIO 15
3. **Calibrate battery monitoring** - Compare ADC readings with actual battery voltage
4. **Connect external load** - Attach MOSFET driver and electromagnetic coil

---

**Next**: Continue with usage guide and safety information in section 10.4.