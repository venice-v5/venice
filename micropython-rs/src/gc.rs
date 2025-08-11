use core::{arch::naked_asm, ffi::c_void};

use crate::{
    MicroPython,
    raw::{gc_collect_root, gc_collect_start, gc_init},
};

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
    pub unsafe fn init_gc(&mut self, start: *mut u8, end: *mut u8) {
        if self.global_data().gc_init {
            return;
        }

        unsafe {
            gc_init(start as *mut c_void, end as *mut c_void);
        }

        self.global_data_mut().gc_init = true;
    }

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
        }
    }
}
