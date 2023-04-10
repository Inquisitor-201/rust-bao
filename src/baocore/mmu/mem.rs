use crate::{
    arch::aarch64::armv8_a::pagetable::{HYP_PT_DSCR, VM_PT_DSCR},
    baocore::{
        cpu::cpu,
        pagetable::{root_pt_addr, Pagetable},
        types::{AsType, Asid, ColorMap, Vaddr},
    },
};

pub const HYP_ASID: u64 = 0;

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
    pub fn init(&mut self, as_type: AsType, id: Asid, root_pt: Vaddr, colors: ColorMap) {
        self.as_type = as_type;
        self.colors = colors;
        self.id = id;

        if root_pt == 0 {
            unimplemented!();
        }

        self.pt = Pagetable {
            root: root_pt,
            desc: match as_type {
                AsType::AsVM => VM_PT_DSCR,
                _ => HYP_PT_DSCR,
            },
        };

        self.arch_init();
    }
}

pub fn mem_prot_init() {
    // let root_pt = (((cpu() as usize) + size_of::<Cpu>()) as u64).align_up(PAGE_SIZE) as *mut pte_t;
    let root_pt = root_pt_addr();
    cpu()
        .addr_space
        .lock()
        .init(AsType::AsHyp, HYP_ASID, root_pt, 0);
}
