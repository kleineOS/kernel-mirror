unsafe extern "C" {
    fn ktrapvec();
}

pub fn set_trap() {
    let addr = ktrapvec as *const () as usize;
    unsafe { crate::arch::write_stvec(addr) };
}

#[repr(C)]
#[derive(Debug)]
pub struct TrapFrame {
    pub ra: usize,
    pub sp: usize,
    pub gp: usize,
    pub tp: usize,
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub s0: usize,
    pub s1: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
}

#[unsafe(no_mangle)]
extern "C" fn kerneltrap(frame: &mut TrapFrame) {
    let scause = unsafe {
        let scause: usize;
        core::arch::asm!("csrr {}, scause", out(reg) scause, options(nomem, nostack));
        scause
    };

    let sepc = unsafe {
        let sepc: usize;
        core::arch::asm!("csrr {}, sepc", out(reg) sepc, options(nomem, nostack));
        sepc
    };

    let stval = unsafe {
        let stval: usize;
        core::arch::asm!("csrr {}, stval", out(reg) stval, options(nomem, nostack));
        stval
    };

    let int = ((scause >> 63) & 1) == 1;
    let cause = scause & ((1 << 63) - 1);

    todo!(
        "Kernel trap is not yet implemented. int={} cause={} sepc={sepc:#x?} stval={stval:#x?} {:#x?}",
        int,
        cause,
        frame
    );
}
