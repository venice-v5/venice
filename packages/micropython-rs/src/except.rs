use std::{ffi::c_int, marker::PhantomData};

use crate::{init::InitToken, obj::Obj};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RomErrorText<'a> {
    ptr: *const u8,
    _phantom: PhantomData<&'a [u8]>,
}

unsafe extern "C" {
    fn mp_raise_ValueError(msg: RomErrorText) -> !;
    fn mp_raise_TypeError(msg: RomErrorText) -> !;
    fn mp_raise_NotImplementedError(msg: RomErrorText) -> !;
    fn mp_raise_StopIteration(arg: Obj) -> !;
    fn mp_raise_OSError(errno_: c_int) -> !;
    fn mp_raise_OSError_with_filename(errno_: c_int, filename: *const u8) -> !;
}

impl<'a> RomErrorText<'a> {
    pub const fn new(text: &'a str) -> Self {
        Self {
            ptr: text.as_ptr(),
            _phantom: PhantomData,
        }
    }
}

pub fn raise_value_error(_: InitToken, msg: RomErrorText) -> ! {
    unsafe { mp_raise_ValueError(msg) };
}

pub fn raise_type_error(_: InitToken, msg: RomErrorText) -> ! {
    unsafe { mp_raise_TypeError(msg) };
}

pub fn raise_not_implemented_error(_: InitToken, msg: RomErrorText) -> ! {
    unsafe { mp_raise_NotImplementedError(msg) };
}

pub fn raise_stop_iteration(_: InitToken, arg: Obj) -> ! {
    unsafe { mp_raise_StopIteration(arg) };
}

pub fn raise_os_error(_: InitToken, errno: c_int) -> ! {
    unsafe { mp_raise_OSError(errno) };
}

pub fn raise_os_error_with_filename(_: InitToken, errno: c_int, filename: *const u8) -> ! {
    unsafe { mp_raise_OSError_with_filename(errno, filename) };
}
