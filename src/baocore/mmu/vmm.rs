use crate::baocore::{
    cpu::mycpu,
    vm::{VMAllocation, VMInstallInfo},
};

pub fn vmm_get_vm_install_info(vm_alloc: &VMAllocation) -> VMInstallInfo {
    VMInstallInfo {
        base: vm_alloc.base,
        vm_section_pte: unsafe { *mycpu().addr_space.pt.pt_get_pte(0, vm_alloc.base) },
    }
}

pub fn vmm_vm_install(install_info: &VMInstallInfo) {
    let pte = mycpu().addr_space.pt.pt_get_pte(0, install_info.base);
    unsafe {
        *pte = install_info.vm_section_pte;
    }
}
