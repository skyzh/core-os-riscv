# boot.S
# bootloader for SoS
# Stephen Marz
# 8 February 2019
.section .text.init
.global _start
.global start

_start:
	# setup stack
	# sp = __stack + (hartid * 4096)
	la sp, __stack
	li a0, 1024*4
	csrr a1, mhartid
	addi a1, a1, 1
	mul a0, a0, a1
	add sp, sp, a0
	# jump to start()
	call start
