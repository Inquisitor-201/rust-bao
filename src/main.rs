#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(alloc_error_handler)]
#![feature(int_roundings)]
#![feature(panic_info_message)]

extern crate alloc;

pub mod arch;
pub mod baocore;
pub mod platform;
pub mod util;
pub mod config;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("Panicked: {}", info.message().unwrap());
    }
    loop {}
}
