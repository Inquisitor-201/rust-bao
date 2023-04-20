use crate::arch::aarch64::vmm::vmm_arch_init;

use super::cpu::CPU_SYNC_TOKEN;

fn vmm_assign_vcpu() {
    // for i in 0..
}

pub fn init() {
    vmm_arch_init();
    CPU_SYNC_TOKEN.sync_barrier();

    vmm_assign_vcpu();
}