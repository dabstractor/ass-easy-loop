# Battery Charging Implementation Checklist

## Pre-Implementation Validation ✅

**MANDATORY: Complete before any hardware work**

- [ ] Run safety tests: `cargo test battery_safety_tests --features testing`
- [ ] Validate voltage thresholds: `cargo test test_voltage_threshold_boundaries`
- [ ] Verify over-voltage protection: `cargo test test_over_voltage_protection`
- [ ] Check state transitions: `cargo test test_safe_state_transitions`

**All tests must pass before proceeding to implementation.**

## Implementation Phase 1: Core Logic (Days 1-2)

### File Modifications Required

**1. Update `src/types/battery.rs`:**
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatteryState {
    Low,        // < 3.1V - Shutdown protection
    Normal,     // 3.1V - 3.6V - Normal operation  
    Charging,   // > 3.6V - Charging detected
    Full,       // ~4.2V - Fully charged
    Fault,      // Safety violation detected
}

impl BatteryState {
    pub fn from_adc_reading(adc_value: u16) -> Self {
        match adc_value {
            0..=1425 => BatteryState::Low,      // ≤ 3.1V
            1426..=1674 => BatteryState::Normal, // 3.1V - 3.6V  
            1675..=1769 => BatteryState::Charging, // 3.6V - 4.2V
            1770..=1800 => BatteryState::Full,   // ~4.2V
            _ => BatteryState::Fault,            // > 4.2V - SAFETY
        }
    }
}
```

**2. Enhance `src/drivers/adc_battery.rs`:**
```rust
use rp2040_hal::adc::{Adc, AdcPin};
use rp2040_hal::gpio::Pin;

impl BatteryMonitor {
    pub fn read_voltage_mv(&mut self) -> u16 {
        let adc_reading: u16 = self.adc.read(&mut self.adc_pin).unwrap();
        // Convert ADC to millivolts: (adc * 3300 / 4095) / 0.337
        ((adc_reading as u32 * 3300 / 4095) * 1000 / 337) as u16
    }
    
    pub fn read_state(&mut self) -> BatteryState {
        let adc_reading: u16 = self.adc.read(&mut self.adc_pin).unwrap();
        BatteryState::from_adc_reading(adc_reading)
    }
}
```

**3. Add to `src/main.rs` RTIC app:**
```rust
// Add to shared resources
#[shared]
struct Shared {
    // existing resources...
    battery_monitor: BatteryMonitor,
    battery_state: BatteryState,
}

// Add battery monitoring task
#[task(shared = [battery_monitor, battery_state, log_queue], priority = 2)]
fn battery_monitor_task(mut ctx: battery_monitor_task::Context) {
    let current_state = ctx.shared.battery_monitor.lock(|monitor| monitor.read_state());
    
    ctx.shared.battery_state.lock(|state| {
        if *state != current_state {
            // State changed - log transition
            #[cfg(feature = "battery-logs")]
            {
                // Add log message to queue
            }
            *state = current_state;
        }
    });
    
    // Reschedule every 100ms (10Hz sampling)
    battery_monitor_task::spawn_after(Duration::<u64, 1, 1000>::millis(100)).unwrap();
}
```

### Validation After Phase 1
```bash
cargo check --target thumbv6m-none-eabi --features battery-logs
cargo build --target thumbv6m-none-eabi --features development
cargo test battery_integration_tests --features testing
```

## Implementation Phase 2: Hardware Integration (Days 3-5)

### Hardware Setup Checklist

**SAFETY FIRST - Use these exact components:**
- [ ] TP4056 charging module (with protection circuit)  
- [ ] 10kΩ resistor (1% tolerance) for voltage divider
- [ ] 5.1kΩ resistor (1% tolerance) for voltage divider
- [ ] Current-limited power supply (max 1.2A output)
- [ ] Multimeter for voltage verification
- [ ] **NO ACTUAL BATTERY YET** - Use power supply only

**Wiring Verification:**
- [ ] TP4056 VCC to 5V USB power
- [ ] TP4056 BAT+ through voltage divider to GPIO 26
- [ ] Voltage divider: BAT+ → 10kΩ → GPIO 26 → 5.1kΩ → GND
- [ ] All grounds connected
- [ ] Power supply current limit set to 1.2A
- [ ] Power supply voltage adjustable 3.0V - 4.3V

### Hardware Tests
```bash
# Flash with hardware testing
cargo run --features hardware-testing,battery-logs

# Test connection
python3 host_tools/battery_monitor.py --test-connection

# Validate ADC accuracy  
# Set power supply to 3.0V, verify reading ~1426
# Set power supply to 3.6V, verify reading ~1675  
# Set power supply to 4.2V, verify reading ~1769

# Run automated hardware validation
cargo test --features hardware-testing test_adc_accuracy_with_reference
```

## Implementation Phase 3: TP4056 Integration (Days 6-8)

### TP4056 Charging Circuit
**Required Components:**
- [ ] TP4056 module with built-in protection
- [ ] 1.2kΩ resistor for R_PROG (1A charge current)
- [ ] LED indicators for charge status (optional)
- [ ] USB-C connector for power input

**Integration Steps:**
1. [ ] Connect TP4056 to power supply
2. [ ] Verify TP4056 outputs correct voltage (should match input when not charging)
3. [ ] Test charge detection by applying load to BAT+ terminal
4. [ ] Validate voltage monitoring through entire range

**Critical Safety Test:**
```bash
# OVER-VOLTAGE PROTECTION TEST
# Slowly increase power supply voltage from 4.0V to 4.3V
# System MUST enter fault state before 4.3V
cargo test --features hardware-testing test_over_voltage_protection

# Monitor for safety violations
python3 host_tools/battery_monitor.py --duration 10 --log safety_test.csv
```

## Implementation Phase 4: Battery Integration (Days 9-11)

**⚠️ EXTREME CAUTION REQUIRED ⚠️**

**Safety Protocol - MANDATORY:**
- [ ] Use only new, quality LiPo battery (3.7V, 1000mAh max)
- [ ] Battery must be at ~3.0V (partially discharged) for first test
- [ ] Temperature monitoring - battery should stay <40°C
- [ ] Emergency disconnect switch within arm's reach
- [ ] Fire-safe work area (metal table, fire extinguisher nearby)
- [ ] Never leave charging unattended

**Battery Integration Steps:**
1. [ ] Connect partially discharged battery (3.0-3.2V)
2. [ ] Verify system detects "Low" or "Normal" state correctly
3. [ ] Connect USB power to begin charging
4. [ ] Verify system detects "Charging" state
5. [ ] Monitor complete charge cycle (may take 2+ hours)

**Continuous Monitoring During Battery Testing:**
```bash
# Start monitoring before connecting battery
python3 host_tools/battery_monitor.py --duration 180 --log battery_charge_cycle.csv

# Monitor system logs simultaneously
python3 host_tools/log_monitor.py --file charge_cycle_logs.txt

# Run performance validation during charging
cargo test --features hardware-testing test_complete_charge_cycle_timing
```

## Implementation Phase 5: Performance Validation (Days 12-14)

### pEMF Timing Validation

**Critical Requirement:** pEMF timing must remain within ±1% during charging

**Setup:**
1. [ ] Configure pEMF generation at known frequency (e.g., 1kHz)
2. [ ] Setup oscilloscope or frequency counter for timing measurement
3. [ ] Begin battery charging cycle
4. [ ] Measure pEMF frequency throughout charging

**Performance Tests:**
```bash
# Timing validation during different charge phases
cargo test test_pemf_timing_constant_current_phase --features hardware-testing
cargo test test_pemf_timing_constant_voltage_phase --features hardware-testing

# ADC sampling impact on timing
cargo test test_adc_sampling_timing_impact --features hardware-testing

# Long-term stability test (60+ minutes)
cargo test test_long_term_timing_stability --features hardware-testing
```

## Production Readiness Checklist

**Final Validation Suite:**
- [ ] All unit tests pass: `cargo test battery_safety_validation`
- [ ] Hardware tests pass: `cargo test --features hardware-testing`  
- [ ] Automation tests pass: `cargo test battery_automation_tests`
- [ ] Performance tests pass: `cargo test battery_performance_validation`
- [ ] Multiple complete charge cycles validated
- [ ] System operates correctly on battery power
- [ ] pEMF timing maintained within ±1% throughout charging
- [ ] No safety violations in 100+ hours of testing

**Production Build:**
```bash
cargo build --target thumbv6m-none-eabi --features production --release
# Binary should be <50KB for RP2040
```

## Emergency Procedures

**If Safety Violation Detected:**
1. **IMMEDIATELY** disconnect USB power
2. Disconnect battery if safe to do so
3. Allow components to cool
4. Review logs: `python3 host_tools/log_monitor.py --file emergency.log`
5. Run safety validation: `cargo test battery_safety_validation`
6. Do not reconnect power until root cause identified and resolved

**System Recovery Steps:**
1. Full safety test suite: `cargo test battery_safety_validation`
2. Hardware inspection and continuity testing  
3. Graduated re-testing starting from Phase 1
4. Document incident and corrective actions

## Quick Daily Testing Routine

**Morning Validation (5 minutes):**
```bash
# Quick syntax and safety check
cargo check --target thumbv6m-none-eabi --features testing
cargo test battery_safety_tests --features testing
cargo clippy --target thumbv6m-none-eabi --features testing -- -D warnings
```

**Pre-commit Validation (10 minutes):**
```bash
# Full automated test suite
cargo test battery_automation_tests
cargo build --target thumbv6m-none-eabi --features development --release
```

**Hardware Testing Session (30 minutes):**
```bash
# Hardware validation with monitoring
python3 host_tools/battery_monitor.py --test-connection
cargo run --features hardware-testing,battery-logs  
python3 host_tools/battery_monitor.py --duration 30 --log daily_validation.csv
```

## Success Metrics

**Phase 1 Complete:**
- All safety tests pass
- Core battery logic implemented
- State detection working correctly

**Phase 2 Complete:**  
- ADC readings accurate within ±50mV
- Hardware integration stable
- Voltage divider calibrated

**Phase 3 Complete:**
- TP4056 charging detection working
- Over-voltage protection verified
- Charging phases detected correctly

**Phase 4 Complete:**
- Complete battery charge cycle validated
- System operates on battery power
- No safety violations during testing

**Phase 5 Complete:**
- pEMF timing within ±1% during charging
- Performance maintained across all charge phases  
- Long-term stability demonstrated
- Production-ready implementation

## Files Created by This Validation Strategy

**Test Files:**
- `/home/dustin/projects/asseasyloop/tests/battery_safety_validation.rs` - Critical safety tests
- `/home/dustin/projects/asseasyloop/tests/battery_hardware_validation.rs` - Hardware integration tests  
- `/home/dustin/projects/asseasyloop/tests/battery_automation_tests.rs` - Automated testing workflows
- `/home/dustin/projects/asseasyloop/tests/battery_performance_validation.rs` - Performance and timing tests

**Host Tools:**
- `/home/dustin/projects/asseasyloop/host_tools/battery_monitor.py` - Continuous monitoring and validation

**Documentation:**
- `/home/dustin/projects/asseasyloop/BATTERY_VALIDATION_STRATEGY.md` - Complete validation strategy
- `/home/dustin/projects/asseasyloop/IMPLEMENTATION_CHECKLIST.md` - This implementation guide

**All files are ready for immediate use with your existing `cargo run` and `python host_tools/` workflow.**