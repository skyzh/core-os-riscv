OUTPUT_ARCH( "riscv" )

ENTRY( _start )

SECTIONS
{
    . = 0x80000000;

    .text :
    {
        *(.text._start) *(.text .text.*)
    }

    .rodata :
    {
        *(.rdata .rodata .rodata.*)
    }

    .data :
    {
        *(.data .data.*)
    }

    .bss :
    {
        *(.bss bss.*)
    }
}
