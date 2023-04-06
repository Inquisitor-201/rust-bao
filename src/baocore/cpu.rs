use crate::arch::aarch64::defs::STACK_SIZE;

use super::types::CpuID;

#[repr(C)]
#[repr(align(0x1000))]
pub struct CpuStack {
    stack: [u8; STACK_SIZE]
}

#[repr(C)]
pub struct Cpu {
    id: CpuID,
    
    // handling_msgs: bool,
    
    // addr_space: AddrSpace,

    // vcpu: *mut Vcpu,

    // arch: CpuArch,

    // interface: *mut CpuIf,

    stack: CpuStack
}

pub const CPU_SIZE: usize = core::mem::size_of::<Cpu>();
