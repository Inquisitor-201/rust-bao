pub mod cpu_arch_profile;
pub mod mem;
#[macro_use]
pub mod pagetable;
pub mod fences;
pub mod vm;
pub mod vmm;

use super::{defs::*, sysregs::*};
use crate::baocore::cpu::CPU_SIZE;
use core::arch::global_asm;
use pagetable::*;

pub const PT_LVLS: usize = 4;

global_asm!(include_str!("boot.S"),
    PTE_HYP_FLAGS = const PTE_HYP_FLAGS,
    PTE_SUPERPAGE = const PTE_SUPERPAGE,
    BAO_VAS_BASE = const BAO_VAS_BASE,
    BAO_CPU_BASE = const BAO_CPU_BASE,
    PAGE_SIZE = const PAGE_SIZE,
    PTE_PAGE = const PTE_PAGE,
    PTE_TABLE = const PTE_TABLE,
    PT_SIZE = const PAGE_SIZE,
    PT_LVLS = const PT_LVLS,
    CPU_SIZE = const CPU_SIZE,
    TCR_EL2_DFLT = const TCR_EL2_DFLT,
    MAIR_EL2_DFLT = const MAIR_EL2_DFLT,
    SCTLR_DFLT = const SCTLR_DFLT);

global_asm!(include_str!("pagetables.S"),
    PAGE_SIZE = const PAGE_SIZE);
