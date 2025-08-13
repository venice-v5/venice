use core::{
    ffi::c_void,
    ptr::{null, null_mut},
};

use crate::{
    MicroPython,
    map::Dict,
    obj::{Obj, ObjType},
    qstr::{Qstr, QstrShort},
    raw::{mp_obj_base_t, mp_obj_type_t, mp_raise_ValueError},
};

unsafe extern "C" {
    static mp_type_module: mp_obj_type_t;

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
}

/// From: `py/emitglue.h`
pub type ProtoFun = *const c_void;

/// From: `py/obj.h`
#[repr(C)]
pub struct Module {
    base: mp_obj_base_t,
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
    const TYPE_OBJ: *const mp_obj_type_t = &raw const mp_type_module;
}

/// From: `py/bc.h`
#[repr(C)]
pub struct CompiledModule {
    context: *mut ModuleContext,
    rc: *const RawCode,
}

impl MicroPython {
    pub fn exec_module(&mut self, name: Obj, bytecode: &[u8]) -> Obj {
        let loaded_module = self.state_ctx().vm.mp_loaded_modules_dict.map.get(name);
        if let Some(module) = loaded_module {
            return module;
        }

        let context = ModuleContext {
            module: Module {
                base: mp_obj_base_t {
                    r#type: &raw const mp_type_module,
                },
                globals: self.state_ctx().thread.dict_globals,
            },
            constants: ModuleConstants {
                qstr_table: null_mut(),
                obj_table: null_mut(),
            },
        };

        let context_obj = Obj::new::<ModuleContext>(context).unwrap();
        let context_ptr = context_obj.as_obj().unwrap();

        let mut cm = CompiledModule {
            context: context_ptr,
            rc: null(),
        };

        self.push_nlr(|this| unsafe {
            mp_raw_code_load_mem(bytecode.as_ptr(), bytecode.len(), &raw mut cm);
            let f = mp_make_function_from_proto_fun(cm.rc.cast(), context_ptr, null());
            this.allow_reentry(|| mp_call_function_0(f));
        });

        context_obj
    }

    pub fn import(&mut self, module_name_obj: Obj, _fromtuple: Obj, level: i32) -> Obj {
        let module_name = module_name_obj
            .get_str()
            .expect("module name not a qstr or a str");

        if level != 0 {
            unimplemented!("relative imports not supported");
        }

        if module_name.is_empty() {
            unsafe {
                mp_raise_ValueError(null());
            }
        }

        let qstr = Qstr::from_bytes(module_name);

        let loaded_module = self
            .state_ctx()
            .vm
            .mp_loaded_modules_dict
            .map
            .get(module_name_obj);
        if let Some(module) = loaded_module {
            return module;
        }

        let builtin = unsafe { mp_module_get_builtin(qstr, false) };
        if !builtin.is_null() {
            return builtin;
        }

        let bytecode = self.module_map.get(module_name).expect("module not found");
        self.exec_module(module_name_obj, *bytecode)
    }
}
