use super::{mmu::mem::mem_prot_init, types::Paddr};

pub fn init(load_addr: Paddr) {
    mem_prot_init();
}
