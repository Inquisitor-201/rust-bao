use crate::{
    baocore::vm::{VMArchTrait, VM},
    config::VMConfig,
};

impl VMArchTrait for VM {
    fn arch_init(&mut self, _config: &VMConfig) {
        // TODO: Vgic init
    }
}
