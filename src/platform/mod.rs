pub mod qemu_aarch64_virt;

use crate::baocore::cache::Cache;
use core::mem::size_of;

#[repr(C)]
struct ArchPlatform {
    gic: GICDescriptor,
    smmu: SMMUDescriptor,
    generic_timer: GenericTimerDescriptor,
    clusters: ClustersDescriptor,
}

#[repr(C)]
struct GICDescriptor {
    gicc_addr: u64,
    gich_addr: u64,
    gicv_addr: u64,
    gicd_addr: u64,
    gicr_addr: u64,
    maintenance_id: u32,
}

#[repr(C)]
struct SMMUDescriptor {
    base: u64,
    interrupt_id: u32,
    global_mask: usize,
}

const fn default_smmu_desc() -> SMMUDescriptor {
    SMMUDescriptor {
        base: 0,
        interrupt_id: 0,
        global_mask: 0,
    }
}

#[repr(C)]
struct GenericTimerDescriptor {
    base_addr: u64,
}

const fn default_generic_timer_desc() -> GenericTimerDescriptor {
    GenericTimerDescriptor { base_addr: 0 }
}

#[repr(C)]
struct ClustersDescriptor {
    num: usize,
    core_nums: [usize; 4],
}

const fn default_clusters_desc() -> ClustersDescriptor {
    ClustersDescriptor {
        num: 0,
        core_nums: [0; 4],
    }
}

#[repr(C)]
pub struct Platform {
    cpu_num: usize,
    region_num: usize,
    // struct mem_region *regions;
    console: usize,
    cache: Cache,
    arch: ArchPlatform,
}

const PLAT_ARCH_OFF: usize = size_of::<Platform>() - size_of::<ArchPlatform>();
const PLAT_ARCH_CLUSTERS_OFF: usize = size_of::<ArchPlatform>() - size_of::<ClustersDescriptor>();
const PLAT_CLUSTERS_CORES_NUM_OFF: usize = size_of::<usize>();
pub const PLATFORM_OFFSET: usize = PLAT_ARCH_OFF + PLAT_ARCH_CLUSTERS_OFF + PLAT_CLUSTERS_CORES_NUM_OFF;

pub use qemu_aarch64_virt::PLATFORM;