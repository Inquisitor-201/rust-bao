#![allow(unused)]
#![allow(non_upper_case_globals)]

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
pub const PTE_ATTR_MSK: u64 = 0x7 << PTE_ATTR_OFF;
pub const PTE_AP_OFF: u64 = 6;
pub const PTE_AP_RW: u64 = 0x1 << PTE_AP_OFF;
pub const PTE_SH_OFF: u64 = 8;
pub const PTE_SH_NS: u64 = 0x0 << PTE_SH_OFF;
pub const PTE_SH_OS: u64 = 0x2 << PTE_SH_OFF;
pub const PTE_SH_IS: u64 = 0x3 << PTE_SH_OFF;
pub const PTE_AF: u64 = 1 << 10;
pub const PTE_TABLE: u64 = 3;
pub const PTE_PAGE: u64 = 3;
pub const PTE_TYPE_MSK: u64 = 0x3;
pub const PTE_XN: u64 = 1 << 54;

pub const PTE_INVALID: u64 = 0;
pub const PTE_VALID: u64 = 0x1;
pub const PTE_SUPERPAGE: u64 = 0x1;
/* Stage 2 fields */
const PTE_MEMATTR_OFF: u64 = 2;
const PTE_MEMATTR_DEV_nGnRnE: u64 = (0x00 << 0) << PTE_MEMATTR_OFF;
const PTE_MEMATTR_DEV_nGnRE: u64 = (0x01 << 0) << PTE_MEMATTR_OFF;
const PTE_MEMATTR_DEV_nGRE: u64 = (0x02 << 0) << PTE_MEMATTR_OFF;
const PTE_MEMATTR_DEV_GRE: u64 = (0x03 << 0) << PTE_MEMATTR_OFF;
const PTE_MEMATTR_NRML_ONC: u64 = (0x01 << 2) << PTE_MEMATTR_OFF;
const PTE_MEMATTR_NRML_OWTC: u64 = (0x02 << 2) << PTE_MEMATTR_OFF;
const PTE_MEMATTR_NRML_OWBC: u64 = (0x03 << 2) << PTE_MEMATTR_OFF;
const PTE_MEMATTR_NRML_INC: u64 = (0x01 << 0) << PTE_MEMATTR_OFF;
const PTE_MEMATTR_NRML_IWTC: u64 = (0x02 << 0) << PTE_MEMATTR_OFF;
const PTE_MEMATTR_NRML_IWBC: u64 = (0x03 << 0) << PTE_MEMATTR_OFF;

const PTE_S2AP_RO: u64 = (0x1 << PTE_AP_OFF);
const PTE_S2AP_WO: u64 = (0x2 << PTE_AP_OFF);
const PTE_S2AP_RW: u64 = (0x3 << PTE_AP_OFF);

pub const PTE_HYP_FLAGS: u64 = pte_attr(1) | PTE_AP_RW | PTE_SH_IS | PTE_AF;
pub const PTE_VM_FLAGS: u64 =
    PTE_MEMATTR_NRML_OWBC | PTE_MEMATTR_NRML_IWBC | PTE_SH_NS | PTE_S2AP_RW | PTE_AF;
pub const PTE_HYP_DEV_FLAGS: u64 = pte_attr(2) | PTE_AP_RW | PTE_SH_IS | PTE_AF | PTE_XN;
pub const PTE_VM_DEV_FLAGS: u64 = PTE_MEMATTR_DEV_GRE | PTE_SH_NS | PTE_S2AP_RW | PTE_AF;

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

pub const ARMV8_PT_S2_SMALL_DSCR: PageTableDescriptor = PageTableDescriptor {
    lvls: 3,
    lvl_wdt: [39, 30, 21, 0],
    lvl_off: [30, 21, 12, 0],
    lvl_term: [false, true, true, true],
};

pub const PARANGE_TABLE: [usize; 6] = [32, 36, 40, 42, 44, 48];

pub const HYP_PT_DSCR: &PageTableDescriptor = &ARMV8_PT_DSCR;
pub static mut VM_PT_DSCR: &PageTableDescriptor = &ARMV8_PT_S2_DSCR;

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

    pub fn pa(&self) -> Paddr {
        self.0 & PTE_ADDR_MSK
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

    pub fn is_page(&self, pt: &Pagetable, lvl: usize) -> bool {
        if !pt.dscr.lvl_term[lvl] {
            return false;
        }
        (lvl != pt.dscr.lvls - 1 && (self.0 & PTE_TYPE_MSK == PTE_SUPERPAGE))
            || (lvl == pt.dscr.lvls - 1 && (self.0 & PTE_TYPE_MSK == PTE_PAGE))
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
