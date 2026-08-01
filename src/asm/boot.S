.section .init

.extern __bss_start
.extern __bss_end

.extern __stack_end
.global _start
_start:

        trap_handler_load:
                la t0, trap_handler
                csrw stvec, t0

        stack_pointer_load:
                la sp, __stack_end

        la t0, __bss_start
        la t1, __bss_end

        clear_bss_loop:

                addi t0, t0, 4
                sw x0, 0(t0)
                blt t0, t1, clear_bss_loop

        jump_to_main:
                j main
