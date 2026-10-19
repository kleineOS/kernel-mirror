#[derive(Debug, Clone, Copy)]
pub struct MemRegion {
    pub address: u64,
    pub size: u64,
}

impl MemRegion {
    pub const fn new(address: u64, size: u64) -> Self {
        Self { address, size }
    }

    pub fn kernel_region() -> Self {
        unsafe extern "C" {
            static __kernel_start: u8;
            static __kernel_end: u8;
        }

        let kernel_start = (&raw const __kernel_start) as u64;
        let kernel_end = (&raw const __kernel_end) as u64;
        let kernel_size = kernel_end - kernel_start;

        Self::new(kernel_start, kernel_size)
    }

    pub fn find_free_region(&self, size: u64, alignment: u64, exclusions: &[Self]) -> Option<Self> {
        if alignment == 0 {
            return None;
        }

        if !alignment.is_power_of_two() {
            return None; // peraps I can only accept Alignment::PageAligned or Alignment::None
        }

        let mut start = (self.address + alignment - 1) & !(alignment - 1);

        loop {
            if start + size > self.address + self.size {
                return None;
            }

            let overlaps = exclusions.iter().any(|e| e.is_within(start, size));
            if !overlaps {
                return Some(Self::new(start, size));
            }

            start += alignment;
        }
    }

    const fn is_within(&self, start: u64, size: u64) -> bool {
        let end = start + size;
        let region_end = self.address + self.size;

        (self.address <= start && start < region_end) || (self.address <= end && end < region_end)
    }
}
