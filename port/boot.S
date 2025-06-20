@ Putting this in a naked function makes GCC emit illegal instructions
.section .boot, "ax"
.global _boot

_boot:
	ldr sp, =__stack_top
	b _start
