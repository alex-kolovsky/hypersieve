.section .text
.global guest_boot

# Load the legacy putchar extension ID
# into the ecall extension register (a7).
li a7, 0x01

# If the Hart ID is non-zero, this is a secondary Hart.
bnez a0, second_vcpu

# Load the ASCII character 'F' into the f10 register.
li t0, 'F'
fmv.w.x f10, t0

first_vcpu:
    # Move the character from f10 to a0 and print it to the screen.
    fmv.x.w a0, f10
    ecall

    j first_vcpu

second_vcpu:
    # Print the ASCII character 'D' on the screen.
    li a0, 'D'
    ecall

    j second_vcpu
