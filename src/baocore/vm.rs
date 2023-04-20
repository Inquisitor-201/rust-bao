use alloc::vec::Vec;

use crate::arch::aarch64::armv8_a::vm::ArchVMPlatform;

use super::types::{IrqID, Paddr, Vaddr};

pub struct VMMemRegion {
    pub base: Paddr,
    pub size: usize,
}

pub struct VMDeviceRegion {
    pub pa: Paddr,
    pub va: Vaddr,
    pub size: usize,
    pub interrupts: Vec<IrqID>,
}

pub struct VMPlatform {
    pub cpu_num: usize,
    pub vm_regions: Vec<VMMemRegion>,
    pub devs: Vec<VMDeviceRegion>,
    pub arch: ArchVMPlatform
}
