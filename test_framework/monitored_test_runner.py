"""
Monitored Test Runner

Comprehensive test runner that integrates real-time monitoring, debugging capabilities,
and enhanced reporting for automated testing workflows.
"""

import time
import logging
import json
from typing import Dict, List, Any, Optional
from pathlib import Path

from .device_manager import UsbHidDeviceManager
from .command_handler import CommandHandler
from .test_sequencer import TestSequencer, TestConfiguration
from .real_time_monitor import RealTimeMonitor, LogLevel, MonitoringEvent
from .result_collector import ResultCollector, TestSuiteResult
from .report_generator import ReportGenerator


class MonitoredTestRunner:
    """
    Enhanced test runner with comprehensive monitoring and debugging capabilities.
    
    Provides real-time progress tracking, verbose logging, failure analysis,
    and detailed reporting for automated testing workflows.
    """
    
    def __init__(self, log_level: LogLevel = LogLevel.NORMAL,
                 enable_snapshots: bool = True,
                 output_dir: str = "test_output"):
        """
        Initialize the monitored test runner.
        
        Args:
            log_level: Verbosity level for monitoring and logging
            enable_snapshots: Whether to capture system state snapshots on failures
            output_dir: Directory for test outputs and reports
        """
        self.log_level = log_level
        self.enable_snapshots = enable_snapshots
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(exist_ok=True)
        
        # Initialize components
        self.device_manager = UsbHidDeviceManager()
        self.monitor = RealTimeMonitor(
            log_level=log_level,
            enable_snapshots=enable_snapshots
        )
        self.command_handler = CommandHandler(
            self.device_manager,
            monitor=self.monitor
        )
        self.test_sequencer = TestSequencer(
            self.device_manager,
            self.command_handler,
            monitor=self.monitor
        )
        self.result_collector = ResultCollector()
        self.report_generator = ReportGenerator(str(self.output_dir))
        
        self.logger = logging.getLogger(__name__)
        
        # Setup monitoring callbacks
        self._setup_monitoring_callbacks()
    
    def _setup_monitoring_callbacks(self):
        """Setup callbacks for monitoring events"""
        # Progress update callback
        def progress_callback(event):
            progress = self.monitor.get_device_progress(event.device_serial)
            if progress and progress.completed_tests > 0:
                completion_pct = (progress.completed_tests / progress.total_tests * 100) if progress.total_tests > 0 else 0
                self.logger.info(f"[{event.device_serial}] Progress: {progress.completed_tests}/{progress.total_tests} "
                               f"({completion_pct:.1f}%) - Success: {progress.success_count}, Failed: {progress.failure_count}")
        
        # Failure analysis callback
        def failure_callback(event):
            if self.log_level in [LogLevel.VERBOSE, LogLevel.DEBUG]:
                self.logger.warning(f"[{event.device_serial}] Test failure captured: {event.test_name}")
                if 'error_message' in event.data:
                    self.logger.warning(f"  Error: {event.data['error_message']}")
        
        # Communication debugging callback
        def communication_callback(event):
            if self.log_level == LogLevel.DEBUG:
                direction = event.data.get('direction', 'unknown')
                message = event.data.get('message', '')[:100]  # Truncate long messages
                self.logger.debug(f"[{event.device_serial}] {direction.upper()}: {message}")
        
        self.monitor.register_event_callback(MonitoringEvent.TEST_COMPLETED, progress_callback)
        self.monitor.register_event_callback(MonitoringEvent.TEST_FAILED, failure_callback)
        self.monitor.register_event_callback(MonitoringEvent.DEVICE_COMMUNICATION, communication_callback)
    
    def run_test_suite(self, config: TestConfiguration,
                      target_devices: List[str] = None,
                      export_monitoring_data: bool = True) -> TestSuiteResult:
        """
        Run a comprehensive test suite with full monitoring.
        
        Args:
            config: Test configuration to execute
            target_devices: List of device serial numbers (None for all connected)
            export_monitoring_data: Whether to export detailed monitoring data
            
        Returns:
            Comprehensive test suite results
        """
        self.logger.info(f"Starting monitored test suite: {config.name}")
        
        # Start monitoring
        self.monitor.start_monitoring()
        
        try:
            # Discover and connect to devices
            if target_devices is None:
                target_devices = self._discover_and_connect_devices()
            else:
                self._connect_to_devices(target_devices)
            
            if not target_devices:
                raise RuntimeError("No devices available for testing")
            
            self.logger.info(f"Running tests on {len(target_devices)} devices with {self.log_level.value} logging")
            
            # Execute test sequence
            execution_results = self.test_sequencer.execute_test_sequence(
                config, target_devices
            )
            
            # Collect and analyze results
            suite_result = self.result_collector.collect_suite_results(
                config.name,
                config.description,
                execution_results
            )
            
            # Generate reports
            self._generate_comprehensive_reports(suite_result)
            
            # Export monitoring data if requested
            if export_monitoring_data:
                self._export_monitoring_data(suite_result.suite_name)
            
            # Log final summary
            self._log_final_summary(suite_result)
            
            return suite_result
            
        finally:
            # Stop monitoring and cleanup
            self.monitor.stop_monitoring()
            self.device_manager.disconnect_all()
    
    def _discover_and_connect_devices(self) -> List[str]:
        """Discover and connect to all available devices"""
        self.logger.info("Discovering devices...")
        devices = self.device_manager.discover_devices()
        
        if not devices:
            self.logger.error("No devices found")
            return []
        
        connected_devices = []
        for device in devices:
            if device.status.value == 'connected':
                if self.device_manager.connect_device(device.serial_number):
                    connected_devices.append(device.serial_number)
                    self.logger.info(f"Connected to device: {device.serial_number}")
                else:
                    self.logger.warning(f"Failed to connect to device: {device.serial_number}")
        
        return connected_devices
    
    def _connect_to_devices(self, device_serials: List[str]):
        """Connect to specific devices"""
        for serial in device_serials:
            if not self.device_manager.connect_device(serial):
                self.logger.warning(f"Failed to connect to device: {serial}")
    
    def _generate_comprehensive_reports(self, suite_result: TestSuiteResult):
        """Generate comprehensive test reports"""
        self.logger.info("Generating test reports...")
        
        # Determine report formats based on log level
        formats = ['json', 'junit']  # Always generate these for CI/CD
        
        if self.log_level in [LogLevel.NORMAL, LogLevel.VERBOSE, LogLevel.DEBUG]:
            formats.extend(['html', 'csv'])
        
        if self.log_level == LogLevel.DEBUG:
            formats.append('pdf')  # Only generate PDF in debug mode
        
        try:
            report_files = self.report_generator.generate_comprehensive_report(
                suite_result, formats
            )
            
            self.logger.info("Generated reports:")
            for format_name, filepath in report_files.items():
                self.logger.info(f"  {format_name.upper()}: {filepath}")
                
        except Exception as e:
            self.logger.error(f"Failed to generate some reports: {e}")
    
    def _export_monitoring_data(self, suite_name: str):
        """Export detailed monitoring data"""
        timestamp = time.strftime("%Y%m%d_%H%M%S")
        monitoring_file = self.output_dir / f"{suite_name}_monitoring_{timestamp}.json"
        
        try:
            self.monitor.export_monitoring_data(str(monitoring_file))
            
            # Also export system snapshots separately if any exist
            snapshots = self.monitor.get_system_snapshots()
            if snapshots:
                snapshots_file = self.output_dir / f"{suite_name}_snapshots_{timestamp}.json"
                with open(snapshots_file, 'w') as f:
                    json.dump([self._snapshot_to_dict(snap) for snap in snapshots], f, indent=2, default=str)
                self.logger.info(f"Exported {len(snapshots)} system snapshots to {snapshots_file}")
                
        except Exception as e:
            self.logger.error(f"Failed to export monitoring data: {e}")
    
    def _snapshot_to_dict(self, snapshot) -> Dict[str, Any]:
        """Convert system snapshot to dictionary"""
        return {
            'timestamp': snapshot.timestamp,
            'device_serial': snapshot.device_serial,
            'test_name': snapshot.test_name,
            'system_state': snapshot.system_state,
            'device_logs': snapshot.device_logs,
            'communication_history': snapshot.communication_history,
            'performance_metrics': snapshot.performance_metrics,
            'error_context': snapshot.error_context
        }
    
    def _log_final_summary(self, suite_result: TestSuiteResult):
        """Log final test summary"""
        self.logger.info("=" * 60)
        self.logger.info(f"TEST SUITE COMPLETED: {suite_result.suite_name}")
        self.logger.info("=" * 60)
        self.logger.info(f"Total Tests: {suite_result.aggregate_metrics.total_tests}")
        self.logger.info(f"Passed: {suite_result.aggregate_metrics.passed_tests}")
        self.logger.info(f"Failed: {suite_result.aggregate_metrics.failed_tests}")
        self.logger.info(f"Skipped: {suite_result.aggregate_metrics.skipped_tests}")
        self.logger.info(f"Success Rate: {suite_result.aggregate_metrics.success_rate:.1f}%")
        self.logger.info(f"Duration: {suite_result.duration:.1f} seconds")
        self.logger.info(f"Devices Tested: {len(suite_result.device_results)}")
        
        # Log device-specific results
        for device_serial, device_result in suite_result.device_results.items():
            self.logger.info(f"  {device_serial}: {device_result.metrics.passed_tests}/"
                           f"{device_result.metrics.total_tests} passed "
                           f"({device_result.metrics.success_rate:.1f}%)")
        
        # Log performance regressions if any
        regressions = [t for t in suite_result.performance_trends if t.regression_detected]
        if regressions:
            self.logger.warning(f"Performance regressions detected: {len(regressions)}")
            for regression in regressions:
                self.logger.warning(f"  {regression.metric_name}: {regression.trend_direction}")
        
        # Log system snapshots if any
        snapshots = self.monitor.get_system_snapshots()
        if snapshots:
            self.logger.info(f"System snapshots captured: {len(snapshots)}")
        
        self.logger.info("=" * 60)
    
    def get_real_time_status(self) -> Dict[str, Any]:
        """Get current real-time status of all devices"""
        status = {
            'devices': {},
            'overall_progress': {
                'total_devices': len(self.monitor.get_all_progress()),
                'active_tests': 0,
                'completed_devices': 0
            }
        }
        
        for device_serial, progress in self.monitor.get_all_progress().items():
            status['devices'][device_serial] = {
                'current_test': progress.current_test,
                'completed_tests': progress.completed_tests,
                'total_tests': progress.total_tests,
                'success_count': progress.success_count,
                'failure_count': progress.failure_count,
                'completion_percentage': (progress.completed_tests / progress.total_tests * 100) if progress.total_tests > 0 else 0,
                'estimated_completion': progress.estimated_completion,
                'status': progress.current_status
            }
            
            if progress.current_test:
                status['overall_progress']['active_tests'] += 1
            if progress.current_status == 'completed':
                status['overall_progress']['completed_devices'] += 1
        
        return status
    
    def get_communication_debug_info(self, device_serial: str = None) -> Dict[str, Any]:
        """Get detailed communication debugging information"""
        comm_logs = self.monitor.get_communication_logs(device_serial, max_entries=50)
        
        debug_info = {
            'total_communications': len(comm_logs),
            'sent_commands': len([log for log in comm_logs if log.direction == 'sent']),
            'received_responses': len([log for log in comm_logs if log.direction == 'received']),
            'recent_communications': []
        }
        
        for log in comm_logs[-20:]:  # Last 20 communications
            debug_info['recent_communications'].append({
                'timestamp': log.timestamp,
                'device': log.device_serial,
                'direction': log.direction,
                'message_type': log.message_type,
                'correlation_id': log.correlation_id,
                'data_summary': str(log.data)[:100] + "..." if len(str(log.data)) > 100 else str(log.data)
            })
        
        return debug_info
    
    def get_failure_analysis(self) -> Dict[str, Any]:
        """Get detailed failure analysis from system snapshots"""
        return self.monitor.get_enhanced_failure_analysis()
    
    def get_enhanced_debug_info(self, device_serial: str = None) -> Dict[str, Any]:
        """Get comprehensive debugging information for troubleshooting"""
        return self.monitor.get_real_time_debug_info(device_serial)
    
    def get_protocol_debug_logs(self, device_serial: str = None, max_entries: int = 50) -> List[Dict[str, Any]]:
        """Get detailed protocol debugging logs"""
        comm_logs = self.monitor.get_communication_logs(device_serial, max_entries)
        
        protocol_logs = []
        for log in comm_logs:
            if self.monitor.protocol_debug_enabled and log.protocol_details:
                protocol_logs.append({
                    'timestamp': log.timestamp,
                    'device': log.device_serial,
                    'direction': log.direction,
                    'message_type': log.message_type,
                    'correlation_id': log.correlation_id,
                    'sequence_number': log.sequence_number,
                    'latency_ms': log.latency_ms,
                    'protocol_details': log.protocol_details
                })
            else:
                # Basic log entry without protocol details
                protocol_logs.append({
                    'timestamp': log.timestamp,
                    'device': log.device_serial,
                    'direction': log.direction,
                    'message_type': log.message_type,
                    'correlation_id': log.correlation_id,
                    'latency_ms': log.latency_ms
                })
        
        return protocol_logs
    
    def generate_debug_report(self, output_file: str = None) -> str:
        """Generate a comprehensive debugging report"""
        if output_file is None:
            timestamp = time.strftime("%Y%m%d_%H%M%S")
            output_file = str(self.output_dir / f"debug_report_{timestamp}.json")
        
        debug_report = {
            'report_metadata': {
                'timestamp': time.time(),
                'log_level': self.log_level.value,
                'snapshots_enabled': self.enable_snapshots,
                'monitoring_active': self.monitor.monitoring_active
            },
            'real_time_status': self.get_real_time_status(),
            'enhanced_debug_info': self.get_enhanced_debug_info(),
            'failure_analysis': self.get_failure_analysis(),
            'protocol_debug_logs': self.get_protocol_debug_logs(max_entries=100),
            'system_snapshots': [
                self._snapshot_to_dict(snapshot) 
                for snapshot in self.monitor.get_system_snapshots()
            ]
        }
        
        with open(output_file, 'w') as f:
            json.dump(debug_report, f, indent=2, default=str)
        
        self.logger.info(f"Debug report generated: {output_file}")
        return output_file
    
    def create_debug_test_config(self) -> TestConfiguration:
        """Create a test configuration optimized for debugging"""
        from .test_sequencer import TestStep, TestType
        
        return TestConfiguration(
            name="Debug Test Suite",
            description="Comprehensive debugging and validation test suite",
            steps=[
                TestStep(
                    name="communication_test",
                    test_type=TestType.USB_COMMUNICATION_TEST,
                    parameters={"message_count": 5, "timeout_ms": 1000},
                    timeout=10.0,
                    retry_count=1
                ),
                TestStep(
                    name="system_health_check",
                    test_type=TestType.PEMF_TIMING_VALIDATION,
                    parameters={"duration_ms": 2000, "tolerance_percent": 2.0},
                    timeout=15.0,
                    depends_on=["communication_test"]
                ),
                TestStep(
                    name="battery_validation",
                    test_type=TestType.BATTERY_ADC_CALIBRATION,
                    parameters={"reference_voltage": 3.3},
                    timeout=10.0,
                    depends_on=["communication_test"]
                ),
                TestStep(
                    name="led_test",
                    test_type=TestType.LED_FUNCTIONALITY,
                    parameters={"pattern": "debug", "duration_ms": 1000},
                    timeout=8.0,
                    depends_on=["communication_test"]
                )
            ],
            parallel_execution=False,
            global_timeout=120.0
        )


def create_monitored_runner(log_level: str = "normal", 
                          enable_snapshots: bool = True,
                          output_dir: str = "test_output") -> MonitoredTestRunner:
    """
    Factory function to create a monitored test runner.
    
    Args:
        log_level: Logging verbosity ("minimal", "normal", "verbose", "debug")
        enable_snapshots: Whether to capture failure snapshots
        output_dir: Output directory for reports and data
        
    Returns:
        Configured MonitoredTestRunner instance
    """
    log_level_map = {
        "minimal": LogLevel.MINIMAL,
        "normal": LogLevel.NORMAL,
        "verbose": LogLevel.VERBOSE,
        "debug": LogLevel.DEBUG
    }
    
    return MonitoredTestRunner(
        log_level=log_level_map.get(log_level.lower(), LogLevel.NORMAL),
        enable_snapshots=enable_snapshots,
        output_dir=output_dir
    )