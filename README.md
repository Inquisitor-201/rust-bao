# rust-bao

todo:
1. automatically detect qemu & atf-fip
2. cpu_init sync & handler
3. as_init: is null ?
5. `colors` unimplemented
7. todo: cache_enumerate
8. todo: Config init
9. todo: mem_create_ppools (additional pools)
10. todo: activate maintanence intr & IPI_CPU_MSG
11. todo: smmu init
12. todo: ipc init
13. todo: vcpu->arch.psci_ctx
15. todo: tlb disable?
16. todo: copy vm-image: cache_flush_range
17. todo: mem_unmap
18. vmm_arch_profile_init: parange??
rust-lld -> gnu ld

--------------------------------------------

1. vbar_el2: Holds the vector base address for any exception that is taken to EL2.
2. esr_el2: Holds syndrome information for an exception taken to EL2.

s2_pt_va = 0xfe8000001000

--------------------------------------------

upd: 2023-5-16

1. vm_msg_broadcast(cpu().vcpu.vm, &msg);
2. vgic_check_reg_alignment
3. vgic_remove_lr
4. todo: reuse VGicHandlerInfo
5. todo: vm interrupts set
6. todo: vgic_write_lr: pend, act

7. smc_call
8. icfg????