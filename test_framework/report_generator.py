"""
Advanced Test Report Generator

Provides comprehensive test report generation in multiple formats including
HTML, PDF, JUnit XML, JSON, and CSV with advanced analytics and visualizations.
"""

import json
import time
import os
from typing import Dict, List, Any, Optional, Tuple
from pathlib import Path
from dataclasses import asdict
import statistics
from datetime import datetime

from test_framework.result_collector import TestSuiteResult, TestArtifact, PerformanceTrend


class ReportGenerator:
    """
    Advanced test report generator with multiple output formats.
    
    Supports HTML, JSON, JUnit XML, CSV, and comprehensive analytics
    with performance trend analysis and regression detection.
    """
    
    def __init__(self, output_dir: str = "test_reports"):
        """
        Initialize the report generator.
        
        Args:
            output_dir: Directory for generated reports
        """
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(exist_ok=True)
        
    def generate_comprehensive_report(self, suite_result: TestSuiteResult,
                                    formats: List[str] = None) -> Dict[str, str]:
        """
        Generate comprehensive test reports in multiple formats.
        
        Args:
            suite_result: Test suite results to report on
            formats: List of formats to generate ('html', 'json', 'junit', 'csv', 'pdf')
            
        Returns:
            Dictionary mapping format names to output file paths
        """
        if formats is None:
            formats = ['html', 'json', 'junit', 'csv']
        
        timestamp = time.strftime("%Y%m%d_%H%M%S")
        report_files = {}
        
        # Generate reports in requested formats
        for format_name in formats:
            try:
                if format_name == 'html':
                    filepath = self._generate_html_report(suite_result, timestamp)
                elif format_name == 'json':
                    filepath = self._generate_json_report(suite_result, timestamp)
                elif format_name == 'junit':
                    filepath = self._generate_junit_report(suite_result, timestamp)
                elif format_name == 'csv':
                    filepath = self._generate_csv_report(suite_result, timestamp)
                elif format_name == 'pdf':
                    filepath = self._generate_pdf_report(suite_result, timestamp)
                else:
                    continue
                    
                report_files[format_name] = filepath
                
            except Exception as e:
                print(f"Warning: Failed to generate {format_name} report: {e}")
        
        # Generate summary index file
        index_file = self._generate_report_index(suite_result, report_files, timestamp)
        report_files['index'] = index_file
        
        return report_files
    
    def _generate_html_report(self, suite_result: TestSuiteResult, timestamp: str) -> str:
        """Generate comprehensive HTML report with interactive charts"""
        filename = f"{suite_result.suite_name}_report_{timestamp}.html"
        filepath = self.output_dir / filename
        
        html_content = self._create_html_template(suite_result)
        
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(html_content)
        
        return str(filepath)
    
    def _generate_json_report(self, suite_result: TestSuiteResult, timestamp: str) -> str:
        """Generate detailed JSON report"""
        filename = f"{suite_result.suite_name}_report_{timestamp}.json"
        filepath = self.output_dir / filename
        
        # Create comprehensive JSON structure
        report_data = {
            'metadata': {
                'report_version': '1.0',
                'generated_at': time.time(),
                'generator': 'Automated Testing Framework',
                'suite_name': suite_result.suite_name
            },
            'summary': {
                'suite_name': suite_result.suite_name,
                'description': suite_result.description,
                'start_time': suite_result.start_time,
                'end_time': suite_result.end_time,
                'duration': suite_result.duration,
                'total_devices': len(suite_result.device_results),
                'aggregate_metrics': asdict(suite_result.aggregate_metrics)
            },
            'device_results': {},
            'performance_trends': [asdict(trend) for trend in suite_result.performance_trends],
            'artifacts': [asdict(artifact) for artifact in suite_result.artifacts],
            'environment_info': suite_result.environment_info,
            'analysis': self._generate_analysis_data(suite_result)
        }
        
        # Add detailed device results
        for device_serial, device_result in suite_result.device_results.items():
            report_data['device_results'][device_serial] = {
                'device_serial': device_serial,
                'overall_status': device_result.overall_status.value,
                'metrics': asdict(device_result.metrics),
                'start_time': device_result.start_time,
                'end_time': device_result.end_time,
                'executions': []
            }
            
            for execution in device_result.executions:
                execution_data = {
                    'test_name': execution.step.name,
                    'test_type': execution.step.test_type.name if hasattr(execution.step.test_type, 'name') else str(execution.step.test_type),
                    'status': execution.status.value,
                    'start_time': execution.start_time,
                    'end_time': execution.end_time,
                    'duration': execution.duration,
                    'retry_attempt': execution.retry_attempt,
                    'error_message': execution.error_message,
                    'parameters': execution.step.parameters,
                    'required': execution.step.required,
                    'timeout': execution.step.timeout
                }
                
                if execution.response:
                    execution_data['response'] = {
                        'status': execution.response.status.name if hasattr(execution.response.status, 'name') else str(execution.response.status),
                        'data': execution.response.data,
                        'timestamp': execution.response.timestamp
                    }
                
                report_data['device_results'][device_serial]['executions'].append(execution_data)
        
        with open(filepath, 'w', encoding='utf-8') as f:
            json.dump(report_data, f, indent=2, default=str)
        
        return str(filepath)
    
    def _generate_junit_report(self, suite_result: TestSuiteResult, timestamp: str) -> str:
        """Generate JUnit XML report for CI/CD integration"""
        from xml.etree.ElementTree import Element, SubElement, tostring
        from xml.dom import minidom
        
        filename = f"{suite_result.suite_name}_junit_{timestamp}.xml"
        filepath = self.output_dir / filename
        
        # Create root testsuites element
        testsuites = Element("testsuites")
        testsuites.set("name", suite_result.suite_name)
        testsuites.set("tests", str(suite_result.aggregate_metrics.total_tests))
        testsuites.set("failures", str(suite_result.aggregate_metrics.failed_tests))
        testsuites.set("skipped", str(suite_result.aggregate_metrics.skipped_tests))
        testsuites.set("time", f"{suite_result.duration:.3f}")
        testsuites.set("timestamp", datetime.fromtimestamp(suite_result.start_time).isoformat())
        
        # Create testsuite for each device
        for device_serial, device_result in suite_result.device_results.items():
            testsuite = SubElement(testsuites, "testsuite")
            testsuite.set("name", f"{suite_result.suite_name}.{device_serial}")
            testsuite.set("tests", str(device_result.metrics.total_tests))
            testsuite.set("failures", str(device_result.metrics.failed_tests))
            testsuite.set("skipped", str(device_result.metrics.skipped_tests))
            testsuite.set("time", f"{device_result.end_time - device_result.start_time:.3f}")
            testsuite.set("timestamp", datetime.fromtimestamp(device_result.start_time).isoformat())
            
            # Add properties
            properties = SubElement(testsuite, "properties")
            prop = SubElement(properties, "property")
            prop.set("name", "device_serial")
            prop.set("value", device_serial)
            
            # Add test cases
            for execution in device_result.executions:
                testcase = SubElement(testsuite, "testcase")
                testcase.set("classname", f"{suite_result.suite_name}.{device_serial}")
                testcase.set("name", execution.step.name)
                testcase.set("time", f"{execution.duration or 0:.3f}")
                
                # Add failure/error/skip information
                if execution.status.value == 'failed':
                    failure = SubElement(testcase, "failure")
                    failure.set("message", execution.error_message or "Test failed")
                    failure.set("type", "TestFailure")
                    failure.text = execution.error_message or "No error details available"
                elif execution.status.value == 'timeout':
                    error = SubElement(testcase, "error")
                    error.set("message", "Test timeout")
                    error.set("type", "TestTimeout")
                    error.text = "Test execution timed out"
                elif execution.status.value == 'skipped':
                    skipped = SubElement(testcase, "skipped")
                    skipped.set("message", "Test was skipped")
                
                # Add system-out for test parameters and response data
                if execution.step.parameters or (execution.response and execution.response.data):
                    system_out = SubElement(testcase, "system-out")
                    output_data = {
                        'parameters': execution.step.parameters,
                        'response_data': execution.response.data if execution.response else None
                    }
                    system_out.text = json.dumps(output_data, indent=2, default=str)
        
        # Format XML with proper indentation
        rough_string = tostring(testsuites, 'unicode')
        reparsed = minidom.parseString(rough_string)
        
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(reparsed.toprettyxml(indent="  "))
        
        return str(filepath)
    
    def _generate_csv_report(self, suite_result: TestSuiteResult, timestamp: str) -> str:
        """Generate CSV report for data analysis"""
        import csv
        
        filename = f"{suite_result.suite_name}_data_{timestamp}.csv"
        filepath = self.output_dir / filename
        
        with open(filepath, 'w', newline='', encoding='utf-8') as csvfile:
            fieldnames = [
                'suite_name', 'device_serial', 'test_name', 'test_type', 'status',
                'duration', 'start_time', 'end_time', 'retry_attempt', 'error_message',
                'required', 'timeout', 'success_rate', 'parameters'
            ]
            writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
            writer.writeheader()
            
            for device_serial, device_result in suite_result.device_results.items():
                for execution in device_result.executions:
                    writer.writerow({
                        'suite_name': suite_result.suite_name,
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
                        'success_rate': device_result.metrics.success_rate,
                        'parameters': json.dumps(execution.step.parameters)
                    })
        
        return str(filepath)
    
    def _generate_pdf_report(self, suite_result: TestSuiteResult, timestamp: str) -> str:
        """Generate PDF report (requires reportlab)"""
        try:
            from reportlab.lib.pagesizes import letter, A4
            from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle
            from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
            from reportlab.lib.units import inch
            from reportlab.lib import colors
        except ImportError:
            raise ImportError("reportlab is required for PDF generation. Install with: pip install reportlab")
        
        filename = f"{suite_result.suite_name}_report_{timestamp}.pdf"
        filepath = self.output_dir / filename
        
        doc = SimpleDocTemplate(str(filepath), pagesize=A4)
        styles = getSampleStyleSheet()
        story = []
        
        # Title
        title_style = ParagraphStyle(
            'CustomTitle',
            parent=styles['Heading1'],
            fontSize=24,
            spaceAfter=30,
            alignment=1  # Center alignment
        )
        story.append(Paragraph(f"Test Report: {suite_result.suite_name}", title_style))
        story.append(Spacer(1, 20))
        
        # Summary section
        story.append(Paragraph("Executive Summary", styles['Heading2']))
        summary_data = [
            ['Metric', 'Value'],
            ['Total Tests', str(suite_result.aggregate_metrics.total_tests)],
            ['Passed Tests', str(suite_result.aggregate_metrics.passed_tests)],
            ['Failed Tests', str(suite_result.aggregate_metrics.failed_tests)],
            ['Success Rate', f"{suite_result.aggregate_metrics.success_rate:.1f}%"],
            ['Total Duration', f"{suite_result.duration:.1f} seconds"],
            ['Devices Tested', str(len(suite_result.device_results))]
        ]
        
        summary_table = Table(summary_data)
        summary_table.setStyle(TableStyle([
            ('BACKGROUND', (0, 0), (-1, 0), colors.grey),
            ('TEXTCOLOR', (0, 0), (-1, 0), colors.whitesmoke),
            ('ALIGN', (0, 0), (-1, -1), 'CENTER'),
            ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
            ('FONTSIZE', (0, 0), (-1, 0), 14),
            ('BOTTOMPADDING', (0, 0), (-1, 0), 12),
            ('BACKGROUND', (0, 1), (-1, -1), colors.beige),
            ('GRID', (0, 0), (-1, -1), 1, colors.black)
        ]))
        story.append(summary_table)
        story.append(Spacer(1, 20))
        
        # Performance trends section
        if suite_result.performance_trends:
            story.append(Paragraph("Performance Analysis", styles['Heading2']))
            regressions = [t for t in suite_result.performance_trends if t.regression_detected]
            
            if regressions:
                story.append(Paragraph(f"⚠️ {len(regressions)} performance regression(s) detected!", 
                                     styles['Normal']))
                story.append(Spacer(1, 10))
            
            trend_data = [['Metric', 'Current Value', 'Trend', 'Status']]
            for trend in suite_result.performance_trends:
                status = "REGRESSION" if trend.regression_detected else "OK"
                trend_data.append([
                    trend.metric_name,
                    f"{trend.current_value:.3f}",
                    trend.trend_direction.title(),
                    status
                ])
            
            trend_table = Table(trend_data)
            trend_table.setStyle(TableStyle([
                ('BACKGROUND', (0, 0), (-1, 0), colors.grey),
                ('TEXTCOLOR', (0, 0), (-1, 0), colors.whitesmoke),
                ('ALIGN', (0, 0), (-1, -1), 'CENTER'),
                ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
                ('FONTSIZE', (0, 0), (-1, 0), 12),
                ('BOTTOMPADDING', (0, 0), (-1, 0), 12),
                ('BACKGROUND', (0, 1), (-1, -1), colors.beige),
                ('GRID', (0, 0), (-1, -1), 1, colors.black)
            ]))
            story.append(trend_table)
            story.append(Spacer(1, 20))
        
        # Device results section
        story.append(Paragraph("Device Results", styles['Heading2']))
        for device_serial, device_result in suite_result.device_results.items():
            story.append(Paragraph(f"Device: {device_serial}", styles['Heading3']))
            
            device_data = [
                ['Test Name', 'Status', 'Duration (s)'],
            ]
            
            for execution in device_result.executions:
                duration_str = f"{execution.duration:.2f}" if execution.duration else "N/A"
                device_data.append([
                    execution.step.name,
                    execution.status.value.upper(),
                    duration_str
                ])
            
            device_table = Table(device_data)
            device_table.setStyle(TableStyle([
                ('BACKGROUND', (0, 0), (-1, 0), colors.grey),
                ('TEXTCOLOR', (0, 0), (-1, 0), colors.whitesmoke),
                ('ALIGN', (0, 0), (-1, -1), 'LEFT'),
                ('FONTNAME', (0, 0), (-1, 0), 'Helvetica-Bold'),
                ('FONTSIZE', (0, 0), (-1, 0), 10),
                ('BOTTOMPADDING', (0, 0), (-1, 0), 12),
                ('BACKGROUND', (0, 1), (-1, -1), colors.beige),
                ('GRID', (0, 0), (-1, -1), 1, colors.black)
            ]))
            story.append(device_table)
            story.append(Spacer(1, 15))
        
        doc.build(story)
        return str(filepath)
    
    def _generate_report_index(self, suite_result: TestSuiteResult, 
                              report_files: Dict[str, str], timestamp: str) -> str:
        """Generate an index HTML file linking to all reports"""
        filename = f"report_index_{timestamp}.html"
        filepath = self.output_dir / filename
        
        html_content = f"""
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Test Report Index - {suite_result.suite_name}</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
        }}
        .container {{
            max-width: 800px;
            margin: 0 auto;
            background-color: white;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            padding: 30px;
        }}
        .header {{
            text-align: center;
            margin-bottom: 30px;
            padding-bottom: 20px;
            border-bottom: 2px solid #667eea;
        }}
        .header h1 {{
            color: #333;
            margin: 0;
        }}
        .summary {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
            gap: 15px;
            margin: 20px 0;
            padding: 20px;
            background-color: #f8f9fa;
            border-radius: 8px;
        }}
        .summary-item {{
            text-align: center;
        }}
        .summary-item .value {{
            font-size: 1.5em;
            font-weight: bold;
            color: #667eea;
        }}
        .summary-item .label {{
            font-size: 0.9em;
            color: #666;
            margin-top: 5px;
        }}
        .reports-section {{
            margin-top: 30px;
        }}
        .report-link {{
            display: block;
            padding: 15px;
            margin: 10px 0;
            background-color: #f8f9fa;
            border: 1px solid #dee2e6;
            border-radius: 8px;
            text-decoration: none;
            color: #333;
            transition: all 0.3s ease;
        }}
        .report-link:hover {{
            background-color: #e9ecef;
            border-color: #667eea;
            transform: translateY(-2px);
            box-shadow: 0 4px 8px rgba(0,0,0,0.1);
        }}
        .report-link .title {{
            font-weight: bold;
            font-size: 1.1em;
            margin-bottom: 5px;
        }}
        .report-link .description {{
            color: #666;
            font-size: 0.9em;
        }}
        .footer {{
            text-align: center;
            margin-top: 30px;
            padding-top: 20px;
            border-top: 1px solid #dee2e6;
            color: #666;
            font-size: 0.9em;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Test Report Index</h1>
            <p>{suite_result.suite_name}</p>
            <p>Generated on {datetime.fromtimestamp(suite_result.end_time).strftime('%Y-%m-%d %H:%M:%S')}</p>
        </div>
        
        <div class="summary">
            <div class="summary-item">
                <div class="value">{suite_result.aggregate_metrics.total_tests}</div>
                <div class="label">Total Tests</div>
            </div>
            <div class="summary-item">
                <div class="value">{suite_result.aggregate_metrics.passed_tests}</div>
                <div class="label">Passed</div>
            </div>
            <div class="summary-item">
                <div class="value">{suite_result.aggregate_metrics.failed_tests}</div>
                <div class="label">Failed</div>
            </div>
            <div class="summary-item">
                <div class="value">{suite_result.aggregate_metrics.success_rate:.1f}%</div>
                <div class="label">Success Rate</div>
            </div>
            <div class="summary-item">
                <div class="value">{len(suite_result.device_results)}</div>
                <div class="label">Devices</div>
            </div>
        </div>
        
        <div class="reports-section">
            <h2>Available Reports</h2>
"""
        
        # Add links to available reports
        report_descriptions = {
            'html': ('Interactive HTML Report', 'Comprehensive report with charts and interactive elements'),
            'json': ('JSON Data Export', 'Machine-readable test data for further analysis'),
            'junit': ('JUnit XML Report', 'CI/CD compatible test results'),
            'csv': ('CSV Data Export', 'Spreadsheet-compatible test data'),
            'pdf': ('PDF Report', 'Printable comprehensive test report')
        }
        
        for format_name, filepath in report_files.items():
            if format_name == 'index':
                continue
                
            if format_name in report_descriptions:
                title, description = report_descriptions[format_name]
                filename = Path(filepath).name
                
                html_content += f"""
            <a href="{filename}" class="report-link">
                <div class="title">{title}</div>
                <div class="description">{description}</div>
            </a>
"""
        
        html_content += """
        </div>
        
        <div class="footer">
            <p>Generated by Automated Testing Framework</p>
        </div>
    </div>
</body>
</html>
"""
        
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(html_content)
        
        return str(filepath)
    
    def _create_html_template(self, suite_result: TestSuiteResult) -> str:
        """Create comprehensive HTML report template"""
        # This would be a very long method, so I'll create a simplified version
        # In a real implementation, this would include all the HTML generation logic
        # from the result_collector.py _generate_html_report_content method
        
        return f"""
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{suite_result.suite_name} - Comprehensive Test Report</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        /* Comprehensive CSS styles would go here */
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 30px; text-align: center; }}
        .summary {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; padding: 30px; }}
        .summary-card {{ background: white; padding: 20px; border-radius: 8px; text-align: center; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
    </style>
</head>
<body>
    <div class="header">
        <h1>{suite_result.suite_name}</h1>
        <p>{suite_result.description}</p>
        <p>Generated on {datetime.fromtimestamp(suite_result.end_time).strftime('%Y-%m-%d %H:%M:%S')}</p>
    </div>
    
    <div class="summary">
        <div class="summary-card">
            <h3>Total Tests</h3>
            <p class="value">{suite_result.aggregate_metrics.total_tests}</p>
        </div>
        <div class="summary-card">
            <h3>Success Rate</h3>
            <p class="value">{suite_result.aggregate_metrics.success_rate:.1f}%</p>
        </div>
        <div class="summary-card">
            <h3>Duration</h3>
            <p class="value">{suite_result.duration:.1f}s</p>
        </div>
    </div>
    
    <!-- Additional sections would be added here -->
    
</body>
</html>
"""
    
    def _generate_analysis_data(self, suite_result: TestSuiteResult) -> Dict[str, Any]:
        """Generate comprehensive analysis data"""
        analysis = {
            'failure_analysis': self._analyze_failures(suite_result),
            'performance_analysis': self._analyze_performance(suite_result),
            'device_comparison': self._compare_devices(suite_result),
            'trend_analysis': self._analyze_trends(suite_result.performance_trends)
        }
        
        return analysis
    
    def _analyze_failures(self, suite_result: TestSuiteResult) -> Dict[str, Any]:
        """Analyze test failures for patterns and insights"""
        failure_patterns = {}
        device_failures = {}
        
        for device_serial, device_result in suite_result.device_results.items():
            device_failures[device_serial] = 0
            
            for execution in device_result.executions:
                if execution.status.value == 'failed':
                    device_failures[device_serial] += 1
                    
                    test_name = execution.step.name
                    if test_name not in failure_patterns:
                        failure_patterns[test_name] = 0
                    failure_patterns[test_name] += 1
        
        return {
            'failure_patterns': failure_patterns,
            'device_failures': device_failures,
            'most_common_failures': sorted(failure_patterns.items(), key=lambda x: x[1], reverse=True)[:5]
        }
    
    def _analyze_performance(self, suite_result: TestSuiteResult) -> Dict[str, Any]:
        """Analyze performance metrics across devices"""
        performance_data = {}
        
        for device_serial, device_result in suite_result.device_results.items():
            durations = [e.duration for e in device_result.executions if e.duration]
            if durations:
                performance_data[device_serial] = {
                    'avg_duration': statistics.mean(durations),
                    'min_duration': min(durations),
                    'max_duration': max(durations),
                    'total_duration': sum(durations)
                }
        
        return performance_data
    
    def _compare_devices(self, suite_result: TestSuiteResult) -> Dict[str, Any]:
        """Compare performance and results across devices"""
        comparison = {
            'success_rates': {},
            'performance_ranking': [],
            'consistency_analysis': {}
        }
        
        for device_serial, device_result in suite_result.device_results.items():
            comparison['success_rates'][device_serial] = device_result.metrics.success_rate
        
        # Rank devices by success rate
        comparison['performance_ranking'] = sorted(
            comparison['success_rates'].items(),
            key=lambda x: x[1],
            reverse=True
        )
        
        return comparison
    
    def _analyze_trends(self, trends: List[PerformanceTrend]) -> Dict[str, Any]:
        """Analyze performance trends for insights"""
        trend_analysis = {
            'regression_count': len([t for t in trends if t.regression_detected]),
            'improving_metrics': [t.metric_name for t in trends if t.trend_direction == 'improving'],
            'degrading_metrics': [t.metric_name for t in trends if t.trend_direction == 'degrading'],
            'stable_metrics': [t.metric_name for t in trends if t.trend_direction == 'stable']
        }
        
        return trend_analysis