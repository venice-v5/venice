use hashbrown::HashMap;

unsafe extern "C" {
    static __bytecode_ram_start: BytecodeTable;
}

pub const BYTECODE_TABLE_MAGIC: u32 = 0x675c3ed9;

lazy_static::lazy_static! {
    pub static ref MODULE_MAP: HashMap<&'static [u8], Bytecode<'static>> = {
        let mut hashmap = HashMap::new();
        let bytecode_table = BytecodeTable::get().unwrap_or_else(|magic| {
            panic!(
                "invalid bytecode table magic: should be 0x{BYTECODE_TABLE_MAGIC:08x}, got 0x{magic:08x}"
            );
        });

        for i in 0..bytecode_table.module_count {
            let module = bytecode_table.module(i).unwrap();
            hashmap.insert(module.name, module.bytecode);
        }

        hashmap
    };
}

#[repr(C)]
pub struct BytecodeTable {
    magic: u32,
    name_pool_offset: u32,
    bytecode_pool_offset: u32,
    module_count: u32,
    modules_ptrs_start: (),
}

mod sealed {
    #[derive(Clone, Copy)]
    #[repr(transparent)]
    pub struct Offset(u32);

    impl Offset {
        pub const fn inner(&self) -> usize {
            self.0 as usize
        }
    }
}

use sealed::Offset;

#[repr(C)]
pub struct ModulePtr {
    name_len: u32,
    name_offset: Offset,
    bytecode_len: u32,
    bytecode_offset: Offset,
}

#[derive(Clone, Copy)]
pub struct Bytecode<'a>(&'a [u8]);

pub struct Module<'a> {
    name: &'a [u8],
    bytecode: Bytecode<'a>,
}

impl BytecodeTable {
    pub fn get() -> Result<&'static Self, u32> {
        unsafe {
            if __bytecode_ram_start.magic == BYTECODE_TABLE_MAGIC {
                Ok(&__bytecode_ram_start)
            } else {
                Err(__bytecode_ram_start.magic)
            }
        }
    }

    fn name_ptr(&self, offset: Offset) -> *const u8 {
        unsafe {
            (&raw const __bytecode_ram_start as *const u8)
                .add(self.name_pool_offset as usize)
                .add(offset.inner())
        }
    }

    const fn bytecode_ptr(&self, offset: Offset) -> *const u8 {
        unsafe {
            (&raw const __bytecode_ram_start as *const u8)
                .add(self.bytecode_pool_offset as usize)
                .add(offset.inner())
        }
    }

    fn module(&self, i: u32) -> Option<Module> {
        if i >= self.module_count {
            return None;
        }

        let module =
            unsafe { &*(&raw const self.modules_ptrs_start as *const ModulePtr).add(i as usize) };

        let name = unsafe {
            core::slice::from_raw_parts(self.name_ptr(module.name_offset), module.name_len as usize)
        };

        let bytecode = unsafe {
            core::slice::from_raw_parts(
                self.bytecode_ptr(module.bytecode_offset),
                module.bytecode_len as usize,
            )
        };

        Some(Module {
            name,
            bytecode: Bytecode(bytecode),
        })
    }
}

impl Bytecode<'_> {
    pub const fn bytes(&self) -> &[u8] {
        self.0
    }
}
