.section .text.boot
.global _start

_start:
    la sp, __boot_stack_top

    la t0, __bss_start
    la t1, __bss_end

1:
    # Clear the BSS section:
    bgeu t0, t1, 2f # if t0 >= t1: goto 2f
    sb zero, 0(t0)  # write 0 (one byte) to the addr at t0
    addi t0, t0, 1  # advance the address at t0 by one byte
    j 1b            # goto start of the loop
2:
    call start
3:
    j .
