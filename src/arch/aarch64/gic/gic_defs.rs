#![allow(non_upper_case_globals)]

pub const GIC_MAX_INTERUPTS: usize = 1024;
pub const GIC_PRIO_BITS: usize = 8;
pub const GIC_TARGET_BITS: usize = 8;
pub const GIC_CONFIG_BITS: usize = 2;
pub const GIC_SEC_BITS: usize = 2;
pub const GIC_SGI_BITS: usize = 8;

pub const fn gic_int_regs(nint: usize) -> usize {
    nint / (core::mem::size_of::<u32>() * 8)
}

pub const fn gic_prio_regs(nint: usize) -> usize {
    nint * GIC_PRIO_BITS / (core::mem::size_of::<u32>() * 8)
}

pub const fn gic_target_regs(nint: usize) -> usize {
    nint * GIC_TARGET_BITS / (core::mem::size_of::<u32>() * 8)
}

pub const fn gic_config_regs(nint: usize) -> usize {
    nint * GIC_CONFIG_BITS / (core::mem::size_of::<u32>() * 8)
}

pub const fn gic_sec_regs(nint: usize) -> usize {
    nint * GIC_SEC_BITS / (core::mem::size_of::<u32>() * 8)
}

pub const fn gic_sgi_regs(nint: usize) -> usize {
    nint * GIC_SGI_BITS / (core::mem::size_of::<u32>() * 8)
}

pub const GIC_MAX_SGIS: usize = 16;
pub const GIC_MAX_PPIS: usize = 16;
pub const GIC_CPU_PRIV: usize = GIC_MAX_SGIS + GIC_MAX_PPIS;

pub const GIC_NUM_SGI_REGS: usize =
    (GIC_MAX_SGIS * GIC_SGI_BITS) / (core::mem::size_of::<u32>() * 8);
pub const GIC_NUM_PRIVINT_REGS: usize = GIC_CPU_PRIV / (core::mem::size_of::<u32>() * 8);
pub const GIC_LOWEST_PRIO: usize = 0xff;

// ****************  GICD  ******************
pub const MPIDR_AFF_MSK: u64 = 0xffff;
pub const GICD_IROUTER_INV: u64 = !MPIDR_AFF_MSK;

pub const GICD_CTLR_ARE_NS_BIT: u32 = 0x10;
pub const GICD_CTLR_EN_BIT: u32 = 0x1;
pub const GICD_CTLR_ENA_BIT: u32 = 0x2;

// ****************  GICR  ******************
pub const GICR_WAKER_ProcessorSleep_BIT: u32 = 0x2;
pub const GICR_WAKER_ChildrenASleep_BIT: u32 = 0x4;

// ****************  GICH  ******************
pub const GICH_VTR_OFF: u32 = 0;
pub const GICH_VTR_LEN: u32 = 6;
pub const GICH_VTR_MSK: u32 = ((1 << GICH_VTR_LEN) - 1) << GICH_VTR_OFF;
pub const GICH_HCR_LRENPIE_BIT: u32 = 1 << 2;
// gicc...........

