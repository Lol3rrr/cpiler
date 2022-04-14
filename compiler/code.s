
.global _start
.align 2
_start: stp x29, x30, [sp, #-32]!
        stur x19, [sp, #16]
        b block_600001b88190
block_600001b88190: stp x0, x1, [sp, #-16]!
                    stp x2, x3, [sp, #-16]!
                    stp x4, x5, [sp, #-16]!
                    stp x6, x7, [sp, #-16]!
                    bl g_init
                    ldp x6, x7, [sp], #16
                    ldp x4, x5, [sp], #16
                    ldp x2, x3, [sp], #16
                    ldp x0, x1, [sp], #16
                    stp x0, x1, [sp, #-16]!
                    stp x2, x3, [sp, #-16]!
                    stp x4, x5, [sp, #-16]!
                    stp x6, x7, [sp, #-16]!
                    bl main
                    mov w19, w0
                    ldp x6, x7, [sp], #16
                    ldp x4, x5, [sp], #16
                    ldp x2, x3, [sp], #16
                    ldp x0, x1, [sp], #16
                    mov w0, w19
                    ldur x19, [sp, #16]
                    ldp x29, x30, [sp], #32
                    ret
g_init: stp x29, x30, [sp, #-16]!
        b block_600001b8c190
block_600001b8c190: ldp x29, x30, [sp], #16
                    ret
main: stp x29, x30, [sp, #-48]!
      stur x19, [sp, #16]
      stur x20, [sp, #24]
      b block_600001b8c110
block_600001b8c110: b block_600001b8c210
block_600001b8c210: movz w19, #10, LSL #0
                    stur w19, [sp, #32]
                    b block_600001b8c290
block_600001b8c290: ldursw x20, [sp, #32]
                    ldursw x19, [sp, #32]
                    mov w19, w19
                    mov w19, w19
                    movz w9, #0, LSL #0
                    cmp w19, w9, LSL #0
                    cset x19, gt
                    cbnz x19, block_600001b8c310
                    b block_600001b8c390
block_600001b8c390: mov w19, w20
                    mov w0, w19
                    ldur x19, [sp, #16]
                    ldur x20, [sp, #24]
                    ldp x29, x30, [sp], #48
                    ret
block_600001b8c310: mov x19, x20
                    movz x9, #1, LSL #0
                    sub x19, x19, x9, LSL #0
                    mov w19, w19
                    stur w19, [sp, #32]
                    b block_600001b8c290
.data
