# Copyright (c) 2020 Alex Chi
#
# This software is released under the MIT License.
# https://opensource.org/licenses/MIT

# Define a .text.init section.
.section .text.init

# Execution starts here.
.global _start
.global __kernel_stack_start
.global kinit
_start:
	# Allocate 64K stack for each hart
	la sp, __kernel_stack_start
	li a0, 0x10000
	csrr a1, mhartid
	addi a1, a1, 1
    mul a0, a0, a1
    add sp, sp, a0
    # jump to kinit in lib.rs
    call kinit