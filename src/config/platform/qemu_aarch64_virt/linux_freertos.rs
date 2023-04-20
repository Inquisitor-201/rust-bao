use alloc::vec;
use spin::{Lazy, Mutex};

use crate::{
    baocore::vm::{VMDeviceRegion, VMMemRegion, VMPlatform},
    config::{Config, VMConfig},
    def_vm_image, arch::aarch64::armv8_a::vm::{ArchVMPlatform, VGicDscr},
};

def_vm_image!("linux_image", "linux.bin");
def_vm_image!("freertos_image", "freertos.bin");

pub static CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| {
    extern "C" {
        fn _linux_vm_begin();
        fn _freertos_vm_begin();
        static _linux_vm_size: usize;
        static _freertos_vm_size: usize;
    }

    let vm_config_linux = VMConfig {
        base_addr: 0x60000000,
        load_addr: _linux_vm_begin as u64,
        size: unsafe { _linux_vm_size },
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

    Mutex::new(Config {
        shared_mem: None,
        vmlist: vec![vm_config_linux],
    })
});
