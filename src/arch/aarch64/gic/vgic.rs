use alloc::vec::Vec;
use spin::{Mutex, RwLock};

use crate::{
    arch::aarch64::gic::{
        gic_is_priv, gicd_reg_mask, gich_get_hcr, gich_set_hcr,
        gicv3::{gicd_set_act, gicd_set_pend, gicd_set_prio, gicd_set_route},
        VGIC_ENABLE_MASK,
    },
    baocore::{
        emul::EmulAccess,
        types::{IrqID, VCpuID, Vaddr},
        vm::{myvcpu, myvm, VCpu, VM},
    },
    debug,
    platform::{ArchPlatformTrait, PLATFORM},
    read_reg,
    util::bit64_extract,
};

use super::{
    gic_defs::{
        GICD_CTLR_ARE_NS_BIT, GICD_IROUTER_INV, GICH_LR_GRP_BIT, GICH_LR_HW_BIT, GIC_CPU_PRIV,
        GIC_MAX_SGIS,
    },
    gic_is_sgi, gich_write_lr,
    gicv3::gicd_set_enable,
    vgicv3::VGicR,
    GicVersion, GIC_VERSION,
};

pub struct VGicIntr {
    pub inner: RwLock<VGicIntrInner>,
}

impl VGicIntr {
    pub fn new(id: IrqID) -> Self {
        Self {
            inner: RwLock::new(VGicIntrInner::new(id)),
        }
    }

    pub fn set_field(&mut self, handlers: &VGicHandlerInfo, data: u64, vcpu: *mut VCpu) {
        // todo: vgic_int_set_field
        let mut intr_inner = self.inner.write();
        if intr_inner.set_ownership(vcpu) {
            // todo: vgic_remove_lr
            let update_field = handlers.update_field.unwrap();
            if update_field(vcpu, &mut intr_inner, data) && intr_inner.is_hw() {
                let update_hw = handlers.update_hw.unwrap();
                update_hw(vcpu, &mut intr_inner);
            }
        }
    }
}

pub struct VGicIntrInner {
    pub owner: Option<*mut VCpu>,
    pub enabled: bool,
    pub pend: bool,
    pub active: bool,
    pub id: IrqID,
    pub hw: bool,
    pub prio: u8,
    pub route: u64,
    pub phys_route: u64,
    pub in_lr: bool,
}

impl VGicIntrInner {
    pub fn new(id: IrqID) -> Self {
        Self {
            owner: None,
            enabled: false,
            pend: false,
            active: false,
            hw: false,
            in_lr: false,
            prio: u8::MAX, // lowest PRIO
            route: GICD_IROUTER_INV,
            phys_route: GICD_IROUTER_INV,
            id,
        }
    }

    pub fn set_ownership(&mut self, vcpu: *mut VCpu) -> bool {
        match &self.owner {
            Some(owner) => *owner == vcpu,
            None => {
                self.owner = Some(vcpu);
                true
            }
        }
    }

    pub fn is_hw(&mut self) -> bool {
        self.id >= GIC_MAX_SGIS as _ && self.hw
    }
}

pub struct VGicD {
    pub interrupts: Vec<VGicIntr>,
    pub int_num: usize,
    pub ctlr: u32,
    pub typer: u32,
    pub lock: Mutex<()>,
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
                for i in 0..GIC_CPU_PRIV {
                    intrs.push(VGicIntr::new(i as _));
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
            read_field: None,
            update_field: None,
            update_hw: None,
        },
        GICD_REG_GROUP_IGROUPR => VGicHandlerInfo {
            reg_access: vgic_emul_razwi,
            field_width: 0,
            regroup_base: 0,
            read_field: None,
            update_field: None,
            update_hw: None,
        },
        GICD_REG_GROUP_ISENABLER => VGicHandlerInfo {
            reg_access: vgic_emul_generic_access,
            field_width: 1,
            regroup_base: GICD_REG_ISENABLER_OFF,
            read_field: Some(vgic_int_get_enable),
            update_field: Some(vgic_int_set_enable),
            update_hw: Some(vgic_int_enable_hw),
        },
        GICD_REG_GROUP_ICENABLER => VGicHandlerInfo {
            reg_access: vgic_emul_generic_access,
            field_width: 1,
            regroup_base: GICD_REG_ICENABLER_OFF,
            read_field: Some(vgic_int_get_enable),
            update_field: Some(vgic_int_clear_enable),
            update_hw: Some(vgic_int_enable_hw),
        },
        GICD_REG_GROUP_ICPENDR => VGicHandlerInfo {
            reg_access: vgic_emul_generic_access,
            field_width: 1,
            regroup_base: GICD_REG_ICPENDR_OFF,
            read_field: Some(vgic_int_get_pend),
            update_field: Some(vgic_int_clear_pend),
            update_hw: Some(vgic_int_state_hw),
        },
        GICD_REG_GROUP_ICACTIVER => VGicHandlerInfo {
            reg_access: vgic_emul_generic_access,
            field_width: 1,
            regroup_base: GICD_REG_ICACTIVER_OFF,
            read_field: Some(vgic_int_get_act),
            update_field: Some(vgic_int_clear_act),
            update_hw: Some(vgic_int_state_hw),
        },
        _ => {
            let gicd_reg = gicd_reg_mask(acc.addr);
            if gicd_reg >= GICD_REG_IPRIORITYR_OFF && gicd_reg < (GICD_REG_IPRIORITYR_OFF + 0x400) {
                VGicHandlerInfo {
                    reg_access: vgic_emul_generic_access,
                    regroup_base: GICD_REG_IPRIORITYR_OFF,
                    field_width: 8,
                    read_field: Some(vgic_int_get_prio),
                    update_field: Some(vgic_int_set_prio),
                    update_hw: Some(vgic_int_set_prio_hw),
                }
            } else if gicd_reg >= GICD_REG_ITARGETSR_OFF
                && gicd_reg < (GICD_REG_ITARGETSR_OFF + 0x400)
            {
                VGicHandlerInfo {
                    reg_access: vgic_emul_razwi,
                    field_width: 0,
                    regroup_base: 0,
                    read_field: None,
                    update_field: None,
                    update_hw: None,
                }
            } else if gicd_reg >= GICD_REG_IROUTER_OFF && gicd_reg < (GICD_REG_IROUTER_OFF + 0x2000)
            {
                VGicHandlerInfo {
                    reg_access: vgic_emul_generic_access,
                    field_width: 64,
                    regroup_base: GICD_REG_IROUTER_OFF,
                    read_field: Some(vgic_int_get_route),
                    update_field: Some(vgic_int_set_route),
                    update_hw: Some(vgic_int_set_route_hw),
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
    pub read_field: Option<fn(*mut VCpu, &mut VGicIntrInner) -> u64>,
    pub update_field: Option<fn(*mut VCpu, &mut VGicIntrInner, data: u64) -> bool>,
    pub update_hw: Option<fn(*mut VCpu, &mut VGicIntrInner)>,
}

fn vgic_get_int(int_id: IrqID, vgicr_id: VCpuID) -> Option<&'static mut VGicIntr> {
    if gic_is_priv(int_id) {
        Some(&mut myvm().get_vcpu_mut(vgicr_id).arch.vgic_priv.interrupts[int_id as usize])
    } else if int_id < myvm().arch.vgicd.int_num as _ {
        // assert!(myvm().arch.vgicd.lock.is_locked());
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
                let prev_ctrl = vgicd.ctlr;
                vgicd.ctlr = myvcpu().read_reg(acc.reg) as u32 & VGIC_ENABLE_MASK;
                debug!("write gicd.ctlr: {:#x?}", vgicd.ctlr);

                if prev_ctrl ^ vgicd.ctlr != 0 {
                    vgic_update_enable();
                    // let msg = CpuMsg {
                    //     id: vgic_initPI_ID,
                    //     data: [
                    //         VGIC_UPDATE_ENABLE,
                    //         VGIC_MSG_DATA(cpu().vcpu.vm.id, 0, 0, 0, 0),
                    //     ],
                    // };
                    // vm_msg_broadcast(cpu().vcpu.vm, &msg);
                }
            } else {
                myvcpu().write_reg(acc.reg as u64, (vgicd.ctlr | GICD_CTLR_ARE_NS_BIT) as u64);
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
    let mut val = if acc.write {
        myvcpu().read_reg(acc.reg)
    } else {
        0
    };
    let mask = if field_width == 64 {
        u64::MAX
    } else {
        (1u64 << field_width) - 1
    };
    let valid_access = if let GicVersion::GicVersion2 = GIC_VERSION {
        true
    } else {
        gicr_access == gic_is_priv(first_int as _)
    };

    if valid_access {
        for i in 0..(1.max((acc.width * 8) / field_width)) {
            let interrupt = vgic_get_int((first_int + i) as _, vgicr_id);
            if interrupt.is_none() {
                break;
            }
            if acc.write {
                let data = bit64_extract(val, i * field_width, field_width);
                interrupt.unwrap().set_field(handlers, data, myvcpu());
            } else {
                let read_field = handlers.read_field.unwrap();
                let mut intr_inner = interrupt.unwrap().inner.write();
                val |= (read_field(myvcpu(), &mut intr_inner) & mask) << (i * field_width);
            }
        }
    }

    if !acc.write {
        myvcpu().write_reg(acc.reg, val);
    }
}

fn vgic_update_enable() {
    if myvm().arch.vgicd.ctlr & VGIC_ENABLE_MASK != 0 {
        gich_set_hcr(gich_get_hcr() | 0x1);
        debug!("GicH HCR enabled.");
    } else {
        gich_set_hcr(gich_get_hcr() & !0x1);
        debug!("GicH HCR disabled.");
    }
}

pub fn gic_maintenance_handler() {
    todo!("gic_maintenance_handler");
}

pub fn vgic_set_hw(vm: &mut VM, id: IrqID) {
    if gic_is_sgi(id) {
        return;
    }
    if gic_is_priv(id) {
        for vcpuid in 0..vm.cpu_num {
            let interrupt = vgic_get_int(id, vcpuid as _).unwrap();
            interrupt.inner.write().hw = true;
        }
    } else {
        let _lock = myvm().arch.vgicd.lock.lock();
        let interrupt = vgic_get_int(id, myvcpu().id).unwrap();
        interrupt.inner.write().hw = true;
    }
}

// ----------------------------
pub fn vgic_int_get_enable(_vcpu: *mut VCpu, intr: &mut VGicIntrInner) -> u64 {
    debug!("get intr {} enable: {}.", intr.id, intr.enabled);
    intr.enabled as _
}

pub fn vgic_int_set_enable(_vcpu: *mut VCpu, intr: &mut VGicIntrInner, data: u64) -> bool {
    if data == 0 {
        return false;
    }
    intr.enabled = true;
    debug!("intr {} enabled.", intr.id);
    true
}

pub fn vgic_int_clear_enable(_vcpu: *mut VCpu, intr: &mut VGicIntrInner, data: u64) -> bool {
    if data == 0 {
        return false;
    }
    intr.enabled = false;
    debug!("intr {} disabled.", intr.id);
    true
}

pub fn vgic_int_enable_hw(_vcpu: *mut VCpu, intr: &mut VGicIntrInner) {
    if gic_is_priv(intr.id) {
        todo!("gicr set enable");
    } else {
        gicd_set_enable(intr.id, intr.enabled);
    }
    debug!(
        "intr {} (hardware) {}.",
        intr.id,
        if intr.enabled { "enabled" } else { "disabled" }
    );
}

pub fn vgic_int_get_pend(_vcpu: *mut VCpu, intr: &mut VGicIntrInner) -> u64 {
    debug!("get intr {} pend: {}.", intr.id, intr.pend);
    intr.pend as _
}

pub fn vgic_int_clear_pend(_vcpu: *mut VCpu, intr: &mut VGicIntrInner, data: u64) -> bool {
    if data == 0 {
        return false;
    }
    intr.pend = false;
    debug!("intr {} clear pend.", intr.id);
    true
}

pub fn vgic_int_get_act(_vcpu: *mut VCpu, intr: &mut VGicIntrInner) -> u64 {
    debug!("get intr {} act: {}.", intr.id, intr.active);
    intr.active as _
}

pub fn vgic_int_clear_act(_vcpu: *mut VCpu, intr: &mut VGicIntrInner, data: u64) -> bool {
    if data == 0 {
        return false;
    }
    intr.active = false;
    debug!("intr {} clear active.", intr.id);
    true
}

pub fn vgic_int_state_hw(_vcpu: *mut VCpu, intr: &mut VGicIntrInner) {
    if gic_is_priv(intr.id) {
        todo!("gicr set state");
    } else {
        gicd_set_act(intr.id, intr.active);
        gicd_set_pend(intr.id, intr.pend);
    }
    debug!(
        "intr {} (hardware) set state: active = {}, pend = {}.",
        intr.id, intr.active, intr.pend
    );
}

pub fn vgic_int_get_prio(_vcpu: *mut VCpu, intr: &mut VGicIntrInner) -> u64 {
    debug!("get intr {} prio: {:#x?}.", intr.id, intr.prio);
    intr.prio as _
}

pub fn vgic_int_set_prio(_vcpu: *mut VCpu, intr: &mut VGicIntrInner, data: u64) -> bool {
    intr.prio = data as _;
    debug!("intr {} set prio -> {:#x?}.", intr.id, intr.prio);
    true
}

pub fn vgic_int_set_prio_hw(_vcpu: *mut VCpu, intr: &mut VGicIntrInner) {
    if gic_is_priv(intr.id) {
        todo!("gicr set prio");
    } else {
        gicd_set_prio(intr.id, intr.prio);
    }
    debug!("intr {} (hardware) set prio", intr.id);
}

pub fn vgic_int_get_route(_vcpu: *mut VCpu, intr: &mut VGicIntrInner) -> u64 {
    if gic_is_priv(intr.id) {
        return 0;
    }
    debug!("get intr {} route: {:#x?}.", intr.id, intr.route);
    intr.route
}

pub fn vgic_int_set_route(_vcpu: *mut VCpu, intr: &mut VGicIntrInner, data: u64) -> bool {
    if gic_is_priv(intr.id) {
        return false;
    }
    intr.route = data;
    intr.phys_route = PLATFORM.cpu_id_to_mpidr(myvm().get_vcpu(data & 0xff).phys_id);
    debug!("intr {} set route -> {:#x?}.", intr.id, intr.route);
    true
}

pub fn vgic_int_set_route_hw(_vcpu: *mut VCpu, intr: &mut VGicIntrInner) {
    if gic_is_priv(intr.id) {
        panic!("gicr: cannot set route");
    } else {
        gicd_set_route(intr.id, intr.phys_route);
    }
    debug!("intr {} (hardware) set route", intr.id);
}

// --------------------------------------------------

pub fn vgic_write_lr(_vcpu: &'static mut VCpu, intr: &mut VGicIntrInner, lr_ind: u64) {
    let mut lr = intr.id as u64  // vINTid
        | ((intr.prio as u64) << 48)
        | GICH_LR_GRP_BIT;
    if intr.is_hw() {
        lr |= GICH_LR_HW_BIT;
        lr |= (intr.id as u64) << 32; // pINTid
        lr |= (intr.pend as u64) << 62; // LR_STATE
    }
    debug!("gich_write_lr({}) -> {:#x?}", lr_ind, lr);
    gich_write_lr(lr_ind as _, lr);
}

pub fn vgic_add_lr(vcpu: &'static mut VCpu, intr: &mut VGicIntrInner) -> bool {
    if !intr.enabled || intr.in_lr {
        return false;
    }

    let elrsr = read_reg!(ich_elrsr_el2);
    let lr_ind = (0..16u64).find(|i| bit64_extract(elrsr, *i, 1) != 0);

    if let Some(lr_ind) = lr_ind {
        vgic_write_lr(vcpu, intr, lr_ind);
        true
    } else {
        todo!("vgic_add_lr: no empty lr found");
    }
}

pub fn vgic_inject_hw(vcpu: &'static mut VCpu, id: IrqID) {
    let interrupt = vgic_get_int(id, vcpu.id).unwrap();

    let mut intr = interrupt.inner.write();
    intr.owner = Some(vcpu);
    intr.pend = true;
    intr.in_lr = false;
    if !vgic_add_lr(vcpu, &mut intr) {
        panic!("vgic: failed to inject hw")
    }
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
