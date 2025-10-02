use core::ops::Deref;

use bitflags::bitflags;
use hashbrown::HashMap;
use lazy_static::lazy_static;
use spin::{Mutex, MutexGuard};
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

lazy_static! {
    static ref MODULE_MAP: Mutex<ModuleMap> = Mutex::new(HashMap::new());
}

pub struct ModuleMapLock<'a> {
    guard: MutexGuard<'a, ModuleMap>,
}

impl Deref for ModuleMapLock<'_> {
    type Target = ModuleMap;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

pub fn lock_module_map<'a>() -> ModuleMapLock<'a> {
    ModuleMapLock {
        guard: MODULE_MAP.lock(),
    }
}

pub fn add_vpt(vpt: Vpt<'static>) {
    let mut lock = MODULE_MAP.lock();
    for program in vpt.program_iter() {
        lock.insert(
            program.name(),
            VptModule {
                data: program.payload(),
            },
        );
    }
}
