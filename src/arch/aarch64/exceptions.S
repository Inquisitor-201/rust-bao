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