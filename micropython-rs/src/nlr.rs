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
    fn nlr_push(nlr: *mut NlrBuf) -> c_uint;

    /// From: `py/nlr.h`
    fn nlr_pop();

    /// From: `py/nlr.h`
    fn nlr_push_jump_callback(node: *mut NlrJumpCallbackNode<()>, fun: NlrJumpCallback);

    /// From: `py/nlr.h`
    fn nlr_pop_jump_callback(run_callback: bool);
}

pub type NlrJumpCallback = extern "C" fn(ctx: *mut c_void);

/// From: `py/nlr.h`
#[repr(C)]
pub struct NlrBuf {
    prev: *mut Self,
    ret_val: *mut c_void,
    regs: [*mut c_void; NLR_REG_COUNT],
}

#[repr(C)]
pub struct NlrJumpCallbackNode<T> {
    prev: *const Self,
    fun: NlrJumpCallback,
    data: T,
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

    pub fn push_nlr_callback<F, R, C, T>(
        &mut self,
        f: F,
        callback: C,
        data: T,
        run_after_pop: bool,
    ) -> R
    where
        F: FnOnce(&mut Self) -> R,
        C: FnOnce(T),
    {
        extern "C" fn callback_bootstrap<C, T>(ctx_ptr: *mut c_void)
        where
            C: FnOnce(T),
        {
            let ctx_ptr = ctx_ptr as *const NlrJumpCallbackNode<(C, T)>;
            unsafe {
                let ctx = core::ptr::read(ctx_ptr);
                (ctx.data.0)(ctx.data.1)
            }
        }

        let mut node = NlrJumpCallbackNode {
            prev: core::ptr::null(),
            fun: callback_bootstrap::<C, T>,
            data: (callback, data),
        };

        unsafe {
            nlr_push_jump_callback(
                &raw mut node as *mut NlrJumpCallbackNode<()>,
                callback_bootstrap::<C, T>,
            )
        };
        let ret = f(self);
        unsafe {
            nlr_pop_jump_callback(run_after_pop);
        }

        ret
    }
}
