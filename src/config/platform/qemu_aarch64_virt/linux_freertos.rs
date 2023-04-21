use alloc::vec;
use spin::{Lazy, RwLock};

use crate::{
    baocore::vm::{VMDeviceRegion, VMMemRegion, VMPlatform},
    config::{Config, VMConfig},
    def_vm_image, arch::aarch64::armv8_a::vm::{ArchVMPlatform, VGicDscr}, println,
};

def_vm_image!("linux", "imgs/qemu-aarch64-virt/linux.bin");
def_vm_image!("freertos", "imgs/qemu-aarch64-virt/freertos.bin");

pub static CONFIG: Lazy<RwLock<Config>> = Lazy::new(|| {
    extern "C" {
        fn _linux_vm_beg();
        fn _freertos_vm_beg();
        fn _linux_vm_end();
        fn _freertos_vm_end();
    }

    println!("_linux_vm_begin = {:#x?}, _linux_vm_end = {:#x?}", _linux_vm_beg as u64, _linux_vm_end as u64);

    let vm_config_linux = VMConfig {
        base_addr: 0x60000000,
        load_addr: _linux_vm_beg as u64,
        size: (_linux_vm_end as usize - _linux_vm_beg as usize),
        separately_loaded: false,
        inplace: false,
        entry: 0x60000000,
        vm_platform: VMPlatform {
            cpu_num: 3,
            vm_regions: vec![VMMemRegion {
                base: 0x60000000,
                size: 0x40000000,
            }],
            devs: vec![
                VMDeviceRegion {
                    /* Arch timer interrupt */
                    pa: 0,
                    va: 0,
                    size: 0,
                    interrupts: vec![27],
                },
                VMDeviceRegion {
                    /* virtio devices */
                    pa: 0xa003000,
                    va: 0xa003000,
                    size: 0x1000,
                    interrupts: vec![72, 73, 74, 75, 76, 77, 78, 79],
                },
            ],
            arch: ArchVMPlatform {
                gic: VGicDscr {
                    gicd_addr: 0xf9010000,
                    gicc_addr: 0xf9020000,
                    gicr_addr: 0,
                    interrupt_num: 0,
                },
            }
        },
    };

    RwLock::new(Config {
        shared_mem: None,
        vmlist: vec![vm_config_linux],
    })
});
