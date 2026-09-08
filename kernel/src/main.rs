#![no_std]
#![no_main]
#![feature(riscv_ext_intrinsics)]

use core::{fmt::Write, panic::PanicInfo};

unsafe extern "C" {
    static mut __bss_start: usize;
    static mut __bss_end: usize;

    fn ktrapvec();
}

#[unsafe(no_mangle)]
extern "C" fn kerneltrap() {
    let scause = unsafe {
        let scause: usize;
        core::arch::asm!("csrr {}, scause", out(reg) scause, options(nomem, nostack));
        scause
    };

    let int = ((scause >> 63) & 1) == 1;
    let cause = scause & ((1 << 63) - 1);

    todo!(
        "Kernel trap is not yet implemented. int={} cause={}",
        int,
        cause
    );
}

#[unsafe(no_mangle)]
extern "C" fn start(hart_id: usize, dtb: usize) {
    unsafe {
        let start = &raw mut __bss_start;
        let end = &raw mut __bss_end;

        let len = end as usize - start as usize;

        core::ptr::write_bytes(start, 0, len);
    };

    let addr = ktrapvec as *const () as usize;
    unsafe { core::arch::asm!("csrw stvec, {0}", in(reg) addr, options(nostack, preserves_flags)) };

    let _ = writeln!(Console, "Hello, World!");
    let _ = writeln!(Console, "Hart ID: {hart_id}, dtb ptr: {dtb:#x}");

    unsafe { core::arch::asm!("unimp") };

    todo!("Zero out BSS and initialise the harts");
}

struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Sbi::dbcn_console_write(s);
        Ok(())
    }
}

/// Represents the required data for calling SBI
#[derive(Debug)]
pub struct Sbi {
    /// First argument for the SBI call
    a0: usize,
    /// Second argument for the SBI call
    a1: usize,
    /// Third argument for the SBI call
    a2: usize,
    /// Fourth argument for the SBI call
    a3: usize,
    /// Fifth argument for the SBI call
    a4: usize,
    /// Sixth argument for the SBI call
    a5: usize,
    /// Function ID for the SBI call
    fid: usize, // a6
    /// Extension ID for the SBI call
    eid: usize, // a7
}

impl Sbi {
    pub fn dbcn_console_write(data: &str) {
        const EID: usize = 0x4442434E;
        const FID: usize = 0x0;

        let num_bytes = data.len();
        let addr = data.as_ptr() as usize;

        #[cfg(target_pointer_width = "64")]
        let (base_addr_lo, base_addr_hi) = (addr, 0);

        let sbi = Self {
            a0: num_bytes,
            a1: base_addr_lo,
            a2: base_addr_hi,
            a3: 0,
            a4: 0,
            a5: 0,
            fid: FID,
            eid: EID,
        };

        sbi.ecall();
    }

    pub fn dbcn_console_byte(data: u8) {
        const EID: usize = 0x4442434E;
        const FID: usize = 0x2;

        let sbi = Self {
            a0: data as usize,
            a1: 0,
            a2: 0,
            a3: 0,
            a4: 0,
            a5: 0,
            fid: FID,
            eid: EID,
        };

        sbi.ecall();
    }

    fn ecall(self) {
        unsafe {
            core::arch::asm!(
                "ecall",
                in("a0") self.a0,
                in("a1") self.a1,
                in("a2") self.a2,
                in("a3") self.a3,
                in("a4") self.a4,
                in("a5") self.a5,
                in("a6") self.fid,
                in("a7") self.eid,
            );
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let _ = writeln!(Console, "\n{info}");

    loop {
        unsafe { core::arch::riscv64::wfi() };
    }
}

#[macro_export]
macro_rules! include_asm {
    ($file:expr $(,)?) => {
        core::arch::global_asm!(include_str!($file));
    };
}

include_asm!("entry.s");
include_asm!("ktrapvec.s");
