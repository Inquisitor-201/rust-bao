use core::arch::global_asm;

pub mod armv8_a;
pub mod defs;
use defs::*;

global_asm!(include_str!("boot.S"),
    platform = sym crate::platform::PLATFORM,
    cores_num_off = const crate::platform::PLATFORM_OFFSET);

global_asm!(include_str!("exceptions.S"));
