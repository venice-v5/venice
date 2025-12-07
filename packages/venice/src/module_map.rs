use std::{collections::HashMap, sync::OnceLock};

use venice_program_table::{Program, Vpt};

type ModuleMap = HashMap<&'static [u8], Program<'static>>;

pub static MODULE_MAP: OnceLock<ModuleMap> = OnceLock::new();

pub fn init_module_map(vpt: Vpt<'static>) -> Result<(), ModuleMap> {
    let mut map: ModuleMap = HashMap::new();
    map.insert(
        b"typing",
        Program {
            name: b"typing",
            payload: include_bytes!("modtyping/typing.mpy"),
            flags: 0,
        }
    );
    for program in vpt.program_iter() {
        map.insert(
            program.name(),
            program,
        );
    }
    MODULE_MAP.set(map)
}
