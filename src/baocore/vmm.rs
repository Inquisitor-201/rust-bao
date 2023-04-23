use core::{mem::MaybeUninit, ptr::null_mut};

use alloc::vec::Vec;
use spin::{Lazy, RwLock, Mutex};

use crate::{
    arch::aarch64::{defs::PAGE_SIZE, vmm::vmm_arch_init},
    config::{platform::qemu_aarch64_virt::linux_freertos::CONFIG, VMConfig},
    util::{align, num_pages},
};

use super::{
    cpu::{mycpu, CPU_SYNC_TOKEN, SyncToken},
    mem::mem_alloc_page,
    mmu::{
        sections::SEC_HYP_VM,
        vmm::{vmm_get_vm_install_info, vmm_vm_install}
    },
    types::CpuMap,
    vm::{vm_init, VCpu, VMAllocation, VMInstallInfo, VM},
};

struct VMAssign {
    master: bool,
    ncpus: usize,
    cpus: CpuMap,
    vm_alloc: Option<VMAllocation>,
    vm_install_info: Option<VMInstallInfo>,
}

static VM_ASSIGN: Lazy<Vec<RwLock<VMAssign>>> = Lazy::new(|| {
    let mut vm_assign = Vec::new();
    for _ in 0..CONFIG.read().vmlist.len() {
        vm_assign.push(RwLock::new(VMAssign {
            master: false,
            ncpus: 0,
            cpus: 0,
            vm_alloc: None,
            vm_install_info: None,
        }));
    }
    vm_assign
});

fn vmm_assign_vcpu() -> (bool, Option<usize>) {
    let config = CONFIG.read();
    let mut master = false;
    let mut vm_id = None;
    for i in 0..config.vmlist.len() {
        let mut vm_assign = VM_ASSIGN[i].write();
        if vm_assign.ncpus < config.vmlist[i].vm_platform.cpu_num {
            if !vm_assign.master {
                vm_assign.master = true;
                master = true;
            }
            vm_assign.ncpus += 1;
            vm_assign.cpus |= 1 << mycpu().id;
            vm_id = Some(i);
            return (master, vm_id);
        }
    }
    (master, vm_id)
}

#[allow(invalid_value)]
fn vmm_alloc_vm(config: &VMConfig) -> VMAllocation {
    let vcpus_offset = align(core::mem::size_of::<VM>(), core::mem::align_of::<VCpu>());
    let vcpu_size = config.vm_platform.cpu_num * core::mem::size_of::<VCpu>();
    let total_size = align(vcpus_offset + vcpu_size, PAGE_SIZE);

    let allocation = mem_alloc_page(num_pages(total_size), SEC_HYP_VM, false).unwrap();
    unsafe { *(allocation as *mut VM) = VM {
        vcpus: null_mut(),
        master: 0,
        id: 0,
        cpu_num: 0,
        cpus: 0,
        addr_space: MaybeUninit::uninit().assume_init(),
        sync_token: SyncToken::new(),
        lock: Mutex::new(()),
    }};
    VMAllocation {
        base: allocation,
        size: total_size,
        vcpus_offset,
    }
}

fn vmm_alloc_install_vm(vm_id: usize, master: bool) -> VMAllocation {
    let config = CONFIG.read();
    let vm_assign = &VM_ASSIGN[vm_id as usize];
    let vm_config = &config.vmlist[vm_id as usize];

    if master {
        let allocation = vmm_alloc_vm(vm_config);
        let install_info = vmm_get_vm_install_info(&allocation);
        let mut _lock = vm_assign.write();
        _lock.vm_alloc = Some(allocation);
        _lock.vm_install_info = Some(install_info);
        allocation
    } else {
        while vm_assign.read().vm_install_info.is_none() {}
        vmm_vm_install(vm_assign.read().vm_install_info.as_ref().unwrap());
        vm_assign.read().vm_alloc.unwrap()
    }
}

pub fn init() {
    vmm_arch_init();
    CPU_SYNC_TOKEN.sync_barrier();

    let (master, vm_id) = vmm_assign_vcpu();
    match vm_id {
        Some(vm_id) => {
            let vm_alloc = vmm_alloc_install_vm(vm_id, master);
            let cfg = CONFIG.read();
            vm_init(&vm_alloc, &cfg.vmlist[vm_id], master, vm_id);
        }
        _ => todo!("cpu_idle"),
    }
}
