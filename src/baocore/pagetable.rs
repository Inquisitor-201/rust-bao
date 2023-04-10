use core::mem::size_of;

use crate::arch::aarch64::{armv8_a::pagetable::PageTableDescriptor, defs::BAO_CPU_BASE};

use super::{cpu::Cpu, types::Vaddr};

#[repr(C)]
pub struct Pagetable {
    pub root: Vaddr,
    pub desc: &'static PageTableDescriptor,
}

pub fn root_pt_addr() -> Vaddr {
    (BAO_CPU_BASE + size_of::<Cpu>()) as _
}
