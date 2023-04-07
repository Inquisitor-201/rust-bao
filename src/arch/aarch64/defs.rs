pub const BAO_VAS_BASE: usize = 0xfd8000000000;
pub const BAO_CPU_BASE: usize = 0xfe0000000000;
pub const BAO_VM_BASE: usize = 0xfe8000000000;
pub const BAO_VAS_TOP: usize = 0xff0000000000;

pub const PAGE_SIZE: usize = 0x1000;
pub const CPU_STACK_SIZE: usize = 2 * PAGE_SIZE;
