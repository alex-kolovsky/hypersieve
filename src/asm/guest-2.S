.section .text
.global guest_boot

li a7, 0x01

bnez a0, second_vcpu

first_vcpu:
    li a0, '1'
    ecall

    j first_vcpu

second_vcpu:
    li a0, '2'
    ecall

    j second_vcpu
