//! Raw bindings to MicroPython

#![warn(missing_docs)]

use core::ffi::{c_char, c_uint, c_void};

use super::obj::Obj;

/// From: `py/emitglue.h`
pub type ProtoFun = *const c_void;

/// From: `py/qstr.h`
pub type QStrShort = u16;

/// From: `py/mpprint.h`
pub type PrintStrn = extern "C" fn(data: *mut c_void, str: *const c_char, len: usize);

/// From: `py/misc.h`
pub type RomErrorText = *const c_char;

/// From: `py/obj.h`
///
/// This struct actually corresponds to `mp_obj_full_type_t`.
#[repr(C)]
pub struct ObjType {
    pub base: ObjBase,

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
pub struct ObjBase {
    r#type: *const ObjType,
}

/// Temporary type alias until a proper binding is written.
pub type ObjDict = c_void;

/// From: `py/obj.h`
#[repr(C)]
pub struct ObjModule {
    pub base: ObjBase,
    pub globals: *mut ObjDict,
}

/// From: `py/bc.h`
#[repr(C)]
pub struct ModuleConstants {
    pub qstr_table: *mut QStrShort,
    pub obj_table: *mut Obj,
}

/// From: `py/bc.h`
#[repr(C)]
pub struct ModuleContext {
    pub module: ObjModule,
    pub constants: ModuleConstants,
}

/// From: `py/emitglue.h`
#[repr(C)]
pub struct RawCode {
    pub proto_fun_indicator: [u8; 2],
    pub kind: u8,
    pub is_generator: bool,
    pub fun_data: *const c_void,
    pub children: *mut *mut RawCode,
}

/// From: `py/bc.h`
#[repr(C)]
pub struct CompiledModule {
    pub context: *mut ModuleContext,
    pub rc: *const RawCode,
}

pub const NLR_REG_COUNT: usize = 16;

// From: `py/nlr.h`
#[repr(C)]
pub struct NlrBuf {
    pub prev: *mut NlrBuf,
    pub ret_val: *mut c_void,
    pub regs: [*mut c_void; NLR_REG_COUNT],
}

/// From: `py/mpprint.h`
#[repr(C)]
pub struct Print {
    data: *mut c_void,
    print_strn: PrintStrn,
}

/// From: `py/mpstate.h`
///
/// This is an incomplete binding; the omitted fields are currently not needed.
#[repr(C)]
pub struct StateThread {
    pub stack_top: *mut c_char,
    pub gc_lock_depth: u16,
    pub dict_locals: *mut ObjDict,
    pub dict_globals: *mut ObjDict,
    // more unneeded fields
}

/// From: `py/mpstate.h`
///
/// This is an incomplete binding; the omitted fields are currently not needed.
#[repr(C)]
pub struct StateCtx {
    pub thread: StateThread,
    // more unneeded fields
}

unsafe extern "C" {
    // ----- Statics ----- //

    /// From: `py/mpprint.h`
    pub static mp_plat_print: Print;

    /// From: `py/mp_state.h`
    ///
    /// Currently, MicroPython threads are disabled, so this is always the active [`StateCtx`].
    pub static mut mp_state_ctx: StateCtx;

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
    pub fn nlr_push(nlr: *mut NlrBuf) -> c_uint;
    /// From: `py/nlr.h`
    pub fn nlr_pop();

    // ----- Bytecode loading ----- //

    /// From: `py/persistentcode.h`
    pub fn mp_raw_code_load_mem(buf: *const u8, len: usize, ctx: *mut CompiledModule);

    /// From: `py/emitglue.h`
    pub fn mp_make_function_from_proto_fun(
        proto_fun: ProtoFun,
        context: *const ModuleContext,
        def_args: *const Obj,
    ) -> Obj;

    /// From: `py/runtime.h`
    pub fn mp_call_function_0(fun: Obj) -> Obj;

    // ----- Exceptions ----- //

    /// From: `py/runtime.h`
    pub fn mp_raise_ValueError(msg: RomErrorText) -> !;

    // ----- Object methods ----- //

    /// From: `py/obj.h`
    pub fn mp_obj_print_exception(print: *const Print, exc: Obj) -> !;

    /// From: `py/obj.h`
    pub fn mp_obj_str_get_data(self_in: Obj, len: *mut usize) -> *const c_char;
}
