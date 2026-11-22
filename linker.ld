ENTRY(_start)

PHDRS {
  text PT_LOAD FLAGS(5);
  rodata PT_LOAD FLAGS(4);
  data PT_LOAD FLAGS(6);
  bss PT_LOAD FLAGS(6);
  stack PT_LOAD FLAGS(6);
  heap PT_LOAD FLAGS(6);
}

SECTIONS {
  . = 0x80200000;

  .text : ALIGN(8) {
      KEEP(*(.init));
      *(.text .text.*);
      . = ALIGN(8);
      *(.stvec);
  } : text

  .rodata : ALIGN(8) {
    *(.rodata .rodata.*);
  } : rodata

  .data : ALIGN(8) {
    *(.data .data.*);
    *(.sdata .sdata.*)
  } : data

  .bss (NOLOAD) : ALIGN(8) {
    __bss_start = .;
    *(.bss .bss.*);
    *(.sbss .sbss.*);
    . = ALIGN(8);
    __bss_end = .;
  } : bss

  .stack (NOLOAD) : ALIGN(16) {
    __stack_start = .;
    . += 1024 * 1024; /* 1 MiB */
    __stack_end = .;
  } : stack

  .heap (NOLOAD) : ALIGN(4096) {
    __heap_start = .;
    . += 64 * 1024 * 1024; /* 64 MiB */
    __heap_end = .;
  } : heap

  /DISCARD/ : {
    *(.eh_frame);
    *(.eh_frame_hdr);
    *(.note.GNU-stack);
    *(.comment);
  }
}
