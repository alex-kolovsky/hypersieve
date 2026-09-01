.section .text
.global guest_boot

li a7, 0x01


bnez a0, second_vcpu

first_vcpu:

    li t0, 1
    vsetvli t1, t0, e32, m1, ta, ma
    li t0, 'V'
    vmv.v.x v31, t0

    first_vcpu_loop:

    vmv.x.s a0, v31
    ecall


    j first_vcpu_loop

second_vcpu:
    li a0, 'B'
    ecall

    j second_vcpu
