pub type Paddr = u64;
pub type Vaddr = u64;

pub type CpuID = u64;
pub type VCpuID = u64;
pub type CpuMap = u64;
pub type ColorMap = u64;
pub type Asid = u64;
pub type MemFlags = u64;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AsType {
    AsHyp = 0,
    AsVM,
    AsHypCry,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AsSec {
    /*--- VM AS SECTIONS -----*/
    HypGlobal = 1,
    HypImage,
    HypPrivate,
    HypVm,
    HypAny, /* must be last */
    /*--- VM AS SECTIONS -----*/
    VmAny = 0, /* must be last */
    /*---- INVALID AS_SECTION ----*/
    Unknown = -1,
}
