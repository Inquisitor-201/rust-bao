pub mod allocator;
pub mod cache;
pub mod cpu;
pub mod mem;
pub mod mmu;
pub mod pagetable;
pub mod types;

use types::{CpuID, Paddr};

#[no_mangle]
pub fn init(cpu_id: CpuID, load_addr: Paddr) {
    // allocator::heap_init(cpu_id);
    cpu::init(cpu_id, load_addr);
    mem::init(load_addr);
    loop {}
}
