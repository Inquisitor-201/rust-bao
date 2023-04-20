use crate::write_reg;

use super::{armv8_a::vmm::vmm_arch_profile_init, sysregs::*};

pub fn vmm_arch_init() {
    vmm_arch_profile_init();
    let hcr = HCR_VM_BIT | HCR_RW_BIT | HCR_IMO_BIT | HCR_FMO_BIT |
                   HCR_TSC_BIT | HCR_APK_BIT | HCR_API_BIT;
    write_reg!(hcr_el2, hcr);
    write_reg!(cptr_el2, 0);
}