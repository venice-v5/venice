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

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum PrintKind {
    Str = 0,
    Repr = 1,
    /// Special format for printing exception in unhandled exception message
    Exc = 2,
    Json = 3,
    /// Special format for printing bytes as an undecorated string
    Raw = 4,
    /// Internal flag for printing exception subclasses
    ExcSubclass = 0x80,
}

unsafe extern "C" {
    /// From: `py/mpprint.h`
    pub static mp_plat_print: Print;

    /// From: `py/obj.h`
    pub fn mp_obj_print_exception(print: *const Print, exc: Obj);
}
