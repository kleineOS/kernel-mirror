OUTPUT_FORMAT("elf64-littleriscv")
OUTPUT_ARCH(riscv)
ENTRY(_start)


KERNEL_BASE = 0x80200000;
PAGE_SIZE = 4K;
BOOT_STACK_SIZE = 16K;

SECTIONS {
    . = KERNEL_BASE;
    __kernel_start  = .;

    .text ALIGN(PAGE_SIZE) : {
        KEEP(*(.text.boot))
        KEEP(*(.text.trap))
        
        *(.text .text.*)
    }

    .rodata ALIGN(PAGE_SIZE) : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    .data ALIGN(PAGE_SIZE) : {
        *(.data. .data.*)
        *(.sdata .sdata.*)
    }

    .bss ALIGN(PAGE_SIZE) (NOLOAD) : {
        __bss_start = .;

        *(.bss .bss.*)
        *(.sbss .sbss.*)

        __bss_end = .;
    }

    __kernel_end = .;

    /*
     * This will be used by the very first init function in the kernel. That
     * function will then set up a stack for every single hart, after which
     * this will become unused.
     */
    .stack ALIGN(PAGE_SIZE) (NOLOAD) : {
        __boot_stack_bottom = .;
        . += BOOT_STACK_SIZE;
        __boot_stack_top = .;
    }
}
