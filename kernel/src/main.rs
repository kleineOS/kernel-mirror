#![no_std]
#![no_main]
#![warn(clippy::pedantic, clippy::nursery)]

use core::{fmt::Write, panic::PanicInfo};

use crate::{
    fdt::{Fdt, MemRegion},
    sbi::Console,
};

mod arch;
mod fdt;
mod init;
mod lock;
mod sbi;
mod trap;
mod writer;

pub struct BitmapAlloc {}

#[unsafe(no_mangle)]
extern "C" fn start(hart_id: usize, dtbp: *mut u8) {
    // Safety: Running the code at the start of our kernel, and only runnig it once
    unsafe { clear_bss() };
    trap::set_trap();

    let dtb: Fdt = unsafe { Fdt::from_raw_ptr(dtbp).unwrap() };

    let mem = MemRegion::get_first_mem_region(&dtb);

    println!();
    println!(
        "Hello, World! hart_id={hart_id} {:x?} {mem:#x?}",
        dtbp as usize
    );

    sbi::Sbi::time_set_timer(usize::MAX);
    arch::sstatus_set_bit(arch::SSTATUS_SIE, true);
    arch::sie_set_bit(arch::SIE_STIE, true);
    sbi::Sbi::time_set_timer(arch::time() + 1_000_000);

    loop {
        arch::wfi();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // println will have a lock, so it may be better to simply write to the consoel directly here
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

    let start = (&raw mut __bss_start).cast::<u8>();
    let end = (&raw mut __bss_end).cast::<u8>();

    let len = end as usize - start as usize;

    // Safety: The linker script defines start and end sections for the BSS section. It must be
    // cleared during early init code.
    unsafe { core::ptr::write_bytes(start, 0, len) };
}

include_asm!("entry.s");
include_asm!("ktrapvec.s");
