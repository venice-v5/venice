#![no_std]

mod gc;
mod init;
mod module;
mod nlr;
mod raw;
mod reentrance;
mod state;

pub(crate) mod print;

pub mod map;
pub mod obj;
pub mod qstr;
pub mod vstr;

use venice_program_table::Vpt;

pub struct MicroPython(());

impl MicroPython {
    pub fn add_vpt(&mut self, vpt: Vpt<'static>) {
        for program in vpt.program_iter() {
            self.global_data_mut()
                .module_map
                .insert(program.name(), program.payload());
        }
    }
}
