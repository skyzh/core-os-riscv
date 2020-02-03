OUTPUT_ARCH( "riscv" )

ENTRY( _start )

MEMORY
{
  ram : ORIGIN = 0x80000000, LENGTH = 128M
}

PHDRS
{
  text PT_LOAD;
  data PT_LOAD;
  bss PT_LOAD;
}

SECTIONS
{
  .text : {
    . = ALIGN(4096);
    PROVIDE(__text_start = .);
    *(.text.init *.text._start) *(.text .text.*)
    . = ALIGN(4096);
    PROVIDE(__trampoline_text_start = .);
    *(trampsec)
    PROVIDE(__text_end = .);
  } >ram AT>ram :text
   PROVIDE(__global_pointer = .);
  .rodata : {
    . = ALIGN(4096);
    PROVIDE(__rodata_start = .);
    *(.rodata .rodata.*)
    PROVIDE(__rodata_end = .);
  } >ram AT>ram :text

  .data : {
    . = ALIGN(4096);
    PROVIDE(__data_start = .);
    *(.sdata .sdata.*) *(.data .data.*)
    PROVIDE(__data_end = .);
  } >ram AT>ram :data

  .bss :{
    . = ALIGN(4096);
    PROVIDE(__bss_start = .);
    *(.sbss .sbss.*) *(.bss .bss.*)
    PROVIDE(__bss_end = .);
  } >ram AT>ram :bss

  PROVIDE(__memory_start = ORIGIN(ram));
  PROVIDE(__kernel_stack_start = __bss_end);
  PROVIDE(__kernel_stack_end = __kernel_stack_start + 0x80000);
  PROVIDE(__memory_end = ORIGIN(ram) + LENGTH(ram));
  PROVIDE(__heap_start = __kernel_stack_end);
  PROVIDE(__heap_size = __memory_end - __kernel_stack_end);
}
