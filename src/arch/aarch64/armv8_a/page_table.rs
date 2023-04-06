#![allow(unused)]

const fn addr_msk(msb: u64, lsb: u64) -> u64 {
    ((1u64 << (msb + 1)) - 1) & !((1u64 << lsb) - 1)
}

const fn pte_mask(off: u64, len: u64) -> u64 {
    ((1 << len) - 1) << off
}

const fn pte_attr(n: u64) -> u64 {
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

pub const PTE_INVALID: u64 = 0;
pub const PTE_SUPERPAGE: u64 = 0x1;
pub const PTE_HYP_FLAGS: u64 = pte_attr(1) | PTE_AP_RW | PTE_SH_IS | PTE_AF;
