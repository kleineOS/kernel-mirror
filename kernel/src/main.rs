#![no_std]
#![no_main]
#![warn(clippy::pedantic, clippy::nursery)]

use core::{fmt::Write, panic::PanicInfo};

use crate::{arch::sbi::Console, fdt::Fdt, mem::MemRegion};

mod arch;
mod fdt;
mod lock;
mod mem;
mod writer;

#[unsafe(no_mangle)]
unsafe extern "C" fn start(_hart_id: usize, dtbp: *mut u8) {
    arch::trap::set_trap();

    let dtb: Fdt = unsafe { Fdt::from_raw_ptr(dtbp) }
        .expect("devicetree pointer should lead to a valid device tree blob");

    println!();
    println!("kleineOS-kernel is starting");
    println!();

    let mem_region = dtb.mem_region();
    let kernel_region = MemRegion::kernel_region();
    // Any memory before the kernel can be considered to be reserved for the firmware. I am unsure
    // if this is a sound assumption for most hardware, but it is correct for the QEMU Virt VM
    let sbi_region = MemRegion::new(
        mem_region.address,
        kernel_region.address - mem_region.address,
    );

    // memory region for physical page allocator
    let phys_alloc_region = mem_region
        .find_free_region(
            0x1000 * 16, // find 16 free pages
            0x1000,      // aligned to the page size
            &[sbi_region, kernel_region, dtb.blob_mem_region()],
        )
        .expect("should be able to find a memory region for a physical page allocator");

    println!("mem region for physical page allocator: {phys_alloc_region:x?}",);

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
