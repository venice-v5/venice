mod readline;

use core::ffi::{c_char, c_void};

use micropython_rs::{MicroPython, obj::Obj};

use crate::serial::print_bytes;

#[unsafe(no_mangle)]
extern "C" fn mp_hal_stdout_tx_strn_cooked(str: *const c_char, len: u32) {
    let slice = unsafe { core::slice::from_raw_parts(str, len as usize) };
    print_bytes(slice);
}

#[unsafe(no_mangle)]
unsafe extern "C" fn gc_collect() {
    MicroPython::reenter(|ptr| unsafe { ptr.expect("reentry_failed").as_mut().collect_garbage() });
}

#[unsafe(no_mangle)]
extern "C" fn nlr_jump_fail(_val: *mut c_void) -> ! {
    panic!("NLR jump fail");
}

#[unsafe(no_mangle)]
unsafe extern "C" fn venice_import(_arg_count: usize, _args: *const Obj) -> Obj {
    todo!()
    // let args = unsafe { core::slice::from_raw_parts(args, arg_count) };

    // let module_name_obj = args[0];
    // let (fromtuple, level) = if args.len() >= 4 {
    //     let level = args[4].as_small_int();
    //     if level < 0 {
    //         // TODO: make safe
    //         unsafe { mp_raise_ValueError(null()) }
    //     } else {
    //         (args[3], level)
    //     }
    // } else {
    //     (Obj::NONE, 0)
    // };

    // MicroPython::reenter(|mp| mp.import(module_name_obj, fromtuple, level))
}

#[allow(non_upper_case_globals)]
mod statics {
    use micropython_rs::obj::Obj;

    #[unsafe(no_mangle)]
    static mp_sys_stdin_obj: Obj = Obj::NONE;

    #[unsafe(no_mangle)]
    static mp_sys_stdout_obj: Obj = Obj::NONE;

    #[unsafe(no_mangle)]
    static mp_sys_stderr_obj: Obj = Obj::NONE;
}
