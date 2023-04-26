use spin::RwLock;

use crate::{
    arch::aarch64::{
        armv8_a::{
            fences::fence_sync_write,
            pagetable::{ARMV8_PT_S2_SMALL_DSCR, PARANGE_TABLE, VM_PT_DSCR},
        },
        sysregs::*,
    },
    baocore::cpu::{mycpu, CPU_SYNC_TOKEN},
    read_reg, write_reg,
};

static MIN_PARANGE: RwLock<u64> = RwLock::new(0x7);

pub fn vmm_arch_profile_init() {
    let temp_parange = read_reg!(id_aa64mmfr0_el1) & 0xf;
    let mut p = MIN_PARANGE.write();
    *p = p.min(temp_parange);
    drop(p);

    CPU_SYNC_TOKEN.sync_barrier();
    if mycpu().is_master() {
        let parange = *MIN_PARANGE.read() as usize;
        if PARANGE_TABLE[parange] < 44 {
            unsafe {
                VM_PT_DSCR = &ARMV8_PT_S2_SMALL_DSCR;
            }
            fence_sync_write();
        }
    }
    let parange = *MIN_PARANGE.read() as usize;

    let vtcr = VTCR_RES1
        | (((parange as u64) << VTCR_PS_OFF) & VTCR_PS_MSK)
        | VTCR_TG0_4K
        | VTCR_ORGN0_WB_RA_WA
        | VTCR_IRGN0_WB_RA_WA
        | vtcr_t0sz(64 - PARANGE_TABLE[parange] as u64)
        | VTCR_SH0_IS
        | if PARANGE_TABLE[parange] < 44 {
            VTCR_SL0_12
        } else {
            VTCR_SL0_01
        };

    write_reg!(vtcr_el2, vtcr);
}
