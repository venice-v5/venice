use core::{
    ffi::c_void,
    ptr::{null, null_mut},
};

use crate::{
    MicroPython,
    obj::{Obj, ObjType},
    qstr::Qstr,
    raw::{
        NLR_REG_COUNT, mp_map_lookup, mp_map_lookup_kind_t, mp_module_get_builtin, mp_obj_base_t,
        mp_obj_dict_t, mp_obj_print_exception, mp_obj_type_t, mp_plat_print, mp_proto_fun_t,
        mp_raise_ValueError, nlr_buf_t, nlr_pop, nlr_push, qstr_short_t,
    },
};

unsafe extern "C" {
    static mp_type_module: mp_obj_type_t;

    /// From: `py/persistentcode.h`
    fn mp_raw_code_load_mem(buf: *const u8, len: usize, ctx: *mut CompiledModule);

    /// From: `py/emitglue.h`
    fn mp_make_function_from_proto_fun(
        proto_fun: mp_proto_fun_t,
        context: *const ModuleContext,
        def_args: *const Obj,
    ) -> Obj;

    /// From: `py/runtime.h`
    fn mp_call_function_0(fun: Obj) -> Obj;
}

/// From: `py/obj.h`
#[repr(C)]
pub struct Module {
    base: mp_obj_base_t,
    globals: *mut mp_obj_dict_t,
}

/// From: `py/bc.h`
#[repr(C)]
pub struct ModuleConstants {
    qstr_table: *mut qstr_short_t,
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
    fn push_nlr<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> Option<R> {
        let mut nlr_buf = nlr_buf_t {
            prev: null_mut(),
            ret_val: null_mut(),
            regs: [null_mut(); NLR_REG_COUNT],
        };

        unsafe {
            if nlr_push(&raw mut nlr_buf) == 0 {
                let ret = f(self);
                nlr_pop();
                Some(ret)
            } else {
                mp_obj_print_exception(
                    &raw const mp_plat_print,
                    Obj::from_raw(nlr_buf.ret_val as u32),
                );
                None
            }
        }
    }

    pub fn exec_module(&mut self, name: Obj, bytecode: &[u8]) -> Obj {
        let elem = unsafe {
            mp_map_lookup(
                &raw mut self.state_ctx_mut().vm.mp_loaded_modules_dict.map,
                name,
                mp_map_lookup_kind_t::MP_MAP_LOOKUP_ADD_IF_NOT_FOUND,
            )
        };
        let elem_value = unsafe { *elem }.value;
        if !elem_value.is_null() {
            return elem_value;
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
            this.allow_reentrance(|| mp_call_function_0(f));
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

        let loaded_module_elem = unsafe {
            mp_map_lookup(
                &raw mut self.state_ctx_mut().vm.mp_loaded_modules_dict.map,
                module_name_obj,
                mp_map_lookup_kind_t::MP_MAP_LOOKUP,
            )
        };

        if !loaded_module_elem.is_null() {
            return unsafe { *loaded_module_elem }.value;
        }

        let builtin = unsafe { mp_module_get_builtin(qstr, false) };
        if !builtin.is_null() {
            return builtin;
        }

        let bytecode = self
            .global_data()
            .module_map
            .get(module_name)
            .expect("module not found");
        self.exec_module(module_name_obj, *bytecode)
    }
}
