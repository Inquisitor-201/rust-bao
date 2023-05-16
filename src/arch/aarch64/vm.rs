use spin::RwLock;

use crate::{
    arch::aarch64::sysregs::*,
    baocore::{
        types::{Paddr, VCpuID, Vaddr},
        vm::{VCpu, VCpuArchTrait, VMArchTrait, VM},
    },
    config::VMConfig,
    write_reg,
};

use super::gic::{vgic::{VGicD, VGicPriv}, vgic_init};

impl VMArchTrait for VM {
    fn arch_init(&mut self, config: &VMConfig) {
        // TODO: Vgic init
        vgic_init(self, &config.vm_platform.arch.gic);
    }
}

pub struct VMArch {
    pub vgicr_addr: Vaddr,
    pub vgicd: VGicD
}

impl VMArch {
    pub fn new() -> Self {
        Self {
            vgicr_addr: 0,
            vgicd: VGicD::new()
        }
    }
}

#[repr(C)]
#[repr(align(16))]
pub struct ArchRegs {
    pub x: [u64; 31],
    pub elr_el2: u64,
    pub spsr_el2: u64,
}

#[repr(C)]
pub struct VCpuArch {
    pub vmpidr: u64,
    pub vgic_priv: VGicPriv,
    pub psci_ctx: RwLock<PsciCtx>,
}

pub enum PsciState {
    Off = 0,
    On,
    OnPending,
}

#[repr(C)]
pub struct PsciCtx {
    pub entrypoint: Paddr,
    pub state: PsciState,
}

pub trait VCpuArchProfileTrait {
    fn arch_profile_init(&mut self, vm: &VM);
}

impl VCpuArchTrait for VCpu {
    fn arch_init(&mut self, vm: &VM) {
        self.arch.vmpidr = cpuid_to_mpidr(vm, self.id);
        write_reg!(vmpidr_el2, self.arch.vmpidr);

        self.arch.psci_ctx.write().state = if self.id == 0 {
            PsciState::On
        } else {
            PsciState::Off
        };

        self.arch_profile_init(vm);
        // self.vgic_cpu_init();
    }
    fn arch_reset(&mut self, entry: Vaddr) {
        self.regs.spsr_el2 = SPSR_EL1h | SPSR_D | SPSR_A | SPSR_I | SPSR_F;
        self.write_pc(entry);
        write_reg!(cntvoff_el2, 0u64);
        write_reg!(sctlr_el1, SCTLR_RES1);
        write_reg!(pmcr_el0, 0u64);
    }
    fn arch_run(&mut self) {
        extern "C" {
            fn vcpu_arch_entry();
        }
        match self.arch.psci_ctx.read().state {
            PsciState::On => unsafe { vcpu_arch_entry() },
            _ => todo!("vcpu_arch_run: idle"),
        }
    }
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
