.section .text
.global guest_boot
guest_boot:

    li a7, 0x01
    li a0, '1'

    ecall

    j guest_boot
