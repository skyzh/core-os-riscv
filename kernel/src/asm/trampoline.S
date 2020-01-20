// Copyright (c) 2020 Alex Chi
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

#include "macro.S"

.section trampsec
trampoline:
.globl uservec
.align 4
uservec:
	# swap t6 into sscratch
	csrrw t6, sscratch, t6
	# save all but t6
	.set 	i, 1
	.rept	30
		save_gp	%i
		.set	i, i+1
	.endr
	# use t5 as trapframe base, save user t6
	mv		t5, t6
	csrr	t6, sscratch
	save_gp 31, t5

	# restore kernel stack pointer from p->tf->kernel_sp
	ld sp, 520(t5)

	# make tp hold the current hartid, from p->tf->kernel_hartid
	ld tp, 528(t5)

	# load the address of usertrap(), p->tf->kernel_trap
	ld t0, 536(t5)

	# restore kernel page table from p->tf->kernel_satp
	ld t1, 512(t5)
	csrw satp, t1
	sfence.vma zero, zero

	# a0 is no longer valid, since the kernel page
	# table does not specially map p->tf.

	# jump to usertrap(), which does not return
	jr t0
.globl userret
.align 4
userret:
	# userret(TRAPFRAME, pagetable)
	# switch from kernel to user.
	# usertrapret() calls here.
	# a0: TRAPFRAME, in user page table.
	# a1: user page table, for satp.
	csrw    satp, a1
	sfence.vma zero, zero

	# save trap frame to sscratch
	csrw sscratch, a0

	# restore all user registers
	mv t6, a0
	.set 	i, 1
	.rept	31
		load_gp	%i
		.set	i, i+1
	.endr

	sret
