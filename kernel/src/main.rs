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
mod lock;
mod sbi;
mod trap;
mod writer;

#[unsafe(no_mangle)]
unsafe extern "C" fn start(hart_id: usize, dtbp: *mut u8) {
    trap::set_trap();

    let dtb: Fdt = unsafe { Fdt::from_raw_ptr(dtbp) }
        .expect("devicetree pointer should lead to a valid device tree blob");

    let mut mem = MemRegion::get_first_mem_region(&dtb);
    // currently, the devicetree pointer in qemu is at the top end of the mem region
    // we exclude it
    mem.size = dtbp as u64 - mem.address;

    println!();
    println!("Hello, World! hart_id={hart_id} {mem:#x?}");

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

include_asm!("entry.s");
include_asm!("ktrapvec.s");
