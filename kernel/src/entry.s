.section .text.boot
.global _start

_start:
    la sp, __boot_stack_top
    call start
