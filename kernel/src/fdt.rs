use core::{
    ffi::{CStr, FromBytesUntilNulError},
    ptr::NonNull,
};

pub const FDT_BEGIN_NODE: u32 = 0x01;
pub const FDT_END_NODE: u32 = 0x02;
pub const FDT_PROP: u32 = 0x03;
pub const FDT_NOP: u32 = 0x04;
pub const FDT_END: u32 = 0x09;

#[derive(Debug, Clone, Copy)]
pub enum FdtError {
    NullPtr,
    MagicInvalid {
        expected: BigEndianU32<true>,
        got: BigEndianU32<true>,
    },
}

impl core::fmt::Display for FdtError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FdtError::NullPtr => write!(f, "Null pointer was provided for parsing the device tree"),
            FdtError::MagicInvalid { expected, got } => write!(
                f,
                "Invalid Magic value was read: expected={expected:#x?},got={got:#x?}"
            ),
        }
    }
}

impl core::error::Error for FdtError {}

#[derive(Debug, Clone, Copy)]
pub enum ParseError {
    NoNullByte,
    StringOutOfBounds,
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ParseError::NoNullByte => write!(f, "A string in the tree has no null terminator"),
            ParseError::StringOutOfBounds => write!(f, "A string is located out of bounds"),
        }
    }
}

impl core::error::Error for ParseError {}

pub struct Fdt<'a> {
    /// All the bytes which represent a flattened device tree
    dtb_bytes: &'a [u8],
    /// The header is parsed once and stored here. The same data is stored in the dtb bytes,
    /// however, this structure can provide aligned access to all the inner fields regardless of the
    /// alignment of the original pointer
    header: FdtHeader,
}

impl<'a> Fdt<'a> {
    pub const VALID_MAGIC: BigEndianU32<true> = BigEndianU32::new(0xd00d_feed);

    /// # Safety
    /// `ptr` must reference a complete and readable device tree blob which is valid for `'a`.
    pub unsafe fn from_raw_ptr(ptr: *mut u8) -> Result<Self, FdtError> {
        let ptr = NonNull::new(ptr).ok_or(FdtError::NullPtr)?;

        let header = unsafe { ptr.as_ptr().cast::<FdtHeader>().read_unaligned() };

        if header.magic != Self::VALID_MAGIC {
            return Err(FdtError::MagicInvalid {
                expected: Self::VALID_MAGIC,
                got: header.magic,
            });
        }

        let dtb_size = header.totalsize.get() as usize;
        let dtb_bytes = unsafe { core::slice::from_raw_parts(ptr.as_ptr(), dtb_size) };

        Ok(Self { dtb_bytes, header })
    }

    pub fn header(&self) -> FdtHeader {
        self.header
    }

    pub fn structure(&self, path: &CStr, props: impl Fn(Prop)) -> Result<(), ParseError> {
        let mut bytes = FdtNodeReader::new(self.struct_bytes());

        let path = path.to_bytes();
        let path_depth = path.split(|&b| b == b'/').count();

        let mut current_depth = 0;
        let mut matched_depth = 0;

        let mut matched_node = c"HELLO";

        loop {
            match bytes.read_u32() {
                FDT_BEGIN_NODE => {
                    // contains the name and optionally the unit address
                    let name = bytes.read_cstr().ok_or(ParseError::NoNullByte)?;

                    // remove the unit address from the name
                    let name_bytes = name.to_bytes();
                    let node_name = name_bytes
                        .split(|&b| b == b'@')
                        .next()
                        .unwrap_or(name_bytes);

                    let expected = path.split(|&b| b == b'/').nth(current_depth);

                    if matched_depth == current_depth && expected == Some(node_name) {
                        matched_depth += 1;

                        if matched_depth == path_depth {
                            matched_node = name;
                        }
                    }

                    current_depth += 1;
                }
                FDT_END_NODE => {
                    current_depth -= 1;
                    if matched_depth > current_depth {
                        matched_depth = current_depth;
                    }
                }
                FDT_PROP => {
                    let len = bytes.read_u32();
                    let name_offset = bytes.read_u32();
                    let data = bytes.read_slice(len as usize);

                    let prop_name = self.get_name(name_offset)?;

                    if matched_depth == path_depth && current_depth == path_depth {
                        props(Prop {
                            node: matched_node,
                            name: prop_name,
                            data,
                        })
                    }
                }
                FDT_NOP => { /* intentionally do nothing */ }
                FDT_END => {
                    // let _ = writeln!(Console, "FDT_END");
                    break;
                }

                node => unreachable!("Node {node:#x?} is invalid"),
            }
        }

        Ok(())
    }

    fn struct_bytes(&self) -> &[u8] {
        let header = &self.header;

        let offset = header.off_dt_struct.get() as usize;
        let size = header.size_dt_struct.get() as usize;
        let end = offset + size;

        &self.dtb_bytes[offset..end]
    }

    fn get_name(&self, name_offset: u32) -> Result<&CStr, ParseError> {
        let header = &self.header;

        let offset = header.off_dt_strings.get() as usize;
        let size = header.size_dt_strings.get() as usize;
        let end = offset + size;

        let strings_blob = &self.dtb_bytes[offset..end];

        let name_offset = name_offset as usize;
        let name_bytes = strings_blob
            .get(name_offset..)
            .ok_or(ParseError::StringOutOfBounds)?;

        CStr::from_bytes_until_nul(name_bytes)
            .map_err(|_: FromBytesUntilNulError| ParseError::NoNullByte)
    }
}

pub struct Prop<'a> {
    pub node: &'a CStr,
    pub name: &'a CStr,
    pub data: &'a [u8],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FdtHeader {
    /// This field shall contain the value 0xd00dfeed (big-endian).
    pub magic: BigEndianU32<true>,
    /// This field shall contain the total size in bytes of the devicetree data structure. This size
    /// shall encompass all sections of the structure: the header, the memory reservation block,
    /// structure block and strings block, as well as any free space gaps between the blocks and
    /// after the final block.
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

pub struct FdtNodeReader<'a> {
    inner: &'a [u8],
    position: usize,
}

impl<'a> FdtNodeReader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self {
            inner: buf,
            position: 0,
        }
    }

    pub fn read_slice(&mut self, size: usize) -> &[u8] {
        let start = self.position;
        let end = start + size;

        let slice = &self.inner[start..end];

        let padded_len = size.next_multiple_of(4);
        self.position += padded_len;

        slice
    }

    pub fn read_u32(&mut self) -> u32 {
        const SIZE: usize = 4;

        let bytes = &self.inner[self.position..];

        let bytes: [u8; SIZE] = core::array::from_fn(|i| bytes[i]);
        let number = u32::from_be_bytes(bytes);

        self.position += SIZE;

        number
    }

    pub fn read_cstr(&mut self) -> Option<&'a CStr> {
        let bytes = &self.inner[self.position..];

        let string = CStr::from_bytes_until_nul(bytes).ok()?;

        let len = string.to_bytes_with_nul().len();
        let padded_len = len.next_multiple_of(4);

        self.position += padded_len;

        Some(string)
    }
}

/// Represent a big endian u32 value
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BigEndianU32<const HEX: bool = false> {
    inner: u32,
}

impl<const HEX: bool> BigEndianU32<HEX> {
    pub const fn new(value: u32) -> Self {
        #[cfg(target_endian = "little")]
        Self {
            inner: value.swap_bytes(),
        }
    }

    pub fn get(&self) -> u32 {
        // we know that our targets will always be le, but it is still better to code that
        // assumption into the software
        #[cfg(target_endian = "little")]
        self.inner.swap_bytes()
    }
}

impl<const HEX: bool> core::fmt::Display for BigEndianU32<HEX> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if HEX {
            core::fmt::LowerHex::fmt(&self.get(), f)
        } else {
            core::fmt::Display::fmt(&self.get(), f)
        }
    }
}

impl<const HEX: bool> core::fmt::Debug for BigEndianU32<HEX> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if HEX {
            core::fmt::LowerHex::fmt(&self.get(), f)
        } else {
            core::fmt::Debug::fmt(&self.get(), f)
        }
    }
}
