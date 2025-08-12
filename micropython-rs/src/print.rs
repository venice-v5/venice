use core::ffi::{c_char, c_void};

use crate::obj::Obj;

/// From: `py/mpprint.h`
pub type PrintStrn = unsafe extern "C" fn(data: *mut c_void, str: *const c_char, len: usize);

/// From: `py/mpprint.h`
#[repr(C)]
pub struct Print {
    pub data: *mut c_void,
    pub print_strn: PrintStrn,
}

unsafe extern "C" {
    /// From: `py/mpprint.h`
    pub static mp_plat_print: Print;

    /// From: `py/obj.h`
    pub fn mp_obj_print_exception(print: *const Print, exc: Obj);
}
