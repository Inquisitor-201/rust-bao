use crate::{arch::aarch64::sysregs::*, write_reg};

pub fn vmm_arch_profile_init() {
    let vctr = VTCR_RES1
        | VTCR_ORGN0_WB_RA_WA
        | VTCR_IRGN0_WB_RA_WA
        | vtcr_t0sz(0)
        | VTCR_SH0_IS
        | VTCR_SL0_12;
    write_reg!(vctr_el2, vctr);
}
