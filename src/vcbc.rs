//! Venice Bytecode Format (VCBC)

unsafe extern "C" {
    static __bytecode_ram_start: BytecodeTable;
}

pub const BYTECODE_TABLE_MAGIC: u32 = 0x675c3ed9;

#[derive(Debug)]
#[repr(C)]
pub struct BytecodeTable {
    magic: u32,
    module_count: u32,
    /// Marker/dummy
    module_ptrs_start: (),
}

#[derive(Debug)]
#[repr(C)]
pub struct BytecodePtr {
    /// Offset from the start of the [`BytecodeTable`]
    offset: u32,
    len: u32,
}

pub struct Bytecode(&'static [u8]);

impl BytecodeTable {
    pub fn get() -> Result<&'static Self, u32> {
        let ret = unsafe { &__bytecode_ram_start };
        if ret.magic == BYTECODE_TABLE_MAGIC {
            Ok(ret)
        } else {
            Err(ret.magic)
        }
    }

    pub fn module(&self, index: u32) -> Option<Bytecode> {
        if index >= self.module_count {
            return None;
        }

        unsafe {
            let bytecode_ptr =
                &*(&raw const self.module_ptrs_start as *const BytecodePtr).add(index as usize);
            let bytecode_start =
                (self as *const Self as *const u8).add(bytecode_ptr.offset as usize);
            Some(Bytecode(core::slice::from_raw_parts(
                bytecode_start,
                bytecode_ptr.len as usize,
            )))
        }
    }
}

impl Bytecode {
    pub fn data(&self) -> &[u8] {
        self.0
    }
}
