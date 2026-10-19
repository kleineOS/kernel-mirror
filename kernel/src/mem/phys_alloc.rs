use core::slice;

use crate::{PAGE_SIZE, mem::MemRegion};

/// Allocate physical pages through a bitmap allocator
pub struct BitmapAlloc {
    region: &'static [u8],
}

impl BitmapAlloc {
    /// Create a bitmap memory allocator for this region
    ///
    /// # Safety
    /// - The allocator must have exclusive access to the memory region
    /// - The entire memory region must be valid and writable
    #[allow(
        clippy::cast_possible_truncation,
        reason = "u64 -> usize is safe, we use only 64bit"
    )]
    pub const unsafe fn new(mem_region: MemRegion) -> Self {
        let region: &'static [u8] = unsafe {
            let ptr = mem_region.address as *const u8;
            let len = mem_region.size as usize;

            slice::from_raw_parts(ptr, len)
        };

        // the first page will be for tracking the free pages

        Self { region }
    }

    pub const fn alloc(&self) -> &[u8; PAGE_SIZE] {
        todo!()
    }
}
