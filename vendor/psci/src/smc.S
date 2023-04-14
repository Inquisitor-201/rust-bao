.section ".text"
.global __smc

.align 2
__smc:
    .word 0xd4000003
    ret

