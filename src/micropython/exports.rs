use core::{
    arch::naked_asm,
    ffi::{c_char, c_void},
};

use super::raw::{gc_collect_end, gc_collect_root, gc_collect_start, mp_state_ctx};

#[unsafe(no_mangle)]
extern "C" fn mp_hal_stdout_tx_strn_cooked(str: *const c_char, len: u32) {
    unsafe {
        vex_sdk::vexSerialWriteBuffer(1, str, len);
    }
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
        );
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn gc_collect() {
    unsafe {
        gc_collect_start();
        let mut regs = [0; 10];
        let sp = collect_gc_regs(&mut regs);
        gc_collect_root(
            sp as *mut *mut c_void,
            ((mp_state_ctx.thread.stack_top as u32 - sp) / size_of::<usize>() as u32) as usize,
        );
        gc_collect_end();
    }
}

#[unsafe(no_mangle)]
extern "C" fn nlr_jump_fail() -> ! {
    panic!(
        "Venice internal error (NLR jump fail). If you are seeing this message as a user, please file a bug report at https://github.com/venice-v5/venice"
    );
}
