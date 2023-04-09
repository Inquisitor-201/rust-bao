pub mod cache;
pub mod cpu;
pub mod types;

use types::{CpuID, Paddr};

#[no_mangle]
pub fn init(cpu_id: CpuID, load_addr: Paddr) {
    cpu::init(cpu_id, load_addr);
}