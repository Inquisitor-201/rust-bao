use crate::{
    arch::aarch64::{
        armv8_a::vm::VGicDscr,
        defs::PAGE_SIZE,
        gic::vgic::{vgic_emul_razwi, VGicHandlerInfo},
    },
    baocore::{
        emul::{EmulAccess, EmulMem},
        types::Vaddr,
        vm::VM,
    },
    println,
    util::align_up,
};

use super::{gicd::GicdHw, gicv3::GicrHw, vgic::vgicd_emul_handler};

pub const fn gicd_reg_mask(addr: Vaddr) -> u64 {
    addr & 0xffff
}

const fn gicr_reg_mask(addr: Vaddr) -> u64 {
    addr & 0x1ffff
}

pub fn vgic_init(vm: &mut VM, vgic_dscrp: &VGicDscr) {
    // vm.arch.vgicr_addr = vgic_dscrp.gicr_addr;
    let vgicd_emul = EmulMem {
        va_base: vgic_dscrp.gicd_addr,
        size: align_up(core::mem::size_of::<GicdHw>(), PAGE_SIZE),
        handler: vgicd_emul_handler,
    };
    vm.emul_add_mem(vgicd_emul);

    let vgicr_emul = EmulMem {
        va_base: vgic_dscrp.gicr_addr,
        size: align_up(core::mem::size_of::<GicrHw>() * vm.cpu_num, PAGE_SIZE),
        handler: vgicr_emul_handler,
    };
    vm.emul_add_mem(vgicr_emul);
}

fn vgicr_emul_handler(acc: &EmulAccess) -> bool {
    let gicr_reg = gicr_reg_mask(acc.addr);
    println!("gicr_reg = {:#x?}", gicr_reg);
    let handler_info = match gicr_reg {
        GICR_REG_WAKER_OFF | GICR_REG_IGROUPR0_OFF => VGicHandlerInfo {
            reg_access: vgic_emul_razwi,
        },
        _ => todo!("vgicd_emul_handler"),
    };
    (handler_info.reg_access)(acc);
    true
}

pub const VGIC_ENABLE_MASK: u32 = 0x2;

// ------------ GICR REGS ------------------

const GICR_REG_WAKER_OFF: u64 = 0x14;
const GICR_REG_IGROUPR0_OFF: u64 = 0x10080;
const GICR_REG_ICENABLER0_OFF: u64 = 0x10180;
const GICR_REG_ICPENDR0_OFF: u64 = 0x10280;
