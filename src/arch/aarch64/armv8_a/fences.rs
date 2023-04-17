use core::arch::asm;

pub fn fence_sync() {
    unsafe { asm!("dsb ish"); }
}