use alloc::vec::Vec;
use spin::{Mutex, RwLock};

use crate::{
    arch::aarch64::{
        armv8_a::{
            pagetable::{PTE, PTE_HYP_FLAGS, PTE_VM_FLAGS},
            vm::ArchVMPlatform,
        },
        vm::{ArchRegs, PsciCtx, PsciState, VCpuArch},
    },
    config::VMConfig,
    println,
    util::{num_pages, range_in_range},
};

use super::{
    cpu::{mycpu, SyncToken},
    mem::PPages,
    mmu::{
        mem::AddrSpace,
        sections::{SEC_HYP_GLOBAL, SEC_HYP_PRIVATE, SEC_VM_ANY},
    },
    types::{AsType, CpuID, CpuMap, IrqID, Paddr, VCpuID, Vaddr},
};

pub struct VMMemRegion {
    pub base: Paddr,
    pub size: usize,
    pub place_phys: bool,
    pub phys: Paddr,
}

pub struct VMDeviceRegion {
    pub va: Option<Vaddr>,
    pub pa: Paddr,
    pub size: usize,
    pub interrupts: Vec<IrqID>,
}

pub struct VMPlatform {
    pub cpu_num: usize,
    pub vm_regions: Vec<VMMemRegion>,
    pub devs: Vec<VMDeviceRegion>,
    pub arch: ArchVMPlatform,
}

#[derive(Clone, Copy)]
pub struct VMAllocation {
    pub base: Vaddr,
    pub size: usize,
    pub vcpus_offset: usize,
}

pub struct VMInstallInfo {
    pub base: Vaddr,
    pub vm_section_pte: PTE,
}

#[repr(C)]
pub struct VCpu {
    pub regs: ArchRegs, // regs: should be put ahead
    pub arch: VCpuArch,
    pub id: VCpuID,
    pub phys_id: CpuID,
    pub active: bool,
    pub vm: *const VM,
}

pub trait VCpuArchTrait {
    fn arch_init(&mut self, vm: &VM);
    fn arch_reset(&mut self, entry: Vaddr);
    fn arch_run(&mut self);
}

impl VCpu {
    pub fn run(&mut self) {
        println!("[vcpu {}] run", self.id);
        self.active = true;
        self.arch_run();
    }
}

#[repr(C)]
pub struct VM {
    pub vcpus: *mut VCpu,
    pub master: CpuID,
    pub id: usize,
    pub cpu_num: usize,
    pub cpus: CpuMap,
    pub addr_space: AddrSpace,
    pub sync_token: SyncToken,
    pub lock: Mutex<()>,
}

pub trait VMArchTrait {
    fn arch_init(&mut self, config: &VMConfig);
}

impl VM {
    fn master_init(&mut self, config: &VMConfig, vm_id: usize) {
        self.master = mycpu().id;
        self.cpu_num = config.vm_platform.cpu_num;
        self.sync_token.sync_init(self.cpu_num);
        self.id = vm_id;
        self.addr_space.init(AsType::AsVM, self.id as _, None, 0)
    }

    fn cpu_init(&mut self) {
        let _lock = self.lock.lock();
        self.cpus |= 1 << mycpu().id;
    }

    fn calc_vcpu_id(&self) -> VCpuID {
        let mut vcpu_id = 0;
        for i in 0..mycpu().id {
            if self.cpus & (1 << i) != 0 {
                vcpu_id += 1;
            }
        }
        vcpu_id
    }

    #[allow(unused)]
    fn get_vcpu(&self, vcpuid: VCpuID) -> &VCpu {
        assert!(vcpuid < self.cpu_num as _);
        unsafe { &*(self.vcpus.add(vcpuid as _)) }
    }

    fn get_vcpu_mut(&self, vcpuid: VCpuID) -> &mut VCpu {
        assert!(vcpuid < self.cpu_num as _);
        unsafe { &mut *(self.vcpus.add(vcpuid as _)) }
    }

    fn vcpu_init(&mut self, config: &VMConfig) {
        let vcpu_id = self.calc_vcpu_id();
        println!("[cpu {}] vcpu_id = {}", mycpu().id, vcpu_id);
        let vcpu = self.get_vcpu_mut(vcpu_id);

        *vcpu = VCpu {
            id: vcpu_id,
            phys_id: mycpu().id,
            vm: self as _,
            arch: VCpuArch {
                vmpidr: 0,
                psci_ctx: RwLock::new(PsciCtx {
                    entrypoint: 0,
                    state: PsciState::Off,
                }),
            },
            regs: ArchRegs {
                x: [0; 31],
                elr_el2: 0,
                spsr_el2: 0,
            },
            active: false,
        };
        mycpu().vcpu = vcpu;

        vcpu.arch_init(self);
        vcpu.arch_reset(config.entry);
    }

    fn init_mem_regions(&mut self, config: &VMConfig) {
        for reg in &config.vm_platform.vm_regions {
            let img_is_in_rgn =
                range_in_range(config.base_addr as _, config.size, reg.base as _, reg.size);
            if img_is_in_rgn {
                self.map_img_rgn(config, reg);
            } else {
                self.map_mem_region(reg);
            }
        }
    }

    fn map_img_rgn(&mut self, config: &VMConfig, reg: &VMMemRegion) {
        if reg.place_phys {
            self.copy_img_to_rgn(config, reg);
            self.map_mem_region(reg);
        } else if config.inplace {
            todo!("config.inplace");
            // self.map_img_rgn_inplace(config, reg);
        } else {
            self.map_mem_region(reg);
            self.install_image(config);
        }
    }

    fn copy_img_to_rgn(&mut self, config: &VMConfig, reg: &VMMemRegion) {
        // Map original image address
        let n_img = num_pages(config.size);
        let src_pa_img = PPages::new(config.load_addr, n_img);
        let src_va = mycpu()
            .addr_space
            .mem_alloc_map(
                SEC_HYP_PRIVATE,
                Some(&src_pa_img),
                None,
                n_img,
                PTE_HYP_FLAGS,
            )
            .unwrap();
        // Map new address
        let offset = config.base_addr - reg.base;
        let dst_phys = reg.phys + offset;
        let dst_pp = PPages::new(dst_phys, n_img);
        let dst_va = mycpu()
            .addr_space
            .mem_alloc_map(SEC_HYP_PRIVATE, Some(&dst_pp), None, n_img, PTE_HYP_FLAGS)
            .unwrap();
        unsafe {
            core::ptr::copy_nonoverlapping(src_va as *const u8, dst_va as *mut u8, config.size);
        }
        // todo: cache_flush_range(dst_va, n_img * PAGE_SIZE);
        // mem_unmap(&cpu().as_, src_va, n_img, false);
        // mem_unmap(&cpu().as_, dst_va, n_img, false);
    }

    fn map_mem_region(&mut self, reg: &VMMemRegion) {
        let n = num_pages(reg.size);

        let ppages = if reg.place_phys {
            // pa_reg.colors = reg.colors;
            Some(PPages::new(reg.phys, n))
        } else {
            None
        };

        let va = self
            .addr_space
            .mem_alloc_map(SEC_VM_ANY, ppages.as_ref(), Some(reg.base), n, PTE_VM_FLAGS)
            .unwrap();

        assert_eq!(va, reg.base);
    }

    fn install_image(&self, config: &VMConfig) {
        let img_num_pages = num_pages(config.size);
        let img_ppages = PPages::new(config.load_addr, img_num_pages);
        let src_va = mycpu()
            .addr_space
            .mem_alloc_map(
                SEC_HYP_GLOBAL,
                Some(&img_ppages),
                None,
                img_num_pages,
                PTE_HYP_FLAGS,
            )
            .unwrap();
        let dst_va =
            mycpu()
                .addr_space
                .mem_map_cpy(&self.addr_space, config.base_addr, None, img_num_pages);
        unsafe {
            core::ptr::copy_nonoverlapping(src_va as *const u8, dst_va as *mut u8, config.size);
        }
    }

    fn init_dev(&mut self, config: &VMConfig) {
        for dev in config.vm_platform.devs.iter() {
            if dev.va.is_some() {
                let va = self
                    .addr_space
                    .mem_alloc_map_dev(SEC_VM_ANY, dev.pa, dev.va, num_pages(dev.size))
                    .unwrap();
                assert_eq!(va, dev.va.unwrap());
            }
        }
    }
}

#[allow(invalid_value)]
fn vm_allocation_init(vm_alloc: &VMAllocation) -> &'static mut VM {
    let vm = unsafe { &mut *(vm_alloc.base as *mut VM) } as &'static mut VM;
    vm.vcpus = (vm_alloc.base + vm_alloc.vcpus_offset as u64) as *mut _;
    vm
}

pub fn vm_init(vm_alloc: &VMAllocation, config: &VMConfig, master: bool, vm_id: usize) {
    let vm = vm_allocation_init(vm_alloc);
    if master {
        vm.master_init(config, vm_id);
    }
    vm.cpu_init();
    vm.sync_token.sync_barrier();

    vm.vcpu_init(config);
    vm.sync_token.sync_barrier();

    vm.arch_init(config);

    if master {
        vm.init_mem_regions(config);
        vm.init_dev(config);
        println!("master finally done");
    }

    vm.sync_token.sync_and_clear_msg();
}
