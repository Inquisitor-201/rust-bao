use core::arch::asm;

use super::gic::{self, gic_defs::GIC_MAX_INTERUPTS};

pub const MAX_INTERUPTS: usize = GIC_MAX_INTERUPTS;

pub fn enable_irqs() {
    unsafe { asm!("msr daifclr, #0xf") };
}

pub fn disable_irqs() {
    unsafe { asm!("msr daifset, #0xf") };
}

pub fn interrupts_arch_init() {
    gic::init();
}
