#![allow(unused)]

use core::mem::size_of;

use crate::{
    arch::aarch64::defs::PAGE_SIZE,
    baocore::{
        pagetable::Pagetable,
        types::{Paddr, Vaddr},
    },
};

pub const fn addr_msk(msb: u64, lsb: u64) -> u64 {
    ((1u64 << (msb + 1)) - 1) & !((1u64 << lsb) - 1)
}

pub const fn pte_mask(off: u64, len: u64) -> u64 {
    ((1 << len) - 1) << off
}

pub const fn pte_attr(n: u64) -> u64 {
    (n << PTE_ATTR_OFF) & PTE_ATTR_MSK
}

pub const PTE_ADDR_MSK: u64 = addr_msk(47, 12);
pub const PTE_FLAGS_MSK: u64 = !PTE_ADDR_MSK;
pub const PTE_ATTR_OFF: u64 = 2;
pub const PTE_ATTR_MSK: u64 = 0x7u64 << PTE_ATTR_OFF;
pub const PTE_AP_OFF: u64 = 6;
pub const PTE_AP_RW: u64 = 0x1u64 << PTE_AP_OFF;
pub const PTE_SH_OFF: u64 = 8;
pub const PTE_SH_IS: u64 = 0x3u64 << PTE_SH_OFF;
pub const PTE_AF: u64 = 1u64 << 10;
pub const PTE_TABLE: u64 = 3;
pub const PTE_PAGE: u64 = 3;
pub const PTE_TYPE_MSK: u64 = 0x3;

pub const PTE_INVALID: u64 = 0;
pub const PTE_VALID: u64 = 0x1;
pub const PTE_SUPERPAGE: u64 = 0x1;
pub const PTE_HYP_FLAGS: u64 = pte_attr(1) | PTE_AP_RW | PTE_SH_IS | PTE_AF;

pub const PTE_RSW_OFF: u64 = 55;
pub const PTE_RSW_WDT: u64 = 4;
pub const PTE_RSW_MSK: u64 =
    ((1u64 << (PTE_RSW_OFF + PTE_RSW_WDT)) - 1) - ((1u64 << (PTE_RSW_OFF)) - 1);
pub const PTE_RSW_RSRV: u64 = 0x3 << PTE_RSW_OFF;

#[repr(C)]
pub struct PageTableDescriptor {
    pub lvls: usize,
    pub lvl_wdt: [usize; 4],
    pub lvl_off: [usize; 4],
    pub lvl_term: [bool; 4],
}

const ARMV8_PT_DSCR: PageTableDescriptor = PageTableDescriptor {
    lvls: 4,
    lvl_wdt: [48, 39, 30, 21],
    lvl_off: [39, 30, 21, 12],
    lvl_term: [false, true, true, true],
};

const ARMV8_PT_S2_DSCR: PageTableDescriptor = PageTableDescriptor {
    lvls: 4,
    lvl_wdt: [48, 39, 30, 21],
    lvl_off: [39, 30, 21, 12],
    lvl_term: [false, true, true, true],
};

pub const HYP_PT_DSCR: &PageTableDescriptor = &ARMV8_PT_DSCR;
pub const VM_PT_DSCR: &PageTableDescriptor = &ARMV8_PT_S2_DSCR;

#[macro_export]
macro_rules! pt_cpu_rec_index {
    () => {
        crate::baocore::cpu::mycpu().addr_space.pt.pt_nentries(0) - 1
    };
}

#[macro_export]
macro_rules! pt_vm_rec_index {
    () => {
        crate::baocore::cpu::mycpu().addr_space.pt.pt_nentries(0) - 2
    };
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PTE(pub u64);

pub const PTE_SIZE: usize = size_of::<PTE>();

impl PTE {
    pub fn new(pa: Paddr, pte_type: u64, pte_flags: u64) -> Self {
        Self((pa & PTE_ADDR_MSK) | ((pte_type | pte_flags) & PTE_FLAGS_MSK))
    }

    pub fn check_rsw(&self, flag: u64) -> bool {
        self.0 & PTE_RSW_MSK == flag & PTE_RSW_MSK
    }

    pub fn is_valid(&self) -> bool {
        (self.0 & PTE_VALID) != 0
    }

    pub fn is_table(&self, pt: &Pagetable, lvl: usize) -> bool {
        lvl != pt.dscr.lvls - 1 && (self.0 & PTE_TYPE_MSK == PTE_TABLE)
    }

    pub fn is_allocable(&self, pt: &Pagetable, lvl: usize, left: usize, addr: Vaddr) -> bool {
        let lvlsize = pt.pt_lvlsize(lvl);
        lvl == pt.dscr.lvls - 1
            || pt.dscr.lvl_term[lvl]
                && !self.is_valid()
                && lvlsize <= left * PAGE_SIZE
                && addr % lvlsize as u64 == 0
    }

    pub fn is_mappable(
        &self,
        pt: &Pagetable,
        lvl: usize,
        left: usize,
        vaddr: Vaddr,
        paddr: Paddr,
    ) -> bool {
        let lvlsize = pt.pt_lvlsize(lvl);
        !self.is_valid()
            && lvlsize <= left * PAGE_SIZE
            && vaddr as usize % lvlsize == 0
            && paddr as usize % lvlsize == 0
    }

    pub fn set_rsw(&mut self, flag: u64) {
        self.0 &= !PTE_RSW_MSK;
        self.0 |= flag & PTE_RSW_MSK;
    }
}

#[repr(C)]
pub struct PageTableArch {
    pub rec_index: usize,
    pub rec_mask: u64,
}
