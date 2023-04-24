use crate::baocore::types::Paddr;

pub struct VGicDscr {
    pub gicd_addr: Paddr,
    pub gicc_addr: Paddr,
    pub gicr_addr: Paddr,
    pub interrupt_num: usize,
}

pub struct ArchVMPlatform {
    pub gic: VGicDscr,
}




