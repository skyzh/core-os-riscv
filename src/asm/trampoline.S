.section trampsec
.globl userret
userret:
        # userret(TRAPFRAME, pagetable)
        # switch from kernel to user.
        # usertrapret() calls here.
        # a0: TRAPFRAME, in user page table.
        # a1: user page table, for satp.

        # switch to the user page table.
        csrw    satp, a1
        sfence.vma zero, zero
        li t0, 0x80000000
        csrw sepc, t0
        li sp, 0x80002000
        sret
1:
        wfi
        j 1b