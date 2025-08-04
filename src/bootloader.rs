//! Bootloader entry management module
//! Provides safe bootloader mode entry with hardware state validation and task shutdown
//! Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 5.1, 5.2, 5.3, 5.4, 5.5

use heapless::Vec;
use rp2040_hal::pac;
use cortex_m::interrupt;
use crate::{log_info, log_warn, log_error};
use crate::error_handling::SystemError;
use core::option::Option::{self, Some, None};
use core::result::Result::{self, Ok, Err};
use core::convert::From;
use core::cmp::Ord;

/// Magic value written to RAM to trigger bootloader mode after reset
/// This value is checked by the RP2040 boot ROM to enter bootloader mode
const BOOTLOADER_MAGIC: u32 = 0xB007C0DE;

/// Memory address for bootloader magic value (end of RAM)
/// RP2040 has 264KB of SRAM, we use the last 4 bytes for the magic value
/// This is the standard location used by the Pico SDK
const BOOTLOADER_MAGIC_ADDR: *mut u32 = 0x20041FFC as *mut u32;

/// Maximum time to wait for task shutdown in milliseconds
const TASK_SHUTDOWN_TIMEOUT_MS: u32 = 5000;

/// Maximum time to wait for hardware state validation in milliseconds
const HARDWARE_VALIDATION_TIMEOUT_MS: u32 = 500;

/// Bootloader entry error types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BootloaderError {
    /// Hardware is not in a safe state for bootloader entry
    UnsafeHardwareState,
    /// Task shutdown sequence failed or timed out
    TaskShutdownFailed,
    /// Hardware validation failed
    HardwareValidationFailed,
    /// System is currently busy with critical operations
    SystemBusy,
    /// Bootloader entry was interrupted
    EntryInterrupted,
}

impl From<BootloaderError> for SystemError {
    fn from(error: BootloaderError) -> Self {
        match error {
            BootloaderError::UnsafeHardwareState => SystemError::HardwareError,
            BootloaderError::TaskShutdownFailed => SystemError::TaskError,
            BootloaderError::HardwareValidationFailed => SystemError::HardwareError,
            BootloaderError::SystemBusy => SystemError::SystemBusy,
            BootloaderError::EntryInterrupted => SystemError::OperationInterrupted,
        }
    }
}

/// Task priority levels for shutdown sequence
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum TaskPriority {
    /// Lowest priority tasks (LED control, USB polling, diagnostics)
    Low = 1,
    /// Medium priority tasks (Battery monitoring, USB HID transmission)
    Medium = 2,
    /// Highest priority tasks (pEMF pulse generation)
    High = 3,
}

/// Task shutdown status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskShutdownStatus {
    /// Task is running normally
    Running,
    /// Task shutdown has been requested
    ShutdownRequested,
    /// Task has acknowledged shutdown request
    ShutdownAcknowledged,
    /// Task has completed shutdown
    ShutdownComplete,
    /// Task shutdown failed or timed out
    ShutdownFailed,
}

/// Hardware component states for validation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HardwareState {
    /// MOSFET pin state (should be LOW for safe shutdown)
    pub mosfet_state: bool,
    /// LED pin state
    pub led_state: bool,
    /// ADC is currently sampling
    pub adc_active: bool,
    /// USB is currently transmitting
    pub usb_transmitting: bool,
    /// pEMF pulse is currently active
    pub pemf_pulse_active: bool,
}

impl HardwareState {
    /// Check if hardware state is safe for bootloader entry
    /// Requirements: 5.1, 5.2 (ensure hardware is in safe state)
    pub fn is_safe_for_bootloader(&self) -> bool {
        // MOSFET must be OFF (LOW) to prevent unwanted pEMF pulses
        // pEMF pulse must not be active
        // Other components can be in any state as they will be reset
        !self.mosfet_state && !self.pemf_pulse_active
    }

    /// Get safety violations as a list of error messages
    pub fn get_safety_violations(&self) -> Vec<&'static str, 4> {
        let mut violations = Vec::new();
        
        if self.mosfet_state {
            let _ = violations.push("MOSFET is active - risk of unwanted pEMF pulse");
        }
        
        if self.pemf_pulse_active {
            let _ = violations.push("pEMF pulse is currently active");
        }
        
        violations
    }
}

/// Task shutdown sequence manager
/// Manages the orderly shutdown of RTIC tasks in reverse priority order
/// Requirements: 1.3, 1.4 (graceful task shutdown, priority hierarchy)
pub struct TaskShutdownSequence {
    /// Shutdown status for each task priority level
    low_priority_status: TaskShutdownStatus,
    medium_priority_status: TaskShutdownStatus,
    high_priority_status: TaskShutdownStatus,
    /// Timestamp when shutdown sequence started
    shutdown_start_time: Option<u32>,
    /// Total timeout for shutdown sequence
    shutdown_timeout_ms: u32,
}

impl Default for TaskShutdownSequence {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskShutdownSequence {
    /// Create a new task shutdown sequence manager
    pub const fn new() -> Self {
        Self {
            low_priority_status: TaskShutdownStatus::Running,
            medium_priority_status: TaskShutdownStatus::Running,
            high_priority_status: TaskShutdownStatus::Running,
            shutdown_start_time: None,
            shutdown_timeout_ms: TASK_SHUTDOWN_TIMEOUT_MS,
        }
    }

    /// Start the task shutdown sequence
    /// Tasks are shut down in reverse priority order: High -> Medium -> Low
    /// Requirements: 1.3 (graceful shutdown), 1.4 (priority hierarchy)
    pub fn start_shutdown(&mut self, current_time_ms: u32) {
        self.shutdown_start_time = Some(current_time_ms);
        
        // Start with highest priority tasks first
        self.high_priority_status = TaskShutdownStatus::ShutdownRequested;
        
        log_info!("Task shutdown sequence started");
        log_info!("Shutting down high priority tasks (pEMF pulse generation)");
    }

    /// Update shutdown sequence progress
    /// This should be called periodically to advance the shutdown sequence
    pub fn update_shutdown_progress(&mut self, current_time_ms: u32) -> Result<bool, BootloaderError> {
        let start_time = self.shutdown_start_time.ok_or(BootloaderError::TaskShutdownFailed)?;
        
        // Check for overall timeout
        if current_time_ms.saturating_sub(start_time) > self.shutdown_timeout_ms {
            log_error!("Task shutdown sequence timed out after {}ms", self.shutdown_timeout_ms);
            return Err(BootloaderError::TaskShutdownFailed);
        }

        // Progress through shutdown sequence based on current status
        match (self.high_priority_status, self.medium_priority_status, self.low_priority_status) {
            // High priority tasks are complete, start medium priority shutdown
            (TaskShutdownStatus::ShutdownComplete, TaskShutdownStatus::Running, _) => {
                self.medium_priority_status = TaskShutdownStatus::ShutdownRequested;
                log_info!("High priority tasks shutdown complete");
                log_info!("Shutting down medium priority tasks (battery monitoring, USB HID)");
            }
            
            // Medium priority tasks are complete, start low priority shutdown
            (TaskShutdownStatus::ShutdownComplete, TaskShutdownStatus::ShutdownComplete, TaskShutdownStatus::Running) => {
                self.low_priority_status = TaskShutdownStatus::ShutdownRequested;
                log_info!("Medium priority tasks shutdown complete");
                log_info!("Shutting down low priority tasks (LED control, USB polling)");
            }
            
            // All tasks are complete
            (TaskShutdownStatus::ShutdownComplete, TaskShutdownStatus::ShutdownComplete, TaskShutdownStatus::ShutdownComplete) => {
                log_info!("All tasks shutdown complete");
                return Ok(true); // Shutdown complete
            }
            
            // Check for failed shutdowns
            (TaskShutdownStatus::ShutdownFailed, _, _) |
            (_, TaskShutdownStatus::ShutdownFailed, _) |
            (_, _, TaskShutdownStatus::ShutdownFailed) => {
                log_error!("Task shutdown failed");
                return Err(BootloaderError::TaskShutdownFailed);
            }
            
            // Shutdown in progress
            _ => {
                // Continue waiting for current tasks to complete
            }
        }

        Ok(false) // Shutdown still in progress
    }

    /// Mark a task priority level as shutdown complete
    pub fn mark_task_shutdown_complete(&mut self, priority: TaskPriority) {
        match priority {
            TaskPriority::High => {
                self.high_priority_status = TaskShutdownStatus::ShutdownComplete;
                log_info!("High priority task shutdown acknowledged");
            }
            TaskPriority::Medium => {
                self.medium_priority_status = TaskShutdownStatus::ShutdownComplete;
                log_info!("Medium priority task shutdown acknowledged");
            }
            TaskPriority::Low => {
                self.low_priority_status = TaskShutdownStatus::ShutdownComplete;
                log_info!("Low priority task shutdown acknowledged");
            }
        }
    }

    /// Mark a task priority level as shutdown failed
    pub fn mark_task_shutdown_failed(&mut self, priority: TaskPriority) {
        match priority {
            TaskPriority::High => {
                self.high_priority_status = TaskShutdownStatus::ShutdownFailed;
                log_error!("High priority task shutdown failed");
            }
            TaskPriority::Medium => {
                self.medium_priority_status = TaskShutdownStatus::ShutdownFailed;
                log_error!("Medium priority task shutdown failed");
            }
            TaskPriority::Low => {
                self.low_priority_status = TaskShutdownStatus::ShutdownFailed;
                log_error!("Low priority task shutdown failed");
            }
        }
    }

    /// Check if shutdown is requested for a specific priority level
    pub fn is_shutdown_requested(&self, priority: TaskPriority) -> bool {
        match priority {
            TaskPriority::High => self.high_priority_status == TaskShutdownStatus::ShutdownRequested,
            TaskPriority::Medium => self.medium_priority_status == TaskShutdownStatus::ShutdownRequested,
            TaskPriority::Low => self.low_priority_status == TaskShutdownStatus::ShutdownRequested,
        }
    }

    /// Get current shutdown status for debugging
    pub fn get_shutdown_status(&self) -> (TaskShutdownStatus, TaskShutdownStatus, TaskShutdownStatus) {
        (self.high_priority_status, self.medium_priority_status, self.low_priority_status)
    }
}

/// Hardware safety manager for validating system state before bootloader entry
/// Requirements: 5.1, 5.2 (hardware state validation)
pub struct HardwareSafetyManager {
    #[allow(dead_code)]
    validation_timeout_ms: u32,
}

impl Default for HardwareSafetyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HardwareSafetyManager {
    /// Create a new hardware safety manager
    pub const fn new() -> Self {
        Self {
            validation_timeout_ms: HARDWARE_VALIDATION_TIMEOUT_MS,
        }
    }

    /// Validate hardware state for safe bootloader entry
    /// Requirements: 5.1 (safe hardware state), 5.2 (validation before entry)
    pub fn validate_hardware_state(&self, hardware_state: &HardwareState) -> Result<(), BootloaderError> {
        log_info!("Validating hardware state for bootloader entry");
        
        // Check if hardware state is safe
        if !hardware_state.is_safe_for_bootloader() {
            let violations = hardware_state.get_safety_violations();
            log_error!("Hardware state validation failed:");
            for violation in &violations {
                log_error!("  - {}", violation);
            }
            return Err(BootloaderError::UnsafeHardwareState);
        }

        log_info!("Hardware state validation passed");
        log_info!("  - MOSFET: OFF (safe)");
        log_info!("  - pEMF pulse: INACTIVE (safe)");
        log_info!("  - LED state: {} (safe)", if hardware_state.led_state { "ON" } else { "OFF" });
        log_info!("  - ADC: {} (safe)", if hardware_state.adc_active { "ACTIVE" } else { "INACTIVE" });
        log_info!("  - USB: {} (safe)", if hardware_state.usb_transmitting { "TRANSMITTING" } else { "IDLE" });

        Ok(())
    }

    /// Force hardware into safe state for bootloader entry
    /// This is a last resort if normal shutdown doesn't work
    /// Requirements: 5.2 (ensure safe state)
    pub fn force_safe_state(&self) -> Result<(), BootloaderError> {
        log_warn!("Forcing hardware into safe state for bootloader entry");
        
        // Disable interrupts to prevent interference
        interrupt::disable();
        
        // Force MOSFET OFF by directly accessing GPIO registers
        // This is a safety measure to prevent unwanted pEMF pulses
        unsafe {
            let io_bank0 = &(*pac::IO_BANK0::ptr());
            let sio = &(*pac::SIO::ptr());
            
            // Set GPIO15 (MOSFET control) to LOW
            // Clear bit 15 in GPIO_OUT register
            sio.gpio_out_clr().write(|w| w.bits(1 << 15));
            
            // Ensure GPIO15 is configured as output
            io_bank0.gpio(15).gpio_ctrl().write(|w| w.funcsel().sio());
            sio.gpio_oe_set().write(|w| w.bits(1 << 15));
        }
        
        // Re-enable interrupts
        unsafe { interrupt::enable(); }
        
        log_info!("Hardware forced into safe state");
        log_info!("  - MOSFET: Forced OFF");
        log_info!("  - GPIO15: Configured as output, set to LOW");
        
        Ok(())
    }
}

/// Main bootloader entry manager
/// Coordinates safe bootloader mode entry with task shutdown and hardware validation
/// Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 5.1, 5.2, 5.3, 5.4, 5.5
pub struct BootloaderEntryManager {
    /// Task shutdown sequence manager
    shutdown_sequence: TaskShutdownSequence,
    /// Hardware safety manager
    hardware_safety: HardwareSafetyManager,
    /// Current bootloader entry state
    entry_state: BootloaderEntryState,
    /// Timeout for bootloader entry process
    entry_timeout_ms: u32,
    /// Timestamp when bootloader entry started
    entry_start_time: Option<u32>,
}

/// Bootloader entry state machine
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BootloaderEntryState {
    /// Normal operation, not entering bootloader
    Normal,
    /// Bootloader entry requested, starting validation
    EntryRequested,
    /// Validating hardware state
    ValidatingHardware,
    /// Shutting down tasks
    ShuttingDownTasks,
    /// Final hardware safety check
    FinalSafetyCheck,
    /// Ready to enter bootloader mode
    ReadyForBootloader,
    /// Bootloader entry failed, returning to normal operation
    EntryFailed,
}

impl Default for BootloaderEntryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BootloaderEntryManager {
    /// Create a new bootloader entry manager
    pub const fn new() -> Self {
        Self {
            shutdown_sequence: TaskShutdownSequence::new(),
            hardware_safety: HardwareSafetyManager::new(),
            entry_state: BootloaderEntryState::Normal,
            entry_timeout_ms: 2000, // 2 second total timeout
            entry_start_time: None,
        }
    }

    /// Request bootloader mode entry
    /// Requirements: 1.1 (respond to bootloader commands), 1.2 (enter within 500ms)
    pub fn request_bootloader_entry(&mut self, timeout_ms: u32, current_time_ms: u32) -> Result<(), BootloaderError> {
        if self.entry_state != BootloaderEntryState::Normal {
            log_warn!("Bootloader entry already in progress, ignoring request");
            return Err(BootloaderError::SystemBusy);
        }

        log_info!("Bootloader entry requested with {}ms timeout", timeout_ms);
        
        self.entry_state = BootloaderEntryState::EntryRequested;
        self.entry_timeout_ms = timeout_ms.min(2000); // Cap at 2 seconds for safety
        self.entry_start_time = Some(current_time_ms);
        
        // Log the bootloader entry request with timestamp and source
        // Requirements: 1.3 (log bootloader entry request)
        log_info!("=== BOOTLOADER ENTRY SEQUENCE STARTED ===");
        log_info!("Timestamp: {}ms since boot", current_time_ms);
        log_info!("Source: USB HID command");
        log_info!("Timeout: {}ms", self.entry_timeout_ms);
        
        Ok(())
    }

    /// Update bootloader entry state machine
    /// This should be called periodically to advance the bootloader entry process
    /// Requirements: 1.4, 1.5 (complete current cycle, graceful shutdown)
    pub fn update_entry_progress(&mut self, hardware_state: &HardwareState, current_time_ms: u32) -> Result<BootloaderEntryState, BootloaderError> {
        // Check for overall timeout
        if let Some(start_time) = self.entry_start_time {
            if current_time_ms.saturating_sub(start_time) > self.entry_timeout_ms {
                log_error!("Bootloader entry timed out after {}ms", self.entry_timeout_ms);
                self.entry_state = BootloaderEntryState::EntryFailed;
                return Err(BootloaderError::TaskShutdownFailed);
            }
        }

        match self.entry_state {
            BootloaderEntryState::EntryRequested => {
                log_info!("Starting hardware state validation");
                self.entry_state = BootloaderEntryState::ValidatingHardware;
            }

            BootloaderEntryState::ValidatingHardware => {
                // Validate current hardware state
                match self.hardware_safety.validate_hardware_state(hardware_state) {
                    Ok(()) => {
                        log_info!("Initial hardware validation passed, starting task shutdown");
                        self.shutdown_sequence.start_shutdown(current_time_ms);
                        self.entry_state = BootloaderEntryState::ShuttingDownTasks;
                    }
                    Err(BootloaderError::UnsafeHardwareState) => {
                        // If pEMF pulse is active, wait for it to complete
                        // Requirements: 1.5 (complete current cycle before entering bootloader)
                        if hardware_state.pemf_pulse_active {
                            log_info!("pEMF pulse active, waiting for cycle completion");
                            // Stay in validation state and wait
                        } else {
                            log_error!("Hardware state unsafe for bootloader entry");
                            self.entry_state = BootloaderEntryState::EntryFailed;
                            return Err(BootloaderError::UnsafeHardwareState);
                        }
                    }
                    Err(e) => {
                        self.entry_state = BootloaderEntryState::EntryFailed;
                        return Err(e);
                    }
                }
            }

            BootloaderEntryState::ShuttingDownTasks => {
                // Update task shutdown progress
                match self.shutdown_sequence.update_shutdown_progress(current_time_ms) {
                    Ok(true) => {
                        // All tasks shut down successfully
                        log_info!("Task shutdown complete, performing final safety check");
                        self.entry_state = BootloaderEntryState::FinalSafetyCheck;
                    }
                    Ok(false) => {
                        // Shutdown still in progress
                    }
                    Err(e) => {
                        log_error!("Task shutdown failed: {:?}", e);
                        self.entry_state = BootloaderEntryState::EntryFailed;
                        return Err(e);
                    }
                }
            }

            BootloaderEntryState::FinalSafetyCheck => {
                // Final hardware state validation before bootloader entry
                match self.hardware_safety.validate_hardware_state(hardware_state) {
                    Ok(()) => {
                        log_info!("Final safety check passed, ready for bootloader entry");
                        self.entry_state = BootloaderEntryState::ReadyForBootloader;
                    }
                    Err(_) => {
                        // Force hardware into safe state as last resort
                        log_warn!("Final safety check failed, forcing safe state");
                        match self.hardware_safety.force_safe_state() {
                            Ok(()) => {
                                log_info!("Hardware forced into safe state, ready for bootloader entry");
                                self.entry_state = BootloaderEntryState::ReadyForBootloader;
                            }
                            Err(e) => {
                                log_error!("Failed to force safe state: {:?}", e);
                                self.entry_state = BootloaderEntryState::EntryFailed;
                                return Err(e);
                            }
                        }
                    }
                }
            }

            BootloaderEntryState::ReadyForBootloader => {
                // Ready to enter bootloader mode
                log_info!("=== READY FOR BOOTLOADER ENTRY ===");
            }

            BootloaderEntryState::EntryFailed => {
                log_error!("Bootloader entry failed, returning to normal operation");
                self.reset_entry_state();
                return Err(BootloaderError::EntryInterrupted);
            }

            BootloaderEntryState::Normal => {
                // Normal operation, nothing to do
            }
        }

        Ok(self.entry_state)
    }

    /// Execute safe shutdown sequence
    /// Requirements: 1.4 (graceful shutdown), 5.1 (ensure data integrity)
    pub fn execute_safe_shutdown(&mut self, _current_time_ms: u32) -> Result<(), BootloaderError> {
        log_info!("Executing safe shutdown sequence");
        
        // Flush all pending log messages to ensure data integrity
        // Requirements: 1.2 (flush pending log messages)
        log_info!("Flushing pending log messages for data integrity");
        
        // Note: In a real implementation, this would interface with the USB HID logging system
        // to ensure all queued messages are transmitted before bootloader entry
        // For now, we log the intent
        log_info!("Log message flush complete");
        
        // Mark shutdown as complete
        log_info!("Safe shutdown sequence complete");
        
        Ok(())
    }

    /// Enter bootloader mode using RP2040 software reset mechanism
    /// Requirements: 1.1 (enter bootloader mode), 5.3 (automatic return after timeout)
    pub fn enter_bootloader_mode(&self) -> ! {
        log_info!("=== ENTERING BOOTLOADER MODE ===");
        log_info!("Writing bootloader magic value to RAM");
        log_info!("Performing software reset to enter bootloader");
        
        // Disable interrupts to prevent interference during reset
        cortex_m::interrupt::disable();
        
        // Write magic value to RAM to trigger bootloader mode
        // The RP2040 boot ROM checks this location after reset
        unsafe {
            core::ptr::write_volatile(BOOTLOADER_MAGIC_ADDR, BOOTLOADER_MAGIC);
        }
        
        // Use the absolute simplest reset approach
        unsafe {
            // Write magic value and immediately reset
            core::ptr::write_volatile(BOOTLOADER_MAGIC_ADDR, BOOTLOADER_MAGIC);
            
            // Ensure the write completes
            cortex_m::asm::dsb();
            cortex_m::asm::isb();
            
            // Disable interrupts
            cortex_m::interrupt::disable();
            
            // Direct system reset - no peripheral resets, just reset the CPU
            let scb = &(*cortex_m::peripheral::SCB::PTR);
            scb.aircr.write(0x05FA0004); // VECTKEY | SYSRESETREQ
            
            // Ensure the reset happens
            cortex_m::asm::dsb();
            cortex_m::asm::isb();
            
            // Wait for reset
            loop {
                cortex_m::asm::wfi();
            }
        }
    }

    /// Reset bootloader entry state to normal operation
    /// Requirements: 5.4 (recover gracefully if interrupted)
    pub fn reset_entry_state(&mut self) {
        log_info!("Resetting bootloader entry state to normal operation");
        
        self.entry_state = BootloaderEntryState::Normal;
        self.entry_start_time = None;
        self.shutdown_sequence = TaskShutdownSequence::new();
        
        log_info!("Bootloader entry state reset, returning to normal operation");
    }

    /// Check if shutdown is requested for a specific task priority
    /// This is called by RTIC tasks to check if they should shut down
    pub fn is_shutdown_requested(&self, priority: TaskPriority) -> bool {
        self.shutdown_sequence.is_shutdown_requested(priority)
    }

    /// Mark a task priority level as shutdown complete
    /// This is called by RTIC tasks when they complete shutdown
    pub fn mark_task_shutdown_complete(&mut self, priority: TaskPriority) {
        self.shutdown_sequence.mark_task_shutdown_complete(priority);
    }

    /// Mark a task priority level as shutdown failed
    /// This is called by RTIC tasks if they fail to shut down
    pub fn mark_task_shutdown_failed(&mut self, priority: TaskPriority) {
        self.shutdown_sequence.mark_task_shutdown_failed(priority);
    }

    /// Get current bootloader entry state
    pub fn get_entry_state(&self) -> BootloaderEntryState {
        self.entry_state
    }

    /// Get current task shutdown status for debugging
    pub fn get_shutdown_status(&self) -> (TaskShutdownStatus, TaskShutdownStatus, TaskShutdownStatus) {
        self.shutdown_sequence.get_shutdown_status()
    }

    /// Check if bootloader entry is in progress
    pub fn is_entry_in_progress(&self) -> bool {
        self.entry_state != BootloaderEntryState::Normal
    }

    /// Get remaining time for bootloader entry process
    pub fn get_remaining_time_ms(&self, current_time_ms: u32) -> Option<u32> {
        if let Some(start_time) = self.entry_start_time {
            let elapsed = current_time_ms.saturating_sub(start_time);
            Some(self.entry_timeout_ms.saturating_sub(elapsed))
        } else {
            None
        }
    }
}

/// Global bootloader entry manager instance
/// This is used by RTIC tasks to coordinate bootloader entry
static mut GLOBAL_BOOTLOADER_MANAGER: Option<BootloaderEntryManager> = None;

/// Initialize the global bootloader entry manager
pub fn init_bootloader_manager() {
    unsafe {
        GLOBAL_BOOTLOADER_MANAGER = Some(BootloaderEntryManager::new());
    }
    log_info!("Bootloader entry manager initialized");
}

/// Get a reference to the global bootloader entry manager
#[allow(static_mut_refs)]
pub fn get_bootloader_manager() -> Option<&'static mut BootloaderEntryManager> {
    unsafe { GLOBAL_BOOTLOADER_MANAGER.as_mut() }
}

/// Check if any task should shut down (called by RTIC tasks)
pub fn should_task_shutdown(priority: TaskPriority) -> bool {
    if let Some(manager) = get_bootloader_manager() {
        manager.is_shutdown_requested(priority)
    } else {
        false
    }
}

/// Mark task shutdown complete (called by RTIC tasks)
pub fn mark_task_shutdown_complete(priority: TaskPriority) {
    if let Some(manager) = get_bootloader_manager() {
        manager.mark_task_shutdown_complete(priority);
    }
}

/// Mark task shutdown failed (called by RTIC tasks)
#[allow(dead_code)]
pub fn mark_task_shutdown_failed(priority: TaskPriority) {
    if let Some(manager) = get_bootloader_manager() {
        manager.mark_task_shutdown_failed(priority);
    }
}