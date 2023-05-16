use alloc::vec::Vec;

use crate::{
    arch::aarch64::defs::BAO_VAS_BASE,
    baocore::{
        types::{Paddr, Vaddr},
        vm::VMPlatform, ipc::SharedMemConfig,
    },
};

use self::platform::qemu_aarch64_virt::linux_freertos::CONFIG;

pub mod platform;

#[macro_export]
macro_rules! def_vm_image {
    ($img_name:literal, $img_path:literal) => {
        core::arch::global_asm!(
            concat!(".pushsection .vm_image_", $img_name, ", \"a\"",),
            concat!(".global _", $img_name, "_vm_beg"),
            concat!(".global _", $img_name, "_vm_end"),
            concat!("_", $img_name, "_vm_beg:"),
            concat!(".incbin \"", $img_path, "\""),
            concat!("_", $img_name, "_vm_end:"),
            ".popsection"
        );
    };
}

pub struct VMConfig {
    pub base_addr: Vaddr,
    pub load_addr: Paddr,
    pub size: usize,
    pub separately_loaded: bool,
    pub inplace: bool,
    pub entry: Vaddr,
    pub vm_platform: VMPlatform,
}

pub struct Config {
    pub shared_mem: Vec<SharedMemConfig>,
    pub vmlist: Vec<VMConfig>,
}

fn adjust_vm_image_addr(load_addr: Paddr) {
    let mut config = CONFIG.write();
    for vm_config in config.vmlist.iter_mut() {
        if !vm_config.separately_loaded {
            vm_config.load_addr = vm_config.load_addr - BAO_VAS_BASE as u64 + load_addr;
        }
    }
}

pub fn init(load_addr: Paddr) {
    adjust_vm_image_addr(load_addr);
}
