#![no_std]
#![no_main]
#![warn(clippy::pedantic, clippy::nursery)]

use core::{fmt::Write, panic::PanicInfo};

use crate::{
    arch::sbi::Console,
    fdt::{Fdt, MemRegion},
};

mod arch;
mod fdt;
mod lock;
mod writer;

#[unsafe(no_mangle)]
unsafe extern "C" fn start(hart_id: usize, dtbp: *mut u8) {
    arch::trap::set_trap();

    let dtb: Fdt = unsafe { Fdt::from_raw_ptr(dtbp) }
        .expect("devicetree pointer should lead to a valid device tree blob");

    let mut mem = MemRegion::get_first_mem_region(&dtb);
    // currently, the devicetree pointer in qemu is at the top end of the mem region
    // we exclude it
    mem.size = dtbp as u64 - mem.address;

    // NOTE: for the virtual machine, the FDT is at the very end of the memory region. So I am going
    // to actually ignore the step which ensures we dont consider the fdt's region as usable memory

    println!();
    println!("Hello, World! hart_id={hart_id} {mem:#x?}");

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
