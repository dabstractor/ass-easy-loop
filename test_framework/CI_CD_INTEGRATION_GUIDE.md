# CI/CD Integration Guide

This guide provides comprehensive instructions for integrating the automated testing framework with various CI/CD systems, enabling headless operation, parallel testing, and standardized reporting.

## Overview

The CI/CD integration module provides:

- **Headless Operation**: Fully automated testing without user interaction
- **Proper Exit Codes**: Standard exit codes for CI system integration
- **Parallel Testing**: Support for testing multiple devices simultaneously
- **Standard Formats**: JUnit XML, JSON, TAP, and other CI-compatible report formats
- **Automated Setup**: Device discovery, connection, and cleanup
- **Environment Detection**: Automatic CI system detection and configuration
- **Comprehensive Reporting**: Multi-format reports with artifacts and trends

## Quick Start

### Basic Usage

```bash
# Run with default configuration
python -m test_framework.ci_integration

# Run with custom configuration
python -m test_framework.ci_integration --config ci_config.json

# Run with firmware flashing
python -m test_framework.ci_integration --firmware firmware.uf2 --devices 2

# Verbose mode for debugging
python -m test_framework.ci_integration --verbose --output-dir ci_results
```

### Exit Codes

The CI integration uses standard exit codes:

- `0`: Success - All tests passed
- `1`: Test failures - Some tests failed
- `2`: Device setup failure - Could not connect to required devices
- `3`: Firmware flash failure - Firmware flashing failed
- `4`: Unexpected error - System error or exception

## Configuration

### Configuration File Format

Create a JSON configuration file to customize test execution:

```json
{
  "test_config": {
    "name": "CI Validation Suite",
    "description": "Automated validation for CI/CD pipeline",
    "steps": [
      {
        "name": "device_communication_test",
        "test_type": "USB_COMMUNICATION_TEST",
        "parameters": {"message_count": 5, "timeout_ms": 1000},
        "timeout": 10.0,
        "required": true
      }
    ],
    "parallel_execution": true,
    "global_timeout": 180.0
  },
  "required_devices": 1,
  "max_parallel_devices": 4,
  "firmware_path": null,
  "timeout_seconds": 300.0,
  "retry_attempts": 1,
  "fail_fast": true,
  "generate_artifacts": true,
  "artifact_retention_days": 30
}
```

### Environment Variables

The system automatically detects CI environments and reads relevant variables:

#### GitHub Actions
- `GITHUB_ACTIONS`: Detected as GitHub Actions
- `GITHUB_RUN_NUMBER`: Build number
- `GITHUB_REF_NAME`: Branch name
- `GITHUB_SHA`: Commit hash

#### Jenkins
- `JENKINS_URL`: Detected as Jenkins
- `BUILD_NUMBER`: Build number
- `GIT_BRANCH`: Branch name
- `GIT_COMMIT`: Commit hash

#### GitLab CI
- `GITLAB_CI`: Detected as GitLab CI
- `CI_PIPELINE_ID`: Pipeline ID
- `CI_COMMIT_REF_NAME`: Branch name
- `CI_COMMIT_SHA`: Commit hash

## CI System Integration

### GitHub Actions

Create `.github/workflows/hardware-testing.yml`:

```yaml
name: Hardware Testing

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  hardware-tests:
    runs-on: self-hosted
    labels: [hardware-testing]
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: '3.9'
    
    - name: Install dependencies
      run: |
        pip install -r test_framework/requirements.txt
    
    - name: Run hardware tests
      run: |
        python -m test_framework.ci_integration \
          --config .github/ci_config.json \
          --devices 2 \
          --parallel 2 \
          --output-dir test_results \
          --verbose
    
    - name: Upload test results
      if: always()
      uses: actions/upload-artifact@v3
      with:
        name: test-results
        path: test_results/
    
    - name: Publish test results
      if: always()
      uses: dorny/test-reporter@v1
      with:
        name: Hardware Tests
        path: test_results/*.xml
        reporter: java-junit
```

### Jenkins

Create a `Jenkinsfile`:

```groovy
pipeline {
    agent { label 'hardware-testing' }
    
    stages {
        stage('Hardware Tests') {
            steps {
                sh '''
                    python -m test_framework.ci_integration \
                        --config jenkins/ci_config.json \
                        --devices 2 \
                        --parallel 2 \
                        --output-dir test_results \
                        --verbose
                '''
            }
            post {
                always {
                    archiveArtifacts artifacts: 'test_results/**/*'
                    publishTestResults testResultsPattern: 'test_results/*.xml'
                }
            }
        }
    }
}
```

### GitLab CI

Create `.gitlab-ci.yml`:

```yaml
hardware_tests:
  stage: test
  tags:
    - hardware-testing
  script:
    - python -m test_framework.ci_integration
        --config .gitlab/ci_config.json
        --devices 2
        --parallel 2
        --output-dir test_results
        --verbose
  artifacts:
    when: always
    reports:
      junit: test_results/*.xml
    paths:
      - test_results/
    expire_in: 30 days
```

## Parallel Testing

### Device Management

The system automatically discovers and manages multiple devices:

```python
# Discover devices
devices, success = ci.discover_and_setup_devices(required_count=4)

# Run tests in parallel
suite_result, success = ci.run_parallel_tests(config, devices)
```

### Configuration for Parallel Testing

```json
{
  "test_config": {
    "parallel_execution": true,
    "max_parallel_devices": 4
  },
  "required_devices": 4,
  "max_parallel_devices": 4
}
```

### Parallel Execution Strategies

1. **Device-Level Parallelism**: Run tests on multiple devices simultaneously
2. **Test-Level Parallelism**: Run different tests in parallel on the same device
3. **Mixed Parallelism**: Combine both strategies for maximum throughput

## Report Formats

### Supported Formats

1. **JUnit XML**: Standard format for CI systems
2. **JSON**: Machine-readable detailed results
3. **HTML**: Human-readable comprehensive reports
4. **CSV**: Data analysis and spreadsheet import
5. **TAP**: Test Anything Protocol for Jenkins/Azure DevOps

### Report Generation

```python
# Generate multiple report formats
report_files = ci.generate_ci_reports(suite_result, config)

# Available formats: json, junit, html, csv, tap
formats = ['json', 'junit', 'html']
report_files = report_generator.generate_comprehensive_report(
    suite_result, formats
)
```

### Report Structure

#### JUnit XML
```xml
<testsuites name="CI Test Suite" tests="5" failures="1">
  <testsuite name="Device_001" tests="5" failures="1">
    <testcase classname="Device_001" name="communication_test" time="2.5"/>
    <testcase classname="Device_001" name="timing_test" time="3.1">
      <failure message="Timing tolerance exceeded">...</failure>
    </testcase>
  </testsuite>
</testsuites>
```

#### JSON Report
```json
{
  "metadata": {
    "report_version": "1.0",
    "generated_at": 1640995200,
    "suite_name": "CI Test Suite"
  },
  "summary": {
    "total_tests": 5,
    "passed": 4,
    "failed": 1,
    "success_rate": 80.0,
    "duration": 45.2
  },
  "device_results": {
    "TEST_DEVICE_001": {
      "tests": [...],
      "metrics": {...}
    }
  }
}
```

## Hardware Setup Requirements

### Self-Hosted Runners

For CI systems requiring hardware access, use self-hosted runners:

#### GitHub Actions Self-Hosted Runner
```bash
# Download and configure runner
./config.sh --url https://github.com/owner/repo --token TOKEN --labels hardware-testing

# Install as service
sudo ./svc.sh install
sudo ./svc.sh start
```

#### Jenkins Agent Setup
```bash
# Install Java and dependencies
sudo apt-get update
sudo apt-get install openjdk-11-jdk python3 python3-pip

# Setup USB permissions
sudo usermod -a -G dialout jenkins
```

### USB Device Permissions

Create udev rules for test devices (`/etc/udev/rules.d/99-test-devices.rules`):

```
# Test devices
SUBSYSTEM=="usb", ATTR{idVendor}=="2e8a", ATTR{idProduct}=="0003", MODE="0666", GROUP="dialout"
SUBSYSTEM=="usb", ATTR{idVendor}=="2e8a", ATTR{idProduct}=="000a", MODE="0666", GROUP="dialout"
```

Reload udev rules:
```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
```

## Troubleshooting

### Common Issues

#### Device Discovery Failures
```bash
# Check USB devices
lsusb

# Check permissions
ls -la /dev/ttyACM*

# Test device manager
python -m test_framework.device_manager --list-devices
```

#### Permission Issues
```bash
# Add user to dialout group
sudo usermod -a -G dialout $USER

# Check group membership
groups $USER
```

#### Timeout Issues
```bash
# Increase timeout
python -m test_framework.ci_integration --timeout 600

# Check device responsiveness
python -m test_framework.device_manager --ping-devices
```

### Debug Mode

Enable verbose logging for troubleshooting:

```bash
python -m test_framework.ci_integration \
  --verbose \
  --output-dir debug_results \
  --config debug_config.json
```

### Log Analysis

Check generated logs:
```bash
# CI logs
cat ci_test_results/logs/ci_test_*.log

# Device communication logs
cat ci_test_results/artifacts/device_communication_*.log

# Test execution logs
cat ci_test_results/artifacts/test_execution_*.log
```

## Performance Optimization

### Parallel Execution Tuning

```json
{
  "max_parallel_devices": 4,
  "test_config": {
    "parallel_execution": true,
    "max_parallel_devices": 4
  }
}
```

### Resource Management

- **CPU**: Limit parallel operations based on available cores
- **Memory**: Monitor memory usage during parallel testing
- **USB Bandwidth**: Consider USB hub limitations
- **Device Thermal**: Allow cooling time between intensive tests

### Caching Strategies

```yaml
# GitHub Actions caching
- name: Cache test dependencies
  uses: actions/cache@v3
  with:
    path: ~/.cache/pip
    key: ${{ runner.os }}-pip-${{ hashFiles('**/requirements.txt') }}
```

## Security Considerations

### Access Control

- Use dedicated CI service accounts
- Limit hardware access to authorized runners
- Implement device authentication where possible

### Artifact Security

- Sanitize logs before archiving
- Encrypt sensitive test data
- Implement artifact retention policies

### Network Security

- Isolate test networks
- Use VPNs for remote hardware access
- Monitor network traffic during tests

## Best Practices

### Test Design

1. **Idempotent Tests**: Tests should not depend on previous test state
2. **Isolated Tests**: Each test should be independent
3. **Deterministic Results**: Avoid flaky tests with proper timeouts
4. **Resource Cleanup**: Always clean up after tests

### CI Pipeline Design

1. **Fast Feedback**: Run quick tests first
2. **Parallel Execution**: Maximize hardware utilization
3. **Artifact Management**: Archive important results
4. **Notification Strategy**: Alert on failures and regressions

### Monitoring and Alerting

1. **Test Metrics**: Track success rates and execution times
2. **Hardware Health**: Monitor device status and availability
3. **Performance Trends**: Detect regressions early
4. **Capacity Planning**: Monitor resource usage

## Advanced Features

### Custom Test Scenarios

```python
# Create custom CI test configuration
from test_framework.ci_integration import CIIntegration

ci = CIIntegration()
config = ci.load_ci_configuration("custom_config.json")

# Run with custom parameters
result = ci.run_ci_pipeline("custom_config.json")
```

### Integration with External Systems

```python
# Custom notification integration
def send_custom_notification(result):
    if not result.success:
        # Send to Slack, Teams, etc.
        send_slack_message(f"Tests failed: {result.error_summary}")

# Performance trend analysis
def analyze_performance_trends(results):
    # Custom performance analysis
    trends = analyze_timing_data(results)
    if trends.regression_detected:
        create_jira_ticket("Performance regression detected")
```

### Multi-Environment Testing

```yaml
# Test matrix for different environments
strategy:
  matrix:
    environment: [development, staging, production]
    device_count: [1, 2, 4]
    test_suite: [smoke, regression, performance]
```

## API Reference

### CIIntegration Class

```python
class CIIntegration:
    def __init__(self, output_dir: str = "ci_test_results", verbose: bool = False)
    def detect_ci_environment(self) -> CIEnvironmentInfo
    def load_ci_configuration(self, config_path: Optional[str] = None) -> CITestConfiguration
    def discover_and_setup_devices(self, required_count: int) -> Tuple[List[str], bool]
    def flash_firmware_parallel(self, devices: List[str], firmware_path: str) -> Tuple[Dict, bool]
    def run_parallel_tests(self, config: CITestConfiguration, devices: List[str]) -> Tuple[TestSuiteResult, bool]
    def generate_ci_reports(self, suite_result: TestSuiteResult, config: CITestConfiguration) -> List[str]
    def run_ci_pipeline(self, config_path: Optional[str] = None) -> CITestResult
```

### Command Line Interface

```bash
python -m test_framework.ci_integration [OPTIONS]

Options:
  --config, -c TEXT       CI configuration file (JSON)
  --firmware, -f TEXT     Firmware file to flash before testing
  --devices, -d INTEGER   Minimum number of devices required
  --parallel, -p INTEGER  Maximum parallel operations
  --timeout, -t FLOAT     Global timeout in seconds
  --output-dir, -o TEXT   Output directory for results
  --verbose, -v           Enable verbose logging
  --fail-fast             Stop on first failure
  --no-artifacts          Skip artifact generation
  --help                  Show help message
```

## Support and Contributing

### Getting Help

1. Check the troubleshooting section
2. Review log files for error details
3. Test with verbose mode enabled
4. Verify hardware setup and permissions

### Contributing

1. Follow the existing code style
2. Add tests for new features
3. Update documentation
4. Test with multiple CI systems

### Reporting Issues

Include the following information:
- CI system and version
- Hardware configuration
- Error logs and stack traces
- Configuration files used
- Steps to reproduce the issue