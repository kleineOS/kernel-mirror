#![no_std]
#![no_main]

use core::{fmt::Write, panic::PanicInfo};

use crate::{
    fdt::{Fdt, Prop},
    sbi::Console,
};

mod arch;
mod fdt;
mod init;
mod lock;
mod sbi;
mod trap;

static WRITER: lock::Mutex<Console> = lock::Mutex::new(Console);

#[unsafe(no_mangle)]
extern "C" fn start(hart_id: usize, dtb: *mut u8) {
    // Safety: Running the code at the start of our kernel, and only runnig it once
    unsafe { clear_bss() };
    trap::set_trap();

    let dtb = unsafe { Fdt::from_raw_ptr(dtb).unwrap() };

    println!();
    println!("Hello, World! hart_id={hart_id}");

    dtb.structure(c"/memory", |Prop { node, name, data }| {
        println!("{node:?} PROP: name={name:?},data=[{} bytes]", data.len());
    })
    .expect("could not parse devicetree");

    arch::unimp();

    todo!("Zero out BSS and initialise the harts");
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

    let start = &raw mut __bss_start as *mut u8;
    let end = &raw mut __bss_end as *mut u8;

    let len = end as usize - start as usize;

    // Safety: The linker script defines start and end sections for the BSS section. It must be
    // cleared during early init code.
    unsafe { core::ptr::write_bytes(start, 0, len) };
}

include_asm!("entry.s");
include_asm!("ktrapvec.s");

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write as _;
    WRITER.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
