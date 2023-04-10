use crate::{
    baocore::types::CpuID,
    platform::{ArchPlatformTrait, Platform},
};

use super::sysregs::*;

impl ArchPlatformTrait for Platform {
    fn cpu_id_to_mpidr(&self, id: CpuID) -> u64 {
        if id as usize > self.cpu_num {
            return !(!MPIDR_RES1 & MPIDR_RES0_MSK); // return an invalid mpidr by inverting res bits
        }

        let mut mpidr = 0;
        let mut found = false;
        if self.arch.clusters.num > 0 {
            let mut j = 0;
            for i in 0..self.arch.clusters.num {
                if id < (j + self.arch.clusters.core_nums[i] as u64) {
                    mpidr = ((i as u64) << 8) | ((id - j) & 0xff);
                    found = true;
                    break;
                }

                j += self.arch.clusters.core_nums[i] as u64;
            }

            if !found {
                panic!("failed cpuid to mpdir translation");
            }
        } else {
            /* No cluster information in configuration. Assume a single cluster. */
            mpidr = id;
        }

        mpidr |= MPIDR_RES1;
        if self.cpu_num == 1 {
            mpidr |= MPIDR_U_BIT;
        }

        mpidr
    }
}
