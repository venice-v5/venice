#![no_std]

mod global_data;
mod raw;
mod reentrance;

pub mod obj;
pub mod qstr;
pub mod singleton;

use core::sync::atomic::{AtomicBool, Ordering};

use hashbrown::HashMap;

use crate::{
    global_data::GlobalData,
    raw::{mp_deinit, mp_init, mp_stack_ctrl_init},
};

pub static MICROPYTHON_CREATED: AtomicBool = AtomicBool::new(false);

pub struct MicroPython(());

unsafe extern "C" {
    fn __libc_init_array();
    fn __libc_fini_array();
}

impl MicroPython {
    pub fn new(module_map: HashMap<&'static [u8], &'static [u8]>) -> Option<Self> {
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
            this.set_global_data(GlobalData { module_map });
        }

        Some(this)
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
