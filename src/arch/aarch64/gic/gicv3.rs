#![allow(non_snake_case)]
use super::{gic_defs::*, gicd::GicdHw};
use crate::arch::aarch64::sysregs::*;
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
    max_irqs: usize,
}

impl GicV3 {
    pub fn new(gicd_base: Vaddr, gicr_base: Vaddr) -> Self {
        let mut gic = Self {
            gicd_base,
            gicr_base,
            max_irqs: 0,
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

fn gich_num_lrs() -> u32 {
    ((read_reg!(ich_vtr_el2) as u32 & GICH_VTR_MSK) >> GICH_VTR_OFF) + 1
}

fn gich_write_lr(i: u32, val: u64) {
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
