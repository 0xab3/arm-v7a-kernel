ENTRY(_start)

SECTIONS {
  . = 0x80010000;
  _KERNEL_START = .;

  .stub   : { _start.o(.text) }
  .text ALIGN(32): { *(.text) }
  .rodata : { *(.rodata) }
  .data   : { *(.data) }
  .bss    : { *(.bss COMMON) }

  STACK_BASE = .;
  . += 0x10000;
  STACK_TOP = .;
  KERNEL_END = .;
}
