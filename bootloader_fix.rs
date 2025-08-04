// Simplified bootloader entry fix
// This creates a more direct path to bootloader mode that bypasses
// the complex task shutdown and hardware validation sequence

use rp2040_hal::pac;
use cortex_m::interrupt;

/// Simplified bootloader entry function that bypasses complex validation
/// This is called directly from the command handler for immediate bootloader entry
pub fn enter_bootloader_mode_direct() -> ! {
    // Log the entry attempt
    crate::log_info!("=== DIRECT BOOTLOADER ENTRY ===");
    crate::log_info!("Bypassing complex validation for immediate entry");
    
    // Disable interrupts immediately to prevent interference
    cortex_m::interrupt::disable();
    
    // Force hardware into safe state directly
    unsafe {
        let io_bank0 = &(*pac::IO_BANK0::ptr());
        let sio = &(*pac::SIO::ptr());
        
        // Force MOSFET OFF (GPIO15) for safety
        sio.gpio_out_clr().write(|w| w.bits(1 << 15));
        io_bank0.gpio(15).gpio_ctrl().write(|w| w.funcsel().sio());
        sio.gpio_oe_set().write(|w| w.bits(1 << 15));
        
        // Turn off LED (GPIO25) 
        sio.gpio_out_clr().write(|w| w.bits(1 << 25));
        io_bank0.gpio(25).gpio_ctrl().write(|w| w.funcsel().sio());
        sio.gpio_oe_set().write(|w| w.bits(1 << 25));
    }
    
    // Write bootloader magic value
    const BOOTLOADER_MAGIC: u32 = 0xB007C0DE;
    const BOOTLOADER_MAGIC_ADDR: *mut u32 = 0x20041FFC as *mut u32;
    
    unsafe {
        core::ptr::write_volatile(BOOTLOADER_MAGIC_ADDR, BOOTLOADER_MAGIC);
        
        // Ensure write completes
        cortex_m::asm::dsb();
        cortex_m::asm::isb();
    }
    
    // Small delay to ensure any pending operations complete
    for _ in 0..1000 {
        cortex_m::asm::nop();
    }
    
    // Perform system reset
    unsafe {
        let scb = &(*cortex_m::peripheral::SCB::PTR);
        
        // Use the standard ARM system reset
        scb.aircr.write(0x05FA0004); // VECTKEY | SYSRESETREQ
        
        // Ensure reset happens
        cortex_m::asm::dsb();
        cortex_m::asm::isb();
        
        // Wait for reset
        loop {
            cortex_m::asm::wfi();
        }
    }
}

/// Alternative bootloader entry using watchdog reset
pub fn enter_bootloader_mode_watchdog() -> ! {
    crate::log_info!("=== WATCHDOG BOOTLOADER ENTRY ===");
    crate::log_info!("Using watchdog reset method");
    
    // Write bootloader magic
    const BOOTLOADER_MAGIC: u32 = 0xB007C0DE;
    const BOOTLOADER_MAGIC_ADDR: *mut u32 = 0x20041FFC as *mut u32;
    
    unsafe {
        core::ptr::write_volatile(BOOTLOADER_MAGIC_ADDR, BOOTLOADER_MAGIC);
        cortex_m::asm::dsb();
        cortex_m::asm::isb();
    }
    
    // Force hardware safe state
    unsafe {
        let sio = &(*pac::SIO::ptr());
        sio.gpio_out_clr().write(|w| w.bits((1 << 15) | (1 << 25))); // MOSFET and LED off
    }
    
    // Use watchdog to reset
    unsafe {
        let watchdog = &(*pac::WATCHDOG::ptr());
        
        // Enable watchdog with very short timeout
        watchdog.ctrl().write(|w| w.enable().set_bit());
        watchdog.load().write(|w| w.bits(1)); // 1 tick = very short timeout
        
        // Wait for watchdog reset
        loop {
            cortex_m::asm::wfi();
        }
    }
}

/// Test if bootloader magic is already set
pub fn is_bootloader_magic_set() -> bool {
    const BOOTLOADER_MAGIC: u32 = 0xB007C0DE;
    const BOOTLOADER_MAGIC_ADDR: *mut u32 = 0x20041FFC as *mut u32;
    
    unsafe {
        let current_value = core::ptr::read_volatile(BOOTLOADER_MAGIC_ADDR);
        current_value == BOOTLOADER_MAGIC
    }
}

/// Clear bootloader magic (for testing)
pub fn clear_bootloader_magic() {
    const BOOTLOADER_MAGIC_ADDR: *mut u32 = 0x20041FFC as *mut u32;
    
    unsafe {
        core::ptr::write_volatile(BOOTLOADER_MAGIC_ADDR, 0);
        cortex_m::asm::dsb();
        cortex_m::asm::isb();
    }
}