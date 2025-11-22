use std::{
    ffi::c_void,
    sync::atomic::{AtomicBool, Ordering},
};

use thiserror::Error;

use crate::gc::{Gc, gc_init};

static INIT: AtomicBool = AtomicBool::new(false);

unsafe extern "C" {
    /// From: `py/runtime.h`
    fn mp_init();

    /// From: `py/stackctrl.h`
    fn mp_stack_ctrl_init();

    fn __libc_init_array();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitToken(());

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("micropython already initialized")]
pub struct AlreadyInit;

pub unsafe fn init_mp(
    heap_start: *mut u8,
    heap_end: *mut u8,
) -> Result<(InitToken, Gc), AlreadyInit> {
    if let Err(_) = INIT.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed) {
        return Err(AlreadyInit);
    }

    unsafe {
        __libc_init_array();

        mp_stack_ctrl_init();
        gc_init(heap_start as *mut c_void, heap_end as *mut c_void);
        mp_init();

        let token = InitToken(());
        Ok((token, Gc::new(token)))
    }
}

pub fn token() -> Option<InitToken> {
    match INIT.load(Ordering::Relaxed) {
        true => Some(InitToken(())),
        false => None,
    }
}
