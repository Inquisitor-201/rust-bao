pub type Paddr = u64;
pub type Vaddr = u64;

pub type CpuID = u64;
pub type VCpuID = u64;
pub type CpuMap = u64;
pub type ColorMap = u64;
pub type Asid = u64;
pub type MemFlags = u64;
pub type AsSecID = u64;

pub const MAX_VA: Vaddr = Vaddr::MAX - 1;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AsType {
    AsHyp = 0,
    AsVM,
    AsHypCry,
}


