pub mod cache;
pub mod cpu;
pub mod heap;
pub mod intr;
pub mod mem;
pub mod mmu;
pub mod pagetable;
pub mod types;
pub mod vm;
pub mod vmm;
pub mod emul;
pub mod ipc;

#[macro_use]
pub mod console;

use types::{CpuID, Paddr};

use crate::baocore::cpu::mycpu;

#[no_mangle]
pub fn init(cpu_id: CpuID, load_addr: Paddr) -> ! {
    // allocator::heap_init(cpu_id);
    cpu::init(cpu_id, load_addr);
    mem::init(load_addr);
    console::init();
    println!("[Cpu {}] Welcome to rust-bao!", mycpu().id);
    intr::init();
    vmm::init();
    panic!("Should never reach here");
}
