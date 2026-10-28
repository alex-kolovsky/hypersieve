.section .text
.global guest_boot

# If the Hart ID is non-zero, this is a secondary Hart.
bnez a0, second_hart

first_hart:

    # Load the legacy putchar extension ID
    # into the ecall extension register (a7).
    li a7, 0x01

    # Configure vector registers to a 32-bit width per register.
    li t0, 1
    vsetvli t1, t0, e32, m1, ta, ma

    # Load the ASCII character 'V' into the v31 register.
    li t0, 0x10000000
    li t1, 'V'
    vmv.v.x v31, t1

    first_hart_loop:

    # Print the 'V' letter from the v31 register via Uart.
    vse8.v v31, (t0)

    # It tests the save-and-restore logic of the vector extension.
    ecall

    j first_hart_loop

second_hart:

    # Write the ASCII character 'U' into the UART.
    li a0, 'U'
    li a1, 0x10000000
    amoswap.w zero, a0, (a1)
    li a1, 0xFFFFFFFFDEFEC8ED
    sw a0, 0(a1)

    # sw a0, 0(a1)

    j second_hart
