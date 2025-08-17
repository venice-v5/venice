//! Raw bindings to MicroPython

#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_void};

use crate::{
    map::Dict,
    nlr::NlrBuf,
    obj::{Obj, ObjBase},
    qstr::QstrPool,
};

/// From: `py/nlr.h`
pub type nlr_jump_callback_fun_t = extern "C" fn(ctx: *mut c_void);

/// From: `py/nlr.h`
#[repr(C)]
pub struct nlr_jump_callback_node_t {
    pub prev: *const Self,
    pub fun: nlr_jump_callback_fun_t,
}

#[repr(C)]
pub struct mp_obj_tuple_t {
    pub base: ObjBase,
    pub len: usize,
    // mp_obj_t items[];
    pub items: (),
}

#[repr(C)]
pub struct mp_obj_exception_t {
    pub base: ObjBase,
    pub traceback_alloc: u16,
    pub traceback_len: u16,
    pub traceback_data: *mut usize,
    pub args: *mut mp_obj_tuple_t,
}

/// From: `py/mpstate.h`
#[repr(C)]
pub struct mp_state_thread_t {
    pub stack_top: *mut c_char,
    pub gc_lock_depth: u16,

    pub dict_locals: *mut Dict,
    pub dict_globals: *mut Dict,

    pub nlr_top: *mut NlrBuf,
    pub nlr_jump_callback_top: *mut nlr_jump_callback_node_t,

    // originally marked as volatile
    pub pending_exception: Obj,

    pub stop_iteration_arg: Obj,
}

/// From: `py/mpstate.h`
///
/// This is an incomplete binding; the omitted fields are currently not needed.
#[repr(C)]
pub struct mp_state_vm_t {
    pub last_pool: *mut QstrPool,
    pub mp_emergency_exception_obj: mp_obj_exception_t,
    pub mp_kbd_exception: mp_obj_exception_t,
    pub mp_loaded_modules_dict: Dict,
    pub dict_main: Dict,
    pub mp_module_builtins_override_dict: *mut Dict,
    // more unneeded fields
}

/// From: `py/mpstate.h`
///
/// This is an incomplete binding; the omitted fields are currently not needed.
#[repr(C)]
pub struct mp_state_ctx_t {
    pub thread: mp_state_thread_t,
    pub vm: mp_state_vm_t,
    // more unneeded fields
}
