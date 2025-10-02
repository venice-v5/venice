// Safety: We can never create references to global statics used by MicroPython. So, the only way
// to access them is through pointers, or carefully writing functions that with specific purposes
// like get x, set y, that never call into MicroPython.

use crate::{init::InitToken, map::Dict, raw::mp_state_ctx_t};

fn state_ctx_raw() -> *mut mp_state_ctx_t {
    unsafe extern "C" {
        /// From: `py/mp_state.h`
        ///
        /// Currently, MicroPython threads are disabled, so this is always the active [`StateCtx`].
        static mut mp_state_ctx: mp_state_ctx_t;
    }

    &raw mut mp_state_ctx
}

pub fn globals(_: InitToken) -> *mut Dict {
    unsafe { (*state_ctx_raw()).thread.dict_globals }
}

pub fn locals(_: InitToken) -> *mut Dict {
    unsafe { (*state_ctx_raw()).thread.dict_locals }
}

pub fn loaded_modules(_: InitToken) -> *mut Dict {
    unsafe { &raw mut (*state_ctx_raw()).vm.mp_loaded_modules_dict }
}

// TODO: place safety invariants

pub unsafe fn set_globals(_: InitToken, dict: *mut Dict) {
    unsafe { (*state_ctx_raw()).thread.dict_globals = dict };
}

pub unsafe fn set_locals(_: InitToken, dict: *mut Dict) {
    unsafe { (*state_ctx_raw()).thread.dict_locals = dict };
}

pub fn stack_top(_: InitToken) -> *mut u8 {
    unsafe { (*state_ctx_raw()).thread.stack_top }
}
