pub type Paddr = u64;
pub type Vaddr = u64;

pub type CpuID = u64;
pub type VCpuID = u64;
pub type CpuMap = u64;
pub type ColorMap = u64;
pub type Asid = u64;

#[repr(C)]
#[derive(Clone, Copy)]
pub enum AsType {
    AsHyp = 0,
    AsVM,
    AsHypCry,
}
