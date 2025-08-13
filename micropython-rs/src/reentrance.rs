use core::{
    ptr::NonNull,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::MicroPython;

pub static REENTRY_PTR: AtomicPtr<MicroPython> = AtomicPtr::new(core::ptr::null_mut());

impl MicroPython {
    pub(crate) fn allow_reentry<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let restore_ptr = REENTRY_PTR.swap(self as *mut Self, Ordering::Relaxed);
        let ret = f();
        REENTRY_PTR.store(restore_ptr, Ordering::Relaxed);
        ret
    }

    pub fn reenter<R>(f: impl FnOnce(Option<NonNull<Self>>) -> R) -> R {
        let ptr = REENTRY_PTR.swap(core::ptr::null_mut(), Ordering::Relaxed);
        let ret = f(NonNull::new(ptr));
        REENTRY_PTR.store(ptr, Ordering::Relaxed);
        ret
    }
}
