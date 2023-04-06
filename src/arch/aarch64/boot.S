.section ".boot.entry", "ax"
.global _el2_entry

_el2_entry:
	mrs  x0, MPIDR_EL1
	adrp x1, _image_start

	/* 
	 * Install vector table physical address early, in case exception occurs
	 * during this initialization.
	 */
	adr	x3, _hyp_vector_table
	msr	VBAR_EL2, x3

	/**
	 * 	Linearize cpu id according to the number of clusters and processors per
	 * cluster. We are only considering two levels of affinity.
	 *  TODO: this should be done some other way. We shouln'd depend on the platform
	 * description this early in the initialization.
	 */

	lsr x3, x0, #8 
	and x3, x3, 0xff
	adr x4, {platform}
	add x4, x4, {cores_num_off}
	mov x5, xzr
	mov x7, xzr
1:
	cmp x5, x3
	b.ge	2f
	ldrb x6, [x4]
	add x4, x4, #1
	add x5, x5, #1
	add x7, x7, x6
	b 	1b
2:
	and x0, x0, #0xff
	add x0, x0, x7

.pushsection .data
_master_set:
	.8byte 	0

.global CPU_MASTER
CPU_MASTER:
	.8byte 	0
.popsection

	/**
	* Setting CPU_MASTER by lottery, the first cpu to atomically set _master_set,
	* becomes the cpu master. As a result x9 should is set to !is_cpu_master.
	*/
    mov x5, #1
    adr	x3, _master_set
_set_master_cpu:
	ldaxr w9, [x3]
	cbnz w9, 1f
	stlxr w9, w5, [x3]
	cbnz w9, _set_master_cpu
	adr x3, CPU_MASTER
	str x0, [x3]
1:

	/** 
	 * TODO: bring the system to a well known state. This includes disabling 
	 * the MMU (done), all caches (missing i$), BP and others...
	 * and invalidating them.	
	 */
 
	/* boot_clear stack pointer to avoid unaligned SP exceptions during boot */
    mov x3, xzr
	mov sp, x3

	/* Invalidate caches */
	/* Should we also clean ? */
	mov x19, x0
	mov x20, x1

	mov x0, #0
	bl boot_cache_invalidate
	cbnz x9, 1f
	mov x0, #2
	bl	boot_cache_invalidate
1:
	mov x0,	x19
	mov x1,	x20

	ic iallu

	bl boot_arch_profile_init

/*
 * Code taken from "Application Note Bare-metal Boot Code for ARMv8-A
 * Processors - Version 1.0"
 *
 * x0 - cache level to be invalidated (0 - dl1$, 1 - il1$, 2 - l2$)
 */
.global boot_cache_invalidate
boot_cache_invalidate:
	msr csselr_el1, x0 
	mrs x4, ccsidr_el1 // read cache size id.
	and x1, x4, #0x7
	add x1, x1, #0x4 // x1 = cache line size.
	ldr x3, =0x7fff
	and x2, x3, x4, lsr #13 // x2 = cache set number – 1.
	ldr x3, =0x3ff
	and x3, x3, x4, lsr #3 // x3 = cache associativity number – 1.
	clz w4, w3 // x4 = way position in the cisw instruction.
	mov x5, #0 // x5 = way counter way_loop.
way_loop:
	mov x6, #0 // x6 = set counter set_loop.
set_loop:
	lsl x7, x5, x4
	orr x7, x0, x7 // set way.
	lsl x8, x6, x1
	orr x7, x7, x8 // set set.
	dc cisw, x7 // clean and invalidate cache line.
	add x6, x6, #1 // increment set counter.
	cmp x6, x2 // last set reached yet?
	ble set_loop // if not, iterate set_loop,
	add x5, x5, #1 // else, next way.
	cmp x5, x3 // last way reached yet?
	ble way_loop // if not, iterate way_loop
	ret

/***** 	Helper functions for boot code. ******/

/* boot_clear: Clear the memory in the range [x16, x17). */

.global boot_clear
boot_clear:
2:
	cmp	x16, x17			
	b.ge 1f				
	str	xzr, [x16], #8	
	b	2b				
1:
	ret
