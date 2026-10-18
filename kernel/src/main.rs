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

    println!();
    println!("Hello, World! hart_id={hart_id}");

    let mut mem_region = MemRegion::get_first_mem_region(&dtb);
    // currently, the devicetree pointer in qemu is at the top end of the mem region
    // we exclude it
    mem_region.size = dtbp as u64 - mem_region.address;

    let kernel_region = {
        unsafe extern "C" {
            static __kernel_start: u8;
            static __kernel_end: u8;
        }

        let kernel_start = (&raw const __kernel_start) as u64;
        let kernel_end = (&raw const __kernel_end) as u64;

        let kernel_size = kernel_end - kernel_start;
        MemRegion::new(kernel_start, kernel_size)
    };

    // any memory before the kernel can be considered to be reserved for the firmware
    let sbi_region = MemRegion::new(
        mem_region.address,
        kernel_region.address - mem_region.address,
    );

    let r = mem_region
        .find_free_region(
            0x1000 * 16, // find 16 free pages
            0x1000,      // aligned to the page size
            &[sbi_region, kernel_region, dtb.blob_mem_region()],
        )
        .expect("should be able to find a memory region for a physical page allocator");

    println!(
        "mem region for physical page allocator: {r:x?} reserved={:#x?} total-region={:#x?}",
        &[sbi_region, kernel_region, dtb.blob_mem_region()],
        mem_region,
    );

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
