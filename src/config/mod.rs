use core::arch::global_asm;

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
            concat!(".pushsection .vm_image_", $img_name, " \"a\"",),
            concat!(".global _", $img_name, "_vm_beg"),
            concat!("_", $img_name, "_vm_beg:"),
            concat!(".incbin ", $img_path),
            concat!("_", $img_name, "_vm_end:"),
            concat!(".global _", $img_name, "_vm_size"),
            concat!(
                ".set _",
                $img_name,
                "_vm_size,  (_",
                $img_name,
                "_vm_end - _",
                $img_name,
                "_vm_beg"
            ),
            ".popsection"
        );
    };
}

pub struct VMConfig {
    base_addr: Vaddr,
    load_addr: Paddr,
    size: usize,
    separately_loaded: bool,
    inplace: bool,
    entry: Vaddr,
    vm_platform: VMPlatform,
}

pub struct Config {
    shared_mem: Option<()>,
    vmlist: Vec<VMConfig>,
}
