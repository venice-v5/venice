use core::{
    arch::naked_asm,
    ffi::{c_uint, c_void},
};

use crate::{init::InitToken, state::stack_top};

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

pub unsafe fn alloc(_: InitToken, size: usize, enable_finaliser: bool) -> *mut u8 {
    const FINALISER_BIT: c_uint = 1;
    unsafe { gc_alloc(size, if enable_finaliser { FINALISER_BIT } else { 0 }) as *mut u8 }
}

pub unsafe fn dealloc(_: InitToken, ptr: *mut u8) {
    unsafe { gc_free(ptr as *mut c_void) };
}

pub unsafe fn realloc(_: InitToken, ptr: *mut u8, new_size: usize) -> *mut u8 {
    unsafe { gc_realloc(ptr as *mut c_void, new_size, true) as *mut u8 }
}

pub fn collect_garbage(token: InitToken) {
    let mut regs = [0; 10];
    let sp = collect_gc_regs(&mut regs);

    unsafe {
        gc_collect_start();
        gc_collect_root(
            sp as *mut *mut c_void,
            ((stack_top(token) as u32 - sp) / size_of::<usize>() as u32) as usize,
        );
        gc_collect_end();
    }
}
