use crate::{
    arch::aarch64::sysregs::*,
    baocore::{
        types::{Paddr, VCpuID, Vaddr},
        vm::{VCpu, VCpuArchTrait, VM},
    },
    write_reg,
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

fn cpuid_to_mpidr(vm: &VM, cpuid: VCpuID) -> u64 {
    if cpuid > vm.cpu_num as _ {
        return !(!MPIDR_RES1 & MPIDR_RES0_MSK); //invert res bits to return an invalid mpidr value
    }

    let mut mpidr = cpuid as u64 | MPIDR_RES1;

    if vm.cpu_num == 1 {
        mpidr |= MPIDR_U_BIT;
    }

    mpidr
}

#[repr(align(16))]
pub struct ArchRegs {
    pub x: [u64; 31],
    pub elr_el2: u64,
    pub spsr_el2: u64,
}

pub struct VCpuArch {
    pub vmpidr: u64,
    // psci_ctx:
}

impl VCpuArchTrait for VCpu {
    fn arch_init(&mut self, vm: &VM) {
        self.arch.vmpidr = cpuid_to_mpidr(vm, self.id);
        write_reg!(vmpidr_el2, self.arch.vmpidr);

        // self.arch.psci_ctx.state = if self.id == 0 { PsciState::On } else { PsciState::Off };

        // self.vcpu_arch_profile_init(vm);
        // self.vgic_cpu_init();
    }
    fn arch_reset(&mut self, entry: Vaddr) {
        self.regs.spsr_el2 = SPSR_EL1h | SPSR_D | SPSR_A | SPSR_I | SPSR_F;
        self.regs.elr_el2 = entry;
        write_reg!(cntvoff_el2, 0u64);
        write_reg!(sctlr_el1, SCTLR_RES1);
        write_reg!(pmcr_el0, 0u64);
    }
}
