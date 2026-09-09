runner kernel *args:
    qemu-system-riscv64 \
        -cpu rva23s64   \
        -smp 4 -m 1G    \
        -machine virt   \
        -nographic      \
        -kernel {{kernel}} {{args}}
