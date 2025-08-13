use core::cell::UnsafeCell;

use hashbrown::HashMap;
use venice_program_table::Vpt;

use crate::{MicroPython, map::Dict, raw::mp_state_ctx_t};

unsafe extern "C" {
    /// From: `py/mp_state.h`
    ///
    /// Currently, MicroPython threads are disabled, so this is always the active [`StateCtx`].
    static mp_state_ctx: UnsafeCell<mp_state_ctx_t>;
}

impl MicroPython {
    pub fn module_map(&self) -> &HashMap<&'static [u8], &'static [u8]> {
        &self.module_map
    }

    pub fn add_vpt(&mut self, vpt: Vpt<'static>) {
        for program in vpt.program_iter() {
            self.module_map.insert(program.name(), program.payload());
        }
    }

    pub fn state_ctx(&self) -> &mp_state_ctx_t {
        unsafe { &*mp_state_ctx.get() }
    }

    pub fn globals_dict(&self) -> &Dict {
        // SAFETY: As long as `state_ctx` is not modified by the program, safety is ensured by
        // MicroPython.
        unsafe { &*self.state_ctx().thread.dict_globals }
    }
}
