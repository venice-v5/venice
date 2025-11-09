use std::{ffi::c_void, ptr::null};

use crate::{
    init::InitToken,
    map::Dict,
    nlr::push_nlr_callback,
    obj::{Obj, ObjBase, ObjTrait, ObjType},
    qstr::{Qstr, QstrShort},
    state::{globals, locals, set_globals, set_locals},
};

unsafe extern "C" {
    static mp_type_module: ObjType;

    /// From: `py/persistentcode.h`
    fn mp_raw_code_load_mem(buf: *const u8, len: usize, ctx: *mut CompiledModule);

    /// From: `py/emitglue.h`
    fn mp_make_function_from_proto_fun(
        proto_fun: ProtoFun,
        context: *const ModuleContext,
        def_args: *const Obj,
    ) -> Obj;

    /// From: `py/runtime.h`
    fn mp_call_function_0(fun: Obj) -> Obj;

    /// From: `py/objmodule.h`
    fn mp_module_get_builtin(module_name: Qstr, extensible: bool) -> Obj;

    /// From: `py/obj.h`
    fn mp_obj_new_module(module_name: Qstr) -> Obj;
}

/// From: `py/emitglue.h`
pub type ProtoFun = *const c_void;

/// From: `py/obj.h`
#[repr(C)]
pub struct Module {
    base: ObjBase<'static>,
    globals: *mut Dict,
}

/// From: `py/bc.h`
#[repr(C)]
pub struct ModuleConstants {
    qstr_table: *mut QstrShort,
    obj_table: *mut Obj,
}

/// From: `py/emitglue.h`
#[repr(C)]
pub struct RawCode {
    proto_fun_indicator: [u8; 2],
    kind: u8,
    is_generator: bool,
    fun_data: *const c_void,
    children: *mut *mut RawCode,
}

/// From: `py/bc.h`
#[repr(C)]
pub struct ModuleContext {
    module: Module,
    constants: ModuleConstants,
}

unsafe impl ObjTrait for ModuleContext {
    const OBJ_TYPE: &ObjType = unsafe { &mp_type_module };
}

/// From: `py/bc.h`
#[repr(C)]
pub struct CompiledModule {
    context: *mut ModuleContext,
    rc: *const RawCode,
}

impl ModuleContext {
    pub fn new(name: Qstr) -> Obj {
        unsafe { mp_obj_new_module(name) }
    }
}

pub fn exec_module(token: InitToken, name: Qstr, bc: &[u8]) -> Obj {
    let context_obj = ModuleContext::new(name);
    let context_ptr = context_obj.try_to_obj_raw().unwrap().as_ptr();

    let mut cm = CompiledModule {
        context: context_ptr,
        rc: null(),
    };

    let old_globals = globals(token);
    let old_locals = locals(token);
    let new_globals = context_obj
        .try_to_obj::<ModuleContext>()
        .unwrap()
        .module
        .globals;

    unsafe {
        set_globals(token, new_globals);
        set_locals(token, new_globals);
        mp_raw_code_load_mem(bc.as_ptr(), bc.len(), &raw mut cm);
    }

    let f = unsafe { mp_make_function_from_proto_fun(cm.rc.cast(), context_ptr, null()) };

    push_nlr_callback(
        token,
        || unsafe { mp_call_function_0(f) },
        || unsafe {
            set_globals(token, old_globals);
            set_locals(token, old_locals);
        },
        true,
    );

    context_obj
}

pub fn builtin_module(_: InitToken, name: Qstr, extensible: bool) -> Obj {
    unsafe { mp_module_get_builtin(name, extensible) }
}
