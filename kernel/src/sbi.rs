use core::fmt::Write;

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

    // pub fn dbcn_console_byte(data: u8) {
    //     const EID: usize = 0x4442434E;
    //     const FID: usize = 0x2;
    //
    //     let sbi = Self {
    //         a0: data as usize,
    //         a1: 0,
    //         a2: 0,
    //         a3: 0,
    //         a4: 0,
    //         a5: 0,
    //         fid: FID,
    //         eid: EID,
    //     };
    //
    //     sbi.ecall();
    // }

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

pub struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Sbi::dbcn_console_write(s);
        Ok(())
    }
}
