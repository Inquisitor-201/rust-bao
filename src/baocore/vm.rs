use alloc::vec::Vec;

use crate::arch::aarch64::armv8_a::{pagetable::PTE, vm::ArchVMPlatform};

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
    pub arch: ArchVMPlatform,
}

pub struct VMAllocation {
    pub base: Vaddr,
    pub size: usize,
    pub vcpus_offset: usize
}

pub struct VMInstallInfo {
    pub base: Vaddr,
    pub vm_section_pte: PTE,
}

pub struct VM {}

pub struct VCpu {
    pub aaa: usize
}
