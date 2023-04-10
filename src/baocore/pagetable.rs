use super::types::Paddr;

#[repr(C)]
pub struct Pagetable {
    root: Paddr
}