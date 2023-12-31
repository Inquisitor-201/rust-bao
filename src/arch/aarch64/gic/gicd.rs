#![allow(non_snake_case)]

use crate::{baocore::types::IrqID, util::bit32_mask};

use super::{
    gic_defs::{
        gic_config_regs, gic_int_mask, gic_int_regs, gic_prio_off, gic_prio_regs, gic_sec_regs,
        gic_target_regs, GICD_CTLR_ARE_NS_BIT, GICD_CTLR_ENA_BIT, GICD_IROUTER_INV, GIC_CPU_PRIV,
        GIC_MAX_INTERUPTS, GIC_NUM_PRIVINT_REGS, GIC_NUM_SGI_REGS, GIC_PRIO_BITS, gic_config_off, GIC_CONFIG_BITS,
    },
    GicVersion, GIC_VERSION,
};

#[repr(C)]
#[repr(align(0x10000))]
pub struct GicdHw {
    pub CTLR: u32,                         // 0x0
    pub TYPER: u32,                        // 0x4
    pub IIDR: u32,                         // 0x8
    pub pad0: [u8; 0x0010 - 0x000C],
    pub STATUSR: u32,                      // 0x10
    pub pad1: [u8; 0x0040 - 0x0014],
    pub SETSPI_NSR: u32,                   // 0x40
    pub pad2: [u8; 0x0048 - 0x0044],
    pub CLRSPI_NSR: u32,                   // 0x48
    pub pad3: [u8; 0x0050 - 0x004C],
    pub SETSPI_SR: u32,                    // 0x50
    pub pad4: [u8; 0x0058 - 0x0054],
    pub CLRSPI_SR: u32,                    // 0x58
    pub pad9: [u8; 0x0080 - 0x005C],
    pub IGROUPR: [u32; gic_int_regs(GIC_MAX_INTERUPTS)], // 0x80
    pub ISENABLER: [u32; gic_int_regs(GIC_MAX_INTERUPTS)], // 0x100
    pub ICENABLER: [u32; gic_int_regs(GIC_MAX_INTERUPTS)], // 0x180
    pub ISPENDR: [u32; gic_int_regs(GIC_MAX_INTERUPTS)], // 0x200
    pub ICPENDR: [u32; gic_int_regs(GIC_MAX_INTERUPTS)], // 0x280
    pub ISACTIVER: [u32; gic_int_regs(GIC_MAX_INTERUPTS)], // 0x300
    pub ICACTIVER: [u32; gic_int_regs(GIC_MAX_INTERUPTS)], // 0x380
    pub IPRIORITYR: [u32; gic_prio_regs(GIC_MAX_INTERUPTS)], // 0x400
    pub ITARGETSR: [u32; gic_target_regs(GIC_MAX_INTERUPTS)], // 0x800
    pub ICFGR: [u32; gic_config_regs(GIC_MAX_INTERUPTS)], // 0xc00
    pub IGPRMODR: [u32; gic_int_regs(GIC_MAX_INTERUPTS)], // 0xd00
    pub pad5: [u8; 0x0E00 - 0x0D80],
    pub NSACR: [u32; gic_sec_regs(GIC_MAX_INTERUPTS)], // 0xe00
    pub SGIR: u32,
    pub pad6: [u8; 0x0F10 - 0x0F04],
    pub CPENDSGIR: [u32; GIC_NUM_SGI_REGS],
    pub SPENDSGIR: [u32; GIC_NUM_SGI_REGS],
    pub pad7: [u8; 0x6000 - 0x0F30],
    pub IROUTER: [u64; GIC_MAX_INTERUPTS], // 0x6000
    pub pad8: [u8; 0xFFD0 - 0x8000],
    pub ID: [u32; (0x10000 - 0xFFD0) / core::mem::size_of::<u32>()],
}

impl GicdHw {
    pub fn init(&mut self, int_num: usize) {
        // Bring distributor to known state
        for i in GIC_NUM_PRIVINT_REGS..gic_int_regs(int_num) {
            // Make sure all interrupts are not enabled, non-pending,
            // non-active.
            self.IGROUPR[i] = u32::MAX;
            self.ICENABLER[i] = u32::MAX;
            self.ICPENDR[i] = u32::MAX;
            self.ICACTIVER[i] = u32::MAX;
        }

        // All interrupts have the lowest priority possible by default
        for i in gic_prio_regs(GIC_CPU_PRIV)..gic_prio_regs(int_num) {
            self.IPRIORITYR[i] = u32::MAX;
        }

        match GIC_VERSION {
            GicVersion::GicVersion2 => {
                todo!("gicv2: GICD init")
                // // No CPU targets for any interrupt by default
                // for i in GIC_NUM_TARGET_REGS(GIC_CPU_PRIV)..GIC_NUM_TARGET_REGS(int_num) {
                //     self.itargetsr[i].write(0);
                // }

                // // Enable distributor
                // gicd.ctlr.modify(|val| val | GICD_CTLR_EN_BIT);
            }
            GicVersion::GicVersion3 => {
                for i in GIC_CPU_PRIV..GIC_MAX_INTERUPTS {
                    self.IROUTER[i] = GICD_IROUTER_INV;
                }

                // Enable distributor and affinity routing
                self.CTLR |= GICD_CTLR_ARE_NS_BIT | GICD_CTLR_ENA_BIT;
            }
        }
    }

    pub fn set_enable(&mut self, id: IrqID, en: bool) {
        let reg_ind = gic_int_regs(id as _);
        let bit = gic_int_mask(id as _);

        if en {
            self.ISENABLER[reg_ind] = bit as _;
        } else {
            self.ICENABLER[reg_ind] = bit as _;
        }
    }

    pub fn set_act(&mut self, id: IrqID, act: bool) {
        let reg_ind = gic_int_regs(id as _);
        let bit = gic_int_mask(id as _);

        if act {
            self.ISACTIVER[reg_ind] = bit as _;
        } else {
            self.ICACTIVER[reg_ind] = bit as _;
        }
    }

    pub fn set_pend(&mut self, id: IrqID, pend: bool) {
        let reg_ind = gic_int_regs(id as _);
        let bit = gic_int_mask(id as _);

        if pend {
            self.ISPENDR[reg_ind] = bit as _;
        } else {
            self.ICPENDR[reg_ind] = bit as _;
        }
    }

    pub fn set_prio(&mut self, id: IrqID, prio: u8) {
        let reg_ind = gic_prio_regs(id as _);
        let off = gic_prio_off(id as _);
        let mask = bit32_mask(off as _, GIC_PRIO_BITS as _);

        self.IPRIORITYR[reg_ind] =
            (self.IPRIORITYR[reg_ind] & !mask) | (((prio as u32) << off) & mask);
    }

    pub fn set_route(&mut self, id: IrqID, route: u64) {
        self.IROUTER[id as usize] = route;
    }

    pub fn set_cfg(&mut self, id: IrqID, cfg: u8) {
        let reg_ind = gic_config_regs(id as _);
        let off = gic_config_off(id as _);
        let mask = bit32_mask(off as _, GIC_CONFIG_BITS as _);
        self.ICFGR[reg_ind] = (self.ICFGR[reg_ind] & !mask) | (((cfg as u32) << off) & mask);
    }
}
