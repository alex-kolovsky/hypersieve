.section .init

.extern __bss_start
.extern __bss_end

.extern __stack_end
.global _start
_start:

        # Load trap handler address to stvec csr so traps in HS/U/VS/VU mode will be processed in trap_handler function.
        la t0, trap_handler
        csrw stvec, t0

        # Load stack pointer to sp register.
        la sp, __stack_end

        # Clear BSS section.
        la t0, __bss_start
        la t1, __bss_end

        sw x0, 0(t0)

        clear_bss_loop:

                addi t0, t0, 4
                sw x0, 0(t0)
                blt t0, t1, clear_bss_loop

        # Jump to rust main function.
        j main
