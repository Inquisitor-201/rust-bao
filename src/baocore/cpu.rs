use spin::Mutex;

use crate::arch::aarch64::{defs::{CPU_STACK_SIZE, BAO_CPU_BASE}, cpu::CpuArch};

use super::{types::{CpuID, Paddr}, mmu::mem::AddrSpace};

#[repr(C)]
#[repr(align(0x1000))]
pub struct CpuStack {
    stack: [u8; CPU_STACK_SIZE],
}

#[repr(C)]
pub struct Cpu {
    pub id: CpuID,
    pub handling_msgs: bool,
    pub addr_space: Mutex<AddrSpace>,
    // vcpu: *mut Vcpu,
    pub arch: CpuArch,
    // interface: *mut CpuIf,
    stack: CpuStack,
}

pub trait CpuArchTrait {
    fn arch_init(&mut self, load_addr: Paddr);
}

pub const CPU_SIZE: usize = core::mem::size_of::<Cpu>();

pub fn cpu() -> &'static mut Cpu {
    unsafe { &mut *(BAO_CPU_BASE as *mut Cpu) }
}

pub fn init(cpu_id: CpuID, load_addr: Paddr) {
    let mycpu = cpu();
    mycpu.id = cpu_id;
    mycpu.handling_msgs = false;
    mycpu.arch_init(load_addr);
}