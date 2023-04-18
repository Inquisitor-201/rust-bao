pub mod armv8_a;
pub mod cpu;
pub mod defs;
pub mod interrupts;
pub mod platform;
pub mod sysregs;

use core::arch::global_asm;

global_asm!(include_str!("boot.S"),
    platform = sym crate::platform::PLATFORM,
    cores_num_off = const crate::platform::PLATFORM_OFFSET,
    CPU_SIZE = const crate::baocore::cpu::CPU_SIZE);

global_asm!(include_str!("exceptions.S"));
