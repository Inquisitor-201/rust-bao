#![allow(unused)]
#![allow(non_upper_case_globals)]

use core::arch::asm;

use aarch64::regs::PAR_EL1;
use tock_registers::interfaces::Readable;

use crate::{arch::aarch64::armv8_a::fences::isb, baocore::types::Vaddr};

// SPSR - Saved Program Status Register
pub const SPSR_EL_MSK: u64 = 0x0f;
pub const SPSR_EL0t: u64 = 0x0;
pub const SPSR_EL1t: u64 = 0x4;
pub const SPSR_EL1h: u64 = 0x5;
pub const SPSR_EL2t: u64 = 0x8;
pub const SPSR_EL2h: u64 = 0x9;
pub const SPSR_EL3t: u64 = 0xc;
pub const SPSR_EL3h: u64 = 0xd;

pub const SPSR_F: u64 = 1 << 6;
pub const SPSR_I: u64 = 1 << 7;
pub const SPSR_A: u64 = 1 << 8;
pub const SPSR_D: u64 = 1 << 9;
pub const SPSR_IL: u64 = 1 << 20;
pub const SPSR_SS: u64 = 1 << 21;

pub const SPSR_USR: u64 = 0x10;
pub const SPSR_IRQ: u64 = 0x12;
pub const SPSR_SVC: u64 = 0x13;
pub const SPSR_ABT: u64 = 0x17;
pub const SPSR_UND: u64 = 0x1b;
pub const SPSR_SYS: u64 = 0x1f;

// SCR - Secure Configuration Register
pub const SCR_NS: u64 = 1 << 0;
pub const SCR_IRQ: u64 = 1 << 1;
pub const SCR_FIQ: u64 = 1 << 2;
pub const SCR_EA: u64 = 1 << 3;
pub const SCR_SMD: u64 = 1 << 7;
pub const SCR_HCE: u64 = 1 << 8;
pub const SCR_SIF: u64 = 1 << 9;
pub const SCR_RW: u64 = 1 << 10;
pub const SCR_ST: u64 = 1 << 11;
pub const SCR_TWI: u64 = 1 << 12;
pub const SCR_TWE: u64 = 1 << 13;
pub const SCR_TLOR: u64 = 1 << 14;
pub const SCR_TERR: u64 = 1 << 15;
pub const SCR_APK: u64 = 1 << 16;
pub const SCR_API: u64 = 1 << 17;

// TCR - Translation Control Register
pub const TCR_RES1: u64 = (1 << 23) | (1 << 31);
pub const TCR_T0SZ_MSK: u64 = 0x1f << 0;
pub const TCR_T0SZ_OFF: u64 = 0;
pub const fn tcr_t0sz(sz: u64) -> u64 {
    (sz << TCR_T0SZ_OFF) & TCR_T0SZ_MSK
}
pub const TCR_IRGN0_MSK: u64 = 0x3 << 8;
pub const TCR_IRGN0_NC: u64 = 0 << 8;
pub const TCR_IRGN0_WB_RA_WA: u64 = 1 << 8;
pub const TCR_IRGN0_WT_RA_NWA: u64 = 2 << 8;
pub const TCR_IRGN0_WB_RA_NWA: u64 = 3 << 8;
pub const TCR_ORGN0_MSK: u64 = 0x3 << 10;
pub const TCR_ORGN0_NC: u64 = 0 << 10;
pub const TCR_ORGN0_WB_RA_WA: u64 = 1 << 10;
pub const TCR_ORGN0_WT_RA_NWA: u64 = 2 << 10;
pub const TCR_ORGN0_WB_RA_NWA: u64 = 3 << 10;
pub const TCR_SH0_MSK: u64 = 0x3 << 12;
pub const TCR_SH0_NS: u64 = 0 << 12;
pub const TCR_SH0_OS: u64 = 2 << 12;
pub const TCR_SH0_IS: u64 = 3 << 12;
pub const TCR_TG0_MSK: u64 = 0x3 << 14;
pub const TCR_TG0_4K: u64 = 0 << 14;
pub const TCR_TG0_16K: u64 = 2 << 14;
pub const TCR_TG0_64K: u64 = 1 << 14;
pub const TCR_PS_OFF: u64 = 16;
pub const TCR_PS_MSK: u64 = 0x7 << TCR_PS_OFF;
pub const TCR_PS_32B: u64 = 0 << 16;
pub const TCR_PS_36B: u64 = 1 << 16;
pub const TCR_PS_40B: u64 = 2 << 16;
pub const TCR_PS_42B: u64 = 3 << 16;
pub const TCR_PS_44B: u64 = 4 << 16;
pub const TCR_PS_48B: u64 = 5 << 16;
pub const TCR_PS_52B: u64 = 6 << 16;
pub const TCR_TBI: u64 = 1 << 20;

/**
 * Default hypervisor translation control
 * The PS field must be filled at runtime by first reading parange
 */
pub const TCR_EL2_DFLT: u64 = TCR_RES1
    | TCR_TG0_4K
    | TCR_PS_48B
    | TCR_ORGN0_WB_RA_WA
    | TCR_IRGN0_WB_RA_WA
    | tcr_t0sz(16)
    | TCR_SH0_IS;

pub const HTCR_DFLT: u64 = TCR_SH0_IS | TCR_ORGN0_WB_RA_WA | TCR_IRGN0_WB_RA_WA | tcr_t0sz(0);

// VTCR - Translation Control Register
pub const VTCR_RES1: u64 = (1 << 31);
pub const VTCR_MSA: u64 = (1 << 31);
pub const VTCR_T0SZ_MSK: u64 = 0x1f << 0;
pub const VTCR_T0SZ_OFF: u64 = 0;
pub const fn vtcr_t0sz(sz: u64) -> u64 {
    (sz << VTCR_T0SZ_OFF) & VTCR_T0SZ_MSK
}
pub const VTCR_SL0_OFF: u64 = 6;
pub const VTCR_SL0_MSK: u64 = 0xc0;
pub const VTCR_SL0_01: u64 = ((0x2 << VTCR_SL0_OFF) & VTCR_SL0_MSK);
pub const VTCR_SL0_12: u64 = ((0x1 << VTCR_SL0_OFF) & VTCR_SL0_MSK);
pub const VTCR_SL0_23: u64 = 0;
pub const VTCR_IRGN0_MSK: u64 = 0x3 << 8;
pub const VTCR_IRGN0_NC: u64 = 0 << 8;
pub const VTCR_IRGN0_WB_RA_WA: u64 = 1 << 8;
pub const VTCR_IRGN0_WT_RA_NWA: u64 = 2 << 8;
pub const VTCR_IRGN0_WB_RA_NWA: u64 = 3 << 8;
pub const VTCR_ORGN0_MSK: u64 = 0x3 << 10;
pub const VTCR_ORGN0_NC: u64 = 0 << 10;
pub const VTCR_ORGN0_WB_RA_WA: u64 = 1 << 10;
pub const VTCR_ORGN0_WT_RA_NWA: u64 = 2 << 10;
pub const VTCR_ORGN0_WB_RA_NWA: u64 = 3 << 10;
pub const VTCR_SH0_MSK: u64 = 0x3 << 12;
pub const VTCR_SH0_NS: u64 = 0 << 12;
pub const VTCR_SH0_OS: u64 = 2 << 12;
pub const VTCR_SH0_IS: u64 = 3 << 12;
pub const VTCR_TG0_MSK: u64 = 0x3 << 14;
pub const VTCR_TG0_4K: u64 = 0 << 14;
pub const VTCR_TG0_16K: u64 = 2 << 14;
pub const VTCR_TG0_64K: u64 = 1 << 14;
pub const VTCR_PS_OFF: u64 = 16;
pub const VTCR_PS_MSK: u64 = 0x7 << VTCR_PS_OFF;
pub const VTCR_PS_32B: u64 = 0 << 16;
pub const VTCR_PS_36B: u64 = 1 << 16;
pub const VTCR_PS_40B: u64 = 2 << 16;
pub const VTCR_PS_42B: u64 = 3 << 16;
pub const VTCR_PS_44B: u64 = 4 << 16;
pub const VTCR_PS_48B: u64 = 5 << 16;
pub const VTCR_PS_52B: u64 = 6 << 16;
pub const VTCR_TBI: u64 = 1 << 20;

/**
 * Default stage-2 translation control
 * ...
 */
pub const VTCR_DFLT: u64 = (VTCR_RES1
    | VTCR_PS_40B
    | VTCR_TG0_4K
    | VTCR_ORGN0_WB_RA_WA
    | VTCR_IRGN0_WB_RA_WA
    | vtcr_t0sz(24)
    | VTCR_SL0_12
    | VTCR_SH0_IS);

// MAIR - Memory Attribute Indirection Register
pub const MAIR_ATTR_WIDTH: u64 = 8;
pub const MAIT_ATTR_NUM: u64 = 8;

pub const MAIR_DEV_nGnRnE: u64 = (0x0 << 2);
pub const MAIR_DEV_nGnRE: u64 = (0x1 << 2);
pub const MAIR_DEV_nGRE: u64 = (0x2 << 2);
pub const MAIR_DEV_GRE: u64 = (0x3 << 2);

pub const MAIR_OWTT: u64 = (0x0 << 6);
pub const MAIR_ONC: u64 = (0x1 << 6);
pub const MAIR_OWBT: u64 = (0x1 << 6);
pub const MAIR_OWTNT: u64 = (0x2 << 6);
pub const MAIR_OWBNT: u64 = (0x3 << 6);
pub const MAIR_ORA: u64 = (0x1 << 5);
pub const MAIR_OWA: u64 = (0x1 << 4);

pub const MAIR_IWTT: u64 = (0x0 << 2);
pub const MAIR_INC: u64 = (0x1 << 2);
pub const MAIR_IWBT: u64 = (0x1 << 2);
pub const MAIR_IWTNT: u64 = (0x2 << 2);
pub const MAIR_IWBNT: u64 = (0x3 << 2);
pub const MAIR_IRA: u64 = (0x1 << 1);
pub const MAIR_IWA: u64 = (0x1 << 0);

/**
 * Default hypervisor memory attributes
 * 0 -> Device-nGnRnE
 * 1 -> Normal, Inner/Outer  WB/WA/RA
 * 2 -> Device-nGnRE
 */
pub const MAIR_EL2_DFLT: u64 =
    ((MAIR_OWBNT | MAIR_ORA | MAIR_OWA | MAIR_IWBNT | MAIR_IRA | MAIR_IWA) << MAIR_ATTR_WIDTH)
        | ((MAIR_DEV_nGnRE) << (MAIR_ATTR_WIDTH * 2));

pub const SCTLR_RES1: u64 = 0x30C50830;
pub const SCTLR_RES1_AARCH32: u64 = 0x30C50818;
pub const SCTLR_M: u64 = 1 << 0;
pub const SCTLR_A: u64 = 1 << 1;
pub const SCTLR_C: u64 = 1 << 2;
pub const SCTLR_SA: u64 = 1 << 3;
pub const SCTLR_I: u64 = 1 << 12;
pub const SCTLR_BR: u64 = 1 << 17;
pub const SCTLR_WXN: u64 = 1 << 19;
pub const SCTLR_EE: u64 = 1 << 25;

pub const SCTLR_DFLT: u64 = SCTLR_RES1 | SCTLR_M | SCTLR_C | SCTLR_I;

pub const MPIDR_RES1: u64 = 0x80000000;
pub const MPIDR_RES0_MSK: u64 = !(0x1f << 25);
pub const MPIDR_AFFINITY_BITS: u64 = 8;
pub const MPIDR_U_BIT: u64 = 1 << 30;
pub const MPIDR_AFF_MSK: u64 = 0xffff;

pub const PAR_F: u64 = 1;
pub const PAR_PA_MSK: u64 = 0x3ffffff << 12;

pub const ICC_SRE_ENB_BIT: u64 = 0x8;
pub const ICC_SRE_DIB_BIT: u64 = 0x4;
pub const ICC_SRE_DFB_BIT: u64 = 0x2;
pub const ICC_SRE_SRE_BIT: u64 = 0x1;
pub const ICC_IGRPEN_EL1_ENB_BIT: u64 = 0x1;
pub const ICC_CTLR_EOIMode_BIT: u32 = 0x1 << 1;

// Hypervisor Configuration Register
pub const HCR_VM_BIT: u64 = 1 << 0;
pub const HCR_SWIO_BIT: u64 = 1 << 1;
pub const HCR_PTW_BIT: u64 = 1 << 2;
pub const HCR_FMO_BIT: u64 = 1 << 3;
pub const HCR_IMO_BIT: u64 = 1 << 4;
pub const HCR_AMO_BIT: u64 = 1 << 5;
pub const HCR_VF_BIT: u64 = 1 << 6;
pub const HCR_VI_BIT: u64 = 1 << 7;
pub const HCR_VSE_BIT: u64 = 1 << 8;
pub const HCR_FB_BIT: u64 = 1 << 9;
pub const HCR_BSU_BIT: u64 = 1 << 10;
pub const HCR_DC_BIT: u64 = 1 << 12;
pub const HCR_TWI_BIT: u64 = 1 << 13;
pub const HCR_TWE_BIT: u64 = 1 << 14;
pub const HCR_TID0_BIT: u64 = 1 << 15;
pub const HCR_TID1_BIT: u64 = 1 << 16;
pub const HCR_TID2_BIT: u64 = 1 << 17;
pub const HCR_TID3_BIT: u64 = 1 << 18;
pub const HCR_TSC_BIT: u64 = 1 << 19;
pub const HCR_TIDCP_BIT: u64 = 1 << 20;
pub const HCR_TACR_BIT: u64 = 1 << 21;
pub const HCR_TSW_BIT: u64 = 1 << 22;
pub const HCR_TPC_BIT: u64 = 1 << 23;
pub const HCR_TPU_BIT: u64 = 1 << 24;
pub const HCR_TTLB_BIT: u64 = 1 << 25;
pub const HCR_TVM_BIT: u64 = 1 << 26;
pub const HCR_TGE_BIT: u64 = 1 << 27;
pub const HCR_TDZ_BIT: u64 = 1 << 28;
pub const HCR_HCD_BIT: u64 = 1 << 29;
pub const HCR_TRVM_BIT: u64 = 1 << 30;
pub const HCR_RW_BIT: u64 = 1 << 31;
pub const HCR_CD_BIT: u64 = 1 << 32;
pub const HCR_ID_BIT: u64 = 1 << 33;
pub const HCR_TERR_BIT: u64 = 1 << 36;
pub const HCR_TEA_BIT: u64 = 1 << 37;
pub const HCR_MIOCNCE_BIT: u64 = 1 << 38;
pub const HCR_APK_BIT: u64 = 1 << 40;
pub const HCR_API_BIT: u64 = 1 << 41;

pub fn mpidr_aff_lvl(mpidr: u64, lvl: u64) -> u64 {
    ((mpidr >> (8 * lvl)) & 0xff) as u64
}

pub fn arm_at_s1e2w(vaddr: Vaddr) -> u64 {
    unsafe {
        asm!("at s1e2w, {}", in(reg) vaddr);
        isb();
        PAR_EL1.get()
    }
}

pub fn arm_at_s12e1w(vaddr: Vaddr) -> u64 {
    unsafe {
        asm!("at s12e1w, {}", in(reg) vaddr);
        isb();
        PAR_EL1.get()
    }
}

// *************** GICH Regs **************

#[macro_export]
macro_rules! read_reg {
    ($reg_name:ident) => {
        {
            let reg_val: u64;
            unsafe {
                core::arch::asm!(
                    concat!("mrs {reg_}, ", stringify!($reg_name)),
                    reg_ = out(reg) reg_val,
                );
            }
            reg_val
        }
    };
}

#[macro_export]
macro_rules! write_reg {
    ($reg_name:ident, $val:expr) => {
        unsafe {
            core::arch::asm!(
                concat!("msr ", stringify!($reg_name), ", {reg_}"),
                reg_ = in(reg) $val,
            );
        }
    };
}
