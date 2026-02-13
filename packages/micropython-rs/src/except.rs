use std::{
    ffi::{CStr, CString, c_int},
    marker::PhantomData,
};

use crate::{
    init::InitToken,
    obj::{Obj, ObjFullType, ObjType, TypeFlags},
    print::{Print, PrintKind},
    qstr::Qstr,
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RomErrorText<'a> {
    ptr: *const u8,
    _phantom: PhantomData<&'a [u8]>,
}

unsafe extern "C" {
    fn mp_obj_exception_make_new(
        ty: *const ObjType,
        n_args: usize,
        n_kw: usize,
        args: *const Obj,
    ) -> Obj;
    fn mp_obj_exception_print(print: *const Print, o: Obj, kind: PrintKind);
    fn mp_obj_exception_attr(self_in: Obj, attr: Qstr, dest: *mut Obj);

    fn mp_raise_msg(exc_type: *const ObjType, msg: RomErrorText) -> !;
    fn mp_raise_ValueError(msg: RomErrorText) -> !;
    fn mp_raise_TypeError(msg: RomErrorText) -> !;
    fn mp_raise_NotImplementedError(msg: RomErrorText) -> !;
    fn mp_raise_StopIteration(arg: Obj) -> !;
    fn mp_raise_OSError(errno_: c_int) -> !;
    fn mp_raise_OSError_with_filename(errno_: c_int, filename: *const u8) -> !;

    pub safe static mp_type_BaseException: ObjType;
    pub safe static mp_type_Exception: ObjType;
    pub safe static mp_type_ImportError: ObjType;
    pub safe static mp_type_RuntimeError: ObjType;
}

impl<'a> RomErrorText<'a> {
    pub const fn new(text: &'a CStr) -> Self {
        Self {
            ptr: text.as_ptr(),
            _phantom: PhantomData,
        }
    }
}

pub const fn new_exception_type(name: Qstr, parent: &'static ObjType) -> ObjFullType {
    unsafe {
        ObjFullType::new(TypeFlags::empty(), name)
            .set_make_new_raw(mp_obj_exception_make_new)
            .set_print_raw(mp_obj_exception_print)
            .set_attr_raw(mp_obj_exception_attr)
            .set_parent(parent)
    }
}

pub fn raise_msg(_: InitToken, exc_type: &ObjType, msg: impl AsRef<str>) -> ! {
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
