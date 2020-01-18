.section trampsec
.globl userret
userret:
        # userret(TRAPFRAME, pagetable)
        # switch from kernel to user.
        # usertrapret() calls here.
        # a0: TRAPFRAME, in user page table.
        # a1: user page table, for satp.

        # switch to the user page table.
        li t0, 0x0000000080000000
        csrw sepc, t0
        csrw stvec, t0
        li sp, 0x0000000080001000
        li		t0, (1 << 5)
        csrw	sstatus, t0
        csrw    sscratch, a0
        csrw    satp, a1
        sfence.vma zero, zero
        sret
