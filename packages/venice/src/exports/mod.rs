mod fs;
mod import;
mod readline;
mod stdio;

use std::{
    ffi::{c_char, c_void},
    io::{Write, stdout},
};

use micropython_rs::init::token;

#[unsafe(no_mangle)]
unsafe extern "C" fn mp_hal_stdout_tx_strn_cooked(str: *const c_char, len: u32) {
    let slice = unsafe { core::slice::from_raw_parts(str, len as usize) };
    stdout().write_all(slice).expect("couldn't write to stdout");
}

#[unsafe(no_mangle)]
unsafe extern "C" fn gc_collect() {
    micropython_rs::gc::collect_garbage(token());
}

#[unsafe(no_mangle)]
extern "C" fn nlr_jump_fail(_val: *mut c_void) -> ! {
    panic!("NLR jump fail");
}
