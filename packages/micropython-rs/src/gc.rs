use core::{
    alloc::{GlobalAlloc, Layout},
    arch::naked_asm,
    ffi::{c_uint, c_void},
    ptr::null_mut,
};
use std::sync::{Mutex, MutexGuard};

use thiserror::Error;

use crate::{init::token, state::stack_top};

unsafe extern "C" {
    /// From: `py/gc.h`
    pub(crate) fn gc_init(start: *mut c_void, end: *mut c_void);

    /// From: `py/gc.h`
    fn gc_collect_start();

    /// From: `py/gc.h`
    fn gc_collect_root(ptrs: *mut *mut c_void, len: usize);

    /// From: `py/gc.h`
    fn gc_collect_end();

    /// From: `py/gc.h`
    fn gc_alloc(n_bytes: usize, alloc_flags: c_uint) -> *mut c_void;

    /// From: `py/gc.h`
    fn gc_free(ptr: *mut c_void);

    /// From: `py/gc.h`
    fn gc_realloc(ptr: *mut c_void, n_bytes: usize, allow_move: bool) -> *mut c_void;
}

#[unsafe(naked)]
extern "C" fn collect_gc_regs(regs: &mut [u32; 10]) -> u32 {
    #[allow(unused_unsafe)]
    unsafe {
        naked_asm!(
            // store registers into regs (r0)
            "str r4, [r0], #4",
            "str r5, [r0], #4",
            "str r6, [r0], #4",
            "str r7, [r0], #4",
            "str r8, [r0], #4",
            "str r9, [r0], #4",
            "str r10, [r0], #4",
            "str r11, [r0], #4",
            "str r12, [r0], #4",
            "str r13, [r0], #4",
            // return stack pointer
            "mov r0, sp",
            "bx lr",
        )
    }
}

pub struct LockedGc {
    gc: Mutex<Option<Gc>>,
}

pub struct Gc(());

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("garbage collector not initialized")]
pub struct NotInitialized;

impl LockedGc {
    pub const fn new(gc: Option<Gc>) -> Self {
        Self { gc: Mutex::new(gc) }
    }

    pub fn lock<'a>(&'a self) -> MutexGuard<'a, Option<Gc>> {
        self.gc.lock().unwrap()
    }
}

impl Gc {
    pub(crate) unsafe fn new() -> Self {
        Self(())
    }

    pub fn collect_garbage(&mut self) -> Result<(), NotInitialized> {
        let mut regs = [0; 10];
        let sp = collect_gc_regs(&mut regs);

        unsafe {
            gc_collect_start();
            gc_collect_root(
                sp as *mut *mut c_void,
                ((stack_top(token().unwrap()) as u32 - sp) / size_of::<usize>() as u32) as usize,
            );
            gc_collect_end();
        }

        Ok(())
    }

    pub unsafe fn alloc(&mut self, size: usize) -> *mut u8 {
        unsafe { gc_alloc(size, 0) as *mut u8 }
    }

    pub unsafe fn dealloc(&mut self, ptr: *mut u8) {
        unsafe { gc_free(ptr as *mut c_void) };
    }

    pub unsafe fn realloc(&mut self, ptr: *mut u8, new_size: usize) -> *mut u8 {
        unsafe { gc_realloc(ptr as *mut c_void, new_size, true) as *mut u8 }
    }
}

unsafe impl GlobalAlloc for LockedGc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() > 32 {
            return null_mut();
        }

        self.lock()
            .as_mut()
            .map(|gc| unsafe { gc.alloc(layout.size()) })
            .unwrap_or(null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if let Some(gc) = self.lock().as_mut() {
            unsafe { gc.dealloc(ptr) };
        }
        // fix: silent fail
    }

    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        self.lock()
            .as_mut()
            .map(|gc| unsafe { gc.realloc(ptr, new_size) })
            .unwrap_or(null_mut())
    }
}
