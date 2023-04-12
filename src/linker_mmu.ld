OUTPUT_ARCH(aarch64)
ENTRY(_el2_entry)

PAGE_SIZE = 0x1000;
BAO_VAS_BASE = 0xfd8000000000;
BAO_CPU_BASE = 0xfe0000000000;
BAO_VM_BASE = 0xfe8000000000;
BAO_VAS_TOP = 0xff0000000000;

SECTIONS
{
    . = BAO_VAS_BASE;
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

	.ipi_cpumsg_handlers : {
		ipi_cpumsg_handlers = .;
		*(.ipi_cpumsg_handlers)
	}

	.ipi_cpumsg_handlers_id : {
		_ipi_cpumsg_handlers_id_start = .;
		*(.ipi_cpumsg_handlers_id)
	}

	_ipi_cpumsg_handlers_size = SIZEOF(.ipi_cpumsg_handlers);

    . = ALIGN(PAGE_SIZE);
    _image_load_end = .;

	/* Sections to be allocated only in physical memory, not in VA space */

	.vm_images : SUBALIGN(PAGE_SIZE) {
		_vm_image_start = .;
		KEEP(*(.vm_image*))
	}
	. = ALIGN(PAGE_SIZE);
	_vm_image_end = .;
    _image_noload_start = .;
    extra_allocated_phys_mem = _image_noload_start - _image_load_end;

	/**
	 * Rewind LC to reuse VMA addresses occupied by config and make VA
	 * start immediately after image.
	 * Also, force start address of next section to enforce this.
	 */
    . = _image_load_end;

	/* Only no load regions below */

	.bss (NOLOAD) : {
		_bss_start = .;
		*(.bss* .sbss*)
		*(COMMON)
		_bss_end = .;
	}

	.glb_page_tables (NOLOAD) : ALIGN(PAGE_SIZE) {
		_page_tables_start = .;
		*(.glb_page_tables)
		_page_tables_end = .;
	}

	. = ALIGN(PAGE_SIZE);
	_image_end = ABSOLUTE(.);
	_dmem_phys_beg = ABSOLUTE(.) + extra_allocated_phys_mem;

	. = ALIGN(PAGE_SIZE);
	_dmem_beg = ABSOLUTE(.);
}