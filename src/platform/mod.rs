#![allow(dead_code)]

pub mod drivers;
pub mod qemu_aarch64_virt;

use crate::baocore::{
    cache::Cache,
    mem::MemRegion,
    types::{CpuID, Paddr, IrqID},
};
use core::mem::size_of;

#[repr(C)]
pub struct ArchPlatform {
    pub gic: GICDescriptor,
    pub smmu: SMMUDescriptor,
    pub generic_timer: GenericTimerDescriptor,
    pub clusters: ClustersDescriptor,
}

#[repr(C)]
pub struct GICDescriptor {
    pub gicc_addr: Paddr,
    pub gich_addr: Paddr,
    pub gicv_addr: Paddr,
    pub gicd_addr: Paddr,
    pub gicr_addr: Paddr,
    pub maintenance_id: IrqID,
}

#[repr(C)]
pub struct SMMUDescriptor {
    pub base: u64,
    pub interrupt_id: u32,
    pub global_mask: usize,
}

const fn default_smmu_desc() -> SMMUDescriptor {
    SMMUDescriptor {
        base: 0,
        interrupt_id: 0,
        global_mask: 0,
    }
}

#[repr(C)]
pub struct GenericTimerDescriptor {
    base_addr: u64,
}

const fn default_generic_timer_desc() -> GenericTimerDescriptor {
    GenericTimerDescriptor { base_addr: 0 }
}

#[repr(C)]
pub struct ClustersDescriptor {
    pub num: usize,
    pub core_nums: [u8; 4],
}

const fn default_clusters_desc() -> ClustersDescriptor {
    ClustersDescriptor {
        num: 0,
        core_nums: [0; 4],
    }
}

#[repr(C)]
pub struct Platform {
    pub cpu_num: usize,
    pub region_num: usize,
    pub regions: [MemRegion; 2],
    pub console_base: Paddr,
    pub cache: Cache,
    pub arch: ArchPlatform,
}

pub trait ArchPlatformTrait {
    fn cpu_id_to_mpidr(&self, id: CpuID) -> u64;
}

const PLAT_ARCH_OFF: usize = size_of::<Platform>() - size_of::<ArchPlatform>();
const PLAT_ARCH_CLUSTERS_OFF: usize = size_of::<ArchPlatform>() - size_of::<ClustersDescriptor>();
const PLAT_CLUSTERS_CORES_NUM_OFF: usize = size_of::<usize>();
pub const PLATFORM_OFFSET: usize =
    PLAT_ARCH_OFF + PLAT_ARCH_CLUSTERS_OFF + PLAT_CLUSTERS_CORES_NUM_OFF;

pub use qemu_aarch64_virt::PLATFORM;
