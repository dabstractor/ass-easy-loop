# pEMF Timing Specifications

## Overview

This document provides detailed specifications for the pulsed electromagnetic field (pEMF) generation system. The system must maintain precise timing to ensure therapeutic effectiveness while allowing for debugging and monitoring through the USB HID logging system.

## Core Timing Requirements

### Target Frequency
- **Frequency**: 2.0 Hz
- **Period**: 500ms (0.5 seconds)

### Pulse Characteristics
- **High Time**: 2ms
- **Low Time**: 498ms
- **Duty Cycle**: 0.4% (2ms/500ms)

### Timing Tolerance
- **Accuracy**: ±1% (±5ms)
- **Jitter**: < 1ms
- **Drift**: < 0.1% over 24 hours

## Implementation Constraints

### RTIC Priority
- **pEMF Task Priority**: Highest (cannot be preempted)
- **Logging Task Priority**: Lowest (priority 3)
- **USB Tasks Priority**: Medium (priority 1)

### Real-Time Requirements
- pEMF pulse generation must never be delayed beyond ±1% tolerance
- Logging operations must not block pEMF task execution
- USB communication must not interfere with pulse timing

## Timing Validation

### Measurement Points
- Pulse start time accuracy
- Pulse duration accuracy
- Period consistency
- Long-term drift characteristics

### Validation Methods
- Internal timestamp comparison using RP2040 hardware timers
- External oscilloscope verification for hardware validation
- Statistical analysis over 1000+ pulse cycles
- Long-term drift measurement (24+ hours)

### Acceptance Criteria
- All pulses within ±1% timing tolerance
- No missed or delayed pulses during logging operations
- Consistent timing over extended periods
- Proper recovery from USB disconnect/reconnect events

## Logging Considerations

### Timing Impact
- Logging operations must complete within available idle time
- Queue operations must be non-blocking
- USB transmission must not delay pulse generation

### Performance Monitoring
- Log timing deviations from specification
- Report frequency calculation vs target frequency
- Warn about timing conflicts with other tasks
- Log pulse generation errors and recovery actions

### Error Handling
- Continue pEMF generation regardless of logging failures
- Log errors locally for debugging when USB unavailable
- Maintain timing accuracy during error recovery