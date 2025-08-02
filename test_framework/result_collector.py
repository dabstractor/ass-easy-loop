"""
Test Result Collection and Analysis

Collects, analyzes, and formats test results from device executions.
"""

import time
import json
import os
import csv
from typing import List, Dict, Any, Optional, Tuple
from dataclasses import dataclass, asdict
from enum import Enum
import statistics
from pathlib import Path
import hashlib
from datetime import datetime, timedelta

from .test_sequencer import TestExecution, TestStatus
from .command_handler import TestResponse, ResponseStatus


class TestResultSeverity(Enum):
    """Test result severity levels"""
    PASS = "pass"
    FAIL = "fail"
    WARNING = "warning"
    INFO = "info"


@dataclass
class TestMetrics:
    """Test execution metrics"""
    total_tests: int
    passed_tests: int
    failed_tests: int
    skipped_tests: int
    timeout_tests: int
    total_duration: float
    average_duration: float
    success_rate: float
    
    
@dataclass
class DeviceTestResult:
    """Test results for a single device"""
    device_serial: str
    executions: List[TestExecution]
    metrics: TestMetrics
    start_time: float
    end_time: float
    overall_status: TestStatus
    
    
@dataclass
class TestArtifact:
    """Test artifact (logs, timing data, error reports)"""
    name: str
    type: str  # 'log', 'timing', 'error', 'performance', 'screenshot'
    content: Any
    timestamp: float
    size_bytes: int
    metadata: Dict[str, Any]


@dataclass
class PerformanceTrend:
    """Performance trend data for regression analysis"""
    metric_name: str
    historical_values: List[float]
    current_value: float
    trend_direction: str  # 'improving', 'degrading', 'stable'
    regression_detected: bool
    confidence_level: float


@dataclass
class TestSuiteResult:
    """Complete test suite results"""
    suite_name: str
    description: str
    device_results: Dict[str, DeviceTestResult]
    aggregate_metrics: TestMetrics
    start_time: float
    end_time: float
    duration: float
    artifacts: List[TestArtifact]
    performance_trends: List[PerformanceTrend]
    environment_info: Dict[str, Any]


class ResultCollector:
    """
    Collects and analyzes test execution results.
    
    Provides comprehensive result analysis, metrics calculation,
    various output formats for test reporting, performance trend analysis,
    and test artifact collection.
    """
    
    def __init__(self, artifacts_dir: str = "test_artifacts"):
        """
        Initialize the result collector
        
        Args:
            artifacts_dir: Directory to store test artifacts
        """
        self.collected_results: List[TestSuiteResult] = []
        self.artifacts_dir = Path(artifacts_dir)
        self.artifacts_dir.mkdir(exist_ok=True)
        self.performance_history: Dict[str, List[Tuple[float, float]]] = {}  # metric_name -> [(timestamp, value)]
        
    def collect_results(self, suite_name: str, description: str,
                       execution_results: Dict[str, List[TestExecution]],
                       start_time: float, end_time: float,
                       environment_info: Optional[Dict[str, Any]] = None) -> TestSuiteResult:
        """
        Collect and analyze test execution results.
        
        Args:
            suite_name: Name of the test suite
            description: Test suite description
            execution_results: Device execution results
            start_time: Suite start time
            end_time: Suite end time
            
        Returns:
            Analyzed test suite results
        """
        device_results = {}
        all_executions = []
        
        # Process results for each device
        for device_serial, executions in execution_results.items():
            device_metrics = self._calculate_device_metrics(executions)
            device_start = min((e.start_time for e in executions if e.start_time), default=start_time)
            device_end = max((e.end_time for e in executions if e.end_time), default=end_time)
            
            # Determine overall device status
            overall_status = self._determine_overall_status(executions)
            
            device_result = DeviceTestResult(
                device_serial=device_serial,
                executions=executions,
                metrics=device_metrics,
                start_time=device_start,
                end_time=device_end,
                overall_status=overall_status
            )
            
            device_results[device_serial] = device_result
            all_executions.extend(executions)
        
        # Calculate aggregate metrics
        aggregate_metrics = self._calculate_device_metrics(all_executions)
        
        # Collect test artifacts
        artifacts = self._collect_test_artifacts(all_executions, suite_name)
        
        # Analyze performance trends
        performance_trends = self._analyze_performance_trends(all_executions)
        
        # Collect environment information
        env_info = environment_info or self._collect_environment_info()
        
        suite_result = TestSuiteResult(
            suite_name=suite_name,
            description=description,
            device_results=device_results,
            aggregate_metrics=aggregate_metrics,
            start_time=start_time,
            end_time=end_time,
            duration=end_time - start_time,
            artifacts=artifacts,
            performance_trends=performance_trends,
            environment_info=env_info
        )
        
        self.collected_results.append(suite_result)
        return suite_result
    
    def _calculate_device_metrics(self, executions: List[TestExecution]) -> TestMetrics:
        """Calculate metrics for a list of test executions"""
        if not executions:
            return TestMetrics(0, 0, 0, 0, 0, 0.0, 0.0, 0.0)
        
        total_tests = len(executions)
        passed_tests = sum(1 for e in executions if e.status == TestStatus.COMPLETED)
        failed_tests = sum(1 for e in executions if e.status == TestStatus.FAILED)
        skipped_tests = sum(1 for e in executions if e.status == TestStatus.SKIPPED)
        timeout_tests = sum(1 for e in executions if e.status == TestStatus.TIMEOUT)
        
        # Calculate duration metrics
        durations = [e.duration for e in executions if e.duration is not None]
        total_duration = sum(durations) if durations else 0.0
        average_duration = statistics.mean(durations) if durations else 0.0
        
        # Calculate success rate
        success_rate = (passed_tests / total_tests * 100) if total_tests > 0 else 0.0
        
        return TestMetrics(
            total_tests=total_tests,
            passed_tests=passed_tests,
            failed_tests=failed_tests,
            skipped_tests=skipped_tests,
            timeout_tests=timeout_tests,
            total_duration=total_duration,
            average_duration=average_duration,
            success_rate=success_rate
        )
    
    def _determine_overall_status(self, executions: List[TestExecution]) -> TestStatus:
        """Determine overall status from execution list"""
        if not executions:
            return TestStatus.SKIPPED
            
        # If any required test failed, overall status is failed
        for execution in executions:
            if execution.status == TestStatus.FAILED and execution.step.required:
                return TestStatus.FAILED
                
        # If all completed tests passed, overall status is completed
        completed_count = sum(1 for e in executions if e.status == TestStatus.COMPLETED)
        total_required = sum(1 for e in executions if e.step.required)
        
        if completed_count >= total_required:
            return TestStatus.COMPLETED
        else:
            return TestStatus.FAILED
    
    def generate_summary_report(self, suite_result: TestSuiteResult) -> Dict[str, Any]:
        """Generate a summary report for a test suite"""
        return {
            "suite_name": suite_result.suite_name,
            "description": suite_result.description,
            "start_time": suite_result.start_time,
            "end_time": suite_result.end_time,
            "duration": suite_result.duration,
            "device_count": len(suite_result.device_results),
            "aggregate_metrics": asdict(suite_result.aggregate_metrics),
            "device_summaries": {
                serial: {
                    "overall_status": result.overall_status.value,
                    "metrics": asdict(result.metrics),
                    "duration": result.end_time - result.start_time
                }
                for serial, result in suite_result.device_results.items()
            }
        }
    
    def generate_detailed_report(self, suite_result: TestSuiteResult) -> Dict[str, Any]:
        """Generate a detailed report including all execution details"""
        detailed_report = self.generate_summary_report(suite_result)
        
        # Add detailed execution information
        detailed_report["device_details"] = {}
        
        for serial, device_result in suite_result.device_results.items():
            device_details = {
                "device_serial": serial,
                "overall_status": device_result.overall_status.value,
                "start_time": device_result.start_time,
                "end_time": device_result.end_time,
                "metrics": asdict(device_result.metrics),
                "test_executions": []
            }
            
            for execution in device_result.executions:
                execution_detail = {
                    "test_name": execution.step.name,
                    "test_type": execution.step.test_type.name,
                    "status": execution.status.value,
                    "start_time": execution.start_time,
                    "end_time": execution.end_time,
                    "duration": execution.duration,
                    "retry_attempt": execution.retry_attempt,
                    "parameters": execution.step.parameters,
                    "error_message": execution.error_message
                }
                
                # Add response data if available
                if execution.response:
                    execution_detail["response"] = {
                        "status": execution.response.status.name,
                        "data": execution.response.data,
                        "timestamp": execution.response.timestamp
                    }
                
                device_details["test_executions"].append(execution_detail)
            
            detailed_report["device_details"][serial] = device_details
        
        return detailed_report
    
    def export_junit_xml(self, suite_result: TestSuiteResult) -> str:
        """Export results in JUnit XML format for CI integration"""
        from xml.etree.ElementTree import Element, SubElement, tostring
        from xml.dom import minidom
        
        # Create root testsuite element
        testsuite = Element("testsuite")
        testsuite.set("name", suite_result.suite_name)
        testsuite.set("tests", str(suite_result.aggregate_metrics.total_tests))
        testsuite.set("failures", str(suite_result.aggregate_metrics.failed_tests))
        testsuite.set("skipped", str(suite_result.aggregate_metrics.skipped_tests))
        testsuite.set("time", f"{suite_result.duration:.3f}")
        testsuite.set("timestamp", time.strftime("%Y-%m-%dT%H:%M:%S", time.localtime(suite_result.start_time)))
        
        # Add test cases for each device and execution
        for device_serial, device_result in suite_result.device_results.items():
            for execution in device_result.executions:
                testcase = SubElement(testsuite, "testcase")
                testcase.set("classname", f"{suite_result.suite_name}.{device_serial}")
                testcase.set("name", execution.step.name)
                testcase.set("time", f"{execution.duration or 0:.3f}")
                
                if execution.status == TestStatus.FAILED:
                    failure = SubElement(testcase, "failure")
                    failure.set("message", execution.error_message or "Test failed")
                    failure.text = execution.error_message or "No error details available"
                elif execution.status == TestStatus.SKIPPED:
                    skipped = SubElement(testcase, "skipped")
                    skipped.set("message", "Test was skipped")
                elif execution.status == TestStatus.TIMEOUT:
                    failure = SubElement(testcase, "failure")
                    failure.set("message", "Test timeout")
                    failure.text = "Test execution timed out"
        
        # Format XML with proper indentation
        rough_string = tostring(testsuite, 'unicode')
        reparsed = minidom.parseString(rough_string)
        return reparsed.toprettyxml(indent="  ")
    
    def export_json(self, suite_result: TestSuiteResult, detailed: bool = True) -> str:
        """Export results in JSON format"""
        if detailed:
            report_data = self.generate_detailed_report(suite_result)
        else:
            report_data = self.generate_summary_report(suite_result)
        
        return json.dumps(report_data, indent=2, default=str)
    
    def get_failure_analysis(self, suite_result: TestSuiteResult) -> Dict[str, Any]:
        """Analyze failures and provide insights"""
        failure_analysis = {
            "total_failures": suite_result.aggregate_metrics.failed_tests,
            "failure_by_test": {},
            "failure_by_device": {},
            "common_failures": [],
            "recommendations": []
        }
        
        # Analyze failures by test type
        for device_result in suite_result.device_results.values():
            for execution in device_result.executions:
                if execution.status == TestStatus.FAILED:
                    test_name = execution.step.name
                    if test_name not in failure_analysis["failure_by_test"]:
                        failure_analysis["failure_by_test"][test_name] = 0
                    failure_analysis["failure_by_test"][test_name] += 1
                    
                    # Track device-specific failures
                    device_serial = execution.device_serial
                    if device_serial not in failure_analysis["failure_by_device"]:
                        failure_analysis["failure_by_device"][device_serial] = 0
                    failure_analysis["failure_by_device"][device_serial] += 1
        
        # Identify common failure patterns
        total_devices = len(suite_result.device_results)
        for test_name, failure_count in failure_analysis["failure_by_test"].items():
            if failure_count > total_devices * 0.5:  # More than 50% of devices failed
                failure_analysis["common_failures"].append({
                    "test_name": test_name,
                    "failure_rate": failure_count / total_devices * 100,
                    "affected_devices": failure_count
                })
        
        # Generate recommendations
        if failure_analysis["common_failures"]:
            failure_analysis["recommendations"].append(
                "Multiple devices failed the same tests - check for firmware or hardware issues"
            )
        
        if suite_result.aggregate_metrics.timeout_tests > 0:
            failure_analysis["recommendations"].append(
                "Some tests timed out - consider increasing timeout values or checking device responsiveness"
            )
        
        return failure_analysis
    
    def get_performance_analysis(self, suite_result: TestSuiteResult) -> Dict[str, Any]:
        """Analyze performance metrics across devices"""
        performance_data = {
            "execution_times": {},
            "device_performance": {},
            "slowest_tests": [],
            "performance_variance": {}
        }
        
        # Collect execution times by test
        test_times = {}
        for device_result in suite_result.device_results.values():
            for execution in device_result.executions:
                if execution.duration:
                    test_name = execution.step.name
                    if test_name not in test_times:
                        test_times[test_name] = []
                    test_times[test_name].append(execution.duration)
        
        # Calculate statistics for each test
        for test_name, times in test_times.items():
            if times:
                performance_data["execution_times"][test_name] = {
                    "min": min(times),
                    "max": max(times),
                    "mean": statistics.mean(times),
                    "median": statistics.median(times),
                    "std_dev": statistics.stdev(times) if len(times) > 1 else 0.0
                }
        
        # Identify slowest tests
        avg_times = [(test, stats["mean"]) for test, stats in performance_data["execution_times"].items()]
        performance_data["slowest_tests"] = sorted(avg_times, key=lambda x: x[1], reverse=True)[:5]
        
        return performance_data
    
    def _collect_test_artifacts(self, executions: List[TestExecution], suite_name: str) -> List[TestArtifact]:
        """Collect test artifacts from executions"""
        artifacts = []
        timestamp = time.time()
        
        # Collect timing data artifacts
        timing_data = []
        for execution in executions:
            if execution.duration:
                timing_data.append({
                    'test_name': execution.step.name,
                    'device_serial': getattr(execution, 'device_serial', 'unknown'),
                    'duration': execution.duration,
                    'start_time': execution.start_time,
                    'end_time': execution.end_time,
                    'status': execution.status.value
                })
        
        if timing_data:
            timing_artifact = TestArtifact(
                name=f"{suite_name}_timing_data",
                type="timing",
                content=timing_data,
                timestamp=timestamp,
                size_bytes=len(json.dumps(timing_data).encode()),
                metadata={'test_count': len(timing_data), 'suite_name': suite_name}
            )
            artifacts.append(timing_artifact)
        
        # Collect error reports
        error_data = []
        for execution in executions:
            if execution.status == TestStatus.FAILED and execution.error_message:
                error_data.append({
                    'test_name': execution.step.name,
                    'device_serial': getattr(execution, 'device_serial', 'unknown'),
                    'error_message': execution.error_message,
                    'timestamp': execution.end_time or execution.start_time,
                    'retry_attempt': execution.retry_attempt,
                    'test_parameters': execution.step.parameters
                })
        
        if error_data:
            error_artifact = TestArtifact(
                name=f"{suite_name}_error_reports",
                type="error",
                content=error_data,
                timestamp=timestamp,
                size_bytes=len(json.dumps(error_data).encode()),
                metadata={'error_count': len(error_data), 'suite_name': suite_name}
            )
            artifacts.append(error_artifact)
        
        # Collect performance data artifacts
        performance_data = []
        for execution in executions:
            if execution.response and hasattr(execution.response, 'data'):
                if isinstance(execution.response.data, dict):
                    perf_metrics = execution.response.data.get('performance_metrics', {})
                    if perf_metrics:
                        performance_data.append({
                            'test_name': execution.step.name,
                            'device_serial': getattr(execution, 'device_serial', 'unknown'),
                            'metrics': perf_metrics,
                            'timestamp': execution.response.timestamp
                        })
        
        if performance_data:
            perf_artifact = TestArtifact(
                name=f"{suite_name}_performance_data",
                type="performance",
                content=performance_data,
                timestamp=timestamp,
                size_bytes=len(json.dumps(performance_data).encode()),
                metadata={'metric_count': len(performance_data), 'suite_name': suite_name}
            )
            artifacts.append(perf_artifact)
        
        return artifacts
    
    def _analyze_performance_trends(self, executions: List[TestExecution]) -> List[PerformanceTrend]:
        """Analyze performance trends for regression detection"""
        trends = []
        current_timestamp = time.time()
        
        # Extract performance metrics from executions
        current_metrics = {}
        for execution in executions:
            if execution.response and hasattr(execution.response, 'data'):
                if isinstance(execution.response.data, dict):
                    perf_metrics = execution.response.data.get('performance_metrics', {})
                    for metric_name, value in perf_metrics.items():
                        if isinstance(value, (int, float)):
                            if metric_name not in current_metrics:
                                current_metrics[metric_name] = []
                            current_metrics[metric_name].append(value)
        
        # Analyze trends for each metric
        for metric_name, values in current_metrics.items():
            if not values:
                continue
                
            current_value = statistics.mean(values)
            
            # Update performance history
            if metric_name not in self.performance_history:
                self.performance_history[metric_name] = []
            
            self.performance_history[metric_name].append((current_timestamp, current_value))
            
            # Keep only recent history (last 30 data points)
            self.performance_history[metric_name] = self.performance_history[metric_name][-30:]
            
            # Analyze trend if we have enough historical data
            historical_values = [v for _, v in self.performance_history[metric_name][:-1]]
            
            if len(historical_values) >= 3:
                trend_direction, regression_detected, confidence = self._detect_performance_regression(
                    historical_values, current_value
                )
                
                trend = PerformanceTrend(
                    metric_name=metric_name,
                    historical_values=historical_values,
                    current_value=current_value,
                    trend_direction=trend_direction,
                    regression_detected=regression_detected,
                    confidence_level=confidence
                )
                trends.append(trend)
        
        return trends
    
    def _detect_performance_regression(self, historical_values: List[float], 
                                     current_value: float) -> Tuple[str, bool, float]:
        """
        Detect performance regression using statistical analysis
        
        Returns:
            Tuple of (trend_direction, regression_detected, confidence_level)
        """
        if len(historical_values) < 3:
            return "unknown", False, 0.0
        
        # Calculate historical statistics
        hist_mean = statistics.mean(historical_values)
        hist_stdev = statistics.stdev(historical_values) if len(historical_values) > 1 else 0
        
        # Calculate trend direction
        recent_values = historical_values[-5:]  # Last 5 values
        if len(recent_values) >= 2:
            trend_slope = (recent_values[-1] - recent_values[0]) / len(recent_values)
            if abs(trend_slope) < hist_stdev * 0.1:
                trend_direction = "stable"
            elif trend_slope > 0:
                trend_direction = "degrading"  # Assuming higher values are worse
            else:
                trend_direction = "improving"
        else:
            trend_direction = "stable"
        
        # Detect regression using z-score
        if hist_stdev > 0:
            z_score = abs(current_value - hist_mean) / hist_stdev
            regression_detected = z_score > 2.0  # 2 standard deviations
            confidence_level = min(z_score / 3.0, 1.0)  # Normalize to 0-1
        else:
            # If no variation in historical data, check for significant change
            change_percent = abs(current_value - hist_mean) / hist_mean if hist_mean != 0 else 0
            regression_detected = change_percent > 0.2  # 20% change
            confidence_level = min(change_percent * 2, 1.0)
        
        return trend_direction, regression_detected, confidence_level
    
    def _collect_environment_info(self) -> Dict[str, Any]:
        """Collect environment information for test context"""
        import platform
        import sys
        
        return {
            'timestamp': time.time(),
            'platform': platform.platform(),
            'python_version': sys.version,
            'architecture': platform.architecture(),
            'processor': platform.processor(),
            'hostname': platform.node(),
            'user': os.environ.get('USER', 'unknown'),
            'working_directory': os.getcwd()
        }
    
    def export_html_report(self, suite_result: TestSuiteResult, 
                          output_path: Optional[str] = None) -> str:
        """Export comprehensive HTML report with charts and analysis"""
        if output_path is None:
            timestamp = time.strftime("%Y%m%d_%H%M%S")
            output_path = f"{suite_result.suite_name}_{timestamp}.html"
        
        html_content = self._generate_html_report_content(suite_result)
        
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(html_content)
        
        return output_path
    
    def _generate_html_report_content(self, suite_result: TestSuiteResult) -> str:
        """Generate comprehensive HTML report content"""
        # Calculate summary statistics
        total_devices = len(suite_result.device_results)
        passed_devices = sum(1 for r in suite_result.device_results.values() 
                           if r.overall_status == TestStatus.COMPLETED)
        
        # Generate performance trend charts data
        trend_charts_data = self._generate_trend_charts_data(suite_result.performance_trends)
        
        # Generate device comparison data
        device_comparison_data = self._generate_device_comparison_data(suite_result.device_results)
        
        html_template = f"""
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{suite_result.suite_name} - Test Report</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background-color: white;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            overflow: hidden;
        }}
        .header {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 30px;
            text-align: center;
        }}
        .header h1 {{
            margin: 0;
            font-size: 2.5em;
            font-weight: 300;
        }}
        .header p {{
            margin: 10px 0 0 0;
            opacity: 0.9;
        }}
        .summary {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            padding: 30px;
            background-color: #f8f9fa;
        }}
        .summary-card {{
            background: white;
            padding: 20px;
            border-radius: 8px;
            text-align: center;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .summary-card h3 {{
            margin: 0 0 10px 0;
            color: #495057;
            font-size: 0.9em;
            text-transform: uppercase;
            letter-spacing: 1px;
        }}
        .summary-card .value {{
            font-size: 2em;
            font-weight: bold;
            margin: 0;
        }}
        .success {{ color: #28a745; }}
        .failure {{ color: #dc3545; }}
        .warning {{ color: #ffc107; }}
        .info {{ color: #17a2b8; }}
        .section {{
            padding: 30px;
            border-bottom: 1px solid #e9ecef;
        }}
        .section h2 {{
            margin: 0 0 20px 0;
            color: #495057;
            border-bottom: 2px solid #667eea;
            padding-bottom: 10px;
        }}
        .chart-container {{
            position: relative;
            height: 400px;
            margin: 20px 0;
        }}
        .device-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
            gap: 20px;
            margin-top: 20px;
        }}
        .device-card {{
            border: 1px solid #dee2e6;
            border-radius: 8px;
            padding: 20px;
            background: white;
        }}
        .device-card h3 {{
            margin: 0 0 15px 0;
            color: #495057;
        }}
        .test-list {{
            list-style: none;
            padding: 0;
            margin: 0;
        }}
        .test-item {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 8px 0;
            border-bottom: 1px solid #f1f3f4;
        }}
        .test-item:last-child {{
            border-bottom: none;
        }}
        .status-badge {{
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 0.8em;
            font-weight: bold;
            text-transform: uppercase;
        }}
        .status-completed {{
            background-color: #d4edda;
            color: #155724;
        }}
        .status-failed {{
            background-color: #f8d7da;
            color: #721c24;
        }}
        .status-timeout {{
            background-color: #fff3cd;
            color: #856404;
        }}
        .artifacts-section {{
            background-color: #f8f9fa;
        }}
        .artifact-item {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 10px;
            margin: 5px 0;
            background: white;
            border-radius: 4px;
            border-left: 4px solid #667eea;
        }}
        .trends-section {{
            background-color: #fff;
        }}
        .trend-item {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 15px;
            margin: 10px 0;
            border: 1px solid #dee2e6;
            border-radius: 8px;
        }}
        .trend-improving {{ border-left: 4px solid #28a745; }}
        .trend-degrading {{ border-left: 4px solid #dc3545; }}
        .trend-stable {{ border-left: 4px solid #6c757d; }}
        .regression-alert {{
            background-color: #f8d7da;
            border: 1px solid #f5c6cb;
            color: #721c24;
            padding: 15px;
            border-radius: 8px;
            margin: 10px 0;
        }}
        .footer {{
            text-align: center;
            padding: 20px;
            color: #6c757d;
            font-size: 0.9em;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>{suite_result.suite_name}</h1>
            <p>{suite_result.description}</p>
            <p>Generated on {datetime.fromtimestamp(suite_result.end_time).strftime('%Y-%m-%d %H:%M:%S')}</p>
        </div>
        
        <div class="summary">
            <div class="summary-card">
                <h3>Total Tests</h3>
                <p class="value info">{suite_result.aggregate_metrics.total_tests}</p>
            </div>
            <div class="summary-card">
                <h3>Passed</h3>
                <p class="value success">{suite_result.aggregate_metrics.passed_tests}</p>
            </div>
            <div class="summary-card">
                <h3>Failed</h3>
                <p class="value failure">{suite_result.aggregate_metrics.failed_tests}</p>
            </div>
            <div class="summary-card">
                <h3>Success Rate</h3>
                <p class="value {'success' if suite_result.aggregate_metrics.success_rate >= 90 else 'warning' if suite_result.aggregate_metrics.success_rate >= 70 else 'failure'}">{suite_result.aggregate_metrics.success_rate:.1f}%</p>
            </div>
            <div class="summary-card">
                <h3>Duration</h3>
                <p class="value info">{suite_result.duration:.1f}s</p>
            </div>
            <div class="summary-card">
                <h3>Devices</h3>
                <p class="value info">{passed_devices}/{total_devices}</p>
            </div>
        </div>
        
        <div class="section">
            <h2>Performance Trends</h2>
            {self._generate_performance_trends_html(suite_result.performance_trends)}
        </div>
        
        <div class="section">
            <h2>Test Execution Timeline</h2>
            <div class="chart-container">
                <canvas id="timelineChart"></canvas>
            </div>
        </div>
        
        <div class="section">
            <h2>Device Results</h2>
            <div class="device-grid">
                {self._generate_device_results_html(suite_result.device_results)}
            </div>
        </div>
        
        <div class="section artifacts-section">
            <h2>Test Artifacts</h2>
            {self._generate_artifacts_html(suite_result.artifacts)}
        </div>
        
        <div class="section">
            <h2>Environment Information</h2>
            {self._generate_environment_html(suite_result.environment_info)}
        </div>
        
        <div class="footer">
            <p>Report generated by Automated Testing Framework</p>
        </div>
    </div>
    
    <script>
        {self._generate_chart_scripts(suite_result)}
    </script>
</body>
</html>
"""
        return html_template
    
    def _generate_performance_trends_html(self, trends: List[PerformanceTrend]) -> str:
        """Generate HTML for performance trends section"""
        if not trends:
            return "<p>No performance trend data available.</p>"
        
        html = ""
        regressions_detected = [t for t in trends if t.regression_detected]
        
        if regressions_detected:
            html += '<div class="regression-alert">'
            html += f'<strong>⚠️ Performance Regression Detected!</strong><br>'
            html += f'Found {len(regressions_detected)} metric(s) showing potential regression.'
            html += '</div>'
        
        for trend in trends:
            trend_class = f"trend-{trend.trend_direction}"
            confidence_percent = trend.confidence_level * 100
            
            html += f'''
            <div class="trend-item {trend_class}">
                <div>
                    <strong>{trend.metric_name}</strong><br>
                    <small>Current: {trend.current_value:.3f} | Trend: {trend.trend_direction.title()}</small>
                </div>
                <div>
                    <span class="status-badge {'status-failed' if trend.regression_detected else 'status-completed'}">
                        {'REGRESSION' if trend.regression_detected else 'OK'}
                    </span><br>
                    <small>Confidence: {confidence_percent:.1f}%</small>
                </div>
            </div>
            '''
        
        return html
    
    def _generate_device_results_html(self, device_results: Dict[str, DeviceTestResult]) -> str:
        """Generate HTML for device results section"""
        html = ""
        
        for serial, result in device_results.items():
            status_class = "success" if result.overall_status == TestStatus.COMPLETED else "failure"
            
            html += f'''
            <div class="device-card">
                <h3>{serial}</h3>
                <p><strong>Status:</strong> <span class="{status_class}">{result.overall_status.value.upper()}</span></p>
                <p><strong>Tests:</strong> {result.metrics.passed_tests}/{result.metrics.total_tests} passed</p>
                <p><strong>Duration:</strong> {result.end_time - result.start_time:.1f}s</p>
                
                <ul class="test-list">
            '''
            
            for execution in result.executions:
                status_class = f"status-{execution.status.value.replace('_', '-')}"
                duration_text = f"{execution.duration:.2f}s" if execution.duration else "N/A"
                
                html += f'''
                <li class="test-item">
                    <span>{execution.step.name}</span>
                    <div>
                        <span class="status-badge {status_class}">{execution.status.value}</span>
                        <small>{duration_text}</small>
                    </div>
                </li>
                '''
            
            html += '''
                </ul>
            </div>
            '''
        
        return html
    
    def _generate_artifacts_html(self, artifacts: List[TestArtifact]) -> str:
        """Generate HTML for artifacts section"""
        if not artifacts:
            return "<p>No test artifacts collected.</p>"
        
        html = ""
        for artifact in artifacts:
            size_kb = artifact.size_bytes / 1024
            
            html += f'''
            <div class="artifact-item">
                <div>
                    <strong>{artifact.name}</strong><br>
                    <small>Type: {artifact.type.title()} | Size: {size_kb:.1f} KB</small>
                </div>
                <div>
                    <small>{datetime.fromtimestamp(artifact.timestamp).strftime('%H:%M:%S')}</small>
                </div>
            </div>
            '''
        
        return html
    
    def _generate_environment_html(self, env_info: Dict[str, Any]) -> str:
        """Generate HTML for environment information"""
        html = "<div style='display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 15px;'>"
        
        for key, value in env_info.items():
            if key == 'timestamp':
                value = datetime.fromtimestamp(value).strftime('%Y-%m-%d %H:%M:%S')
            
            html += f'''
            <div style="padding: 10px; background: white; border-radius: 4px; border-left: 3px solid #667eea;">
                <strong>{key.replace('_', ' ').title()}:</strong><br>
                <small>{value}</small>
            </div>
            '''
        
        html += "</div>"
        return html
    
    def _generate_chart_scripts(self, suite_result: TestSuiteResult) -> str:
        """Generate JavaScript for charts"""
        # Prepare timeline data
        timeline_data = []
        for device_serial, device_result in suite_result.device_results.items():
            for execution in device_result.executions:
                if execution.start_time and execution.end_time:
                    timeline_data.append({
                        'device': device_serial,
                        'test': execution.step.name,
                        'start': execution.start_time,
                        'end': execution.end_time,
                        'status': execution.status.value
                    })
        
        return f'''
        // Timeline Chart
        const timelineCtx = document.getElementById('timelineChart').getContext('2d');
        const timelineData = {json.dumps(timeline_data)};
        
        // Process timeline data for Chart.js
        const datasets = {{}};
        const colors = {{
            'completed': '#28a745',
            'failed': '#dc3545',
            'timeout': '#ffc107',
            'skipped': '#6c757d'
        }};
        
        timelineData.forEach(item => {{
            if (!datasets[item.device]) {{
                datasets[item.device] = [];
            }}
            datasets[item.device].push({{
                x: [item.start * 1000, item.end * 1000],
                y: item.test,
                backgroundColor: colors[item.status] || '#6c757d'
            }});
        }});
        
        new Chart(timelineCtx, {{
            type: 'bar',
            data: {{
                datasets: Object.keys(datasets).map(device => ({{
                    label: device,
                    data: datasets[device],
                    backgroundColor: colors.completed,
                    borderWidth: 1
                }}))
            }},
            options: {{
                responsive: true,
                maintainAspectRatio: false,
                indexAxis: 'y',
                scales: {{
                    x: {{
                        type: 'time',
                        time: {{
                            unit: 'second'
                        }},
                        title: {{
                            display: true,
                            text: 'Time'
                        }}
                    }},
                    y: {{
                        title: {{
                            display: true,
                            text: 'Tests'
                        }}
                    }}
                }},
                plugins: {{
                    title: {{
                        display: true,
                        text: 'Test Execution Timeline'
                    }},
                    legend: {{
                        display: true
                    }}
                }}
            }}
        }});
        '''
    
    def _generate_trend_charts_data(self, trends: List[PerformanceTrend]) -> Dict[str, Any]:
        """Generate data for trend charts"""
        return {
            'metrics': [t.metric_name for t in trends],
            'current_values': [t.current_value for t in trends],
            'historical_means': [statistics.mean(t.historical_values) if t.historical_values else 0 for t in trends],
            'regression_flags': [t.regression_detected for t in trends]
        }
    
    def _generate_device_comparison_data(self, device_results: Dict[str, DeviceTestResult]) -> Dict[str, Any]:
        """Generate data for device comparison charts"""
        return {
            'devices': list(device_results.keys()),
            'success_rates': [r.metrics.success_rate for r in device_results.values()],
            'total_tests': [r.metrics.total_tests for r in device_results.values()],
            'durations': [r.end_time - r.start_time for r in device_results.values()]
        }
    
    def export_csv_report(self, suite_result: TestSuiteResult, output_path: Optional[str] = None) -> str:
        """Export test results in CSV format for data analysis"""
        if output_path is None:
            timestamp = time.strftime("%Y%m%d_%H%M%S")
            output_path = f"{suite_result.suite_name}_{timestamp}.csv"
        
        with open(output_path, 'w', newline='', encoding='utf-8') as csvfile:
            fieldnames = [
                'device_serial', 'test_name', 'test_type', 'status', 'duration',
                'start_time', 'end_time', 'retry_attempt', 'error_message',
                'required', 'timeout', 'parameters'
            ]
            writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
            writer.writeheader()
            
            for device_serial, device_result in suite_result.device_results.items():
                for execution in device_result.executions:
                    writer.writerow({
                        'device_serial': device_serial,
                        'test_name': execution.step.name,
                        'test_type': execution.step.test_type.name if hasattr(execution.step.test_type, 'name') else str(execution.step.test_type),
                        'status': execution.status.value,
                        'duration': execution.duration or '',
                        'start_time': execution.start_time or '',
                        'end_time': execution.end_time or '',
                        'retry_attempt': execution.retry_attempt,
                        'error_message': execution.error_message or '',
                        'required': execution.step.required,
                        'timeout': execution.step.timeout,
                        'parameters': json.dumps(execution.step.parameters)
                    })
        
        return output_path
    
    def save_artifacts_to_disk(self, suite_result: TestSuiteResult) -> Dict[str, str]:
        """Save test artifacts to disk and return file paths"""
        artifact_paths = {}
        timestamp = time.strftime("%Y%m%d_%H%M%S")
        suite_dir = self.artifacts_dir / f"{suite_result.suite_name}_{timestamp}"
        suite_dir.mkdir(exist_ok=True)
        
        for artifact in suite_result.artifacts:
            filename = f"{artifact.name}_{timestamp}.json"
            filepath = suite_dir / filename
            
            with open(filepath, 'w', encoding='utf-8') as f:
                json.dump({
                    'artifact_info': {
                        'name': artifact.name,
                        'type': artifact.type,
                        'timestamp': artifact.timestamp,
                        'size_bytes': artifact.size_bytes,
                        'metadata': artifact.metadata
                    },
                    'content': artifact.content
                }, f, indent=2, default=str)
            
            artifact_paths[artifact.name] = str(filepath)
        
        return artifact_paths
    
    def generate_regression_report(self, suite_result: TestSuiteResult) -> Dict[str, Any]:
        """Generate detailed regression analysis report"""
        regressions = [t for t in suite_result.performance_trends if t.regression_detected]
        
        report = {
            'summary': {
                'total_metrics_analyzed': len(suite_result.performance_trends),
                'regressions_detected': len(regressions),
                'regression_rate': len(regressions) / len(suite_result.performance_trends) * 100 if suite_result.performance_trends else 0
            },
            'regressions': [],
            'recommendations': []
        }
        
        for regression in regressions:
            regression_info = {
                'metric_name': regression.metric_name,
                'current_value': regression.current_value,
                'historical_mean': statistics.mean(regression.historical_values) if regression.historical_values else 0,
                'trend_direction': regression.trend_direction,
                'confidence_level': regression.confidence_level,
                'severity': self._assess_regression_severity(regression)
            }
            report['regressions'].append(regression_info)
        
        # Generate recommendations
        if regressions:
            report['recommendations'].extend([
                "Review recent code changes that might affect performance",
                "Run additional validation tests to confirm regressions",
                "Consider rolling back recent changes if regressions are severe",
                "Investigate hardware or environmental factors"
            ])
        
        return report
    
    def _assess_regression_severity(self, regression: PerformanceTrend) -> str:
        """Assess the severity of a performance regression"""
        if regression.confidence_level >= 0.9:
            return "critical"
        elif regression.confidence_level >= 0.7:
            return "high"
        elif regression.confidence_level >= 0.5:
            return "medium"
        else:
            return "low"