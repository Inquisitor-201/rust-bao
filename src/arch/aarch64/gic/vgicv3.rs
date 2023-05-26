use spin::Mutex;

use crate::{
    arch::aarch64::{
        armv8_a::vm::VGicDscr,
        defs::PAGE_SIZE,
        gic::vgic::{
            vgic_emul_generic_access, vgic_emul_razwi, vgic_int_clear_act, vgic_int_clear_enable,
            vgic_int_clear_pend, vgic_int_enable_hw, vgic_int_get_act, vgic_int_get_enable,
            vgic_int_get_pend, vgic_int_get_prio, vgic_int_set_enable, vgic_int_set_prio,
            vgic_int_set_prio_hw, vgic_int_state_hw, VGicHandlerInfo, GICD_REG_ICACTIVER_OFF,
            GICD_REG_ICENABLER_OFF, GICD_REG_ICPENDR_OFF, GICD_REG_IPRIORITYR_OFF,
            GICD_REG_ISENABLER_OFF,
        },
    },
    baocore::{
        emul::{EmulAccess, EmulMem},
        types::{VCpuID, Vaddr},
        vm::{myvcpu, myvm, VM},
    },
    debug,
    util::{align_up, bit64_mask},
};

use super::{
    gic_defs::GIC_CPU_PRIV,
    gicd::GicdHw,
    gicd_get_iidr, gicr_get_pidr,
    gicv3::GicrHw,
    vgic::{vgicd_emul_handler, VGicIntr, GICD_REG_ICFGR_OFF, vgic_int_get_cfg, vgic_int_set_cfg, vgic_int_set_cfg_hw},
    GIC,
};

pub const fn gicd_reg_mask(addr: Vaddr) -> u64 {
    addr & 0xffff
}

const fn gicr_reg_mask(addr: Vaddr) -> u64 {
    addr & 0x1ffff
}

fn vgicr_get_id(acc: &EmulAccess) -> u64 {
    (acc.addr - myvm().arch.vgicr_addr) / align_up(core::mem::size_of::<GicrHw>(), PAGE_SIZE) as u64
}

pub fn vgic_init(vm: &mut VM, vgic_dscrp: &VGicDscr) {
    vm.arch.vgicr_addr = vgic_dscrp.gicr_addr;

    vm.arch.vgicd.int_num = unsafe { GIC.get().unwrap().max_irqs };
    vm.arch.vgicd.typer = ((vm.arch.vgicd.int_num as u32 / 32 - 1) & 0b11111) // ITLN
        | ((vm.cpu_num as u32 - 1) << 5)        // CPU_NUM
        | ((10 - 1) << 19); // TYPER_IDBITS
    vm.arch.vgicd.iidr = gicd_get_iidr();

    for i in 0..vm.arch.vgicd.int_num {
        let intr = VGicIntr::new((i + GIC_CPU_PRIV) as _, 0);
        // vm.arch.vgicd.interrupts[i].state = INV;
        // vm.arch.vgicd.interrupts[i].prio = GIC_LOWEST_PRIO;
        // vm.arch.vgicd.interrupts[i].cfg = 0;
        // vm.arch.vgicd.interrupts[i].route = GICD_IROUTER_INV;
        // vm.arch.vgicd.interrupts[i].phys.route = GICD_IROUTER_INV;
        // vm.arch.vgicd.interrupts[i].hw = false;
        // vm.arch.vgicd.interrupts[i].in_lr = false;
        // vm.arch.vgicd.interrupts[i].enabled = false;
        vm.arch.vgicd.interrupts.push(intr);
    }

    let vgicd_emul = EmulMem {
        va_base: vgic_dscrp.gicd_addr,
        size: align_up(core::mem::size_of::<GicdHw>(), PAGE_SIZE),
        handler: vgicd_emul_handler,
    };
    vm.emul_add_mem(vgicd_emul);

    for vcpu_id in 0..myvm().cpu_num {
        let vcpu = vm.get_vcpu_mut(vcpu_id as _);

        let mut typer = vcpu.id << 8;
        typer |= (vcpu.arch.vmpidr & 0xffff) << 32;
        typer |= ((vcpu.id as usize == myvm().cpu_num - 1) as u64) << 4;
        vcpu.arch.vgic_priv.vgicr.typer = typer;
    }

    let vgicr_emul = EmulMem {
        va_base: vgic_dscrp.gicr_addr,
        size: align_up(core::mem::size_of::<GicrHw>(), PAGE_SIZE) * vm.cpu_num,
        handler: vgicr_emul_handler,
    };
    vm.emul_add_mem(vgicr_emul);
}

fn vgicr_emul_handler(acc: &EmulAccess) -> bool {
    let gicr_reg = gicr_reg_mask(acc.addr);
    let handler_info = match gicr_reg {
        GICR_REG_CTRL_OFF | GICR_REG_WAKER_OFF | GICR_REG_IGROUPR0_OFF => VGicHandlerInfo {
            reg_access: vgic_emul_razwi,
            regroup_base: 0,
            field_width: 0,
            read_field: None,
            update_field: None,
            update_hw: None,
        },
        GICR_REG_TYPER_OFF => VGicHandlerInfo {
            reg_access: vgicr_emul_typer_access,
            regroup_base: 0,
            field_width: 0,
            read_field: None,
            update_field: None,
            update_hw: None,
        },
        GICR_REG_ISENABLER0_OFF => VGicHandlerInfo {
            reg_access: vgic_emul_generic_access,
            regroup_base: GICD_REG_ISENABLER_OFF,
            field_width: 1,
            read_field: Some(vgic_int_get_enable),
            update_field: Some(vgic_int_set_enable),
            update_hw: Some(vgic_int_enable_hw),
        },
        GICR_REG_ICENABLER0_OFF => VGicHandlerInfo {
            reg_access: vgic_emul_generic_access,
            regroup_base: GICD_REG_ICENABLER_OFF,
            field_width: 1,
            read_field: Some(vgic_int_get_enable),
            update_field: Some(vgic_int_clear_enable),
            update_hw: Some(vgic_int_enable_hw),
        },
        GICR_REG_ICPENDR0_OFF => VGicHandlerInfo {
            reg_access: vgic_emul_generic_access,
            regroup_base: GICD_REG_ICPENDR_OFF,
            field_width: 1,
            read_field: Some(vgic_int_get_pend),
            update_field: Some(vgic_int_clear_pend),
            update_hw: Some(vgic_int_state_hw),
        },
        GICR_REG_ICACTIVER0_OFF => VGicHandlerInfo {
            reg_access: vgic_emul_generic_access,
            regroup_base: GICD_REG_ICACTIVER_OFF,
            field_width: 1,
            read_field: Some(vgic_int_get_act),
            update_field: Some(vgic_int_clear_act),
            update_hw: Some(vgic_int_state_hw),
        },
        GICR_REG_ICFGR0_OFF | GICR_REG_ICFGR1_OFF => VGicHandlerInfo {
            reg_access: vgic_emul_generic_access,
            regroup_base: GICD_REG_ICFGR_OFF,
            field_width: 2,
            read_field: Some(vgic_int_get_cfg),
            update_field: Some(vgic_int_set_cfg),
            update_hw: Some(vgic_int_set_cfg_hw),
        },
        _ => {
            if gicr_reg >= GICR_REG_IPRIORITYR_OFF && gicr_reg < (GICR_REG_IPRIORITYR_OFF + 0x20) {
                VGicHandlerInfo {
                    reg_access: vgic_emul_generic_access,
                    regroup_base: GICD_REG_IPRIORITYR_OFF,
                    field_width: 8,
                    read_field: Some(vgic_int_get_prio),
                    update_field: Some(vgic_int_set_prio),
                    update_hw: Some(vgic_int_set_prio_hw),
                }
            } else if gicr_reg >= GICR_REG_ID_OFF && gicr_reg < 0xfffc {
                VGicHandlerInfo {
                    reg_access: vgicr_emul_pidr_access,
                    field_width: 0,
                    regroup_base: 0,
                    read_field: None,
                    update_field: None,
                    update_hw: None,
                }
            } else {
                todo!("vgicr_emul_handler");
            }
        }
    };

    // todo: check alignment?
    let vgcir_id = vgicr_get_id(acc);
    let vcpu = myvm().get_vcpu_mut(vgcir_id);

    let _gicr_mutex = vcpu.arch.vgic_priv.vgicr.lock.lock();
    (handler_info.reg_access)(acc, &handler_info, true, vgcir_id);
    true
}

fn vgicr_emul_pidr_access(
    acc: &EmulAccess,
    _handlers: &VGicHandlerInfo,
    _gicr_access: bool,
    _vgicr_id: VCpuID,
) {
    if !acc.write {
        debug!("read gicr.pidr: {:#x?}", gicr_get_pidr(acc.addr));
        myvcpu().write_reg(acc.reg, gicr_get_pidr(acc.addr) as _);
    }
}

fn vgicr_emul_typer_access(
    acc: &EmulAccess,
    _handlers: &VGicHandlerInfo,
    _gicr_access: bool,
    vgicr_id: VCpuID,
) {
    let word_access = acc.width == 4;
    let top_access = word_access && (acc.addr & 0x4 != 0);

    if !acc.write {
        let vcpu = myvm().get_vcpu_mut(vgicr_id);
        let typer0 = vcpu.arch.vgic_priv.vgicr.typer;
        let typer = if top_access {
            typer0 >> 32
        } else {
            typer0 & bit64_mask(0, 32)
        };
        myvcpu().write_reg(acc.reg, typer);
        debug!("read gicr({}).typer {:#x?}", vgicr_id, typer0);
    }
}
// ----------------------------------

pub struct VGicR {
    pub lock: Mutex<()>,
    pub typer: u64,
    pub ctlr: u32,
    pub iidr: u32,
}

pub const VGIC_ENABLE_MASK: u32 = 0x2;

// ------------ GICR REGS ------------------

const GICR_REG_CTRL_OFF: u64 = 0x0;
const GICR_REG_TYPER_OFF: u64 = 0x8;
const GICR_REG_WAKER_OFF: u64 = 0x14;
const GICR_REG_IGROUPR0_OFF: u64 = 0x10080;
const GICR_REG_ISENABLER0_OFF: u64 = 0x10100;
const GICR_REG_ICENABLER0_OFF: u64 = 0x10180;
const GICR_REG_ICPENDR0_OFF: u64 = 0x10280;
const GICR_REG_ICACTIVER0_OFF: u64 = 0x10380;
const GICR_REG_ICFGR0_OFF: u64 = 0x10c00;
const GICR_REG_ICFGR1_OFF: u64 = 0x10c04;
const GICR_REG_IPRIORITYR_OFF: u64 = 0x10400;
const GICR_REG_ID_OFF: u64 = 0x0ffd0;
