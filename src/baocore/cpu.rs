use super::types::CpuID;

#[repr(C)]
#[repr(align(0x1000))]
pub struct Cpu {
    // id: CpuID,
    
    // handling_msgs: bool,
    
    // addr_space: AddrSpace,

    // vcpu: *mut Vcpu,

    // arch: CpuArch,

    // interface: *mut CpuIf,

    // #[repr(align(PAGE_SIZE))]
    // stack: [u8; STACK_SIZE],
}
