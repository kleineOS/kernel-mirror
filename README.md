# kleineOS

Klein (adj.) small in size.

As the name suggests, kleineOS is a small operating system. The kernel will be
written with the sole goal of it being small, which can make it a great
educational project.

## Developing

The rust nightly toolchain is required.

For automatically running the build scripts and a virtual machine,
`<https://github.com/casey/just>` is recommended, along with
`qemu-system-riscv64`. Once these are installed, you can run the code with
`cargo run`.

## License

The kernel is licensed under the GNU Public License v3.0.
