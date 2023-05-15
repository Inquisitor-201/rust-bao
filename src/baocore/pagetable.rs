use core::mem::size_of;

use crate::{
    arch::aarch64::{
        armv8_a::pagetable::{
            pte_mask, PageTableArch, PageTableDescriptor, PTE, PTE_HYP_FLAGS, PTE_PAGE, PTE_SIZE,
            PTE_SUPERPAGE, PTE_TABLE,
        },
        defs::{BAO_CPU_BASE, PAGE_SIZE},
    },
    util::align_down,
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
        let hyp_pte = (mycpu().addr_space.pt.root + (index * PTE_SIZE) as u64) as *mut PTE;
        unsafe { *hyp_pte = PTE::new(root_pt_pa, PTE_TABLE, PTE_HYP_FLAGS) };
        self.arch.rec_index = index;
        self.arch.rec_mask = 0;
        let cpu_rec_index = mycpu().addr_space.pt.arch.rec_index;
        for i in 0..self.dscr.lvls {
            let lvl_off = self.dscr.lvl_off[i];
            self.arch.rec_mask |= (cpu_rec_index as u64) << lvl_off;
        }
    }

    pub fn pt_get_pte(&self, lvl: usize, va: Vaddr) -> *mut PTE {
        let cpu_pt = &mycpu().addr_space.pt;
        let rec_ind_off = cpu_pt.dscr.lvl_off[cpu_pt.dscr.lvls - lvl - 1];
        let rec_ind_len = cpu_pt.dscr.lvl_wdt[cpu_pt.dscr.lvls - lvl - 1];
        let rec_ind_mask = pte_mask(rec_ind_off as u64, (rec_ind_len - rec_ind_off) as u64);
        let mut addr = cpu_pt.arch.rec_mask & !pte_mask(0, rec_ind_len as u64);
        addr |= ((self.arch.rec_index as u64) << rec_ind_off as u64) & rec_ind_mask;
        addr |=
            ((va >> self.dscr.lvl_off[lvl]) * PTE_SIZE as u64) & pte_mask(0, rec_ind_off as u64);

        addr as *mut PTE
    }

    pub fn pt_get(&self, lvl: usize, va: Vaddr) -> *mut PTE {
        if lvl == 0 {
            return self.root as _;
        }
        align_down(self.pt_get_pte(lvl, va) as usize, PAGE_SIZE) as _
    }

    pub fn pt_size(&self, lvl: usize) -> usize {
        self.pt_nentries(lvl) * PTE_SIZE
    }

    pub fn pt_getpteindex(&self, pte: *const PTE, lvl: usize) -> usize {
        (pte as usize & (self.pt_size(lvl) - 1)) / PTE_SIZE
    }

    pub fn pt_lvlsize(&self, lvl: usize) -> usize {
        1 << self.dscr.lvl_off[lvl]
    }

    pub fn page_type(&self, lvl: usize) -> u64 {
        if lvl == self.dscr.lvls - 1 {
            PTE_PAGE
        } else {
            PTE_SUPERPAGE
        }
    }
}

pub fn root_pt_addr() -> Vaddr {
    (BAO_CPU_BASE + size_of::<Cpu>()) as _
}
