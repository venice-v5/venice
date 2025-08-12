#![no_std]

mod gc;
mod module;
mod raw;
mod reentrance;
mod state;

pub mod obj;
pub mod qstr;
pub mod vstr;

use core::sync::atomic::{AtomicBool, Ordering};

use hashbrown::HashMap;
use venice_program_table::Vpt;

use crate::{
    raw::{mp_deinit, mp_init, mp_stack_ctrl_init},
    state::GlobalData,
};

pub static MICROPYTHON_CREATED: AtomicBool = AtomicBool::new(false);

pub struct MicroPython(());

unsafe extern "C" {
    fn __libc_init_array();
    fn __libc_fini_array();
}

impl MicroPython {
    pub fn new() -> Option<Self> {
        if let Err(_) =
            MICROPYTHON_CREATED.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        {
            return None;
        }

        unsafe {
            __libc_init_array();

            mp_stack_ctrl_init();
            mp_init();
        }

        let mut this = Self(());

        unsafe {
            this.set_global_data(GlobalData {
                module_map: HashMap::new(),
                gc_init: false,
            });
        }

        Some(this)
    }

    pub fn add_vpt(&mut self, vpt: Vpt<'static>) {
        for program in vpt.program_iter() {
            self.global_data_mut()
                .module_map
                .insert(program.name(), program.payload());
        }
    }
}

impl Drop for MicroPython {
    fn drop(&mut self) {
        unsafe {
            mp_deinit();
            __libc_fini_array();
        }
    }
}
