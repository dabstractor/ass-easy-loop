# Comprehensive Test Scenarios

This document describes the comprehensive test scenarios implemented for hardware validation, stress testing, regression testing, performance benchmarking, and integration testing.

## Overview

The comprehensive test framework provides pre-defined test scenarios that validate all aspects of the RP2040 pEMF/battery monitoring device. These scenarios can be run individually or as a complete test suite.

## Test Scenario Types

### 1. Hardware Validation Suite

**Purpose**: Comprehensive validation of all hardware subsystems

**Tests Included**:
- System startup and communication check
- pEMF timing accuracy validation
- pEMF pulse consistency testing
- Battery ADC calibration and accuracy
- Battery voltage range validation
- LED functionality testing (all patterns)
- LED timing accuracy validation
- USB communication stress testing
- System integration test (all subsystems active)

**Duration**: ~5 minutes
**Requirements**: 9.1, 9.5

### 2. Stress Testing Suite

**Purpose**: Validate system behavior under stress conditions

**Tests Included**:
- Baseline performance measurement
- CPU stress testing
- Memory stress testing
- I/O stress testing
- Combined subsystem stress testing
- Long-term stability testing
- Post-stress recovery validation

**Duration**: ~10 minutes (configurable)
**Requirements**: 9.2, 9.5

### 3. Regression Testing Suite

**Purpose**: Validate existing functionality after firmware changes

**Tests Included**:
- Core functionality regression
- pEMF timing regression
- Battery monitoring regression
- LED control regression
- USB logging regression
- Configuration persistence regression
- Error handling regression
- Performance regression

**Duration**: ~3 minutes
**Requirements**: 9.3, 9.5

### 4. Performance Benchmarking Suite

**Purpose**: Measure and validate system performance metrics

**Tests Included**:
- Timing precision benchmarking
- USB throughput benchmarking
- Memory usage benchmarking
- Task execution time benchmarking
- ADC sampling rate benchmarking
- LED update rate benchmarking
- System responsiveness benchmarking
- Comprehensive performance profiling

**Duration**: ~4 minutes
**Requirements**: 9.4, 9.5

### 5. Integration Testing Suite

**Purpose**: End-to-end system integration validation

**Tests Included**:
- System initialization integration
- Multi-subsystem coordination
- Real-time constraint validation
- Data flow integration
- Error propagation integration
- State management integration
- Resource sharing integration
- End-to-end workflow validation

**Duration**: ~5 minutes
**Requirements**: 9.1, 9.2, 9.3, 9.4, 9.5

## Usage

### Command Line Interface

```bash
# Run specific scenario
python -m test_framework.comprehensive_test_runner --scenario hardware_validation

# Run full test suite
python -m test_framework.comprehensive_test_runner --full-suite

# Run with custom parameters
python -m test_framework.comprehensive_test_runner --scenario stress_testing --stress-duration 60000 --stress-load 90

# Run on specific devices
python -m test_framework.comprehensive_test_runner --full-suite --devices DEVICE001 DEVICE002

# Skip specific scenarios in full suite
python -m test_framework.comprehensive_test_runner --full-suite --skip-scenarios stress_testing
```

### Programmatic Usage

```python
from test_framework import ComprehensiveTestRunner, TestScenarioType, ScenarioParameters

# Initialize runner
runner = ComprehensiveTestRunner("test_results")
runner.setup_logging("INFO")

# Discover and connect devices
devices = runner.discover_and_connect_devices()

# Run specific scenario
result = runner.run_scenario(TestScenarioType.HARDWARE_VALIDATION, devices)

# Run full test suite
full_result = runner.run_full_test_suite(devices)

# Run custom scenario
custom_steps = [
    {
        'name': 'quick_test',
        'test_type': 5,  # USB_COMMUNICATION_TEST
        'parameters': {'message_count': 5},
        'timeout': 10.0
    }
]
custom_result = runner.run_custom_scenario("Quick Test", custom_steps, devices)
```

### Configuration

Test scenarios can be configured using JSON configuration files:

```json
{
  "scenario_parameters": {
    "hardware_validation": {
      "pemf_test_duration_ms": 5000,
      "pemf_tolerance_percent": 1.0,
      "battery_reference_voltage": 3.3,
      "led_test_duration_ms": 2000
    },
    "stress_testing": {
      "stress_test_duration_ms": 30000,
      "stress_load_level": 80,
      "memory_stress_iterations": 1000
    }
  },
  "timeout_configurations": {
    "short_timeout": 10.0,
    "medium_timeout": 30.0,
    "long_timeout": 120.0
  }
}
```

## Test Results and Reporting

### Output Formats

The framework generates multiple report formats:

1. **JSON Reports**: Detailed test results with all execution data
2. **JUnit XML**: Standard format for CI/CD integration
3. **HTML Reports**: Human-readable summary with charts and graphs
4. **Console Output**: Real-time progress and summary

### Result Analysis

The framework provides comprehensive analysis:

- **Performance Analysis**: Execution times, throughput metrics, resource usage
- **Failure Analysis**: Common failure patterns, device-specific issues, recommendations
- **Trend Analysis**: Performance regression detection, baseline comparisons

### Example Output

```
=== Hardware Validation Suite Results ===
Duration: 4.23 seconds
Total tests: 9
Passed: 9
Failed: 0
Success rate: 100.0%

Device TEST001 Results:
  ✓ system_startup_check: completed (0.45s)
  ✓ pemf_timing_accuracy: completed (1.23s)
  ✓ pemf_pulse_consistency: completed (2.01s)
  ✓ battery_adc_calibration: completed (0.34s)
  ✓ battery_voltage_range: completed (0.67s)
  ✓ led_all_patterns: completed (0.89s)
  ✓ led_timing_accuracy: completed (0.45s)
  ✓ usb_communication_stress: completed (1.12s)
  ✓ system_integration_test: completed (0.78s)
```

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: Hardware Validation
on: [push, pull_request]

jobs:
  test:
    runs-on: self-hosted
    steps:
      - uses: actions/checkout@v2
      - name: Run Hardware Validation
        run: |
          python -m test_framework.comprehensive_test_runner \
            --full-suite \
            --output-dir test_results \
            --log-level INFO
      - name: Upload Test Results
        uses: actions/upload-artifact@v2
        with:
          name: test-results
          path: test_results/
```

### Jenkins Pipeline Example

```groovy
pipeline {
    agent any
    stages {
        stage('Hardware Tests') {
            steps {
                sh '''
                    python -m test_framework.comprehensive_test_runner \
                        --full-suite \
                        --output-dir ${WORKSPACE}/test_results
                '''
            }
            post {
                always {
                    publishTestResults testResultsPattern: 'test_results/*.xml'
                    archiveArtifacts artifacts: 'test_results/*'
                }
            }
        }
    }
}
```

## Customization and Extension

### Adding Custom Test Scenarios

```python
from test_framework import TestScenarios, TestStep, TestType

# Create custom scenario
scenarios = TestScenarios()
custom_steps = [
    TestStep(
        name="custom_validation",
        test_type=TestType.USB_COMMUNICATION_TEST,
        parameters={"message_count": 10},
        timeout=15.0
    )
]

config = scenarios.create_custom_scenario(
    "Custom Validation",
    "Custom test scenario description",
    custom_steps
)
```

### Extending Scenario Parameters

```python
from test_framework import ScenarioParameters

# Custom parameters
params = ScenarioParameters(
    pemf_test_duration_ms=10000,
    stress_test_duration_ms=60000,
    benchmark_iterations=200,
    # Add custom parameters as needed
)

scenarios = TestScenarios(params)
```

## Troubleshooting

### Common Issues

1. **Device Not Found**: Ensure device is connected and in normal operation mode
2. **Test Timeouts**: Increase timeout values in configuration
3. **Communication Errors**: Check USB connection and device firmware
4. **Memory Issues**: Reduce stress test parameters or iterations

### Debug Mode

Enable detailed logging for troubleshooting:

```bash
python -m test_framework.comprehensive_test_runner \
    --scenario hardware_validation \
    --log-level DEBUG \
    --log-file debug.log
```

### Performance Optimization

For faster test execution:

1. Reduce test durations in configuration
2. Skip non-critical scenarios
3. Use parallel execution where supported
4. Optimize device communication parameters

## Requirements Mapping

This implementation satisfies the following requirements:

- **9.1**: Hardware validation tests (pEMF, battery, LED)
- **9.2**: Stress testing scenarios with configurable parameters
- **9.3**: Regression test suite for firmware validation
- **9.4**: Performance benchmarking tests
- **9.5**: Integration tests for complete test scenario execution

## Files Created

- `test_scenarios.py`: Core test scenario definitions
- `comprehensive_test_runner.py`: Main test runner implementation
- `config_loader.py`: Configuration management
- `scenario_config.json`: Default configuration file
- `comprehensive_test_example.py`: Usage examples
- `tests/test_comprehensive_scenarios.py`: Integration tests
- `COMPREHENSIVE_TESTING_README.md`: This documentation