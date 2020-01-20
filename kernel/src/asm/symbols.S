.section .rodata
.global HEAP_START
HEAP_START: .dword __heap_start
.global HEAP_SIZE
HEAP_SIZE: .dword __heap_size
.global TEXT_START
TEXT_START: .dword __text_start
.global TEXT_END
TEXT_END: .dword __text_end
.global RODATA_START
RODATA_START: .dword __rodata_start
.global RODATA_END
RODATA_END: .dword __rodata_end
.global DATA_START
DATA_START: .dword __data_start
.global DATA_END
DATA_END: .dword __data_end
.global BSS_START
BSS_START: .dword __bss_start
.global BSS_END
BSS_END: .dword __bss_end
.global KERNEL_STACK_START
KERNEL_STACK_START: .dword __kernel_stack_start
.global KERNEL_STACK_END
KERNEL_STACK_END: .dword __kernel_stack_end
.global TRAMPOLINE_TEXT_START
TRAMPOLINE_TEXT_START: .dword __trampoline_text_start
