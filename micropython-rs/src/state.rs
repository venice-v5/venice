use core::cell::UnsafeCell;

use hashbrown::HashMap;
use venice_program_table::Vpt;

use crate::{MicroPython, map::Dict, raw::mp_state_ctx_t};

impl MicroPython {
    pub fn module_map(&self) -> &HashMap<&'static [u8], &'static [u8]> {
        &self.module_map
    }

    pub fn add_vpt(&mut self, vpt: Vpt<'static>) {
        for program in vpt.program_iter() {
            self.module_map.insert(program.name(), program.payload());
        }
    }

    pub(crate) fn state_ctx_raw(&self) -> *mut mp_state_ctx_t {
        unsafe extern "C" {
            /// From: `py/mp_state.h`
            ///
            /// Currently, MicroPython threads are disabled, so this is always the active [`StateCtx`].
            static mp_state_ctx: UnsafeCell<mp_state_ctx_t>;
        }

        unsafe { mp_state_ctx.get() }
    }

    pub fn state_ctx(&self) -> &mp_state_ctx_t {
        unsafe { &*self.state_ctx_raw() }
    }

    pub fn globals(&self) -> &Dict {
        unsafe { &*self.state_ctx().thread.dict_globals }
    }

    pub fn locals(&self) -> &Dict {
        unsafe { &*self.state_ctx().thread.dict_locals }
    }

    pub unsafe fn set_globals(&mut self, dict: *mut Dict) {
        unsafe { (*self.state_ctx_raw()).thread.dict_globals = dict }
    }

    pub unsafe fn set_locals(&mut self, dict: *mut Dict) {
        unsafe { (*self.state_ctx_raw()).thread.dict_locals = dict }
    }
}
