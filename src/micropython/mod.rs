mod exports;
mod obj;
mod raw;

use core::{
    ffi::c_void,
    ptr::{null, null_mut},
};

use self::{
    obj::Obj,
    raw::{
        CompiledModule, ModuleContext, NLR_REG_COUNT, NlrBuf, m_malloc, mp_call_function_0,
        mp_deinit, mp_make_function_from_proto_fun, mp_obj_print_exception, mp_plat_print,
        mp_raw_code_load_mem, mp_state_ctx, nlr_pop, nlr_push,
    },
};
use crate::vbt::Bytecode;

unsafe extern "C" {
    static mut __stack_top: u8;
    static mut __python_heap_start: u8;
    static mut __python_heap_end: u8;
}

pub struct MicroPython(());

impl MicroPython {
    pub unsafe fn new() -> Self {
        unsafe {
            raw::mp_stack_set_top((&raw mut __stack_top).add(0x10000) as *mut c_void);
            raw::gc_init(
                &raw mut __python_heap_start as *mut c_void,
                &raw mut __python_heap_end as *mut c_void,
            );
            raw::mp_init();
        }
        Self(())
    }

    fn push_nlr<R>(&mut self, f: impl FnOnce() -> R) -> R {
        let mut nlr_buf = NlrBuf {
            prev: null_mut(),
            ret_val: null_mut(),
            regs: [null_mut(); NLR_REG_COUNT],
        };

        unsafe {
            if nlr_push(&raw mut nlr_buf) == 0 {
                let ret = f();
                nlr_pop();
                ret
            } else {
                mp_obj_print_exception(
                    &raw const mp_plat_print,
                    Obj::from_raw(nlr_buf.ret_val as u32),
                );
            }
        }
    }

    pub fn exec_bytecode(&mut self, bytecode: Bytecode) {
        self.push_nlr(|| unsafe {
            let context = m_malloc(size_of::<ModuleContext>()) as *mut ModuleContext;
            (*context).module.globals = mp_state_ctx.thread.dict_globals;
            let mut cm = CompiledModule {
                context,
                rc: null(),
            };
            mp_raw_code_load_mem(
                bytecode.bytes().as_ptr(),
                bytecode.bytes().len(),
                &raw mut cm,
            );
            let f = mp_make_function_from_proto_fun(cm.rc.cast(), context, null());
            mp_call_function_0(f);
        });
    }

    // pub fn import_module(&mut self, module_name: &[u8], vbt: &BytecodeTable) -> Result<(), ()> {}
}

impl Drop for MicroPython {
    fn drop(&mut self) {
        unsafe {
            mp_deinit();
        }
    }
}
