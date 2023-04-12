use core::mem::size_of;

use crate::arch::aarch64::{
    armv8_a::pagetable::{
        PageTableArch, PageTableDescriptor, PTE, PTE_HYP_FLAGS, PTE_SIZE, PTE_TABLE,
    },
    defs::BAO_CPU_BASE,
};

use super::{
    cpu::{mycpu, Cpu},
    types::Vaddr,
};

#[repr(C)]
pub struct Pagetable {
    pub root: Vaddr,
    pub dscr: &'static PageTableDescriptor,
    pub arch: PageTableArch,
}

impl Pagetable {
    pub fn pt_nentries(&self, lvl: usize) -> usize {
        (1 << self.dscr.lvl_wdt[lvl]) >> self.dscr.lvl_off[lvl]
    }

    pub fn pt_set_recursive(&mut self, index: usize) {
        let root_pt_pa = mycpu().addr_space.mem_translate(self.root).unwrap();
        let pte = PTE::refmut_from_va(self.root + (index * PTE_SIZE) as u64);
        *pte = PTE::new(root_pt_pa, PTE_TABLE, PTE_HYP_FLAGS);
        self.arch.rec_index = index;
        self.arch.rec_mask = 0;
        for i in 0..self.dscr.lvls {
            let lvl_off = self.dscr.lvl_off[i];
            self.arch.rec_mask |= (index as u64) << lvl_off;
        }
    }
}

pub fn root_pt_addr() -> Vaddr {
    (BAO_CPU_BASE + size_of::<Cpu>()) as _
}
