.section .text
.global guest_boot

li a7, 0x01

bnez a0, second_vcpu

first_vcpu:
    li a0, 'A'
    ecall

    j first_vcpu

second_vcpu:
    li a0, 'B'
    ecall

    j second_vcpu
