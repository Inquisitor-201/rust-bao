use alloc::vec::Vec;
use spin::{Mutex, RwLock};

use crate::{
    arch::aarch64::gic::{gic_is_priv, gicd_reg_mask, VGIC_ENABLE_MASK},
    baocore::{
        emul::EmulAccess,
        types::{IrqID, VCpuID, Vaddr},
        vm::{myvcpu, myvm},
    },
    println,
    util::bit64_extract, debug,
};

use super::{gic_defs::{GICD_CTLR_ARE_NS_BIT, GIC_CPU_PRIV}, vgicv3::VGicR, GicVersion, GIC_VERSION};

pub struct VGicIntr {
    inner: RwLock<VGicIntrInner>
}

impl VGicIntr {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(VGicIntrInner::new())
        }
    }

    pub fn vgic_int_set_field(&mut self, _handlers: &VGicHandlerInfo, _data: u64) {
        // todo: vgic_int_set_field
    }
}

pub struct VGicIntrInner {}

impl VGicIntrInner {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct VGicD {
    pub interrupts: Vec<VGicIntr>,
    pub int_num: usize,
    pub ctlr: u32,
    pub typer: u32,
    pub lock: Mutex<()>
}

impl VGicD {
    pub fn new() -> Self {
        Self {
            interrupts: Vec::new(),
            int_num: 0,
            ctlr: 0,
            typer: 0,
            lock: Mutex::new(()),
        }
    }
}

pub struct VGicPriv {
    pub vgicr: VGicR,
    pub _curr_lrs: Vec<IrqID>,
    pub interrupts: Vec<VGicIntr>,
}

impl VGicPriv {
    pub fn new() -> Self {
        Self {
            vgicr: VGicR {
                lock: Mutex::new(()),
                typer: 0,
                ctlr: 0,
                iidr: 0,
            },
            _curr_lrs: Vec::new(),
            interrupts: {
                let mut intrs = Vec::with_capacity(GIC_CPU_PRIV);
                for _ in 0..GIC_CPU_PRIV {
                    intrs.push(VGicIntr::new());
                }
                intrs
            },
        }
    }
}

pub fn vgicd_emul_handler(acc: &EmulAccess) -> bool {
    let gicd_reg_group = gicd_reg_mask(acc.addr) >> 7;
    let handler_info = match gicd_reg_group {
        GICD_REG_GROUP_CTLR => VGicHandlerInfo {
            reg_access: vgicd_emul_misc_access,
            field_width: 0,
            regroup_base: 0,
        },
        GICD_REG_GROUP_IGROUPR => VGicHandlerInfo {
            reg_access: vgic_emul_razwi,
            field_width: 0,
            regroup_base: 0,
        },
        GICD_REG_GROUP_ISENABLER => VGicHandlerInfo {
            reg_access: vgic_emul_generic_access,
            field_width: 1,
            regroup_base: GICD_REG_ISENABLER_OFF,
        },
        GICD_REG_GROUP_ICENABLER => VGicHandlerInfo {
            reg_access: vgic_emul_generic_access,
            field_width: 1,
            regroup_base: GICD_REG_ICENABLER_OFF,
        },
        GICD_REG_GROUP_ICPENDR => VGicHandlerInfo {
            reg_access: vgic_emul_generic_access,
            field_width: 1,
            regroup_base: GICD_REG_ICPENDR_OFF,
        },
        GICD_REG_GROUP_ICACTIVER => VGicHandlerInfo {
            reg_access: vgic_emul_generic_access,
            field_width: 1,
            regroup_base: GICD_REG_ICACTIVER_OFF,
        },
        _ => {
            let gicd_reg = gicd_reg_mask(acc.addr);
            if gicd_reg >= GICD_REG_IPRIORITYR_OFF && gicd_reg < (GICD_REG_IPRIORITYR_OFF + 0x400) {
                VGicHandlerInfo {
                    reg_access: vgic_emul_generic_access,
                    regroup_base: GICD_REG_IPRIORITYR_OFF,
                    field_width: 8,
                }
            } else if gicd_reg >= GICD_REG_ITARGETSR_OFF && gicd_reg < (GICD_REG_ITARGETSR_OFF + 0x400) {
                VGicHandlerInfo {
                    reg_access: vgic_emul_razwi,
                    field_width: 0,
                    regroup_base: 0,
                }
            } else if gicd_reg >= GICD_REG_IROUTER_OFF && gicd_reg < (GICD_REG_IROUTER_OFF + 0x2000) {
                VGicHandlerInfo {
                    reg_access: vgic_emul_generic_access,
                    field_width: 64,
                    regroup_base: GICD_REG_IROUTER_OFF,
                }
            } else {
                todo!("vgicd_emul_handler");
            }
        }
    };

    let _vgicd_mutex = myvm().arch.vgicd.lock.lock();
    (handler_info.reg_access)(acc, &handler_info, false, myvcpu().id);
    true
}

pub struct VGicHandlerInfo {
    pub reg_access:
        fn(acc: &EmulAccess, handlers: &VGicHandlerInfo, gicr_access: bool, vgicr_id: VCpuID),
    pub regroup_base: Vaddr,
    pub field_width: u64,
}

fn vgic_get_int(
    int_id: IrqID,
    vgicr_id: VCpuID,
) -> Option<&'static mut VGicIntr> {
    if gic_is_priv(int_id) {
        Some(&mut myvm().get_vcpu_mut(vgicr_id).arch.vgic_priv.interrupts[int_id as usize])
    } else if int_id < myvm().arch.vgicd.int_num as _ {
        assert!(myvm().arch.vgicd.lock.is_locked());
        Some(&mut myvm().arch.vgicd.interrupts[int_id as usize - GIC_CPU_PRIV])
    } else {
        None
    }
}

pub fn vgic_emul_razwi(
    acc: &EmulAccess,
    _handlers: &VGicHandlerInfo,
    _gicr_access: bool,
    _vgicr_id: VCpuID,
) {
    if !acc.write {
        myvcpu().write_reg(acc.reg, 0);
    }
}

pub fn vgicd_emul_misc_access(
    acc: &EmulAccess,
    _handlers: &VGicHandlerInfo,
    _gicr_access: bool,
    _vgicr_id: VCpuID,
) {
    let vgicd = &mut myvm().arch.vgicd;
    let reg = acc.addr & 0x7F;

    match reg {
        GICD_REG_INDEX_CTLR => {
            if acc.write {
                // let prev_ctrl = inner.ctlr;
                vgicd.ctlr = myvcpu().read_reg(acc.reg) as u32 & VGIC_ENABLE_MASK;
                debug!("write gicd.ctlr: {:#x?}", vgicd.ctlr);
                // if prev_ctrl ^ vgicd.CTLR != 0 {
                //     vgic_update_enable(cpu().vcpu);
                //     let msg = CpuMsg {
                //         id: vgic_initPI_ID,
                //         data: [
                //             VGIC_UPDATE_ENABLE,
                //             VGIC_MSG_DATA(cpu().vcpu.vm.id, 0, 0, 0, 0),
                //         ],
                //     };
                //     vm_msg_broadcast(cpu().vcpu.vm, &msg);
                // }
            } else {
                myvcpu().write_reg(
                    acc.reg as u64,
                    (vgicd.ctlr | GICD_CTLR_ARE_NS_BIT) as u64,
                );
                debug!("read gicd.ctlr: {:#x?}", vgicd.ctlr | GICD_CTLR_ARE_NS_BIT);
            }
        }
        GICD_REG_INDEX_TYPER => {
            if !acc.write {
                myvcpu().write_reg(acc.reg, vgicd.typer as _);
                debug!("read gicd.typer: {:#x?}", vgicd.typer);
            }
        }
        // GICD_REG_IND(IIDR) => {
        //     if !acc.write {
        //         vcpu_writereg(cpu().vcpu, acc.reg as u64, vgicd.IIDR);
        //     }
        // }
        _ => todo!(),
    }
}

pub fn vgic_emul_generic_access(
    acc: &EmulAccess,
    handlers: &VGicHandlerInfo,
    gicr_access: bool,
    vgicr_id: VCpuID,
) {
    let field_width = handlers.field_width;
    let first_int = (gicd_reg_mask(acc.addr) - handlers.regroup_base) * 8 / field_width;
    println!("first_int = {:#x?}", first_int);
    let mut val = if acc.write {
        myvcpu().read_reg(acc.reg)
    } else {
        0
    };
    // let mask = (1u64 << field_width) - 1;
    let valid_access = if let GicVersion::GicVersion2 = GIC_VERSION {
        true
    } else {
        gicr_access == gic_is_priv(first_int as _)
    };

    if valid_access {
        for i in 0..(acc.width * 8) / field_width {
            let interrupt = vgic_get_int((first_int + i) as _, vgicr_id);
            if interrupt.is_none() {
                break;
            }
            if acc.write {
                let data = bit64_extract(val, i * field_width, field_width);
                interrupt.unwrap().vgic_int_set_field(handlers, data);
            } else {
                val = 0;
                // val |= (handlers.read_field(cpu().vcpu, interrupt.unwrap()) & mask)
                //     << (i * field_width);
            }
        }
    }

    // if !acc.write {
    //     vcpu_writereg(cpu().vcpu, acc.reg, val);
    // }
}

pub fn gic_maintenance_handler() {
    todo!("gic_maintenance_handler");
}

// --------------- GICD_REG_GROUPS ------------------

// CTLR GROUP
pub const GICD_REG_GROUP_CTLR: u64 = 0;
pub const GICD_REG_INDEX_CTLR: u64 = 0;
pub const GICD_REG_INDEX_TYPER: u64 = 4;

pub const GICD_REG_ISENABLER_OFF: u64 = 0x100;
pub const GICD_REG_ICENABLER_OFF: u64 = 0x180;
pub const GICD_REG_ICPENDR_OFF: u64 = 0x280;
pub const GICD_REG_ICACTIVER_OFF: u64 = 0x380;
pub const GICD_REG_IPRIORITYR_OFF: u64 = 0x400;
pub const GICD_REG_ITARGETSR_OFF: u64 = 0x800;
pub const GICD_REG_IROUTER_OFF: u64 = 0x6000;

// OTHER GROUPS
pub const GICD_REG_GROUP_IGROUPR: u64 = 0x80 >> 7;
pub const GICD_REG_GROUP_ISENABLER: u64 = 0x100 >> 7;
pub const GICD_REG_GROUP_ICENABLER: u64 = 0x180 >> 7;
pub const GICD_REG_GROUP_ICPENDR: u64 = 0x280 >> 7;
pub const GICD_REG_GROUP_ICACTIVER: u64 = 0x380 >> 7;
pub const GICD_REG_GROUP_IPRIORITYR: u64 = 0x400 >> 7;
pub const GICD_REG_GROUP_ITARGETSR: u64 = 0x800 >> 7;
pub const GICD_REG_GROUP_IROUTER: u64 = 0x6000 >> 7;
