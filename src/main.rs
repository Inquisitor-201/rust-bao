#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(alloc_error_handler)]
#![feature(int_roundings)]
#![feature(panic_info_message)]
#![feature(exclusive_range_pattern)]

extern crate alloc;

pub mod arch;
pub mod baocore;
pub mod config;
pub mod platform;
pub mod util;

use core::panic::PanicInfo;

use crate::baocore::cpu::mycpu;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "[Cpu {}] Panicked at {}:{} {}",
            mycpu().id,
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("Panicked: {}", info.message().unwrap());
    }
    loop {}
}
