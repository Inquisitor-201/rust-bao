use crate::baocore::{
    mmu::mem::{AddrSpace, AsArchTrait},
    types::AsType::*,
};

impl AsArchTrait for AddrSpace {
    fn arch_init(&mut self) {
        let index = match self.as_type {
            AsHyp | AsVM => todo!(),
            AsHypCry => todo!(),
        };
    }
}
