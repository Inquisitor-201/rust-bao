#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(alloc_error_handler)]
#![feature(int_roundings)]

extern crate alloc;

pub mod arch;
pub mod baocore;
pub mod platform;
pub mod util;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
