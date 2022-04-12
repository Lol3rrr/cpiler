
.global _start
.align 2
_start: stp x29, x30, [sp, #-32]!
        stur x19, [sp, #16]
        b block_600001430110
block_600001430110: stp x0, x1, [sp, #-16]!
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
        b block_600001438010
block_600001438010: ldp x29, x30, [sp], #16
                    ret
main: stp x29, x30, [sp, #-96]!
      stur x21, [sp, #16]
      stur x20, [sp, #24]
      stur x19, [sp, #32]
      stur x22, [sp, #40]
      b block_600001438090
block_600001438090: b block_600001438110
block_600001438110: movz w19, #100, LSL #0
                    stur w19, [sp, #92]
                    b block_600001438190
block_600001438190: movz w19, #0, LSL #0
                    stur w19, [sp, #84]
                    b block_600001438210
block_600001438210: ldursw x19, [sp, #84]
                    mov w19, w19
                    mov w19, w19
                    movz w9, #10, LSL #0
                    cmp w19, w9, LSL #0
                    cset x19, lt
                    ldursw x22, [sp, #92]
                    ldursw x19, [sp, #92]
                    cbnz x19, block_600001438290
                    b block_600001438310
block_600001438310: b block_600001438610
block_600001438610: mov w19, w22
                    mov w0, w19
                    ldur x21, [sp, #16]
                    ldur x20, [sp, #24]
                    ldur x19, [sp, #32]
                    ldur x22, [sp, #40]
                    ldp x29, x30, [sp], #96
                    ret
block_600001438290: b block_600001438390
block_600001438390: movz w19, #0, LSL #0
                    stur w19, [sp, #88]
                    b block_600001438410
block_600001438410: ldursw x21, [sp, #88]
                    ldursw x19, [sp, #88]
                    mov w19, w19
                    mov w19, w19
                    movz w9, #10, LSL #0
                    cmp w19, w9, LSL #0
                    cset x20, lt
                    ldursw x19, [sp, #92]
                    ldursw x19, [sp, #92]
                    cbnz x20, block_600001438490
                    ldursw x19, [sp, #84]
                    b block_600001438510
block_600001438510: b block_600001438590
block_600001438590: add w19, w20, #1, LSL #0
                    stur w19, [sp, #84]
                    b block_600001438210
block_600001438490: mov x19, x19
                    stur x19, [sp, #64]
                    movz x9, #1, LSL #0
                    sub x19, x19, x9, LSL #0
                    stur x19, [sp, #72]
                    mov w19, w19
                    stur w19, [sp, #92]
                    add w19, w21, #1, LSL #0
                    stur w19, [sp, #88]
                    b block_600001438410
.data
