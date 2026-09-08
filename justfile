runner kernel *args:
    qemu-system-riscv64 -machine virt -nographic -kernel {{kernel}} {{args}}
