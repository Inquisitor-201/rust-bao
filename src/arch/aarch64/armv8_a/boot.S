.macro PTE_INDEX_ASM	index, addr, level
	lsr \index, \addr, ((9 * (3 - \level)) + 12)
	and \index, \index, #0x1ff
	lsl \index, \index, #3
.endm

.data 
.align 3
/**
 * barrier is used to minimal synchronization in boot - other cores wait for
 * bsp to set it.
 */
 .global _boot_barrier
_boot_barrier: .8byte 0		

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

	/* Set global root mappings for hypervisor image */

	adr x4, root_l1_pt
	add x4, x4, x18
	ldr x5, =(({BAO_VAS_BASE} >> (9 * (3 - 1) + 12)) & 0x1FF)*8
	adr x6, root_l2_pt
	add x6, x6, x18
	add x6, x6, #({PTE_HYP_FLAGS} | {PTE_TABLE})
	str x6, [x4, x5]

	adr x4, root_l2_pt
	add x4, x4, x18
	ldr x5, =(({BAO_VAS_BASE} >> (9 * (3 - 2) + 12)) & 0x1FF)*8
	adr x6, root_l3_pt
	add x6, x6, x18
	add x6, x6, #({PTE_HYP_FLAGS} | {PTE_TABLE})
	str x6, [x4, x5]

	adr x4, root_l3_pt
	add x4, x4, x18

	/* map leaf pages: image region */
	/* x7 and x8 save virtual address */
	ldr x7, =_image_start
	ldr x8, =_image_load_end 
	adr x6, _image_start
	add x6, x6, #({PTE_HYP_FLAGS} | {PTE_TABLE})
1:
	cmp	x7, x8
	b.ge 2f

	PTE_INDEX_ASM x5, x7, 3
	str x6, [x4, x5]
	add x6, x6, {PAGE_SIZE}
	add x7, x7, {PAGE_SIZE}
	b	1b

	/* map leaf pages: _image_noload_start ~  */
2:
	ldr x8, =_image_end
	cmp x7, x8
	b.ge 3f
    ldr x6, =_image_noload_start
    ldr x10, ={BAO_VAS_BASE}
    sub x6, x6, x10
    add x6, x6, x1
	add x6, x6, #({PTE_HYP_FLAGS} | {PTE_PAGE})
	b 1b
3:
	adr x5, _boot_barrier
	mov x4, #1
	str x4, [x5]
	//	dsb // arguments?
	sev
	b 	map_cpu

wait_for_bsp:	
/* wait fot the bsp to finish up global mappings */
	wfe
	ldr x4, _boot_barrier
	cmp x4, #1
	b.lt wait_for_bsp

map_cpu:
	/**
	 *    x3 -> cpu base phys
	 *    x4 -> current pt base phys
	 *    x5 -> pte index
	 *    x6 -> phys addr
	 *    x7 -> virt addr
	 *    x8 -> aux 
	 */

	adrp x3, _dmem_phys_beg
	mov x8, #({CPU_SIZE} + ({PT_SIZE}*{PT_LVLS}))
	madd x3, x0, x8, x3

	mov	x16, x3	
	add	x17, x3, x8
	bl	boot_clear

	/* Get pointer to root page table */
	add x4, x3, #{CPU_SIZE}

	/* map original bootstrap flat mappings */
	PTE_INDEX_ASM x5, x1, 0 
	adr x6, root_l1_flat_pt
	add x6, x6, x18
	add x6, x6, #({PTE_HYP_FLAGS} | {PTE_TABLE})
	str x6, [x4, x5]
	
	ldr x5, =(({BAO_VAS_BASE} >> (9 * (3 - 0) + 12)) & 0x1FF)*8
	adr x6, root_l1_pt
	add x6, x6, x18
	add x6, x6, #({PTE_HYP_FLAGS} | {PTE_TABLE})
	str x6, [x4, x5]

	ldr x5, =(({BAO_CPU_BASE} >> (9 * (3 - 0) + 12)) & 0x1FF)*8
	add x6, x4, #{PT_SIZE}
	add x6, x6, #({PTE_HYP_FLAGS} | {PTE_TABLE})
	str x6, [x4, x5]

	add x4, x4, #{PT_SIZE}
	ldr x5, =(({BAO_CPU_BASE} >> (9 * (3 - 1) + 12)) & 0x1FF)*8
	add x6, x4, #{PT_SIZE}
	add x6, x6, #({PTE_HYP_FLAGS} | {PTE_TABLE})
	str x6, [x4, x5]

	add x4, x4, #{PT_SIZE}
	ldr x5, =(({BAO_CPU_BASE} >> (9 * (3 - 2) + 12)) & 0x1FF)*8
	add x6, x4, #{PT_SIZE}
	add x6, x6, #({PTE_HYP_FLAGS} | {PTE_TABLE})
	str x6, [x4, x5]

	add x4, x4, #{PT_SIZE}
	ldr x7, ={BAO_CPU_BASE}
	add x8, x7, #({CPU_SIZE}+{PT_SIZE})
	mov x6, x3
	add x6, x6, #({PTE_HYP_FLAGS} | {PTE_TABLE})
1:
	cmp	x7, x8
	b.ge setup_cpu
	PTE_INDEX_ASM x5, x7, 3
	str x6, [x4, x5]
	add x6, x6, #{PAGE_SIZE}
	add x7, x7, #{PAGE_SIZE}
	b	1b

setup_cpu:

	/**
	 * 	The operation is purposely commented out.
	 *  We are assuming monitor code already enabled smp coherency.
	 */ 
	ldr x3, ={TCR_EL2_DFLT}
	msr	TCR_EL2, x3

	/* set hypervisor default memory attributes */
	ldr x3, ={MAIR_EL2_DFLT}
	msr	MAIR_EL2, x3

    /* set cpu thread pointer */
    ldr x3, ={BAO_CPU_BASE}
    msr tpidr_el2, x3

    adrp x3, _dmem_phys_beg
	mov x8, #({CPU_SIZE} + ({PT_SIZE}*{PT_LVLS}))
	madd x3, x0, x8, x3
	add x3, x3, #{CPU_SIZE}
.arch armv8-a
	msr TTBR0_EL2, x3

	/**	
	 * TODO: set implementation defined registers such as ACTLR or AMAIR.
	 * Maybe define a macro for this in a implementation oriented directory
	 * inside arch.
	 */

	/**
	 * TODO: invalidate caches, TLBs and branch prediction.
	 * Need for barriers?
	 */

	ldr x5, =_enter_vas

	/* Enable MMU and caches */
	ldr x4, ={SCTLR_DFLT}
	msr	SCTLR_EL2, x4
	
	tlbi	alle2
	dsb	nsh
	isb
	
	br  x5

_enter_vas:

	/* Install vector table virtual address*/
	ldr x3, =_hyp_vector_table
	msr VBAR_EL2, x3

	/* Remove temporary mapping - the L1 page holding it leaks */
	ldr x4, ={BAO_CPU_BASE}
	add x4, x4, #{CPU_SIZE}
	PTE_INDEX_ASM x5, x1, 0 
	str xzr, [x4, x5]

    tlbi	alle2
	dsb	nsh
	isb

    sub x20, x20, x1
    ldr x3, =_image_start
    add x30, x20, x3
    ret
