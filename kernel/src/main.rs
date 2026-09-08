#![no_std]
#![no_main]

use core::{fmt::Write, panic::PanicInfo};

use crate::sbi::Console;

mod arch;
mod init;
mod sbi;
mod trap;

#[unsafe(no_mangle)]
extern "C" fn start(hart_id: usize, dtb: usize) {
    // Safety: Running the code at the start of our kernel, and only runnig it once
    unsafe { clear_bss() };
    trap::set_trap();

    let _ = writeln!(Console, "Hello, World!");
    let _ = writeln!(Console, "Hart ID: {hart_id}, dtb ptr: {dtb:#x}");

    arch::unimp();

    todo!("Zero out BSS and initialise the harts");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let _ = writeln!(Console, "\n{info}");

    loop {
        arch::wfi();
    }
}

/// Clear the BSS sector in memory, zeroing it out in the process. This is an important early init step.
///
/// # Safety
/// This must be ran only once, and near the start of the kernel.
unsafe fn clear_bss() {
    unsafe extern "C" {
        static mut __bss_start: usize;
        static mut __bss_end: usize;
    }

    let start = &raw mut __bss_start;
    let end = &raw mut __bss_end;

    let len = end as usize - start as usize;

    // Safety: The linker script defines start and end sections for the BSS section. It must be
    // cleared during early init code.
    unsafe { core::ptr::write_bytes(start, 0, len) };
}

include_asm!("entry.s");
include_asm!("ktrapvec.s");
