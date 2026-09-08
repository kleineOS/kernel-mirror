use core::{ffi::CStr, fmt::Write, ptr::NonNull};

use crate::sbi::Console;

pub const FDT_BEGIN_NODE: u32 = 0x01;
pub const FDT_END_NODE: u32 = 0x02;
pub const FDT_PROP: u32 = 0x03;
pub const FDT_NOP: u32 = 0x04;
pub const FDT_END: u32 = 0x09;

pub struct FdtPtr {
    ptr: NonNull<u8>,
}

impl FdtPtr {
    pub const VALID_MAGIC: u32 = 0xd00dfeed;

    pub fn from_raw_ptr(ptr: *mut u8) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr })
    }

    pub fn get_header(&self) -> &FdtHeader {
        unsafe { &*self.ptr.as_ptr().cast::<FdtHeader>() }
    }

    pub fn get_magic(&self) -> u32 {
        self.get_header().magic.get()
    }

    pub fn structure(&self) {
        let header = self.get_header();
        let offset = header.off_dt_struct.get() as usize;

        let mut node_start: NonNull<BigEndianU32> = unsafe { self.ptr.add(offset).cast() };

        loop {
            let node = unsafe { node_start.read() };

            match node.get() {
                FDT_BEGIN_NODE => {
                    let name = unsafe {
                        let ptr = node_start.add(1).cast::<u8>();
                        CStr::from_ptr(ptr.as_ptr())
                    };

                    let name_len = name.to_bytes_with_nul().len();

                    // pad upwards, to the next 4 byte boundry
                    let padded_name_len = (name_len + 3) & !3;
                    node_start = unsafe { node_start.add(1 + padded_name_len / 4) };

                    let _ = writeln!(Console, "FDT_BEGIN_NODE({name:?})");
                }

                FDT_END_NODE => {
                    let _ = writeln!(Console, "FDT_END_NODE");
                    node_start = unsafe { node_start.add(1) };
                }

                FDT_PROP => {
                    let len = unsafe { node_start.add(1).read().get() };

                    let name_offset = unsafe { node_start.add(2).read().get() };
                    let str = header.get_string(self.ptr, name_offset);

                    let data = unsafe {
                        let data_ptr = node_start.add(3).cast::<u8>();
                        core::slice::from_raw_parts(data_ptr.as_ptr(), len as usize)
                    };

                    let _ = writeln!(Console, "FDT_PROP({str:?}:len={len}:data={data:?})");

                    let padded_len = ((len + 3) & !3) as usize;
                    node_start = unsafe { node_start.add(3 + padded_len / 4) };
                }

                FDT_NOP => {
                    let _ = writeln!(Console, "FDT_NOP");
                    node_start = unsafe { node_start.add(1) };
                }

                FDT_END => {
                    let _ = writeln!(Console, "FDT_END");
                    break;
                }

                node => unreachable!("{node:#x?}"),
            }
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct FdtHeader {
    /// This field shall contain the value 0xd00dfeed (big-endian).
    pub magic: BigEndianU32<true>,
    // This field shall contain the total size in bytes of the devicetree data structure. This size
    // shall encompass all sections of the structure: the header, the memory reservation block,
    // structure block and strings block, as well as any free space gaps between the blocks and
    // after the final block.
    pub totalsize: BigEndianU32,
    /// This field shall contain the offset in bytes of the structure block from the beginning of
    /// the header.
    pub off_dt_struct: BigEndianU32,
    /// This field shall contain the offset in bytes of the strings block from the beginning of the
    /// header.
    pub off_dt_strings: BigEndianU32,
    /// This field shall contain the offset in bytes of the memory reservation block from the
    /// beginning of the header.
    pub off_mem_rsvmap: BigEndianU32,
    /// This field shall contain the version of the devicetree data structure.
    pub version: BigEndianU32,
    /// This field shall contain the lowest version of the devicetree data structure with which the
    /// version used is backwards compatible.
    pub last_comp_version: BigEndianU32,
    /// This field shall contain the physical ID of the system's boot CPU. IT shall be identical to
    /// the physical ID given in the `reg` property of tat CPU node within the devicetree.
    pub boot_cpuid_phys: BigEndianU32,
    /// This field shall contain the length in bytes of the strings block section of the devicetree
    /// blob.
    pub size_dt_strings: BigEndianU32,
    /// This field shall contain the length in bytes of the structure block section of the
    /// devicetree blob.
    pub size_dt_struct: BigEndianU32,
}

impl FdtHeader {
    pub fn get_string(&self, base_ptr: NonNull<u8>, offset: u32) -> Option<&CStr> {
        let str_offset = self.off_dt_strings.get() + offset;

        // if str_offset > self.size_dt_strings.get() {
        //     return None;
        // }

        let str = unsafe {
            let str_ptr = base_ptr.add(str_offset as usize);
            CStr::from_ptr(str_ptr.as_ptr())
        };

        Some(str)
    }
}

#[repr(transparent)]
pub struct BigEndianU32<const HEX: bool = false> {
    inner: u32,
}

impl<const HEX: bool> BigEndianU32<HEX> {
    pub fn get(&self) -> u32 {
        self.inner.swap_bytes()
    }
}

impl<const HEX: bool> core::fmt::Display for BigEndianU32<HEX> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match HEX {
            true => core::fmt::LowerHex::fmt(&self.get(), f),
            false => core::fmt::Display::fmt(&self.get(), f),
        }
    }
}

impl<const HEX: bool> core::fmt::Debug for BigEndianU32<HEX> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match HEX {
            true => core::fmt::LowerHex::fmt(&self.get(), f),
            false => core::fmt::Debug::fmt(&self.get(), f),
        }
    }
}
