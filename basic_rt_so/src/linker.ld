OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x87000000;

SECTIONS
{
    . = BASE_ADDRESS;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }
    . = ALIGN(4K);
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    . = ALIGN(4K);
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }
    . = ALIGN(4K);
    .bss : {
        *(.bss.interface)
        *(.bss.stack)
        *(.bss .bss.*)
    }
    /DISCARD/ : {
        *(.eh_frame)
    }
}