use crate::{baocore::{
    mmu::mem::{AddrSpace, AsArchTrait},
    types::AsType::*,
}, pt_vm_rec_index, pt_cpu_rec_index};

impl AsArchTrait for AddrSpace {
    fn arch_init(&mut self) {
        let _index = match self.as_type {
            AsHypCry | AsVM => pt_vm_rec_index!(),
            AsHyp => pt_cpu_rec_index!(),
        };
        // todo: recursive mapping
    }
}
