use super::{types::Paddr, mmu::mem::mem_prot_init};

pub fn root_pt() -> &'static mut _ {
    // unsafe { &mut *(BAO_CPU_BASE as *mut Cpu) }
}

pub fn init(load_addr: Paddr) {
    mem_prot_init();
}