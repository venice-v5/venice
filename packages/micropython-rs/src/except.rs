use std::{
    ffi::{CStr, CString, c_int},
    marker::PhantomData,
};

use crate::{
    init::InitToken,
    obj::{Obj, ObjType},
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RomErrorText<'a> {
    ptr: *const u8,
    _phantom: PhantomData<&'a [u8]>,
}

unsafe extern "C" {
    fn mp_raise_msg(exc_type: *const ObjType, msg: RomErrorText) -> !;
    fn mp_raise_ValueError(msg: RomErrorText) -> !;
    fn mp_raise_TypeError(msg: RomErrorText) -> !;
    fn mp_raise_NotImplementedError(msg: RomErrorText) -> !;
    fn mp_raise_StopIteration(arg: Obj) -> !;
    fn mp_raise_OSError(errno_: c_int) -> !;
    fn mp_raise_OSError_with_filename(errno_: c_int, filename: *const u8) -> !;

    pub static mp_type_ImportError: ObjType;
}

impl<'a> RomErrorText<'a> {
    pub const fn new(text: &'a CStr) -> Self {
        Self {
            ptr: text.as_ptr(),
            _phantom: PhantomData,
        }
    }
}

pub fn raise_msg<'a>(_: InitToken, exc_type: *const ObjType, msg: impl AsRef<str>) -> ! {
    let cstring = CString::new(msg.as_ref()).unwrap();
    unsafe { mp_raise_msg(exc_type, RomErrorText::new(cstring.as_c_str())) };
}

pub fn raise_value_error(_: InitToken, msg: impl AsRef<str>) -> ! {
    let cstring = CString::new(msg.as_ref()).unwrap();
    unsafe { mp_raise_ValueError(RomErrorText::new(cstring.as_c_str())) };
}

pub fn raise_type_error(_: InitToken, msg: impl AsRef<str>) -> ! {
    let cstring = CString::new(msg.as_ref()).unwrap();
    unsafe { mp_raise_TypeError(RomErrorText::new(cstring.as_c_str())) };
}

pub fn raise_not_implemented_error(_: InitToken, msg: impl AsRef<str>) -> ! {
    let cstring = CString::new(msg.as_ref()).unwrap();
    unsafe { mp_raise_NotImplementedError(RomErrorText::new(cstring.as_c_str())) };
}

pub fn raise_stop_iteration(_: InitToken, arg: Obj) -> ! {
    unsafe { mp_raise_StopIteration(arg) };
}

pub fn raise_os_error(_: InitToken, errno: c_int) -> ! {
    unsafe { mp_raise_OSError(errno) };
}

pub fn raise_os_error_with_filename(_: InitToken, errno: c_int, filename: impl AsRef<str>) -> ! {
    let cstring = CString::new(filename.as_ref()).unwrap();
    unsafe { mp_raise_OSError_with_filename(errno, cstring.as_ptr()) };
}
