use core::{
    ffi::{c_uint, c_void},
    ptr::null_mut,
};

use crate::{
    MicroPython,
    obj::Obj,
    print::{mp_obj_print_exception, mp_plat_print},
};

pub const NLR_REG_COUNT: usize = 16;

unsafe extern "C" {
    /// From: `py/nlr.h`
    pub fn nlr_push(nlr: *mut NlrBuf) -> c_uint;

    /// From: `py/nlr.h`
    pub fn nlr_pop();
}

/// From: `py/nlr.h`
#[repr(C)]
pub struct NlrBuf {
    prev: *mut Self,
    ret_val: *mut c_void,
    regs: [*mut c_void; NLR_REG_COUNT],
}

impl MicroPython {
    pub fn push_nlr<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> Option<R> {
        let mut nlr_buf = NlrBuf {
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
}
