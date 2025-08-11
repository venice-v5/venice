use core::{
    arch::naked_asm,
    ffi::{CStr, c_char, c_int, c_void},
    ptr::null,
};

use super::{
    MicroPython,
    obj::Obj,
    raw::{gc_collect_end, gc_collect_root, gc_collect_start, mp_raise_ValueError},
};
use crate::{
    micropython::{raw::vstr, readline::Readline},
    serial::print_bytes,
};

#[unsafe(no_mangle)]
extern "C" fn mp_hal_stdout_tx_strn_cooked(str: *const c_char, len: u32) {
    let slice = unsafe { core::slice::from_raw_parts(str, len as usize) };
    print_bytes(slice);
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
    MicroPython::reenter(|mp| unsafe {
        gc_collect_start();
        let mut regs = [0; 10];
        let sp = collect_gc_regs(&mut regs);
        gc_collect_root(
            sp as *mut *mut c_void,
            ((mp.state_ctx().thread.stack_top as u32 - sp) / size_of::<usize>() as u32) as usize,
        );
        gc_collect_end();
    })
}

#[unsafe(no_mangle)]
extern "C" fn nlr_jump_fail(_val: *mut c_void) -> ! {
    panic!("NLR jump fail");
}

#[unsafe(no_mangle)]
unsafe extern "C" fn venice_import(arg_count: usize, args: *const Obj) -> Obj {
    let args = unsafe { core::slice::from_raw_parts(args, arg_count) };

    let module_name_obj = args[0];
    let (fromtuple, level) = if args.len() >= 4 {
        let level = args[4].as_small_int();
        if level < 0 {
            // TODO: make safe
            unsafe { mp_raise_ValueError(null()) }
        } else {
            (args[3], level)
        }
    } else {
        (Obj::NONE, 0)
    };

    MicroPython::reenter(|mp| mp.import(module_name_obj, fromtuple, level))
}

#[unsafe(no_mangle)]
unsafe extern "C" fn readline(line: *mut vstr, prompt: *const c_char) -> c_int {
    let mut readline = Readline::new();
    let prompt = unsafe { CStr::from_ptr(prompt) };
    readline.read(line, prompt.to_bytes());
    0
}

#[allow(non_upper_case_globals)]
mod statics {
    use crate::micropython::Obj;

    #[unsafe(no_mangle)]
    static mp_sys_stdin_obj: Obj = Obj::NONE;

    #[unsafe(no_mangle)]
    static mp_sys_stdout_obj: Obj = Obj::NONE;

    #[unsafe(no_mangle)]
    static mp_sys_stderr_obj: Obj = Obj::NONE;
}
