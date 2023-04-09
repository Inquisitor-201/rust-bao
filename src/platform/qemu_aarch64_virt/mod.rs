use crate::baocore::cache::Cache;

use super::*;

pub static PLATFORM: Platform = Platform {
    cpu_num: 4,
    region_num: 1,
    console_base: 0x9000000,
    cache: Cache {},
    arch: ArchPlatform {
        gic: GICDescriptor {
            gicd_addr: 0x08000000,
            gicc_addr: 0x08010000,
            gich_addr: 0x08030000,
            gicv_addr: 0x08040000,
            gicr_addr: 0x080A0000,
            maintenance_id: 25,
        },
        smmu: default_smmu_desc(),
        generic_timer: default_generic_timer_desc(),
        clusters: ClustersDescriptor {
            num: 1,
            core_nums: [4, 0, 0, 0],
        },
    },
    // .regions =  (struct mem_region[]) {
    //     {
    //         .base = 0x40000000,
    //         .size = 0x100000000
    //     }
    // },
};
