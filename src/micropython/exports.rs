use core::{
    arch::naked_asm,
    ffi::{c_char, c_void},
    ptr::null,
};

use super::{
    obj::Obj,
    raw::{
        gc_collect_end, gc_collect_root, gc_collect_start, mp_obj_str_get_data,
        mp_raise_ValueError, mp_state_ctx,
    },
};
use crate::vbt::MODULE_MAP;

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
extern "C" fn nlr_jump_fail(_val: *mut c_void) -> ! {
    panic!(
        "Venice internal error (NLR jump fail). If you are seeing this message as a user, please file a bug report at https://github.com/venice-v5/venice"
    );
}

#[unsafe(no_mangle)]
extern "C" fn mp_builtin___import__(arg_count: usize, args: *const Obj) -> Obj {
    let args = unsafe { core::slice::from_raw_parts(args, arg_count) };

    let module_name_obj = args[0];
    let (_fromtuple, level) = if arg_count >= 4 {
        let level = args[4].as_small_int();
        if level < 0 {
            unsafe { mp_raise_ValueError(null()) }
        } else {
            (args[3], level)
        }
    } else {
        (Obj::NONE, 0)
    };

    let mut module_name_len = 0;
    let module_name = unsafe { mp_obj_str_get_data(module_name_obj, &raw mut module_name_len) };

    if level != 0 {
        unimplemented!("relative imports not supported");
    }

    if module_name_len == 0 {
        unsafe {
            mp_raise_ValueError(null());
        }
    }

    let module_name = unsafe { core::slice::from_raw_parts(module_name, module_name_len) };
    let _module = MODULE_MAP.get(module_name).expect("module not found");

    todo!("importing not finished yet")
}
