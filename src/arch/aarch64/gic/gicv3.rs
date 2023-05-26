#![allow(non_snake_case)]
use spin::Mutex;

use super::GIC;
use super::{gic_defs::*, gicd::GicdHw};
use crate::arch::aarch64::sysregs::*;
use crate::baocore::types::IrqID;
use crate::util::bit32_mask;
use crate::{
    baocore::{
        cpu::mycpu,
        mmu::sections::SEC_HYP_GLOBAL,
        types::{CpuID, Vaddr},
    },
    platform::PLATFORM,
    read_reg,
    util::num_pages,
    write_reg,
};

#[repr(C)]
#[repr(align(0x10000))]
pub struct GicrHw {
    /* RD_base frame */
    CTLR: u32,
    IIDR: u32,
    TYPER: u64,
    STATUSR: u32,
    WAKER: u32,
    pad0: [u8; 0x0040 - 0x0018],
    SETLPIR: u64,
    CLRLPIR: u64,
    pad1: [u8; 0x0070 - 0x0050],
    PROPBASER: u64,
    PENDBASER: u64,
    pad2: [u8; 0x00A0 - 0x0080],
    INVLPIR: u64,
    pad3: [u8; 0x00B0 - 0x00A8],
    INVALLR: u64,
    pad4: [u8; 0x00c0 - 0x00b8],
    SYNCR: u64,
    pad5: [u8; 0xFFD0 - 0x00c8],
    ID: [u32; (0x10000 - 0xFFD0) / core::mem::size_of::<u32>()],

    /* SGI_base frame */
    sgi_base: SgiBase,
    pad6: [u8; 0x0080 - 0x000],
    IGROUPR0: u32,
    pad7: [u8; 0x0100 - 0x084],
    ISENABLER0: u32,
    pad8: [u8; 0x0180 - 0x104],
    ICENABLER0: u32,
    pad9: [u8; 0x0200 - 0x184],
    ISPENDR0: u32,
    pad10: [u8; 0x0280 - 0x204],
    ICPENDR0: u32,
    pad11: [u8; 0x0300 - 0x284],
    ISACTIVER0: u32,
    pad12: [u8; 0x0380 - 0x304],
    ICACTIVER0: u32,
    pad13: [u8; 0x0400 - 0x384],
    IPRIORITYR: [u32; gic_prio_regs(GIC_CPU_PRIV)],
    pad14: [u8; 0x0c00 - 0x420],
    ICFGR0: u32,
    ICFGR1: u32,
    pad15: [u8; 0x0D00 - 0xc08],
    IGRPMODR0: u32,
    pad16: [u8; 0x0e00 - 0xd04],
    NSACR: u32,
}

impl GicrHw {
    pub fn init(&mut self) {
        self.WAKER &= !GICR_WAKER_ProcessorSleep_BIT;

        self.IGROUPR0 = u32::MAX;
        self.ICENABLER0 = u32::MAX;
        self.ICPENDR0 = u32::MAX;
        self.ICACTIVER0 = u32::MAX;

        for i in 0..gic_prio_regs(GIC_CPU_PRIV) {
            self.IPRIORITYR[i] = u32::MAX;
        }
    }
    pub fn set_enable(&mut self, id: IrqID, en: bool) {
        let bit = gic_int_mask(id as _);

        if en {
            self.ISENABLER0 = bit as _;
        } else {
            self.ICENABLER0 = bit as _;
        }
    }

    pub fn set_act(&mut self, id: IrqID, act: bool) {
        let bit = gic_int_mask(id as _);

        if act {
            self.ISACTIVER0 = bit as _;
        } else {
            self.ICACTIVER0 = bit as _;
        }
    }

    pub fn set_pend(&mut self, id: IrqID, pend: bool) {
        let bit = gic_int_mask(id as _);

        if pend {
            self.ISPENDR0 = bit as _;
        } else {
            self.ICPENDR0 = bit as _;
        }
    }

    pub fn set_prio(&mut self, id: IrqID, prio: u8) {
        let reg_ind = gic_prio_regs(id as _);
        let off = gic_prio_off(id as _);
        let mask = bit32_mask(off as _, GIC_PRIO_BITS as _);

        self.IPRIORITYR[reg_ind] =
            (self.IPRIORITYR[reg_ind] & !mask) | (((prio as u32) << off) & mask);
    }

    pub fn set_cfg(&mut self, id: IrqID, cfg: u8) {
        let reg_id = gic_config_regs(id as _);
        let off = gic_config_off(id as _);
        let mask = bit32_mask(off as _, GIC_CONFIG_BITS as _);

        match reg_id {
            0 => self.ICFGR0 = (self.ICFGR0 & !mask) | (((cfg as u32) << off) & mask),
            1 => self.ICFGR1 = (self.ICFGR1 & !mask) | (((cfg as u32) << off) & mask),
            _ => panic!("set_cfg: invalid intr id")
        }
    }
}

#[repr(C)]
#[repr(align(0x10000))]
struct SgiBase();

pub fn gic_map_mmio() -> (Vaddr, Vaddr) {
    let addr_space = &mut mycpu().addr_space;
    let gicd = addr_space
        .mem_alloc_map_dev(
            SEC_HYP_GLOBAL,
            PLATFORM.arch.gic.gicd_addr,
            None,
            num_pages(core::mem::size_of::<GicdHw>()),
        )
        .unwrap();
    let gicr = addr_space
        .mem_alloc_map_dev(
            SEC_HYP_GLOBAL,
            PLATFORM.arch.gic.gicr_addr,
            None,
            PLATFORM.cpu_num * num_pages(core::mem::size_of::<GicrHw>()),
        )
        .unwrap();
    (gicd, gicr)
}

pub struct GicV3 {
    gicd_base: Vaddr,
    gicr_base: Vaddr,
    pub max_irqs: usize,
    pub gicd_lock: Mutex<()>,
    pub gicr_lock: Mutex<()>,
}

impl GicV3 {
    pub fn new(gicd_base: Vaddr, gicr_base: Vaddr) -> Self {
        let mut gic = Self {
            gicd_base,
            gicr_base,
            max_irqs: 0,
            gicd_lock: Mutex::new(()),
            gicr_lock: Mutex::new(()),
        };
        gic.max_irqs = ((gic.gicd().TYPER as usize & 0b11111) + 1) * 32;
        gic
    }

    pub fn gicd_init(&mut self) {
        self.gicd().init(self.max_irqs);
    }

    pub fn each_cpu_init(&mut self, cpu_id: CpuID) {
        self.gicr(cpu_id as _).init();
        self.gicc_init();
    }

    fn gicd(&self) -> &mut GicdHw {
        unsafe { &mut *(self.gicd_base as *mut _) }
    }

    fn gicr(&self, index: usize) -> &mut GicrHw {
        assert!(index < PLATFORM.cpu_num);
        unsafe { &mut *((self.gicr_base as *mut GicrHw).add(index)) }
    }

    fn gicc_init(&self) {
        let num_lrs = gich_num_lrs();
        for i in 0..num_lrs {
            gich_write_lr(i, 0);
        }
        write_reg!(icc_pmr_el1, GIC_LOWEST_PRIO);
        write_reg!(icc_bpr1_el1, 0u64);
        write_reg!(icc_ctlr_el1, ICC_CTLR_EOIMode_BIT as u64);
        let hcr = read_reg!(ich_hcr_el2) as u32 | GICH_HCR_LRENPIE_BIT;
        write_reg!(ich_hcr_el2, hcr as u64);
        write_reg!(icc_igrpen1_el1, ICC_IGRPEN_EL1_ENB_BIT);
    }
}

pub fn gicd_set_enable(id: IrqID, enabled: bool) {
    let gic = unsafe { GIC.get_mut().unwrap() };
    let _lock = gic.gicd_lock.lock();
    gic.gicd().set_enable(id, enabled);
}

pub fn gicd_set_act(id: IrqID, act: bool) {
    let gic = unsafe { GIC.get_mut().unwrap() };
    let _lock = gic.gicd_lock.lock();
    gic.gicd().set_act(id, act);
}

pub fn gicd_set_pend(id: IrqID, pend: bool) {
    let gic = unsafe { GIC.get_mut().unwrap() };
    let _lock = gic.gicd_lock.lock();
    gic.gicd().set_pend(id, pend);
}

pub fn gicd_set_prio(id: IrqID, prio: u8) {
    let gic = unsafe { GIC.get_mut().unwrap() };
    let _lock = gic.gicd_lock.lock();
    gic.gicd().set_prio(id, prio);
}

pub fn gicd_set_route(id: IrqID, route: u64) {
    let gic = unsafe { GIC.get_mut().unwrap() };
    let _lock = gic.gicd_lock.lock();
    gic.gicd().set_route(id, route);
}

pub fn gicd_set_cfg(id: IrqID, cfg: u8) {
    let gic = unsafe { GIC.get_mut().unwrap() };
    let _lock = gic.gicd_lock.lock();
    gic.gicd().set_cfg(id, cfg as _);
}

pub fn gicd_get_pidr(addr: u64) -> u32 {
    let gic = unsafe { GIC.get_mut().unwrap() };
    let _lock = gic.gicd_lock.lock();
    gic.gicd().ID[((addr as usize & 0xffff) - 0xffd0) / 4]
}

pub fn gicd_get_iidr() -> u32 {
    let gic = unsafe { GIC.get_mut().unwrap() };
    let _lock = gic.gicd_lock.lock();
    gic.gicd().IIDR
}
// ---------------------------------------------------------------

pub fn gicr_set_enable(id: IrqID, enabled: bool, gicr_id: CpuID) {
    let gic = unsafe { GIC.get_mut().unwrap() };
    let _lock = gic.gicr_lock.lock();
    gic.gicr(gicr_id as _).set_enable(id, enabled);
}

pub fn gicr_set_act(id: IrqID, act: bool, gicr_id: CpuID) {
    let gic = unsafe { GIC.get_mut().unwrap() };
    let _lock = gic.gicr_lock.lock();
    gic.gicr(gicr_id as _).set_act(id, act);
}

pub fn gicr_set_pend(id: IrqID, pend: bool, gicr_id: CpuID) {
    let gic = unsafe { GIC.get_mut().unwrap() };
    let _lock = gic.gicr_lock.lock();
    gic.gicr(gicr_id as _).set_pend(id, pend);
}

pub fn gicr_set_prio(id: IrqID, prio: u8, gicr_id: CpuID) {
    let gic = unsafe { GIC.get_mut().unwrap() };
    let _lock = gic.gicr_lock.lock();
    gic.gicr(gicr_id as _).set_prio(id, prio);
}

pub fn gicr_set_cfg(id: IrqID, cfg: u8, gicr_id: CpuID) {
    let gic = unsafe { GIC.get_mut().unwrap() };
    let _lock = gic.gicr_lock.lock();
    gic.gicr(gicr_id as _).set_cfg(id, cfg);
}

pub fn gicr_get_pidr(addr: u64) -> u32 {
    let gic = unsafe { GIC.get_mut().unwrap() };
    let _lock = gic.gicr_lock.lock();
    gic.gicr(0).ID[((addr as usize & 0xffff) - 0xffd0) / 4]
}

fn gich_num_lrs() -> u32 {
    ((read_reg!(ich_vtr_el2) as u32 & GICH_VTR_MSK) >> GICH_VTR_OFF) + 1
}

pub fn gich_get_hcr() -> u32 {
    read_reg!(ich_hcr_el2) as u32
}

pub fn gich_set_hcr(hcr: u32) {
    write_reg!(ich_hcr_el2, hcr as u64);
}

pub fn gich_write_lr(i: u32, val: u64) {
    match i {
        0 => write_reg!(ich_lr0_el2, val),
        1 => write_reg!(ich_lr1_el2, val),
        2 => write_reg!(ich_lr2_el2, val),
        3 => write_reg!(ich_lr3_el2, val),
        4 => write_reg!(ich_lr4_el2, val),
        5 => write_reg!(ich_lr5_el2, val),
        6 => write_reg!(ich_lr6_el2, val),
        7 => write_reg!(ich_lr7_el2, val),
        8 => write_reg!(ich_lr8_el2, val),
        9 => write_reg!(ich_lr9_el2, val),
        10 => write_reg!(ich_lr10_el2, val),
        11 => write_reg!(ich_lr11_el2, val),
        12 => write_reg!(ich_lr12_el2, val),
        13 => write_reg!(ich_lr13_el2, val),
        14 => write_reg!(ich_lr14_el2, val),
        15 => write_reg!(ich_lr15_el2, val),
        _ => panic!("gich_write_lr: index out of range"),
    }
}

pub fn gicc_iar() -> u64 {
    read_reg!(icc_iar1_el1)
}

pub fn gicc_eoir(eoir: u32) {
    write_reg!(icc_eoir1_el1, eoir as u64)
}

pub fn gicc_dir(dir: u32) {
    write_reg!(icc_dir_el1, dir as u64)
}
