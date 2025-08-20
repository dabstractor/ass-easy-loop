#![no_std]
#![no_main]

use panic_probe as _;
use cortex_m_rt::entry;
use rp2040_hal as _;

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

#[entry]
fn main() -> ! {
    loop {
        cortex_m::asm::nop();
    }
}