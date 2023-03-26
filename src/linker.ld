OUTPUT_ARCH(aarch64)
ENTRY(_start)
BASE_ADDRESS = 0x50000000;

SECTIONS
{
    . = BASE_ADDRESS;
    .text : {
        . = ALIGN(4K);
        *(.text .text.*)
    }
}