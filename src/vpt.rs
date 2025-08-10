use hashbrown::HashMap;
use lazy_static::lazy_static;
use venice_program_table::Vpt;

unsafe extern "C" {
    static __bytecode_ram_start: u8;
}

#[derive(Clone, Copy)]
pub struct Bytecode(&'static [u8]);

impl Bytecode {
    pub fn bytes(&self) -> &'static [u8] {
        self.0
    }
}

// TODO: pick another ID
pub const VENDOR_ID: u32 = 0x11235813;

lazy_static! {
    pub static ref VPT: Vpt<'static> = unsafe {
        Vpt::from_ptr(&raw const __bytecode_ram_start, VENDOR_ID).expect("invalid VPT was uploaded")
    };
}

pub fn build_module_map() -> HashMap<&'static [u8], Bytecode> {
    let mut hashmap = HashMap::new();
    for program in VPT.program_iter() {
        hashmap.insert(program.name(), Bytecode(program.payload()));
    }

    hashmap
}
