use alloc::vec::Vec;

use crate::baocore::{
    types::{Paddr, Vaddr},
    vm::VMPlatform,
};

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
    pub shared_mem: Option<()>,
    pub vmlist: Vec<VMConfig>,
}
