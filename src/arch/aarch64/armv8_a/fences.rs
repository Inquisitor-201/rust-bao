use core::arch::asm;

pub fn fence_sync() {
    unsafe {
        asm!("dsb ish");
    }
}

pub fn fence_sync_write() {
    unsafe {
        asm!("dsb ishst");
    }
}

pub fn fence_sync_read() {
    unsafe {
        asm!("dsb ishst");
    }
}
