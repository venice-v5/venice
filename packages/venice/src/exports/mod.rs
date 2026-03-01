mod fs;
mod import;
mod stdio;

use std::ffi::c_void;

use micropython_rs::init::token;

#[unsafe(no_mangle)]
unsafe extern "C" fn gc_collect() {
    micropython_rs::gc::collect_garbage(token());
}

#[unsafe(no_mangle)]
extern "C" fn nlr_jump_fail(_val: *mut c_void) -> ! {
    panic!("NLR jump fail");
}
