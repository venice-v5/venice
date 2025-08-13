use core::{
    ptr::NonNull,
    sync::atomic::{AtomicPtr, Ordering},
};

use thiserror::Error;

use crate::MicroPython;

pub static REENTRY_PTR: AtomicPtr<MicroPython> = AtomicPtr::new(core::ptr::null_mut());

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("reentry not permitted")]
pub struct ReentryError;

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

    pub fn reenter<R>(f: impl FnOnce(NonNull<Self>) -> R) -> Result<R, ReentryError> {
        let ptr = REENTRY_PTR.swap(core::ptr::null_mut(), Ordering::Relaxed);
        let ret = NonNull::new(ptr)
            .map(|non_null| f(non_null))
            .ok_or(ReentryError);
        REENTRY_PTR.store(ptr, Ordering::Relaxed);
        ret
    }
}
