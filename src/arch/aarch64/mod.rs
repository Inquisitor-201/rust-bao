use core::arch::global_asm;

pub mod bao_defs;
use bao_defs::*;

global_asm!(include_str!("boot.S"),
    platform = sym crate::platform::PLATFORM,
    cores_num_off = const crate::platform::PLATFORM_OFFSET,
    BAO_VAS_BASE = const BAO_VAS_BASE);

global_asm!(include_str!("exceptions.S"));
