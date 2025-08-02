"""
Real-time Test Monitoring and Debugging

Provides real-time test progress reporting, verbose logging modes, failure point capture,
and device communication logging with timestamp correlation for debugging purposes.
"""

import time
import threading
import logging
import json
from typing import Dict, List, Any, Optional, Callable, Set
from dataclasses import dataclass, field
from enum import Enum
from collections import deque
import queue
from datetime import datetime

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
                    continue
                    
                # Update progress estimates
                self._update_progress_estimates()
                
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
            if 'total_tests' in event.data:
                progress.total_tests = event.data['total_tests']
                
        elif event.event_type == MonitoringEvent.TEST_COMPLETED:
            progress.completed_tests += 1
            progress.success_count += 1
            progress.current_test = None
            progress.current_status = "completed" if progress.completed_tests >= progress.total_tests else "running"
            
        elif event.event_type == MonitoringEvent.TEST_FAILED:
            progress.completed_tests += 1
            progress.failure_count += 1
            progress.current_test = None
            progress.current_status = "completed" if progress.completed_tests >= progress.total_tests else "running"
    
    def _update_progress_estimates(self):
        """Update estimated completion times for active tests"""
        current_time = time.time()
        
        for progress in self.device_progress.values():
            if progress.completed_tests > 0 and progress.total_tests > 0:
                elapsed_time = current_time - progress.start_time
                avg_time_per_test = elapsed_time / progress.completed_tests
                remaining_tests = progress.total_tests - progress.completed_tests
                progress.estimated_completion = current_time + (avg_time_per_test * remaining_tests)
    
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
    
    def log_test_completed(self, device_serial: str, test_name: str, execution: TestExecution):
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
    
    def log_test_failed(self, device_serial: str, test_name: str, execution: TestExecution):
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
        
        comm_log = CommunicationLog(
            timestamp=time.time(),
            device_serial=device_serial,
            direction='sent',
            message_type=command.command_type.name,
            data=command.payload,
            correlation_id=correlation_id
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
                'payload_size': len(json.dumps(command.payload))
            },
            correlation_id=correlation_id
        )
        
        self.event_queue.put(event)
        return correlation_id
    
    def log_response_received(self, device_serial: str, response: TestResponse, correlation_id: str = None):
        """Log that a response was received from a device"""
        comm_log = CommunicationLog(
            timestamp=time.time(),
            device_serial=device_serial,
            direction='received',
            message_type=response.response_type,
            data=response.data,
            correlation_id=correlation_id
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
                'data_size': len(json.dumps(response.data))
            },
            correlation_id=correlation_id
        )
        
        self.event_queue.put(event)
    
    def log_device_communication(self, device_serial: str, message: str, direction: str = 'received'):
        """Log raw device communication"""
        event = MonitoringEventData(
            event_type=MonitoringEvent.DEVICE_COMMUNICATION,
            timestamp=time.time(),
            device_serial=device_serial,
            data={
                'direction': direction,
                'message': message,
                'message_length': len(message)
            }
        )
        
        self.event_queue.put(event)
    
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
    
    def export_monitoring_data(self, filepath: str):
        """Export all monitoring data to a JSON file"""
        export_data = {
            'metadata': {
                'export_timestamp': time.time(),
                'log_level': self.log_level.value,
                'monitoring_duration': time.time() - min(p.start_time for p in self.device_progress.values()) if self.device_progress else 0
            },
            'device_progress': {serial: asdict(progress) for serial, progress in self.device_progress.items()},
            'event_history': [asdict(event) for event in self.event_history],
            'communication_logs': [asdict(log) for log in self.communication_logs],
            'system_snapshots': [asdict(snapshot) for snapshot in self.system_snapshots]
        }
        
        with open(filepath, 'w') as f:
            json.dump(export_data, f, indent=2, default=str)
        
        self.logger.info(f"Monitoring data exported to {filepath}")


# Helper function to convert dataclass to dict
def asdict(obj):
    """Convert dataclass to dictionary, handling nested objects"""
    if hasattr(obj, '__dict__'):
        result = {}
        for key, value in obj.__dict__.items():
            if hasattr(value, '__dict__'):
                result[key] = asdict(value)
            elif isinstance(value, list):
                result[key] = [asdict(item) if hasattr(item, '__dict__') else item for item in value]
            elif isinstance(value, dict):
                result[key] = {k: asdict(v) if hasattr(v, '__dict__') else v for k, v in value.items()}
            else:
                result[key] = value
        return result
    else:
        return obj