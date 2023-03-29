.macro PTE_INDEX_ASM	index, addr, level
	lsr \index, \addr, ((9 * (3 - \level)) + 12)
	and \index, \index, #0x1ff
	lsl \index, \index, #3
.endm

.section ".boot", "ax"
.global boot_arch_profile_init
boot_arch_profile_init:

    mov x20, x30

	/*
     * Register x18 contains the size of the allocated physical memory between
	 * the loadable sections of the image and the non-loadable.
     */
    ldr x18, =extra_allocated_phys_mem

	/* Disable caches and MMU */
	mrs x3, SCTLR_EL2 
	bic x3, x3, #0x7	
	msr SCTLR_EL2, x3

	/* Skip initialy global page tables setup if not bsp (boot cpu) */
	cbnz	x9, wait_for_bsp

    adr x16, _page_tables_start
    adr	x17, _page_tables_end
	add x16, x16, x18
	add x17, x17, x18
	bl	boot_clear

    /* Set temporary flat mapping to switch to VAS. */

    adr x4, root_l1_flat_pt
    add x4, x4, x18
    PTE_INDEX_ASM x5, x1, 1
	add x6, x1, #({PTE_HYP_FLAGS} | {PTE_SUPERPAGE})
	str x6, [x4, x5]
wait_for_bsp: