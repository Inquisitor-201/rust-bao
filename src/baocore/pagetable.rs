use core::mem::size_of;

use crate::arch::aarch64::{armv8_a::pagetable::PageTableDescriptor, defs::BAO_CPU_BASE};

use super::{cpu::Cpu, types::Vaddr};

#[repr(C)]
pub struct Pagetable {
    pub root: Vaddr,
    pub dscr: &'static PageTableDescriptor,
}

impl Pagetable {
    pub fn pt_nentries(&self, lvl: usize) -> usize {
        (1 << self.dscr.lvl_wdt[lvl]) >> self.dscr.lvl_off[lvl]
    }
}

pub fn root_pt_addr() -> Vaddr {
    (BAO_CPU_BASE + size_of::<Cpu>()) as _
}
