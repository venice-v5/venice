//! Raw bindings to MicroPython

#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_uint, c_void};

pub type mp_obj_t = super::obj::Obj;

/// From: `py/emitglue.h`
pub type ProtoFun = *const c_void;

/// From: `py/qstr.h`
pub type qstr = usize;

/// From: `py/qstr.h`
pub type qstr_short_t = u16;

/// From: `py/mpprint.h`
pub type mp_print_strn_t = extern "C" fn(data: *mut c_void, str: *const c_char, len: usize);

/// From: `py/misc.h`
pub type mp_rom_error_text_t = *const c_char;

/// From: `py/obj.h`
#[repr(C)]
pub struct mp_map_elem_t {
    pub key: mp_obj_t,
    pub value: mp_obj_t,
}

/// From: `py/obj.h`
#[repr(C)]
pub struct mp_map_t {
    // this is actually 4 bitfields
    pub used: usize,
    pub alloc: usize,
    pub table: *mut mp_map_elem_t,
}

/// From: `py/obj.h`
#[repr(C)]
pub enum mp_map_lookup_kind_t {
    MP_MAP_LOOKUP = 0,
    MP_MAP_LOOKUP_ADD_IF_NOT_FOUND = 1,
    MP_MAP_LOOKUP_REMOVE_IF_FOUND = 2,
    MP_MAP_LOOKUP_ADD_IF_NOT_FOUND_OR_REMOVE_IF_FOUND = 3,
}

/// From: `py/obj.h`
///
/// This struct actually corresponds to `mp_obj_full_type_t`.
#[repr(C)]
pub struct mp_obj_type_t {
    pub base: mp_obj_base_t,

    pub flags: u16,
    pub name: u16,

    pub slot_index_make_new: u8,
    pub slot_index_print: u8,
    pub slot_index_call: u8,
    pub slot_index_unary_op: u8,
    pub slot_index_binary_op: u8,
    pub slot_index_attr: u8,
    pub slot_index_subscr: u8,
    pub slot_index_iter: u8,
    pub slot_index_buffer: u8,
    pub slot_index_protocol: u8,
    pub slot_index_parent: u8,
    pub slot_index_locals_dict: u8,

    pub slots: [*const c_void; 12],
}

/// From: `py/obj.h`
#[repr(C)]
pub struct mp_obj_base_t {
    pub r#type: *const mp_obj_type_t,
}

/// From: `py/obj.h`
#[repr(C)]
pub struct mp_obj_dict_t {
    pub base: mp_obj_base_t,
    pub map: mp_map_t,
}

/// From: `py/obj.h`
#[repr(C)]
pub struct mp_obj_module_t {
    pub base: mp_obj_base_t,
    pub globals: *mut mp_obj_dict_t,
}

/// From: `py/bc.h`
#[repr(C)]
pub struct mp_module_constants_t {
    pub qstr_table: *mut qstr_short_t,
    pub obj_table: *mut mp_obj_t,
}

/// From: `py/bc.h`
#[repr(C)]
pub struct mp_module_context_t {
    pub module: mp_obj_module_t,
    pub constants: mp_module_constants_t,
}

/// From: `py/emitglue.h`
#[repr(C)]
pub struct mp_raw_code_t {
    pub proto_fun_indicator: [u8; 2],
    pub kind: u8,
    pub is_generator: bool,
    pub fun_data: *const c_void,
    pub children: *mut *mut mp_raw_code_t,
}

/// From: `py/bc.h`
#[repr(C)]
pub struct mp_compiled_module_t {
    pub context: *mut mp_module_context_t,
    pub rc: *const mp_raw_code_t,
}

pub const NLR_REG_COUNT: usize = 16;

/// From: `py/nlr.h`
#[repr(C)]
pub struct nlr_buf_t {
    pub prev: *mut Self,
    pub ret_val: *mut c_void,
    pub regs: [*mut c_void; NLR_REG_COUNT],
}

/// From: `py/nlr.h`
pub type nlr_jump_callback_fun_t = extern "C" fn(ctx: *mut c_void);

/// From: `py/nlr.h`
#[repr(C)]
pub struct nlr_jump_callback_node_t {
    prev: *const Self,
    fun: nlr_jump_callback_fun_t,
}

/// From: `py/mpprint.h`
#[repr(C)]
pub struct mp_print_t {
    pub data: *mut c_void,
    pub print_strn: mp_print_strn_t,
}

/// From: `py/mpstate.h`
#[repr(C)]
pub struct mp_state_thread_t {
    pub stack_top: *mut c_char,
    pub gc_lock_depth: u16,

    pub dict_locals: *mut mp_obj_dict_t,
    pub dict_globals: *mut mp_obj_dict_t,

    pub nlr_top: *mut nlr_buf_t,
    pub nlr_jump_callback_top: *mut nlr_jump_callback_node_t,

    // originally marked as volatile
    pub pending_exception: mp_obj_t,

    pub stop_iteration_arg: mp_obj_t,
}

/// From: `py/mpstate.h`
///
/// This is an incomplete binding; the omitted fields are currently not needed.
#[repr(C)]
pub struct mp_state_ctx_t {
    pub thread: mp_state_thread_t,
    // more unneeded fields
}

unsafe extern "C" {
    // ----- Statics ----- //

    /// From: `py/mpprint.h`
    pub static mp_plat_print: mp_print_t;

    /// From: `py/mp_state.h`
    ///
    /// Currently, MicroPython threads are disabled, so this is always the active [`StateCtx`].
    pub static mut mp_state_ctx: mp_state_ctx_t;

    // ----- Initialization ----- //

    /// From: `py/runtime.h`
    pub fn mp_init();

    /// From: `py/runtime.h`
    pub fn mp_deinit();

    /// From: `py/stackctrl.h`
    pub fn mp_stack_set_top(top: *mut c_void);

    /// From: `py/gc.h`
    pub fn gc_init(start: *mut c_void, end: *mut c_void);

    // ----- Garbage collection ----- //

    /// From: `py/gc.h`
    pub fn gc_collect_start();

    /// From: `py/gc.h`
    pub fn gc_collect_root(ptrs: *mut *mut c_void, len: usize);

    /// From: `py/gc.h`
    pub fn gc_collect_end();

    /// From: `py/malloc.h`
    pub fn m_malloc(size: usize) -> *mut c_void;

    // ----- NLR ----- //

    /// From: `py/nlr.h`
    pub fn nlr_push(nlr: *mut nlr_buf_t) -> c_uint;
    /// From: `py/nlr.h`
    pub fn nlr_pop();

    // ----- Bytecode loading ----- //

    /// From: `py/persistentcode.h`
    pub fn mp_raw_code_load_mem(buf: *const u8, len: usize, ctx: *mut mp_compiled_module_t);

    /// From: `py/emitglue.h`
    pub fn mp_make_function_from_proto_fun(
        proto_fun: ProtoFun,
        context: *const mp_module_context_t,
        def_args: *const mp_obj_t,
    ) -> mp_obj_t;

    /// From: `py/runtime.h`
    pub fn mp_call_function_0(fun: mp_obj_t) -> mp_obj_t;

    // ----- Modules ----- //

    /// From: `py/objmodule.h`
    pub fn mp_module_get_builtin(module_name: qstr, extensible: bool) -> mp_obj_t;

    // ----- Exceptions ----- //

    /// From: `py/runtime.h`
    pub fn mp_raise_ValueError(msg: mp_rom_error_text_t) -> !;

    // ----- mp_obj_tect methods ----- //

    /// From: `py/obj.h`
    pub fn mp_obj_print_exception(print: *const mp_print_t, exc: mp_obj_t) -> !;

    /// From: `py/obj.h`
    pub fn mp_obj_str_get_data(self_in: mp_obj_t, len: *mut usize) -> *const c_char;

    // ----- Map methods ----- //

    /// From: `py/obj.h`
    pub fn mp_map_lookup(
        map: *mut mp_map_t,
        index: mp_obj_t,
        lookup_kind: mp_map_lookup_kind_t,
    ) -> *mut mp_map_elem_t;

    // ----- QStr ----- //

    /// From: `py/qstr.h`
    pub fn qstr_from_strn(str: *const u8, len: usize) -> qstr;
}
