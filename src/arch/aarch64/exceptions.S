.text
.balign 0x800
.global _hyp_vector_table	
_hyp_vector_table:

/* 
 * EL2 with SPx
 */
.balign 0x80
curr_el_sp0_sync:        
    b	.
.balign 0x80
curr_el_sp0_irq:  
    b   .
.balign 0x80
curr_el_sp0_fiq:         
    b	.
.balign 0x80
curr_el_sp0_serror:      
    b	.

.global vcpu_arch_entry
vcpu_arch_entry:
    mrs x0, tpidr_el2
    ldr x0, [x0]
    mov sp, x0

    ldp x0, x1, [sp, #(8*31)]
    msr ELR_EL2, x0
    msr SPSR_EL2, x1

    ldp x0, x1,   [sp, #(8*0)]
    ldp x2, x3,   [sp, #(8*2)]
    ldp x4, x5,   [sp, #(8*4)]
    ldp x6, x7,   [sp, #(8*6)]
    ldp x8, x9,   [sp, #(8*8)]
    ldp x10, x11, [sp, #(8*10)]
    ldp x12, x13, [sp, #(8*12)]
    ldp x14, x15, [sp, #(8*14)]
    ldp x16, x17, [sp, #(8*16)]
    ldp x18, x19, [sp, #(8*18)]
    ldp x20, x21, [sp, #(8*20)]
    ldp x22, x23, [sp, #(8*22)]
    ldp x24, x25, [sp, #(8*24)]
    ldp x26, x27, [sp, #(8*26)]
    ldp x28, x29, [sp, #(8*28)]
    ldr x30,      [sp, #(8*30)]
    
    eret
    b   .