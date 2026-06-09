OUTPUT_ARCH("riscv")
ENTRY(_start)

SECTIONS
{
    . = 0x00000000;

    .text : ALIGN(4)
    {
        *(.text.init)
        *(.text.startup)
        *(.text*)
    }

    . = 0x00004000;

    .data : ALIGN(4)
    {
        *(.rodata*)
        *(.srodata*)
        *(.data*)
        *(.sdata*)
    }

    .bss : ALIGN(4)
    {
        __bss_start = .;
        *(.sbss*)
        *(.bss*)
        *(COMMON)
        __bss_end = .;
    }

    . = ALIGN(16);
    _end = .;

    PROVIDE(__global_pointer$ = 0x00004800);
    PROVIDE(__stack_top = 0x0003fe00);
}
