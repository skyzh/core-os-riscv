# boot.S
# bootloader for SoS
# Stephen Marz
# 8 February 2019

# Define a .text.init section.
.section .text.init

# Execution starts here.
.global _start
_start:
	# Any hardware threads (hart) that are not bootstrapping
	# need to wait for an IPI
	csrr	t0, mhartid
	bnez	t0, 3f
	# SATP should be zero, but let's make sure
	csrw	satp, zero
	
	# Disable linker instruction relaxation for the `la` instruction below.
	# This disallows the assembler from assuming that `gp` is already initialized.
	# Set all bytes in the BSS section to zero.
	la 		a0, __bss_start
	la		a1, __bss_end
	bgeu	a0, a1, 2f
1:
	sd		zero, (a0)
	addi	a0, a0, 8
	bltu	a0, a1, 1b
2:
	# Control registers, set the stack, mstatus, mepc,
	# and mtvec to return to the main function.
	# li		t5, 0xffff;
	# csrw	medeleg, t5
	# csrw	mideleg, t5
	# The stack grows from bottom to top, so we put the stack pointer
	# to the very end of the stack range.
	la		sp, __stack_end
	# Setting `mstatus` register:
	# 0b11 << 11: Machine's previous protection mode is 3 (MPP=3).
	li		t0, 0b11 << 11
	csrw	mstatus, t0
	# Do not allow interrupts while running kinit
	csrw	mie, zero
	# Machine's exception program counter (MEPC) is set to `kinit`.
	la		t1, kinit
	csrw	mepc, t1
	# Set the return address to get us into supervisor mode
	la		ra, 2f
	# We use mret here so that the mstatus register is properly updated.
	mret
2:
	li		t0, (0b01 << 11) | (1 << 7) | (1 << 5)
	csrw	mstatus, t0
	la		t1, kmain
	csrw	mepc, t1
	la		t2, m_trap_vector
	csrw	mtvec, t2
	li		t2, 0x888
	csrw	mie, t2
	mret
3:

	# Parked harts go here. We need to set these
	# to only awaken if it receives a software interrupt,
	# which we're going to call the SIPI (Software Intra-Processor Interrupt).
	# We call the SIPI by writing the software interrupt into the Core Local Interruptor (CLINT)
	# Which is calculated by: base_address + hart * 4
	# where base address is 0x0200_0000 (MMIO CLINT base address)
	# We only use additional harts to run user-space programs, although this may
	# change.
4:
	wfi
	j		4b