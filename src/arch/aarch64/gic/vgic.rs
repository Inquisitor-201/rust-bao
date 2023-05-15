use spin::RwLock;

use crate::{
    arch::aarch64::gic::{gicd_reg_mask, VGIC_ENABLE_MASK},
    baocore::{emul::EmulAccess, vm::{myvm, myvcpu}},
};

use super::gic_defs::GICD_CTLR_ARE_NS_BIT;

#[derive(Clone, Copy)]
pub struct VGicIntr {}

impl VGicIntr {
    pub fn new() -> Self {
        Self {}
    }
}

const MAX_VGICD_INTR_NUM: usize = 32;

pub struct VGicDInner {
    pub interrupts: [VGicIntr; MAX_VGICD_INTR_NUM],
    pub ctlr: u32,
}

pub struct VGicD {
    pub inner: RwLock<VGicDInner>,
}

impl VGicD {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(VGicDInner {
                interrupts: [VGicIntr::new(); MAX_VGICD_INTR_NUM],
                ctlr: 0,
            }),
        }
    }
}

pub fn vgicd_emul_handler(acc: &EmulAccess) -> bool {
    let gicd_reg_group = gicd_reg_mask(acc.addr) >> 7;
    let handler_info = match gicd_reg_group {
        GICD_REG_GROUP_CTLR => VGicHandlerInfo {
            reg_access: vgicd_emul_misc_access,
        },
        _ => todo!("vgicd_emul_handler"),
    };
    (handler_info.reg_access)(acc);
    true
}

pub struct VGicHandlerInfo {
    pub reg_access: fn(acc: &EmulAccess),
}

pub fn vgic_emul_razwi(acc: &EmulAccess) {
    if !acc.write {
        myvcpu().write_reg(acc.reg, 0);
    }
}

pub fn vgicd_emul_misc_access(acc: &EmulAccess) {
    let vgicd = &mut myvm().arch.vgicd;
    let reg = acc.addr & 0x7F;

    match reg {
        GICD_REG_INDEX_CTLR => {
            if acc.write {
                let mut inner = vgicd.inner.write();
                // let prev_ctrl = inner.ctlr;
                inner.ctlr = myvcpu().read_reg(acc.reg) as u32 & VGIC_ENABLE_MASK;

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
                    (vgicd.inner.read().ctlr | GICD_CTLR_ARE_NS_BIT) as u64 ,
                );
            }
        }
        // GICD_REG_IND(TYPER) => {
        //     if !acc.is_write() {
        //         vcpu_writereg(cpu().vcpu, acc.reg as u64, vgicd.TYPER);
        //     }
        // }
        // GICD_REG_IND(IIDR) => {
        //     if !acc.is_write() {
        //         vcpu_writereg(cpu().vcpu, acc.reg as u64, vgicd.IIDR);
        //     }
        // }
        _ => todo!()
    }
}

pub fn gic_maintenance_handler() {
    todo!("gic_maintenance_handler");
}

// --------------- GICD_REG_GROUPS ------------------

// CTLR GROUPS
pub const GICD_REG_GROUP_CTLR: u64 = 0;
pub const GICD_REG_INDEX_CTLR: u64 = 0;
