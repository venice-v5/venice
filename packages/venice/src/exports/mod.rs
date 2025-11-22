mod import;
mod readline;

use std::{
    ffi::{c_char, c_void},
    io::{Write, stdout},
};

use crate::ALLOCATOR;

#[unsafe(no_mangle)]
unsafe extern "C" fn mp_hal_stdout_tx_strn_cooked(str: *const c_char, len: u32) {
    let slice = unsafe { core::slice::from_raw_parts(str, len as usize) };
    stdout().write_all(slice).expect("couldn't write to stdout");
}

#[unsafe(no_mangle)]
unsafe extern "C" fn gc_collect() {
    ALLOCATOR.collect_garbage().unwrap();
}

#[unsafe(no_mangle)]
extern "C" fn nlr_jump_fail(_val: *mut c_void) -> ! {
    panic!("NLR jump fail");
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
