OUTPUT_ARCH(aarch64)
ENTRY(_el2_entry)
BASE_ADDRESS = 0x50000000;

SECTIONS
{
    . = BASE_ADDRESS;
    _image_start = ABSOLUTE(.);
    .boot : {
        *(.boot .boot.*) 
    }
    .text : {
        . = ALIGN(4K);
        *(.text .text.*)
    }
}