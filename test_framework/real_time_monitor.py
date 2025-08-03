"""
Real-time Test Monitoring and Debugging

Provides real-time test progress reporting, verbose logging modes, failure point capture,
and device communication logging with timestamp correlation for debugging purposes.
"""

import time
import threading
import logging
import json
from typing import Dict, List, Any, Optional, Callable, Set, TYPE_CHECKING
from dataclasses import dataclass, field
from enum import Enum
from collections import deque
import queue
from datetime import datetime

# Avoid circular import - use TYPE_CHECKING for type hints
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .test_sequencer import TestExecution, TestStatus
from .command_handler import TestCommand, TestResponse


class LogLevel(Enum):
    """Logging verbosity levels"""
    MINIMAL = "minimal"
    NORMAL = "normal"
    VERBOSE = "verbose"
    DEBUG = "debug"


class MonitoringEvent(Enum):
    """Types of monitoring events"""
    TEST_STARTED = "test_started"
    TEST_COMPLETED = "test_completed"
    TEST_FAILED = "test_failed"
    COMMAND_SENT = "command_sent"
    RESPONSE_RECEIVED = "response_received"
    DEVICE_COMMUNICATION = "device_communication"
    SYSTEM_STATE_SNAPSHOT = "system_state_snapshot"
    PROGRESS_UPDATE = "progress_update"


@dataclass
class MonitoringEventData:
    """Data structure for monitoring events"""
    event_type: MonitoringEvent
    timestamp: float
    device_serial: str
    test_name: Optional[str] = None
    data: Dict[str, Any] = field(default_factory=dict)
    correlation_id: Optional[str] = None


@dataclass
class SystemStateSnapshot:
    """System state snapshot for failure analysis"""
    timestamp: float
    device_serial: str
    test_name: str
    system_state: Dict[str, Any]
    device_logs: List[str]
    communication_history: List[Dict[str, Any]]
    performance_metrics: Dict[str, float]
    error_context: Optional[str] = None


@dataclass
class CommunicationLog:
    """Device communication log entry"""
    timestamp: float
    device_serial: str
    direction: str  # 'sent' or 'received'
    message_type: str
    data: Any
    correlation_id: Optional[str] = None
    raw_bytes: Optional[bytes] = None
    latency_ms: Optional[float] = None  # For request-response pairs
    sequence_number: Optional[int] = None
    protocol_details: Dict[str, Any] = field(default_factory=dict)


@dataclass
class ProgressStatus:
    """Real-time progress status"""
    device_serial: str
    current_test: Optional[str]
    completed_tests: int
    total_tests: int
    success_count: int
    failure_count: int
    start_time: float
    estimated_completion: Optional[float] = None
    current_status: str = "running"
    current_test_start_time: Optional[float] = None
    current_test_progress: float = 0.0  # 0.0 to 1.0
    last_activity_time: float = field(default_factory=time.time)
    health_status: str = "healthy"  # healthy, warning, error
    performance_metrics: Dict[str, float] = field(default_factory=dict)


class RealTimeMonitor:
    """
    Real-time test monitoring and debugging system.
    
    Provides comprehensive monitoring capabilities including progress tracking,
    verbose logging, failure capture, and communication debugging.
    """
    
    def __init__(self, log_level: LogLevel = LogLevel.NORMAL, 
                 max_history_size: int = 1000,
                 enable_snapshots: bool = True):
        """
        Initialize the real-time monitor.
        
        Args:
            log_level: Verbosity level for logging
            max_history_size: Maximum number of events to keep in history
            enable_snapshots: Whether to capture system state snapshots on failures
        """
        self.log_level = log_level
        self.max_history_size = max_history_size
        self.enable_snapshots = enable_snapshots
        
        # Event tracking
        self.event_queue = queue.Queue()
        self.event_history = deque(maxlen=max_history_size)
        self.communication_logs = deque(maxlen=max_history_size)
        self.system_snapshots: List[SystemStateSnapshot] = []
        
        # Progress tracking
        self.device_progress: Dict[str, ProgressStatus] = {}
        self.active_tests: Dict[str, TestExecution] = {}
        
        # Monitoring state
        self.monitoring_active = False
        self.monitor_thread: Optional[threading.Thread] = None
        self.event_callbacks: Dict[MonitoringEvent, List[Callable]] = {}
        
        # Communication tracking
        self.pending_commands: Dict[str, CommunicationLog] = {}
        self.correlation_counter = 0
        self.sequence_counter = 0
        
        # Enhanced monitoring features
        self.health_check_interval = 30.0  # seconds
        self.last_health_check = time.time()
        self.protocol_debug_enabled = (log_level == LogLevel.DEBUG)
        self.periodic_status_interval = 10.0  # seconds
        self.last_status_report = time.time()
        
        # Logger setup
        self.logger = logging.getLogger(__name__)
        self._setup_logging()
        
    def _setup_logging(self):
        """Setup logging configuration based on verbosity level"""
        log_levels = {
            LogLevel.MINIMAL: logging.WARNING,
            LogLevel.NORMAL: logging.INFO,
            LogLevel.VERBOSE: logging.DEBUG,
            LogLevel.DEBUG: logging.DEBUG
        }
        
        # Create formatter based on verbosity
        if self.log_level in [LogLevel.VERBOSE, LogLevel.DEBUG]:
            formatter = logging.Formatter(
                '%(asctime)s.%(msecs)03d [%(levelname)s] %(name)s:%(lineno)d - %(message)s',
                datefmt='%H:%M:%S'
            )
        else:
            formatter = logging.Formatter(
                '%(asctime)s [%(levelname)s] %(message)s',
                datefmt='%H:%M:%S'
            )
        
        # Configure handler
        handler = logging.StreamHandler()
        handler.setFormatter(formatter)
        
        self.logger.setLevel(log_levels[self.log_level])
        if not self.logger.handlers:
            self.logger.addHandler(handler)
    
    def start_monitoring(self):
        """Start the real-time monitoring system"""
        if self.monitoring_active:
            return
            
        self.monitoring_active = True
        self.monitor_thread = threading.Thread(target=self._monitor_loop, daemon=True)
        self.monitor_thread.start()
        
        self.logger.info("Real-time monitoring started")
    
    def stop_monitoring(self):
        """Stop the real-time monitoring system"""
        if not self.monitoring_active:
            return
            
        self.monitoring_active = False
        if self.monitor_thread:
            self.monitor_thread.join(timeout=1.0)
            
        self.logger.info("Real-time monitoring stopped")
    
    def _monitor_loop(self):
        """Main monitoring loop"""
        while self.monitoring_active:
            try:
                # Process events from queue
                try:
                    event = self.event_queue.get(timeout=0.1)
                    self._process_event(event)
                    self.event_queue.task_done()
                except queue.Empty:
                    pass
                    
                # Update progress estimates
                self._update_progress_estimates()
                
                # Perform periodic health checks
                current_time = time.time()
                if current_time - self.last_health_check >= self.health_check_interval:
                    self._perform_health_checks()
                    self.last_health_check = current_time
                
                # Generate periodic status reports
                if current_time - self.last_status_report >= self.periodic_status_interval:
                    self._generate_periodic_status_report()
                    self.last_status_report = current_time
                
            except Exception as e:
                self.logger.error(f"Error in monitoring loop: {e}")
                time.sleep(0.1)
    
    def _process_event(self, event: MonitoringEventData):
        """Process a monitoring event"""
        # Add to history
        self.event_history.append(event)
        
        # Log event based on verbosity level
        self._log_event(event)
        
        # Update progress tracking
        self._update_progress(event)
        
        # Handle special events
        if event.event_type == MonitoringEvent.TEST_FAILED and self.enable_snapshots:
            self._capture_failure_snapshot(event)
        
        # Call registered callbacks
        callbacks = self.event_callbacks.get(event.event_type, [])
        for callback in callbacks:
            try:
                callback(event)
            except Exception as e:
                self.logger.error(f"Error in event callback: {e}")
    
    def _log_event(self, event: MonitoringEventData):
        """Log event based on current verbosity level"""
        timestamp_str = datetime.fromtimestamp(event.timestamp).strftime('%H:%M:%S.%f')[:-3]
        
        if self.log_level == LogLevel.MINIMAL:
            # Only log critical events
            if event.event_type in [MonitoringEvent.TEST_FAILED, MonitoringEvent.PROGRESS_UPDATE]:
                self.logger.info(f"[{event.device_serial}] {event.event_type.value}: {event.test_name}")
                
        elif self.log_level == LogLevel.NORMAL:
            # Log test lifecycle events
            if event.event_type in [MonitoringEvent.TEST_STARTED, MonitoringEvent.TEST_COMPLETED, 
                                  MonitoringEvent.TEST_FAILED, MonitoringEvent.PROGRESS_UPDATE]:
                self.logger.info(f"[{event.device_serial}] {event.event_type.value}: {event.test_name}")
                
        elif self.log_level == LogLevel.VERBOSE:
            # Log all events with basic data
            data_summary = self._summarize_event_data(event.data)
            self.logger.info(f"[{timestamp_str}] [{event.device_serial}] {event.event_type.value}: "
                           f"{event.test_name or 'N/A'} - {data_summary}")
                           
        elif self.log_level == LogLevel.DEBUG:
            # Log everything with full data
            self.logger.debug(f"[{timestamp_str}] [{event.device_serial}] {event.event_type.value}:")
            self.logger.debug(f"  Test: {event.test_name or 'N/A'}")
            self.logger.debug(f"  Correlation ID: {event.correlation_id or 'N/A'}")
            self.logger.debug(f"  Data: {json.dumps(event.data, indent=2, default=str)}")
    
    def _summarize_event_data(self, data: Dict[str, Any]) -> str:
        """Create a brief summary of event data"""
        if not data:
            return "No data"
            
        summary_parts = []
        for key, value in data.items():
            if isinstance(value, (str, int, float, bool)):
                summary_parts.append(f"{key}={value}")
            elif isinstance(value, dict):
                summary_parts.append(f"{key}={{...}}")
            elif isinstance(value, list):
                summary_parts.append(f"{key}=[{len(value)} items]")
            else:
                summary_parts.append(f"{key}={type(value).__name__}")
                
        return ", ".join(summary_parts[:3])  # Limit to first 3 items
    
    def _update_progress(self, event: MonitoringEventData):
        """Update progress tracking based on event"""
        device_serial = event.device_serial
        
        if device_serial not in self.device_progress:
            self.device_progress[device_serial] = ProgressStatus(
                device_serial=device_serial,
                current_test=None,
                completed_tests=0,
                total_tests=0,
                success_count=0,
                failure_count=0,
                start_time=time.time()
            )
        
        progress = self.device_progress[device_serial]
        
        if event.event_type == MonitoringEvent.TEST_STARTED:
            progress.current_test = event.test_name
            progress.current_status = "running"
            progress.current_test_start_time = event.timestamp
            progress.current_test_progress = 0.0
            progress.last_activity_time = event.timestamp
            if 'total_tests' in event.data:
                progress.total_tests = event.data['total_tests']
                
        elif event.event_type == MonitoringEvent.TEST_COMPLETED:
            progress.completed_tests += 1
            progress.success_count += 1
            progress.current_test = None
            progress.current_test_start_time = None
            progress.current_test_progress = 0.0
            progress.last_activity_time = event.timestamp
            progress.current_status = "completed" if progress.completed_tests >= progress.total_tests else "running"
            
        elif event.event_type == MonitoringEvent.TEST_FAILED:
            progress.completed_tests += 1
            progress.failure_count += 1
            progress.current_test = None
            progress.current_test_start_time = None
            progress.current_test_progress = 0.0
            progress.last_activity_time = event.timestamp
            progress.current_status = "completed" if progress.completed_tests >= progress.total_tests else "running"
    
    def _update_progress_estimates(self):
        """Update estimated completion times for active tests"""
        current_time = time.time()
        
        for progress in self.device_progress.values():
            # Update last activity time
            progress.last_activity_time = current_time
            
            # Calculate completion estimates
            if progress.completed_tests > 0 and progress.total_tests > 0:
                elapsed_time = current_time - progress.start_time
                avg_time_per_test = elapsed_time / progress.completed_tests
                remaining_tests = progress.total_tests - progress.completed_tests
                progress.estimated_completion = current_time + (avg_time_per_test * remaining_tests)
            
            # Update current test progress if available
            if progress.current_test and progress.current_test_start_time:
                test_elapsed = current_time - progress.current_test_start_time
                # Estimate progress based on average test time (rough approximation)
                if progress.completed_tests > 0:
                    avg_test_time = (current_time - progress.start_time) / progress.completed_tests
                    progress.current_test_progress = min(1.0, test_elapsed / avg_test_time)
            
            # Update performance metrics
            if progress.completed_tests > 0:
                elapsed_time = current_time - progress.start_time
                progress.performance_metrics.update({
                    'tests_per_second': progress.completed_tests / elapsed_time,
                    'success_rate': progress.success_count / progress.completed_tests,
                    'average_test_duration': elapsed_time / progress.completed_tests,
                    'completion_percentage': (progress.completed_tests / progress.total_tests * 100) if progress.total_tests > 0 else 0
                })
    
    def _capture_failure_snapshot(self, event: MonitoringEventData):
        """Capture system state snapshot when a test fails"""
        try:
            snapshot = SystemStateSnapshot(
                timestamp=event.timestamp,
                device_serial=event.device_serial,
                test_name=event.test_name or "unknown",
                system_state=self._collect_system_state(event.device_serial),
                device_logs=self._collect_recent_logs(event.device_serial),
                communication_history=self._collect_communication_history(event.device_serial),
                performance_metrics=self._collect_performance_metrics(event.device_serial),
                error_context=event.data.get('error_message')
            )
            
            self.system_snapshots.append(snapshot)
            
            if self.log_level in [LogLevel.VERBOSE, LogLevel.DEBUG]:
                self.logger.info(f"Captured failure snapshot for {event.device_serial}:{event.test_name}")
                
        except Exception as e:
            self.logger.error(f"Failed to capture system snapshot: {e}")
    
    def _collect_system_state(self, device_serial: str) -> Dict[str, Any]:
        """Collect current system state for snapshot"""
        return {
            'timestamp': time.time(),
            'device_serial': device_serial,
            'active_test': self.active_tests.get(device_serial),
            'progress': asdict(self.device_progress.get(device_serial)) if device_serial in self.device_progress else None,
            'pending_commands': len([cmd for cmd in self.pending_commands.values() 
                                   if cmd.device_serial == device_serial])
        }
    
    def _collect_recent_logs(self, device_serial: str, max_logs: int = 50) -> List[str]:
        """Collect recent log messages for the device"""
        recent_logs = []
        
        # Get recent events for this device
        for event in list(self.event_history)[-max_logs:]:
            if event.device_serial == device_serial:
                log_entry = f"[{datetime.fromtimestamp(event.timestamp).strftime('%H:%M:%S.%f')[:-3]}] " \
                           f"{event.event_type.value}: {event.test_name or 'N/A'}"
                recent_logs.append(log_entry)
        
        return recent_logs
    
    def _collect_communication_history(self, device_serial: str, max_entries: int = 20) -> List[Dict[str, Any]]:
        """Collect recent communication history for the device"""
        comm_history = []
        
        for comm_log in list(self.communication_logs)[-max_entries:]:
            if comm_log.device_serial == device_serial:
                comm_history.append({
                    'timestamp': comm_log.timestamp,
                    'direction': comm_log.direction,
                    'message_type': comm_log.message_type,
                    'correlation_id': comm_log.correlation_id,
                    'data_summary': str(comm_log.data)[:100] + "..." if len(str(comm_log.data)) > 100 else str(comm_log.data)
                })
        
        return comm_history
    
    def _collect_performance_metrics(self, device_serial: str) -> Dict[str, float]:
        """Collect performance metrics for the device"""
        progress = self.device_progress.get(device_serial)
        if not progress:
            return {}
        
        current_time = time.time()
        elapsed_time = current_time - progress.start_time
        
        metrics = {
            'elapsed_time': elapsed_time,
            'tests_per_second': progress.completed_tests / elapsed_time if elapsed_time > 0 else 0,
            'success_rate': progress.success_count / progress.completed_tests if progress.completed_tests > 0 else 0,
            'completion_percentage': (progress.completed_tests / progress.total_tests * 100) if progress.total_tests > 0 else 0
        }
        
        if progress.estimated_completion:
            metrics['estimated_remaining_time'] = max(0, progress.estimated_completion - current_time)
        
        return metrics
    
    def _perform_health_checks(self):
        """Perform periodic health checks on all monitored devices"""
        current_time = time.time()
        
        for device_serial, progress in self.device_progress.items():
            # Check for stalled tests (no activity for too long)
            time_since_activity = current_time - progress.last_activity_time
            
            if progress.current_test and time_since_activity > 60.0:  # 1 minute threshold
                progress.health_status = "warning"
                self.logger.warning(f"[{device_serial}] Test '{progress.current_test}' may be stalled "
                                  f"(no activity for {time_since_activity:.1f}s)")
            elif time_since_activity > 300.0:  # 5 minute threshold
                progress.health_status = "error"
                self.logger.error(f"[{device_serial}] Device appears unresponsive "
                                f"(no activity for {time_since_activity:.1f}s)")
            else:
                progress.health_status = "healthy"
            
            # Check for excessive failure rates
            if progress.completed_tests > 5:  # Only check after some tests
                failure_rate = progress.failure_count / progress.completed_tests
                if failure_rate > 0.5:  # More than 50% failures
                    progress.health_status = "error"
                    self.logger.error(f"[{device_serial}] High failure rate: {failure_rate:.1%}")
                elif failure_rate > 0.2:  # More than 20% failures
                    if progress.health_status == "healthy":
                        progress.health_status = "warning"
                    self.logger.warning(f"[{device_serial}] Elevated failure rate: {failure_rate:.1%}")
        
        # Check for communication issues
        recent_comm_logs = [log for log in self.communication_logs 
                           if current_time - log.timestamp < 300.0]  # Last 5 minutes
        
        for device_serial in self.device_progress.keys():
            device_comms = [log for log in recent_comm_logs if log.device_serial == device_serial]
            sent_commands = [log for log in device_comms if log.direction == 'sent']
            received_responses = [log for log in device_comms if log.direction == 'received']
            
            # Check for command/response imbalance
            if len(sent_commands) > len(received_responses) + 5:  # Allow some pending
                progress = self.device_progress[device_serial]
                if progress.health_status == "healthy":
                    progress.health_status = "warning"
                self.logger.warning(f"[{device_serial}] Communication imbalance: "
                                  f"{len(sent_commands)} sent, {len(received_responses)} received")
    
    def _generate_periodic_status_report(self):
        """Generate periodic status reports for long-running tests"""
        if not self.device_progress:
            return
        
        if self.log_level in [LogLevel.VERBOSE, LogLevel.DEBUG]:
            self.logger.info("=== Periodic Status Report ===")
            
            for device_serial, progress in self.device_progress.items():
                if progress.total_tests == 0:
                    continue
                
                completion_pct = (progress.completed_tests / progress.total_tests * 100)
                elapsed_time = time.time() - progress.start_time
                
                status_msg = (f"[{device_serial}] {completion_pct:.1f}% complete "
                            f"({progress.completed_tests}/{progress.total_tests}) - "
                            f"Success: {progress.success_count}, Failed: {progress.failure_count} - "
                            f"Health: {progress.health_status}")
                
                if progress.current_test:
                    status_msg += f" - Current: {progress.current_test}"
                    if progress.current_test_progress > 0:
                        status_msg += f" ({progress.current_test_progress:.1%})"
                
                if progress.estimated_completion:
                    remaining_time = max(0, progress.estimated_completion - time.time())
                    status_msg += f" - ETA: {remaining_time:.0f}s"
                
                self.logger.info(status_msg)
            
            self.logger.info("=== End Status Report ===")
    
    def log_test_started(self, device_serial: str, test_name: str, total_tests: int = 0):
        """Log that a test has started"""
        event = MonitoringEventData(
            event_type=MonitoringEvent.TEST_STARTED,
            timestamp=time.time(),
            device_serial=device_serial,
            test_name=test_name,
            data={'total_tests': total_tests}
        )
        
        self.active_tests[f"{device_serial}:{test_name}"] = None
        self.event_queue.put(event)
    
    def log_test_completed(self, device_serial: str, test_name: str, execution: 'TestExecution'):
        """Log that a test has completed successfully"""
        event = MonitoringEventData(
            event_type=MonitoringEvent.TEST_COMPLETED,
            timestamp=time.time(),
            device_serial=device_serial,
            test_name=test_name,
            data={
                'duration': execution.duration,
                'status': execution.status.value,
                'retry_attempt': execution.retry_attempt
            }
        )
        
        self.active_tests.pop(f"{device_serial}:{test_name}", None)
        self.event_queue.put(event)
    
    def log_test_failed(self, device_serial: str, test_name: str, execution: 'TestExecution'):
        """Log that a test has failed"""
        event = MonitoringEventData(
            event_type=MonitoringEvent.TEST_FAILED,
            timestamp=time.time(),
            device_serial=device_serial,
            test_name=test_name,
            data={
                'duration': execution.duration,
                'status': execution.status.value,
                'error_message': execution.error_message,
                'retry_attempt': execution.retry_attempt
            }
        )
        
        self.active_tests.pop(f"{device_serial}:{test_name}", None)
        self.event_queue.put(event)
    
    def log_command_sent(self, device_serial: str, command: TestCommand) -> str:
        """Log that a command was sent to a device"""
        correlation_id = f"cmd_{self.correlation_counter:06d}"
        self.correlation_counter += 1
        self.sequence_counter += 1
        
        # Enhanced protocol details for debugging
        protocol_details = {
            'command_type_value': command.command_type.value,
            'command_id': command.command_id,
            'payload_json': json.dumps(command.payload) if self.protocol_debug_enabled else None,
            'raw_bytes_length': 64,  # HID report size
            'checksum': self._calculate_command_checksum(command) if self.protocol_debug_enabled else None
        }
        
        comm_log = CommunicationLog(
            timestamp=time.time(),
            device_serial=device_serial,
            direction='sent',
            message_type=command.command_type.name,
            data=command.payload,
            correlation_id=correlation_id,
            sequence_number=self.sequence_counter,
            protocol_details=protocol_details
        )
        
        self.communication_logs.append(comm_log)
        self.pending_commands[correlation_id] = comm_log
        
        event = MonitoringEventData(
            event_type=MonitoringEvent.COMMAND_SENT,
            timestamp=comm_log.timestamp,
            device_serial=device_serial,
            data={
                'command_type': command.command_type.name,
                'command_id': command.command_id,
                'payload_size': len(json.dumps(command.payload)),
                'sequence_number': self.sequence_counter,
                'protocol_details': protocol_details if self.protocol_debug_enabled else {}
            },
            correlation_id=correlation_id
        )
        
        self.event_queue.put(event)
        return correlation_id
    
    def log_response_received(self, device_serial: str, response: TestResponse, correlation_id: str = None):
        """Log that a response was received from a device"""
        current_time = time.time()
        
        # Calculate latency if we have the original command
        latency_ms = None
        if correlation_id and correlation_id in self.pending_commands:
            sent_log = self.pending_commands[correlation_id]
            latency_ms = (current_time - sent_log.timestamp) * 1000
        
        # Enhanced protocol details for debugging
        protocol_details = {
            'status_code': response.status.value,
            'command_id': response.command_id,
            'response_timestamp': response.timestamp,
            'data_json': json.dumps(response.data) if self.protocol_debug_enabled else None,
            'latency_ms': latency_ms
        }
        
        comm_log = CommunicationLog(
            timestamp=current_time,
            device_serial=device_serial,
            direction='received',
            message_type=response.response_type,
            data=response.data,
            correlation_id=correlation_id,
            latency_ms=latency_ms,
            protocol_details=protocol_details
        )
        
        self.communication_logs.append(comm_log)
        
        # Remove from pending if we have correlation
        if correlation_id and correlation_id in self.pending_commands:
            del self.pending_commands[correlation_id]
        
        event = MonitoringEventData(
            event_type=MonitoringEvent.RESPONSE_RECEIVED,
            timestamp=comm_log.timestamp,
            device_serial=device_serial,
            data={
                'response_type': response.response_type,
                'status': response.status.name,
                'command_id': response.command_id,
                'data_size': len(json.dumps(response.data)),
                'latency_ms': latency_ms,
                'protocol_details': protocol_details if self.protocol_debug_enabled else {}
            },
            correlation_id=correlation_id
        )
        
        self.event_queue.put(event)
    
    def log_device_communication(self, device_serial: str, message: str, direction: str = 'received', 
                                raw_bytes: bytes = None):
        """Log raw device communication with enhanced debugging info"""
        current_time = time.time()
        
        # Enhanced communication details
        comm_details = {
            'direction': direction,
            'message': message,
            'message_length': len(message),
            'timestamp_precise': current_time,
            'raw_bytes_length': len(raw_bytes) if raw_bytes else None
        }
        
        # Add protocol analysis for debug mode
        if self.protocol_debug_enabled:
            comm_details.update({
                'raw_bytes_hex': raw_bytes.hex() if raw_bytes else None,
                'message_type_detected': self._detect_message_type(message),
                'contains_json': self._contains_json(message),
                'log_level_detected': self._detect_log_level(message)
            })
        
        event = MonitoringEventData(
            event_type=MonitoringEvent.DEVICE_COMMUNICATION,
            timestamp=current_time,
            device_serial=device_serial,
            data=comm_details
        )
        
        self.event_queue.put(event)
    
    def _calculate_command_checksum(self, command) -> int:
        """Calculate command checksum for protocol debugging"""
        try:
            return (command.command_type.value + command.command_id + 
                   len(json.dumps(command.payload).encode('utf-8'))) & 0xFF
        except:
            return 0
    
    def _detect_message_type(self, message: str) -> str:
        """Detect the type of log message for debugging"""
        if message.startswith("TEST_RESPONSE:"):
            return "test_response"
        elif message.startswith("LOG:"):
            return "log_message"
        elif message.startswith("ERROR:"):
            return "error_message"
        elif message.startswith("DEBUG:"):
            return "debug_message"
        elif message.startswith("WARN:"):
            return "warning_message"
        else:
            return "unknown"
    
    def _contains_json(self, message: str) -> bool:
        """Check if message contains JSON data"""
        try:
            if ":" in message:
                json_part = message.split(":", 1)[1].strip()
                json.loads(json_part)
                return True
        except:
            pass
        return False
    
    def _detect_log_level(self, message: str) -> str:
        """Detect log level from message"""
        if message.startswith("ERROR:"):
            return "error"
        elif message.startswith("WARN:"):
            return "warning"
        elif message.startswith("DEBUG:"):
            return "debug"
        elif message.startswith("LOG:"):
            return "info"
        else:
            return "unknown"
    
    def register_event_callback(self, event_type: MonitoringEvent, callback: Callable[[MonitoringEventData], None]):
        """Register a callback for specific monitoring events"""
        if event_type not in self.event_callbacks:
            self.event_callbacks[event_type] = []
        self.event_callbacks[event_type].append(callback)
    
    def get_device_progress(self, device_serial: str) -> Optional[ProgressStatus]:
        """Get current progress status for a device"""
        return self.device_progress.get(device_serial)
    
    def get_all_progress(self) -> Dict[str, ProgressStatus]:
        """Get progress status for all devices"""
        return self.device_progress.copy()
    
    def get_communication_logs(self, device_serial: str = None, max_entries: int = 100) -> List[CommunicationLog]:
        """Get communication logs, optionally filtered by device"""
        logs = list(self.communication_logs)
        
        if device_serial:
            logs = [log for log in logs if log.device_serial == device_serial]
        
        return logs[-max_entries:]
    
    def get_system_snapshots(self, device_serial: str = None) -> List[SystemStateSnapshot]:
        """Get system state snapshots, optionally filtered by device"""
        snapshots = self.system_snapshots
        
        if device_serial:
            snapshots = [snap for snap in snapshots if snap.device_serial == device_serial]
        
        return snapshots
    
    def get_event_history(self, device_serial: str = None, event_types: List[MonitoringEvent] = None,
                         max_entries: int = 100) -> List[MonitoringEventData]:
        """Get event history with optional filtering"""
        events = list(self.event_history)
        
        if device_serial:
            events = [event for event in events if event.device_serial == device_serial]
        
        if event_types:
            events = [event for event in events if event.event_type in event_types]
        
        return events[-max_entries:]
    
    def get_enhanced_failure_analysis(self, device_serial: str = None) -> Dict[str, Any]:
        """Get enhanced failure analysis with detailed debugging information"""
        snapshots = self.get_system_snapshots(device_serial)
        events = self.get_event_history(device_serial, [MonitoringEvent.TEST_FAILED])
        
        analysis = {
            'total_failures': len(snapshots),
            'failure_timeline': [],
            'failure_patterns': {},
            'communication_issues': [],
            'performance_degradation': [],
            'error_categories': {},
            'recovery_suggestions': []
        }
        
        # Analyze failure timeline
        for snapshot in snapshots:
            failure_info = {
                'timestamp': snapshot.timestamp,
                'device': snapshot.device_serial,
                'test': snapshot.test_name,
                'error': snapshot.error_context,
                'system_health': snapshot.system_state.get('progress', {}).get('health_status', 'unknown'),
                'performance_at_failure': snapshot.performance_metrics
            }
            analysis['failure_timeline'].append(failure_info)
        
        # Analyze failure patterns
        for event in events:
            test_name = event.test_name or 'unknown'
            if test_name not in analysis['failure_patterns']:
                analysis['failure_patterns'][test_name] = {
                    'count': 0,
                    'devices': set(),
                    'error_types': {},
                    'avg_time_to_failure': 0
                }
            
            pattern = analysis['failure_patterns'][test_name]
            pattern['count'] += 1
            pattern['devices'].add(event.device_serial)
            
            error_msg = event.data.get('error_message', 'unknown')
            error_type = self._categorize_error(error_msg)
            pattern['error_types'][error_type] = pattern['error_types'].get(error_type, 0) + 1
        
        # Convert sets to lists for JSON serialization
        for pattern in analysis['failure_patterns'].values():
            pattern['devices'] = list(pattern['devices'])
        
        # Analyze communication issues
        recent_comms = [log for log in self.communication_logs 
                       if time.time() - log.timestamp < 300.0]  # Last 5 minutes
        
        for device in (self.device_progress.keys() if not device_serial else [device_serial]):
            device_comms = [log for log in recent_comms if log.device_serial == device]
            
            # Check for high latency
            high_latency_comms = [log for log in device_comms 
                                if log.latency_ms and log.latency_ms > 1000]  # > 1 second
            
            if high_latency_comms:
                analysis['communication_issues'].append({
                    'device': device,
                    'issue_type': 'high_latency',
                    'count': len(high_latency_comms),
                    'max_latency_ms': max(log.latency_ms for log in high_latency_comms),
                    'avg_latency_ms': sum(log.latency_ms for log in high_latency_comms) / len(high_latency_comms)
                })
            
            # Check for missing responses
            sent_commands = [log for log in device_comms if log.direction == 'sent']
            received_responses = [log for log in device_comms if log.direction == 'received']
            
            if len(sent_commands) > len(received_responses) + 3:  # Allow some pending
                analysis['communication_issues'].append({
                    'device': device,
                    'issue_type': 'missing_responses',
                    'sent_count': len(sent_commands),
                    'received_count': len(received_responses),
                    'missing_count': len(sent_commands) - len(received_responses)
                })
        
        # Generate recovery suggestions
        analysis['recovery_suggestions'] = self._generate_recovery_suggestions(analysis)
        
        return analysis
    
    def _categorize_error(self, error_message: str) -> str:
        """Categorize error message for pattern analysis"""
        error_lower = error_message.lower()
        
        if 'timeout' in error_lower:
            return 'timeout'
        elif 'communication' in error_lower or 'usb' in error_lower:
            return 'communication'
        elif 'hardware' in error_lower or 'adc' in error_lower or 'gpio' in error_lower:
            return 'hardware'
        elif 'memory' in error_lower or 'allocation' in error_lower:
            return 'memory'
        elif 'validation' in error_lower or 'assertion' in error_lower:
            return 'validation'
        else:
            return 'other'
    
    def _generate_recovery_suggestions(self, analysis: Dict[str, Any]) -> List[str]:
        """Generate recovery suggestions based on failure analysis"""
        suggestions = []
        
        # Timeout-related suggestions
        timeout_failures = sum(1 for pattern in analysis['failure_patterns'].values() 
                             for error_type, count in pattern['error_types'].items() 
                             if error_type == 'timeout')
        
        if timeout_failures > 0:
            suggestions.append("Consider increasing test timeouts or checking for device responsiveness issues")
        
        # Communication-related suggestions
        comm_issues = len(analysis['communication_issues'])
        if comm_issues > 0:
            suggestions.append("Check USB connections and consider device reset or reconnection")
        
        # High failure rate suggestions
        total_failures = analysis['total_failures']
        if total_failures > 10:
            suggestions.append("High failure count detected - consider hardware validation or firmware debugging")
        
        # Pattern-specific suggestions
        for test_name, pattern in analysis['failure_patterns'].items():
            if pattern['count'] > 3:
                suggestions.append(f"Test '{test_name}' failing repeatedly - review test parameters and implementation")
        
        return suggestions
    
    def get_real_time_debug_info(self, device_serial: str = None) -> Dict[str, Any]:
        """Get comprehensive real-time debugging information"""
        current_time = time.time()
        
        debug_info = {
            'timestamp': current_time,
            'monitoring_status': {
                'active': self.monitoring_active,
                'log_level': self.log_level.value,
                'protocol_debug_enabled': self.protocol_debug_enabled,
                'snapshots_enabled': self.enable_snapshots
            },
            'system_health': {},
            'communication_stats': {},
            'performance_metrics': {},
            'recent_activity': []
        }
        
        # System health for each device
        devices = [device_serial] if device_serial else list(self.device_progress.keys())
        
        for device in devices:
            progress = self.device_progress.get(device)
            if not progress:
                continue
                
            debug_info['system_health'][device] = {
                'health_status': progress.health_status,
                'current_test': progress.current_test,
                'test_progress': progress.current_test_progress,
                'last_activity': current_time - progress.last_activity_time,
                'completion_percentage': progress.performance_metrics.get('completion_percentage', 0),
                'success_rate': progress.performance_metrics.get('success_rate', 0)
            }
        
        # Communication statistics
        recent_comms = [log for log in self.communication_logs 
                       if current_time - log.timestamp < 60.0]  # Last minute
        
        for device in devices:
            device_comms = [log for log in recent_comms if log.device_serial == device]
            
            if device_comms:
                latencies = [log.latency_ms for log in device_comms if log.latency_ms]
                
                debug_info['communication_stats'][device] = {
                    'total_messages': len(device_comms),
                    'sent_commands': len([log for log in device_comms if log.direction == 'sent']),
                    'received_responses': len([log for log in device_comms if log.direction == 'received']),
                    'avg_latency_ms': sum(latencies) / len(latencies) if latencies else None,
                    'max_latency_ms': max(latencies) if latencies else None,
                    'pending_commands': len([cmd for cmd in self.pending_commands.values() 
                                           if cmd.device_serial == device])
                }
        
        # Recent activity (last 10 events)
        recent_events = list(self.event_history)[-10:]
        for event in recent_events:
            if not device_serial or event.device_serial == device_serial:
                debug_info['recent_activity'].append({
                    'timestamp': event.timestamp,
                    'device': event.device_serial,
                    'event_type': event.event_type.value,
                    'test_name': event.test_name,
                    'summary': self._summarize_event_data(event.data)
                })
        
        return debug_info
    
    def export_monitoring_data(self, filepath: str):
        """Export all monitoring data to a JSON file"""
        export_data = {
            'metadata': {
                'export_timestamp': time.time(),
                'log_level': self.log_level.value,
                'protocol_debug_enabled': self.protocol_debug_enabled,
                'snapshots_enabled': self.enable_snapshots,
                'monitoring_duration': time.time() - min(p.start_time for p in self.device_progress.values()) if self.device_progress else 0,
                'health_check_interval': self.health_check_interval,
                'status_report_interval': self.periodic_status_interval
            },
            'device_progress': {serial: asdict(progress) for serial, progress in self.device_progress.items()},
            'event_history': [asdict(event) for event in self.event_history],
            'communication_logs': [asdict(log) for log in self.communication_logs],
            'system_snapshots': [asdict(snapshot) for snapshot in self.system_snapshots],
            'enhanced_analysis': self.get_enhanced_failure_analysis(),
            'real_time_debug_info': self.get_real_time_debug_info()
        }
        
        with open(filepath, 'w') as f:
            json.dump(export_data, f, indent=2, default=str)
        
        self.logger.info(f"Enhanced monitoring data exported to {filepath}")


# Helper function to convert dataclass to dict
try:
    from dataclasses import asdict
except ImportError:
    def asdict(obj):
        """Convert dataclass to dictionary, handling nested objects"""
        if hasattr(obj, '__dict__'):
            result = {}
            for key, value in obj.__dict__.items():
                if hasattr(value, '__dict__') and not isinstance(value, (str, int, float, bool)):
                    result[key] = asdict(value)
                elif isinstance(value, list):
                    result[key] = [asdict(item) if hasattr(item, '__dict__') and not isinstance(item, (str, int, float, bool)) else item for item in value]
                elif isinstance(value, dict):
                    result[key] = {k: asdict(v) if hasattr(v, '__dict__') and not isinstance(v, (str, int, float, bool)) else v for k, v in value.items()}
                else:
                    result[key] = value
            return result
        else:
            return obj