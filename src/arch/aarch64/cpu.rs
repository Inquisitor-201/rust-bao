use crate::baocore::{
    cpu::{Cpu, CpuArchTrait},
    types::{CpuID, Paddr},
};
use aarch64::regs::*;
use tock_registers::interfaces::Readable;

#[repr(C)]
pub struct CpuArch {
    mpdir: u64,
}

pub trait CpuArchProfileTrait {
    fn arch_profile_init(&self, cpu_id: CpuID, load_addr: Paddr);
}

impl CpuArchTrait for Cpu {
    fn arch_init(&mut self, load_addr: Paddr) {
        self.arch.mpdir = MPIDR_EL1.get();
        self.arch.arch_profile_init(self.id, load_addr);
    }
}
