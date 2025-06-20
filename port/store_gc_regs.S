.section .text
.global store_gc_regs

@ returns the stack pointer
@ uint32_t store_gc_regs(r0=uint32_t regs[10])
store_gc_regs:
    @ store registers into given array
    str     r4, [r0], #4
    str     r5, [r0], #4
    str     r6, [r0], #4
    str     r7, [r0], #4
    str     r8, [r0], #4
    str     r9, [r0], #4
    str     r10, [r0], #4
    str     r11, [r0], #4
    str     r12, [r0], #4
    str     r13, [r0], #4

    @ return the sp
    mov     r0, sp
    bx      lr
