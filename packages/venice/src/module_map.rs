use std::{collections::HashMap, sync::OnceLock};

use bitflags::bitflags;
use venice_program_table::Vpt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VptModule<'a> {
    data: &'a [u8],
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct VptModuleFlags: u8 {
        const IS_MODULE = 0b01;
        const IS_PACKAGE = 0b10;
    }
}

impl<'a> VptModule<'a> {
    pub const fn flags(&self) -> VptModuleFlags {
        VptModuleFlags::from_bits(self.data[0]).expect("malformed VPT: unknown module flags set")
    }

    pub fn payload(&self) -> &'a [u8] {
        &self.data[1..]
    }
}

type ModuleMap = HashMap<&'static [u8], VptModule<'static>>;

pub static MODULE_MAP: OnceLock<ModuleMap> = OnceLock::new();

pub fn init_module_map(vpt: Vpt<'static>) -> Result<(), ModuleMap> {
    let mut map = HashMap::new();
    for program in vpt.program_iter() {
        map.insert(
            program.name(),
            VptModule {
                data: program.payload(),
            },
        );
    }
    MODULE_MAP.set(map)
}
