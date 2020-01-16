# boot.S
# bootloader for SoS
# Stephen Marz
# 8 February 2019
.section .text.init
.global _start
_start:
	# Any hardware threads (hart) that are not bootstrapping
	# need to wait for an IPI
	csrr	t0, mhartid
	bnez	t0, 3f
	la		sp, __stack
	# SATP should be zero, but let's make sure
	csrw	satp, zero
	call    start
3:
4:
	wfi
	j		4b