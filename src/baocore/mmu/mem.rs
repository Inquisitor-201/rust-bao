use crate::baocore::{
    cpu::cpu,
    mem::root_pt,
    pagetable::Pagetable,
    types::{AsType, Asid, ColorMap},
};

#[repr(C)]
pub struct AddrSpace {
    pub pt: Pagetable,
    pub as_type: AsType,
    pub colors: ColorMap,
    pub id: Asid,
}

pub trait AsArchTrait {
    fn arch_init(&mut self);
}

impl AddrSpace {
    pub fn init(&mut self, as_type: AsType, colors: ColorMap, id: Asid) {
        self.as_type = as_type;
        self.colors = colors;
        self.id = id
    }
}

pub fn mem_prot_init() {
    // let root_pt = (((cpu() as usize) + size_of::<Cpu>()) as u64).align_up(PAGE_SIZE) as *mut pte_t;
    let root_pt = root_pt();
    unsafe {
        as_init(
            cpu().addr_space,
            AS_HYP,
            HYP_ASID,
            root_pt,
            HypConfig::get().hyp.colors,
        );
    }
}
