#![allow(unused)]

//! RISC-V architecture related code. Currently incomplete. Intention is to provide high level APIs
//! to access RISC-V CSRs, instructuions, the RISC-V SBI etc.

use core::arch::asm;

pub mod sbi;
pub mod trap;

#[macro_export]
macro_rules! include_asm {
    ($file:expr $(,)?) => {
        core::arch::global_asm!(include_str!($file));
    };
}

pub fn wfi() {
    unsafe { asm!("wfi", options(nomem, nostack, preserves_flags)) };
}

pub fn unimp() {
    unsafe { asm!("unimp") };
}

pub unsafe fn write_stvec(addr: usize) {
    unsafe { asm!("csrw stvec, {0}", in(reg) addr, options(nostack, preserves_flags)) };
}

/// `TIME` instruction wrapper
pub fn time() -> usize {
    unsafe {
        let time: usize;
        asm!("csrr {}, time", out(reg) time, options(nomem, nostack));
        time
    }
}

pub const SIE_STIE: usize = 5;

pub fn sie_set_bit(bit: usize, value: bool) {
    let mask = 1_usize << bit;

    unsafe {
        if value {
            asm!("csrs sie, {mask}", mask = in(reg) mask, options(nostack));
        } else {
            asm!("csrc sie, {mask}", mask = in(reg) mask, options(nostack));
        }
    }
}

pub const SSTATUS_SIE: usize = 1;

pub fn sstatus_set_bit(bit: usize, value: bool) {
    let mask = 1_usize << bit;

    unsafe {
        if value {
            asm!("csrs sstatus, {mask}", mask = in(reg) mask, options(nostack));
        } else {
            asm!("csrc sstatus, {mask}", mask = in(reg) mask, options(nostack));
        }
    }
}
