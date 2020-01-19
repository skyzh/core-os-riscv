// Copyright (c) 2020 Alex Chi
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

#include "macro.S"

.section trampsec
trampoline:
.globl uservec
uservec:
1:
	wfi
	j 1b
.globl userret
userret:
	# userret(TRAPFRAME, pagetable)
	# switch from kernel to user.
	# usertrapret() calls here.
	# a0: TRAPFRAME, in user page table.
	# a1: user page table, for satp.
	csrw    satp, a1
	sfence.vma zero, zero

	# restore all user registers
	mv t6, a0
	.set 	i, 1
	.rept	31
		load_gp	%i
		.set	i, i+1
	.endr

	sret