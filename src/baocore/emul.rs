use super::types::Vaddr;

pub struct EmulMem {
    pub va_base: Vaddr,
    pub size: usize,
    pub handler: EmulHandler,
}

pub struct EmulReg {}

#[derive(Debug)]
pub struct EmulAccess {
    pub addr: Vaddr,
    pub width: u64,
    pub write: bool,
    pub reg: u64,
}

pub type EmulHandler = fn(&EmulAccess) -> bool;
