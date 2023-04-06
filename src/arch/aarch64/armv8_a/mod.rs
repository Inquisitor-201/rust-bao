pub mod page_table;

use super::defs::{PAGE_SIZE, BAO_VAS_BASE};
use core::arch::global_asm;
use page_table::*;

global_asm!(include_str!("boot.S"),
    PTE_HYP_FLAGS = const PTE_HYP_FLAGS,
    PTE_SUPERPAGE = const PTE_SUPERPAGE,
    BAO_VAS_BASE = const BAO_VAS_BASE,
    PAGE_SIZE = const PAGE_SIZE,
    PTE_PAGE = const PTE_PAGE);

global_asm!(include_str!("pagetables.S"),
    PAGE_SIZE = const PAGE_SIZE);
