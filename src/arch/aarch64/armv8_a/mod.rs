pub mod page_table;

use super::defs::PAGE_SIZE;
use core::arch::global_asm;
use page_table::*;

global_asm!(include_str!("boot.S"),
    PTE_HYP_FLAGS = const PTE_HYP_FLAGS,
    PTE_SUPERPAGE = const PTE_SUPERPAGE);

global_asm!(include_str!("pagetables.S"),
    PAGE_SIZE = const PAGE_SIZE);
