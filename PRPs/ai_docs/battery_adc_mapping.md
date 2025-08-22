# Battery ADC Mapping Specifications

## Overview

This document provides detailed specifications for the battery voltage monitoring system using the RP2040's ADC peripheral. The system uses a voltage divider circuit to safely measure the battery voltage within the ADC's input range.

## Voltage Divider Circuit

### Component Values
- R1 (High-side): 10kΩ
- R2 (Low-side): 5.1kΩ

### Scale Factor Calculation
```
Scale Factor = R2 / (R1 + R2)
Scale Factor = 5.1kΩ / (10kΩ + 5.1kΩ)
Scale Factor = 5.1 / 15.1
Scale Factor = 0.337
```

This means a 3.7V battery will produce approximately 1.25V at the ADC input.

## ADC Specifications

### Reference Voltage
- ADC Reference: 3.3V
- ADC Resolution: 12-bit (0-4095)

### Conversion Formula
```
Battery Voltage = (ADC Reading * 3.3V / 4095) / 0.337
Battery Voltage = ADC Reading * 0.00238V
```

## Threshold Values

### ADC Thresholds
- **Low Battery**: ADC ≤ 1425 (≈ 3.1V battery)
- **Normal Battery**: 1425 < ADC < 1675 (3.1V - 3.6V)
- **Charging Battery**: ADC ≥ 1675 (> 3.6V)

### Voltage Calculations
- ADC 1425 = 1425 * 0.00238V ≈ 3.39V at ADC pin = 3.39V / 0.337 ≈ 3.1V battery
- ADC 1675 = 1675 * 0.00238V ≈ 3.99V at ADC pin = 3.99V / 0.337 ≈ 3.6V battery

## Implementation Notes

### Safety Margins
- Low threshold includes safety margin to ensure adequate warning before critical battery level
- Charging threshold set above normal range to clearly distinguish charging state

### Accuracy Considerations
- Resistor tolerance affects scale factor accuracy (±1% typical)
- Temperature variations may affect resistor values slightly
- ADC reference voltage tolerance affects measurements

### Logging Requirements
When battery logging is enabled (`battery-logs` feature), the system should log:
- Battery state transitions (Low/Normal/Charging)
- Periodic voltage readings (every 5 seconds by default)
- Critical threshold warnings (immediate logging)
- ADC read errors with diagnostic information
- Both raw ADC values (0-4095) and calculated voltages