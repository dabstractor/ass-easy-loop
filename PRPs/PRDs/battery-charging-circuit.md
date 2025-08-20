# Battery Charging Circuit Requirements

## Overview

The battery charging circuit provides safe, monitored charging for LiPo batteries integrated into the pEMF device. The system must detect charging states, provide protection against overcharge/overdischarge, and maintain operation during charging cycles while ensuring safety.

## Core Requirements

### Battery Management System (BMS) Specifications

**Primary Requirements**:
- Support for single-cell LiPo batteries (3.7V nominal)
- Voltage range: 3.0V (cutoff) to 4.2V (fully charged)
- Automatic charge detection and state reporting
- Integration with existing ADC monitoring (GPIO 26)
- Protection against overcharge, overdischarge, and overcurrent

**Charging Module Specifications**:
- **Recommended IC**: TP4056 or equivalent
- **Input Voltage**: 5V USB (compatible with USB-C power delivery)
- **Charge Current**: 1A maximum (adjustable via R_PROG resistor)
- **Charge Termination**: 4.2V ± 1% with CC/CV charging profile
- **Thermal Protection**: Automatic current reduction at 120°C junction temp
- **Safety Features**: Over-voltage, under-voltage, over-current protection

### Charge Detection and State Management

**Voltage Thresholds** (from pico-dual-function-device specifications):

Using voltage divider (R1=10kΩ, R2=5.1kΩ, scale factor = 0.337):

| Battery Voltage | ADC Voltage | ADC Value (12-bit) | State |
|-----------------|-------------|-------------------|-------|
| < 3.1V | < 1.04V | < 1296 | Low (shutdown protection) |
| 3.1V - 3.6V | 1.04V - 1.21V | 1296 - 1508 | Normal (operating) |
| > 3.6V | > 1.21V | > 1508 | Charging (detected) |
| 4.2V | 1.42V | 1769 | Fully Charged |

**State Detection Logic**:
```rust
impl BatteryState {
    fn from_adc_reading(adc_value: u16) -> Self {
        match adc_value {
            0..=1425 => BatteryState::Low,      // ≤ 3.1V
            1426..=1674 => BatteryState::Normal, // 3.1V - 3.6V
            1675.. => BatteryState::Charging,    // > 3.6V
        }
    }
}
```

**Charging State Indicators**:
- **LED Behavior**: Solid ON when charging detected (ADC ≥ 1675)
- **USB Logging**: Charge state transitions logged with voltage readings
- **Safety Monitoring**: Continuous voltage monitoring during charge cycle

### Hardware Integration Requirements

**Circuit Components**:
```
LiPo Battery → Protection Circuit → TP4056 Charging IC → Power Management
     ↓                                        ↓
Voltage Divider (R1:R2 = 10k:5.1k) → ADC (GPIO 26) → RP2040
```

**Protection Circuit Requirements**:
- **Battery Protection IC**: DW01A or equivalent
- **MOSFETs**: Dual N-channel (charging/discharging control)
- **Overcurrent Protection**: 3A maximum discharge, 1A charge
- **Voltage Protection**: 2.4V under-voltage, 4.3V over-voltage cutoff
- **Short Circuit Protection**: < 1ms response time

**Power Path Management**:
- USB power takes priority when connected
- Battery backup when USB disconnected
- Seamless switching without operation interruption
- Isolation between USB power and battery during charging

### Charging Safety Requirements

**Thermal Management**:
- **Operating Temperature**: 0°C to 45°C during charging
- **Storage Temperature**: -20°C to 60°C
- **Thermal Cutoff**: Charging disabled above 50°C battery temperature
- **Heat Dissipation**: Adequate copper pour for IC thermal pad

**Electrical Safety**:
- **Reverse Polarity Protection**: Schottky diode or MOSFET protection
- **Overcurrent Protection**: Fuse or PTC resetable fuse (1.5A rating)
- **ESD Protection**: TVS diodes on USB input and battery connections
- **Isolation**: Proper ground plane design to prevent noise coupling

**Chemical Safety**:
- **Vent Holes**: PCB design includes battery expansion venting
- **Enclosure**: Fire-retardant materials (UL94 V-0 rating)
- **Spacing**: Minimum 3mm clearance around battery for thermal expansion
- **Marking**: Clear polarity and voltage markings on PCB

### Charging Algorithm and Control

**CC/CV Charging Profile**:
1. **Pre-charge Phase** (< 3.0V): 100mA constant current until 3.0V
2. **Constant Current Phase** (3.0V - 4.2V): 1A constant current
3. **Constant Voltage Phase** (4.2V): Constant 4.2V until current drops to 100mA
4. **Termination**: Charging complete when current < C/10 (typically 100mA)

**Charge Timing**:
- **Maximum Charge Time**: 4 hours total (safety timeout)
- **Pre-charge Timeout**: 30 minutes maximum
- **Trickle Charge**: Disabled (safety requirement for LiPo)
- **Recharge Threshold**: Resume charging when voltage drops below 4.05V

**Status Monitoring**:
- **Charge Progress**: Estimated from voltage curve and time
- **Fault Detection**: Over-temperature, over-time, voltage anomalies
- **Health Monitoring**: Capacity degradation tracking via charge/discharge cycles

### Integration with Existing System

**ADC Monitoring Enhancement**:
- **Sampling Rate**: Maintain 10Hz during normal operation
- **Charging Detection**: Increase to 1Hz during charging for safety monitoring
- **Noise Filtering**: Additional smoothing during switching transients
- **Calibration**: Temperature compensation for accurate voltage readings

**Task Priority Integration**:
- **Battery Monitoring**: Maintain medium priority (priority 2)
- **Charge Safety**: Highest priority for over-voltage protection
- **Status Updates**: Low priority LED updates during charging
- **Emergency Shutdown**: Immediate response to safety violations

**System Operation During Charging**:
- **pEMF Generation**: Full operation maintained during charging
- **USB Logging**: Enhanced logging of charging parameters
- **Performance**: No degradation of timing accuracy (±1% pEMF tolerance)
- **Power Budget**: Ensure sufficient power for full operation + charging

### Charging Circuit Implementation

**TP4056 Reference Design**:
```
USB 5V → Protection Diode → TP4056 VCC
                ↓
TP4056 CHRG/STDBY → Status LEDs (optional)
TP4056 BAT+ → Protection Circuit → Battery+
TP4056 BAT- → Battery-
TP4056 PROG → R_PROG (1.2kΩ for 1A charge current)
```

**Voltage Divider for ADC**:
```
Battery+ → R1 (10kΩ) → ADC_INPUT (GPIO 26) → R2 (5.1kΩ) → GND
```

**Protection Circuit**:
```
Battery+ → DW01A → MOSFET1 (charge control) → TP4056 BAT+
Battery- → MOSFET2 (discharge control) → System Ground
```

### Testing and Validation Requirements

**Electrical Testing**:
- **Charge Current Accuracy**: ±5% of programmed current (1A)
- **Voltage Regulation**: 4.2V ± 1% at full charge
- **Protection Testing**: Verify all protection thresholds
- **Temperature Testing**: Charge cycle across operating temperature range
- **Efficiency**: >80% charging efficiency from USB to battery

**Safety Testing**:
- **Overcharge Protection**: Verify cutoff at 4.3V battery voltage
- **Overcurrent Protection**: Verify 3A discharge current limit
- **Thermal Protection**: Verify thermal shutdown at temperature limits
- **Reverse Polarity**: Verify protection against reverse battery connection
- **Short Circuit**: Verify protection response time < 1ms

**Integration Testing**:
- **System Operation**: Verify pEMF timing maintained during charging
- **ADC Accuracy**: Validate voltage readings during charge cycle
- **State Detection**: Verify correct charging state detection and reporting
- **Power Management**: Verify seamless USB/battery switching
- **Long-term**: 100+ charge/discharge cycles without degradation

**Compliance Testing**:
- **USB Compliance**: USB 2.0 power delivery compliance
- **Battery Safety**: UN38.3 shipping regulations (if applicable)
- **EMC Testing**: Electromagnetic compatibility during charging
- **Thermal Testing**: UL compliance for thermal safety

### Failure Modes and Recovery

**Charge Failure Scenarios**:
1. **Over-voltage**: TP4056 internal protection + external monitoring
2. **Over-temperature**: Thermal shutdown + system notification
3. **Time-out**: 4-hour charge timeout with fault indication
4. **Battery Fault**: Protection circuit isolation + system alert
5. **USB Power Loss**: Seamless transition to battery operation

**Recovery Mechanisms**:
- **Automatic Restart**: Resume charging after fault clearance
- **Manual Reset**: Reset via USB command or power cycle
- **Degraded Operation**: Continue operation on battery if charging fails
- **Fault Reporting**: Detailed fault logs via USB HID interface
- **System Protection**: Automatic shutdown if battery critically low

### Environmental and Regulatory Requirements

**Operating Conditions**:
- **Ambient Temperature**: 0°C to 40°C during charging
- **Humidity**: 5% to 85% RH non-condensing
- **Altitude**: Sea level to 2000m operational
- **Vibration**: IEC 60068-2-6 (10-500Hz, 1g RMS)
- **Shock**: IEC 60068-2-27 (50g, 11ms half-sine)

**Regulatory Compliance**:
- **FCC Part 15**: EMI/EMC compliance for switching power supply
- **CE Marking**: European conformity for electronic devices
- **RoHS Compliance**: Lead-free materials and processes
- **WEEE Directive**: Waste electrical and electronic equipment
- **Battery Directive**: Proper battery disposal and recycling markings

## Implementation Timeline

**Phase 1: Hardware Design** (Week 1-2)
- TP4056 charging circuit schematic design
- Protection circuit integration
- PCB layout with thermal considerations
- Component selection and sourcing

**Phase 2: Software Integration** (Week 3-4)
- Enhanced ADC monitoring for charging states
- Charge detection algorithm implementation
- Safety monitoring and protection logic
- USB logging integration for charging status

**Phase 3: Testing and Validation** (Week 5-6)
- Hardware validation testing
- Software integration testing
- Safety compliance testing
- Long-term reliability testing

**Phase 4: Documentation** (Week 7)
- Complete technical documentation
- User safety instructions
- Manufacturing and assembly guides
- Compliance certification documentation

## Success Criteria

1. **Safety**: 100% compliance with LiPo charging safety standards
2. **Performance**: No degradation of existing pEMF timing accuracy
3. **Reliability**: >1000 charge/discharge cycles without failure
4. **Integration**: Seamless operation with existing system functionality
5. **User Experience**: Clear charging status indication and safety warnings
6. **Compliance**: Full regulatory compliance for target markets
7. **Cost**: BOM cost increase <$5 USD for charging circuit components
8. **Size**: Circuit fits within existing enclosure constraints

## Test-Driven Development Requirements

**Implementation Order** (per project requirements):
1. **Safety Tests First**: Comprehensive safety testing framework before implementation
2. **Hardware Simulation**: SPICE simulation of charging circuit before fabrication
3. **Progressive Testing**: Each component validated before integration
4. **Compliance Validation**: Regulatory testing throughout development process
5. **Integration Testing**: Validate with existing system at each milestone

**Required Test Coverage**:
- Unit tests for charge detection algorithms
- Hardware-in-loop testing with actual batteries
- Safety testing for all failure modes
- Thermal testing across operating temperature range
- Long-term reliability and cycle testing
- Integration testing with complete system functionality