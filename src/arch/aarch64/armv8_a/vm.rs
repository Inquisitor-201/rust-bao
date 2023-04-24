use aarch64::regs::VTTBR_EL2;
use tock_registers::interfaces::Writeable;

use crate::{
    arch::aarch64::{
        armv8_a::fences::isb,
        sysregs::{VTTBR_VMID_MSK, VTTBR_VMID_OFF},
        vm::VCpuArchProfileTrait,
    },
    baocore::{
        cpu::mycpu,
        types::Paddr,
        vm::{VCpu, VM},
    },
};

pub struct VGicDscr {
    pub gicd_addr: Paddr,
    pub gicc_addr: Paddr,
    pub gicr_addr: Paddr,
    pub interrupt_num: usize,
}

pub struct ArchVMPlatform {
    pub gic: VGicDscr,
}

impl VCpuArchProfileTrait for VCpu {
    fn arch_profile_init(&mut self, vm: &VM) {
        let s2pt_root = mycpu()
            .addr_space
            .mem_translate(vm.addr_space.pt.root)
            .unwrap();

        let vttbr =
            (((vm.id as u64) << VTTBR_VMID_OFF) & VTTBR_VMID_MSK) | (s2pt_root & !VTTBR_VMID_MSK);
        VTTBR_EL2.set(vttbr);
        isb();
    }
}
