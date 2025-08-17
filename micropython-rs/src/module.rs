use core::{ffi::c_void, ptr::null};

use bitflags::bitflags;
use hashbrown::HashMap;
use venice_program_table::Vpt;

use crate::{
    MicroPython,
    map::Dict,
    obj::{Obj, ObjBase, ObjFullType, ObjType},
    qstr::{Qstr, QstrShort},
};

unsafe extern "C" {
    static mp_type_module: ObjFullType;

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
    base: ObjBase,
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

unsafe impl ObjType for ModuleContext {
    const TYPE_OBJ: *const ObjFullType = &raw const mp_type_module;
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

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct VptModuleFlags: u8 {
        const IS_MODULE = 0b01;
        const IS_PACKAGE = 0b10;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VptModule<'a> {
    data: &'a [u8],
}

impl<'a> VptModule<'a> {
    pub const fn flags(&self) -> VptModuleFlags {
        VptModuleFlags::from_bits(self.data[0]).expect("malformed VPT: unknown module flags set")
    }

    pub fn payload(&self) -> &'a [u8] {
        &self.data[1..]
    }
}

impl MicroPython {
    pub fn module_map(&self) -> &HashMap<&'static [u8], VptModule<'static>> {
        &self.module_map
    }

    pub fn add_vpt(&mut self, vpt: Vpt<'static>) {
        for program in vpt.program_iter() {
            self.module_map.insert(
                program.name(),
                VptModule {
                    data: program.payload(),
                },
            );
        }
    }

    pub fn exec_module(&mut self, name: Qstr, bc: &[u8]) -> Obj {
        let context_obj = ModuleContext::new(name);
        let context_ptr = context_obj.as_obj_raw().unwrap();

        let mut cm = CompiledModule {
            context: context_ptr,
            rc: null(),
        };

        let (old_globals, old_locals) = (
            self.globals() as *const Dict as *mut Dict,
            self.locals() as *const Dict as *mut Dict,
        );
        let new_globals = context_obj
            .as_obj::<ModuleContext>()
            .unwrap()
            .module
            .globals;

        unsafe {
            self.set_globals(new_globals);
            self.set_locals(new_globals);
            mp_raw_code_load_mem(bc.as_ptr(), bc.len(), &raw mut cm);
        }

        let f = unsafe { mp_make_function_from_proto_fun(cm.rc.cast(), context_ptr, null()) };

        self.push_nlr_callback(
            |this| this.allow_reentry(|| unsafe { mp_call_function_0(f) }),
            |this| unsafe {
                this.set_globals(old_globals);
                this.set_locals(old_locals);
            },
            true,
        );

        context_obj
    }

    pub fn builtin_module(&self, name: Qstr, extensible: bool) -> Obj {
        unsafe { mp_module_get_builtin(name, extensible) }
    }
}
