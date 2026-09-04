.section .text
.global guest_boot

li a7, 0x01

bnez a0, second_vcpu

li t0, 'F'
fmv.w.x f10, t0
fmv.x.w a0, f10

first_vcpu:
    ecall

    j first_vcpu

second_vcpu:
    li a0, 'D'
    ecall

    j second_vcpu
