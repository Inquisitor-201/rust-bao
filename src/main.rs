#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(concat_idents)]

pub mod arch;
pub mod baocore;
pub mod platform;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
