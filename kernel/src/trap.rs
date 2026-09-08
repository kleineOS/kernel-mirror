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

impl core::fmt::Display for TrapFrame {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f)?;
        writeln!(f, "t0={:#018x}  a0={:#018x}  s0={:#018x}   s8={:#018x}", self.t0, self.a0, self.s0, self.s8)?;
        writeln!(f, "t1={:#018x}  a1={:#018x}  s1={:#018x}   s9={:#018x}", self.t1, self.a1, self.s1, self.s9)?;
        writeln!(f, "t2={:#018x}  a2={:#018x}  s2={:#018x}  s10={:#018x}", self.t2, self.a2, self.s2, self.s10)?;
        writeln!(f, "t3={:#018x}  a3={:#018x}  s3={:#018x}  s11={:#018x}", self.t3, self.a3, self.s3, self.s11)?;
        writeln!(f, "t4={:#018x}  a4={:#018x}  s4={:#018x}   ra={:#018x}", self.t4, self.a4, self.s4, self.ra)?;
        writeln!(f, "t5={:#018x}  a5={:#018x}  s5={:#018x}   sp={:#018x}", self.t5, self.a5, self.s5, self.sp)?;
        writeln!(f, "t6={:#018x}  a6={:#018x}  s6={:#018x}   gp={:#018x}", self.t6, self.a6, self.s6, self.gp)?;
        writeln!(f, "tp={:#018x}  a7={:#018x}  s7={:#018x}",              self.tp, self.a7, self.s7)?;

        Ok(())
    }
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
        "Kernel trap is not yet implemented. int={} cause={} sepc={sepc:#x?} stval={stval:#x?} {}",
        int,
        cause,
        frame
    );
}
