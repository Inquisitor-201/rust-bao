OUTPUT_ARCH(aarch64)
ENTRY(_el2_entry)

PAGE_SIZE = 0x1000;
BASE_ADDRESS = 0x50000000;

SECTIONS
{
    . = BASE_ADDRESS;
    _image_start = ABSOLUTE(.);
    .boot : {
        *(.boot.entry)
        *(.boot .boot.*) 
    }
    .text : {
        . = ALIGN(4K);
        *(.text .text.*)
    }
    . = ALIGN(PAGE_SIZE); /* start RO sections in separate page */

	.rodata :  {
		*(.rdata .rodata .rodata.*)
	}

    . = ALIGN(PAGE_SIZE); /* start RW sections in separate page */
	
	.data : {
		*(.data .data.*)
        PROVIDE(__global_pointer$ = . + 0x800);
		*(.sdata .sdata.* .sdata2.*)
	}
}