use core::{arch::naked_asm, ffi::c_void};

use crate::MicroPython;

unsafe extern "C" {
    /// From: `py/gc.h`
    pub fn gc_init(start: *mut c_void, end: *mut c_void);

    /// From: `py/gc.h`
    fn gc_collect_start();

    /// From: `py/gc.h`
    fn gc_collect_root(ptrs: *mut *mut c_void, len: usize);

    /// From: `py/gc.h`
    fn gc_collect_end();

    /// From: `py/malloc.h`
    pub fn m_malloc(size: usize) -> *mut c_void;
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

impl MicroPython {
    pub fn collect_garbage(&mut self) {
        let mut regs = [0; 10];
        let sp = collect_gc_regs(&mut regs);

        unsafe {
            gc_collect_start();
            gc_collect_root(
                sp as *mut *mut c_void,
                ((self.state_ctx().thread.stack_top as u32 - sp) / size_of::<usize>() as u32)
                    as usize,
            );
            gc_collect_end();
        }
    }
}
