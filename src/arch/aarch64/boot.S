.section ".boot", "ax"
.global _el2_entry

_el2_entry:
    # x0 = MPIDR
	mrs  x0, MPIDR_EL1
	adrp x1, _image_start
