use alloc::vec;
use spin::{Lazy, RwLock};

use crate::{
    arch::aarch64::armv8_a::vm::{ArchVMPlatform, VGicDscr},
    baocore::{
        ipc::{SharedMemConfig, IPC},
        vm::{VMDeviceRegion, VMMemRegion, VMPlatform},
    },
    config::{Config, VMConfig},
    def_vm_image, println,
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

    println!(
        "_linux_vm_begin = {:#x?}, _linux_vm_end = {:#x?}",
        _linux_vm_beg as u64, _linux_vm_end as u64
    );

    let vm_config_freertos = VMConfig {
        base_addr: 0x0,
        load_addr: _freertos_vm_beg as u64,
        size: (_freertos_vm_end as usize - _freertos_vm_beg as usize),
        separately_loaded: false,
        inplace: false,
        entry: 0x0,
        vm_platform: VMPlatform {
            cpu_num: 1,
            vm_regions: vec![VMMemRegion {
                base: 0x0,
                size: 0x8000000,
                place_phys: false,
                phys: 0,
            }],
            devs: vec![
                VMDeviceRegion {
                    /* Pl011 */
                    pa: 0x9000000,
                    va: Some(0xff000000),
                    size: 0x10000,
                    interrupts: vec![33],
                },
                VMDeviceRegion {
                    /* Arch timer interrupt */
                    pa: 0,
                    va: None,
                    size: 0,
                    interrupts: vec![27],
                },
                // VMDeviceRegion {
                //     /* virtio devices */
                //     pa: 0xa003000,
                //     va: 0xa003000,
                //     size: 0x1000,
                //     interrupts: vec![72, 73, 74, 75, 76, 77, 78, 79],
                // },
            ],
            arch: ArchVMPlatform {
                gic: VGicDscr {
                    gicd_addr: 0xf9010000,
                    gicc_addr: 0,
                    gicr_addr: 0xf9020000,
                    interrupt_num: 0,
                },
            },
            ipcs: vec![IPC {
                base: 0x70000000,
                size: 0x00010000,
                shmem_id: 0,
                interrupts: vec![52],
            }],
        },
    };

    let vm_config_linux = VMConfig {
        base_addr: 0x60000000,
        load_addr: _linux_vm_beg as u64,
        size: (_linux_vm_end as usize - _linux_vm_beg as usize),
        separately_loaded: false,
        inplace: false,
        entry: 0x60000000,
        vm_platform: VMPlatform {
            cpu_num: 1,
            vm_regions: vec![VMMemRegion {
                base: 0x60000000,
                size: 0x40000000,
                place_phys: true,
                phys: 0x60000000,
            }],
            devs: vec![
                VMDeviceRegion {
                    /* Arch timer interrupt */
                    pa: 0,
                    va: None,
                    size: 0,
                    interrupts: vec![27],
                },
                VMDeviceRegion {
                    /* virtio devices */
                    pa: 0xa003000,
                    va: Some(0xa003000),
                    size: 0x1000,
                    interrupts: vec![72, 73, 74, 75, 76, 77, 78, 79],
                },
            ],
            arch: ArchVMPlatform {
                gic: VGicDscr {
                    gicd_addr: 0x8000000,
                    gicc_addr: 0,
                    gicr_addr: 0x80a0000,
                    interrupt_num: 0,
                },
            },
            ipcs: vec![],
        },
    };

    RwLock::new(Config {
        shared_mem: vec![SharedMemConfig { size: 0x10000 }],
        vmlist: vec![vm_config_freertos, vm_config_linux],
    })
});
