# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

Build firmware:
```bash
cargo build --release
```

Flash device:
```bash
elf2uf2-rs target/thumbv6m-none-eabi/release/ass-easy-loop
```

Run validation tests:
```bash
cargo run --bin validate_battery_logging
```

Run unit tests:
```bash
cargo test --test logging_tests
```

## Architecture Overview

### RTIC Task System
- **pEMF Task**: Highest priority (GPIO 15), generates 2Hz waveform (2ms HIGH/498ms LOW)
- **Battery Monitor**: Medium priority (GPIO 26 ADC), 10Hz sampling with state detection
- **LED Control**: Low priority, responds to battery state changes

### Key Modules
- `battery.rs`: Manages state transitions between Low/Normal/Charging (src/battery.rs:45)
- `config.rs`: Contains voltage thresholds and timing constants
- `logging.rs`: USB HID logging implementation
- RTIC resources shared between tasks with compile-time safety