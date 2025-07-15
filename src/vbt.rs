//! Venice Bytecode Table (VBT)

use hashbrown::HashMap;

unsafe extern "C" {
    static __bytecode_ram_start: BytecodeTable;
}

pub const BYTECODE_TABLE_MAGIC: u32 = 0x675c3ed9;

pub fn build_module_map() -> HashMap<&'static [u8], Bytecode<'static>> {
    let mut hashmap = HashMap::new();
    let bytecode_table = BytecodeTable::get().unwrap_or_else(|magic| {
            panic!(
                "invalid bytecode table magic: should be 0x{BYTECODE_TABLE_MAGIC:08x}, got 0x{magic:08x}"
            );
        });

    for module in bytecode_table.module_iter() {
        hashmap.insert(module.name, module.bytecode);
    }

    hashmap
}

#[repr(C)]
pub struct BytecodeTable {
    magic: u32,
    name_pool_offset: u32,
    bytecode_pool_offset: u32,
    module_count: u32,
    modules_ptrs_start: (),
}

#[repr(C)]
pub struct ModulePtr {
    name_len: u32,
    bytecode_len: u32,
}

#[derive(Clone, Copy)]
pub struct Bytecode<'a>(&'a [u8]);

pub struct Module<'a> {
    name: &'a [u8],
    bytecode: Bytecode<'a>,
}

pub struct ModuleIter<'a> {
    table: &'a BytecodeTable,
    current_name_offset: u32,
    current_bytecode_offset: u32,
    i: u32,
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

    const fn name_ptr(&self, offset: u32) -> *const u8 {
        unsafe {
            (&raw const __bytecode_ram_start as *const u8)
                .add(self.name_pool_offset as usize)
                .add(offset as usize)
        }
    }

    const fn bytecode_ptr(&self, offset: u32) -> *const u8 {
        unsafe {
            (&raw const __bytecode_ram_start as *const u8)
                .add(self.bytecode_pool_offset as usize)
                .add(offset as usize)
        }
    }

    const fn module_iter(&self) -> ModuleIter {
        ModuleIter {
            table: self,
            current_name_offset: 0,
            current_bytecode_offset: 0,
            i: 0,
        }
    }
}

impl<'a> Iterator for ModuleIter<'a> {
    type Item = Module<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.table.module_count {
            return None;
        }

        let module_ptr = unsafe {
            &*(&raw const self.table.modules_ptrs_start as *const ModulePtr).add(self.i as usize)
        };

        let name = unsafe {
            core::slice::from_raw_parts(
                self.table.name_ptr(self.current_name_offset),
                module_ptr.name_len as usize,
            )
        };

        let bytecode = unsafe {
            core::slice::from_raw_parts(
                self.table.bytecode_ptr(self.current_bytecode_offset),
                module_ptr.bytecode_len as usize,
            )
        };

        self.i += 1;
        self.current_name_offset += module_ptr.name_len;
        self.current_bytecode_offset += module_ptr.bytecode_len;

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
