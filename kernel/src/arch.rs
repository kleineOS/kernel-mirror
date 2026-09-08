#[macro_export]
macro_rules! include_asm {
    ($file:expr $(,)?) => {
        core::arch::global_asm!(include_str!($file));
    };
}

pub fn wfi() {
    unsafe { core::arch::asm!("wfi", options(nomem, nostack, preserves_flags)) };
}

pub fn unimp() {
    unsafe { core::arch::asm!("unimp") };
}

pub unsafe fn write_stvec(addr: usize) {
    unsafe { core::arch::asm!("csrw stvec, {0}", in(reg) addr, options(nostack, preserves_flags)) };
}
