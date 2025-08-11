use core::{
    cell::UnsafeCell,
    ffi::c_void,
    mem::ManuallyDrop,
    ptr::{null, null_mut},
    sync::atomic::{AtomicBool, Ordering},
};

use hashbrown::HashMap;

use super::{
    obj::Obj,
    qstr::Qstr,
    raw::{
        NLR_REG_COUNT, gc_init, mp_call_function_0, mp_compiled_module_t, mp_deinit, mp_init,
        mp_make_function_from_proto_fun, mp_map_lookup, mp_map_lookup_kind_t, mp_module_context_t,
        mp_module_get_builtin, mp_obj_print_exception, mp_plat_print, mp_raise_ValueError,
        mp_raw_code_load_mem, mp_stack_set_top, mp_state_ctx, nlr_buf_t, nlr_pop, nlr_push,
    },
};
use crate::raw::mp_state_ctx_t;

unsafe extern "C" {
    static mut __stack_top: u8;
    static mut __python_heap_start: u8;
    static mut __python_heap_end: u8;

    /// Calls libc global constructors.
    ///
    /// # Safety
    ///
    /// Must be called once at the start of the program.
    fn __libc_init_array();

    /// Calls libc global destructors.
    ///
    /// # Safety
    ///
    /// Must be called once at the end of the program.
    fn __libc_fini_array();
}

static REENTRANCE_ALLOWED: AtomicBool = AtomicBool::new(false);

static GLOBAL_DATA: GdContainer = GdContainer::new();

pub struct GdContainer {
    inner: UnsafeCell<Option<GlobalData>>,
}

unsafe impl Sync for GdContainer {}

impl GdContainer {
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(None),
        }
    }
}

pub struct GlobalData {
    pub module_map: HashMap<&'static [u8], &'static [u8]>,
}

pub struct MicroPython(());

impl MicroPython {
    pub unsafe fn new(module_map: HashMap<&'static [u8], &'static [u8]>) -> Self {
        unsafe {
            __libc_init_array();

            mp_stack_set_top((&raw mut __stack_top) as *mut c_void);
            gc_init(
                &raw mut __python_heap_start as *mut c_void,
                &raw mut __python_heap_end as *mut c_void,
            );
            mp_init();

            *GLOBAL_DATA.inner.get() = Some(GlobalData { module_map });
        }
        Self(())
    }

    pub fn global_data(&self) -> &GlobalData {
        unsafe { (*GLOBAL_DATA.inner.get()).as_ref().unwrap_unchecked() }
    }

    pub fn state_ctx(&self) -> &mp_state_ctx_t {
        unsafe { &*mp_state_ctx.get() }
    }

    pub fn state_ctx_mut(&mut self) -> &mut mp_state_ctx_t {
        unsafe { &mut *mp_state_ctx.get() }
    }

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

    unsafe fn allow_reentrance<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let old = REENTRANCE_ALLOWED.swap(true, Ordering::Acquire);
        let ret = f(self);
        REENTRANCE_ALLOWED.store(old, Ordering::Release);
        ret
    }

    pub fn reenter<R>(f: impl FnOnce(&mut Self) -> R) -> R {
        match REENTRANCE_ALLOWED.compare_exchange(true, false, Ordering::Release, Ordering::Acquire)
        {
            Ok(_) => {
                let mut this = ManuallyDrop::new(Self(()));
                let ret = f(&mut this);
                REENTRANCE_ALLOWED.store(true, Ordering::Release);
                ret
            }
            Err(_) => panic!("reetrance attempted while prohibited"),
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
            this.allow_reentrance(|_| mp_call_function_0(f));
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

impl Drop for MicroPython {
    fn drop(&mut self) {
        unsafe {
            mp_deinit();
            __libc_fini_array();
        }
    }
}
