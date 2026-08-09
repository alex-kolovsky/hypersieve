.section .text
.global guest_boot
guest_boot:

    vsetvli t0, x0, e32, m1, ta, ma
    vadd.vv v1, v2, v3

    j guest_boot
