use crate::{
    arch::aarch64::cpu::{CpuArch, CpuArchProfileTrait},
    baocore::types::{CpuID, Paddr},
    platform::{ArchPlatformTrait, PLATFORM},
};

extern "C" {
    pub fn CPU_MASTER();
}

impl CpuArchProfileTrait for CpuArch {
    fn arch_profile_init(&self, cpu_id: CpuID, load_addr: Paddr) {
        if cpu_id == unsafe { *(CPU_MASTER as *mut u64) } {
            for cpu_core_id in 0..PLATFORM.cpu_num as CpuID {
                if cpu_core_id == cpu_id {
                    continue;
                }
                let mpdir = PLATFORM.cpu_id_to_mpidr(cpu_core_id);
                // TODO: pass config addr in contextid (x0 register)
                psci::cpu_on(mpdir, load_addr, 0).unwrap_or_else(|err| {
                    if let psci::error::Error::AlreadyOn = err {
                    } else {
                        panic!("can't wake up cpu {}", cpu_core_id);
                    }
                });
            }
        }
    }
}
