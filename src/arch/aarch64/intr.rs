use core::arch::asm;

use crate::baocore::{types::IrqID, intr::{IntrHandleResult, interrupts_is_reserved}, vm::myvcpu};

use super::{gic::{self, gic_defs::GIC_MAX_INTERUPTS}, armv8_a::vm::vcpu_arch_inject_hw_irq};

pub const MAX_INTERUPTS: usize = GIC_MAX_INTERUPTS;

pub fn enable_irqs() {
    unsafe { asm!("msr daifclr, #0xf") };
}

pub fn disable_irqs() {
    unsafe { asm!("msr daifset, #0xf") };
}

pub fn interrupts_handle(int_id: IrqID) -> IntrHandleResult {
    if interrupts_is_reserved(int_id) {
        todo!("handle reserved intr {}", int_id);
    }
    vcpu_arch_inject_hw_irq(myvcpu(), int_id);
    IntrHandleResult::ForwardToVM
}

pub fn interrupts_arch_init() {
    gic::init();
}
