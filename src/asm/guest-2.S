.section .text
.global guest_boot

li a7, 0x01

bnez a0, second_vcpu

first_vcpu:
    li a0, 'C'
    ecall

    j first_vcpu

second_vcpu:
    li a0, 'D'
    ecall

    j second_vcpu
