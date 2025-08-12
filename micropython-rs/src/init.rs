use core::{
    ffi::c_void,
    sync::atomic::{AtomicBool, Ordering},
};

use hashbrown::HashMap;

use crate::{MicroPython, gc::gc_init, state::GlobalData};

pub static MICROPYTHON_CREATED: AtomicBool = AtomicBool::new(false);

unsafe extern "C" {
    /// From: `py/runtime.h`
    pub fn mp_init();

    /// From: `py/runtime.h`
    pub fn mp_deinit();

    /// From: `py/stackctrl.h`
    pub fn mp_stack_ctrl_init();

    fn __libc_init_array();
    fn __libc_fini_array();
}

impl MicroPython {
    pub unsafe fn new(heap_start: *mut u8, heap_end: *mut u8) -> Option<Self> {
        if let Err(_) =
            MICROPYTHON_CREATED.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        {
            return None;
        }

        unsafe {
            __libc_init_array();

            mp_stack_ctrl_init();
            gc_init(heap_start as *mut c_void, heap_end as *mut c_void);
            mp_init();
        }

        let mut this = Self(());

        unsafe {
            this.set_global_data(GlobalData {
                module_map: HashMap::new(),
            });
        }

        Some(this)
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
