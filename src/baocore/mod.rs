pub mod cache;
pub mod cpu;
pub mod types;
pub mod allocator;
pub mod mmu;
pub mod mem;
pub mod pagetable;

use types::{CpuID, Paddr};

#[no_mangle]
pub fn init(cpu_id: CpuID, load_addr: Paddr) {
    // allocator::heap_init(cpu_id);
    cpu::init(cpu_id, load_addr);
    mem::init(load_addr);
    loop {}
}