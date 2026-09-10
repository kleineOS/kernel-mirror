use core::{
    ffi::{CStr, FromBytesUntilNulError},
    ptr::NonNull,
    str::Utf8Error,
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
            Self::NullPtr => write!(f, "Null pointer was provided for parsing the device tree"),
            Self::MagicInvalid { expected, got } => write!(
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
    InvalidUtf8,
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoNullByte => write!(f, "A string in the tree has no null terminator"),
            Self::StringOutOfBounds => write!(f, "A string is located out of bounds"),
            Self::InvalidUtf8 => write!(f, "Invalid UTF-8 characters when parsing string"),
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

impl Fdt<'_> {
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

    pub fn root(&self) -> Node<'_> {
        let blob = self.struct_bytes();
        Node::new(blob)
    }

    pub fn get_name(&self, name_offset: &NameOffset) -> Result<&str, ParseError> {
        let header = &self.header;

        let offset = header.off_dt_strings.get() as usize;
        let size = header.size_dt_strings.get() as usize;
        let end = offset + size;

        let strings_blob = &self.dtb_bytes[offset..end];

        let name_offset = name_offset.0 as usize;
        let name_bytes = strings_blob
            .get(name_offset..)
            .ok_or(ParseError::StringOutOfBounds)?;

        CStr::from_bytes_until_nul(name_bytes)
            .map_err(|_: FromBytesUntilNulError| ParseError::NoNullByte)?
            .to_str()
            .map_err(|_: Utf8Error| ParseError::InvalidUtf8)
    }

    fn struct_bytes(&self) -> &[u8] {
        let header = &self.header;

        let offset = header.off_dt_struct.get() as usize;
        let size = header.size_dt_struct.get() as usize;
        let end = offset + size;

        &self.dtb_bytes[offset..end]
    }
}

#[derive(Clone, Copy)]
pub struct Node<'a> {
    blob: &'a [u8],
    offset: usize,
}

impl<'a> Node<'a> {
    pub const fn new(blob: &'a [u8]) -> Self {
        Self { blob, offset: 0 }
    }

    const fn copy_with_offset(&self, offset: usize) -> Self {
        Self {
            blob: self.blob,
            offset,
        }
    }

    pub fn properties(&self) -> PropertyIter<'_> {
        let reader = self.as_node_reader();
        PropertyIter::new(reader)
    }

    /// Find a node one level deeper in relation to the current root node
    pub fn find(&self, target: &str) -> Option<Node<'_>> {
        let target = target.trim_matches('/');

        // we always assume our current node to be the root, and the node query from the user to be
        // relative to it. If the query is for the root node itself, then the current node is copied
        if target.is_empty() {
            return Some(*self);
        }

        let mut reader = self.as_node_reader();
        let mut depth = 0;

        loop {
            let offset = reader.position;

            match reader.read_u32() {
                FDT_BEGIN_NODE => {
                    let name = reader.read_cstr_as_str()?;
                    // the name will be in node-name@unit-address format
                    // we only need to match against the name
                    let name = name.split_once('@').map_or(name, |(name, _)| name);

                    if depth == 1 && name == target {
                        return Some(self.copy_with_offset(offset));
                    }

                    depth += 1;
                }

                FDT_PROP => {
                    // skip through the FDT_PROP data so it does not interfere with parsing
                    let len = reader.read_u32();
                    let _ = reader.read_u32();
                    let _ = reader.read_slice(len as usize);
                }

                FDT_END_NODE => match depth {
                    0 => return None,
                    _ => depth -= 1,
                },
                FDT_END => return None,

                FDT_NOP => (),

                unknown => unreachable!("unreachable branch in fdt parsing: {unknown:#x}"),
            }
        }
    }

    fn as_node_reader(&self) -> FdtNodeReader<'_> {
        FdtNodeReader::new(&self.blob[self.offset..])
    }
}

#[derive(Debug)]
pub struct NameOffset(u32);

#[derive(Debug)]
pub struct Property<'a> {
    pub data: &'a [u8],
    pub name_offset: NameOffset,
}

pub struct PropertyIter<'a> {
    reader: FdtNodeReader<'a>,
    depth: i32,
}

impl<'a> PropertyIter<'a> {
    pub const fn new(reader: FdtNodeReader<'a>) -> Self {
        Self { reader, depth: -1 }
    }
}

impl<'a> Iterator for PropertyIter<'a> {
    type Item = Property<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.reader.read_u32() {
                FDT_BEGIN_NODE => {
                    let _ = self.reader.read_cstr();
                    self.depth += 1;
                }

                FDT_END_NODE => {
                    if self.depth == 0 {
                        break;
                    }
                    self.depth -= 1;
                }

                FDT_PROP if self.depth == 0 => {
                    let len = self.reader.read_u32();
                    let name_offset = self.reader.read_u32();
                    let data = self.reader.read_slice(len as usize);

                    let property = Property {
                        data,
                        name_offset: NameOffset(name_offset),
                    };

                    return Some(property);
                }

                FDT_PROP => {
                    let len = self.reader.read_u32();
                    let _ = self.reader.read_u32();
                    let _ = self.reader.read_slice(len as usize);
                }

                FDT_NOP => (),
                FDT_END => break,

                _ => todo!(),
            }
        }

        None
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FdtHeader {
    /// This field shall contain the value `0xd00d_feed` (big-endian).
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
    pub const fn new(buf: &'a [u8]) -> Self {
        Self {
            inner: buf,
            position: 0,
        }
    }

    pub fn read_slice(&mut self, size: usize) -> &'a [u8] {
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

    pub fn read_cstr_as_str(&mut self) -> Option<&'a str> {
        let bytes = &self.inner[self.position..];

        let string = CStr::from_bytes_until_nul(bytes).ok()?;

        let len = string.to_bytes_with_nul().len();
        let padded_len = len.next_multiple_of(4);

        self.position += padded_len;

        let str_slice = string.to_str().ok()?;

        Some(str_slice)
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

    pub const fn get(self) -> u32 {
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
