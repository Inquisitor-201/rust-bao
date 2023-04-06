#![allow(unused)]
#![allow(non_upper_case_globals)]

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