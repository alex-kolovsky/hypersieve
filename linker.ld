ENTRY(_start)
MEMORY {
  HYP_MEM (rwx) : ORIGIN = 0x80200000, LENGTH = 128M
}

SECTIONS {
  .text : ALIGN(8) {
    KEEP(*(.init));
    *(.text .text.*);
    . = ALIGN(8);
    *(.stvec);
  }

  .rodata : ALIGN(8) {
    *(.rodata .rodata.*);
  }

  .data : ALIGN(8) {
    *(.data .data.*);
    *(.sdata .sdata.*)
  }

  .bss (NOLOAD) : ALIGN(8) {
    __bss_start = .;
    *(.bss .bss.*);
    *(.sbss .sbss.*);
    . = ALIGN(8);
    __bss_end = .;
  }

  .stack (NOLOAD) : ALIGN(16) {
    __stack_start = .;
    . += 1024 * 1024; /* 1 MB */
    __stack_end = .;
  }

  .heap (NOLOAD) : ALIGN(16) {
    __heap_start = .;
    . += 64 * 1024 * 1024; /* 64 MiB */
    __heap_end = .;
  }

  /DISCARD/ : {
    *(.eh_frame);
    *(.eh_frame_hdr);
    *(.note.GNU-stack);
    *(.comment);
  }
}
