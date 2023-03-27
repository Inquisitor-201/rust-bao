#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(include_str!("arch/aarch64/boot.S"));

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}