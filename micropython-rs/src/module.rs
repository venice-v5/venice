use core::ptr::{null, null_mut};

use crate::{
    MicroPython,
    obj::Obj,
    qstr::Qstr,
    raw::{
        NLR_REG_COUNT, mp_call_function_0, mp_compiled_module_t, mp_make_function_from_proto_fun,
        mp_map_lookup, mp_map_lookup_kind_t, mp_module_context_t, mp_module_get_builtin,
        mp_obj_print_exception, mp_plat_print, mp_raise_ValueError, mp_raw_code_load_mem,
        nlr_buf_t, nlr_pop, nlr_push,
    },
};

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

        let context_obj = Obj::new::<mp_module_context_t>();
        let context_ptr = context_obj.as_obj::<mp_module_context_t>().unwrap();

        unsafe {
            (*context_ptr).module.globals = self.state_ctx().thread.dict_globals;
            (*elem).value = context_obj;
        }

        let mut cm = mp_compiled_module_t {
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
