use std::{
    ffi::{c_uint, c_void},
    ptr::null_mut,
};

use crate::{
    init::InitToken,
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

pub type NlrJumpCallback = unsafe extern "C" fn(ctx: *mut c_void);

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

pub fn push_nlr<R>(_: InitToken, f: impl FnOnce() -> R) -> Option<R> {
    let mut nlr_buf = NlrBuf {
        prev: null_mut(),
        ret_val: null_mut(),
        regs: [null_mut(); NLR_REG_COUNT],
    };

    unsafe {
        if nlr_push(&raw mut nlr_buf) == 0 {
            let ret = f();
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

unsafe extern "C" fn callback_trampoline<C>(ctx_ptr: *mut c_void)
where
    C: FnOnce(),
{
    let ctx_ptr = ctx_ptr as *const NlrJumpCallbackNode<C>;
    unsafe {
        let ctx = core::ptr::read(ctx_ptr);
        (ctx.data)()
    }
}

pub fn push_nlr_callback<F, R, C>(_: InitToken, f: F, callback: C, run_after_pop: bool) -> R
where
    F: FnOnce() -> R,
    C: FnOnce(),
{
    let mut node = NlrJumpCallbackNode {
        prev: core::ptr::null(),
        fun: callback_trampoline::<C>,
        data: callback,
    };

    unsafe {
        nlr_push_jump_callback(
            &raw mut node as *mut NlrJumpCallbackNode<()>,
            callback_trampoline::<C>,
        )
    };

    let ret = f();
    unsafe {
        nlr_pop_jump_callback(run_after_pop);
    }

    ret
}

pub fn raise(_: InitToken, ex: Obj) -> ! {
    unsafe extern "C" {
        fn mp_obj_is_exception_instance(self_in: Obj) -> bool;
        fn nlr_jump(val: *mut c_void) -> !;
    }

    unsafe {
        if !mp_obj_is_exception_instance(ex) {
            panic!("attempt to raise non-exception object");
        }

        nlr_jump(ex.inner());
    }
}
