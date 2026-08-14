.section .text
.global guest_boot
guest_boot:

    li a7, 0x01
    li a0, 'H'

    ecall

    vsetvli t0, x0, e32, m1, ta, ma
    vadd.vv v1, v2, v3

    li a7, 0x01
    li a0, 'I'

    ecall

    j guest_boot
