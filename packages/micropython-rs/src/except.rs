use std::{
    ffi::{CStr, c_int},
    marker::PhantomData,
};

use crate::{
    init::{InitToken, token},
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

    safe static mp_type_BaseException: ObjType;
    safe static mp_type_Exception: ObjType;
    safe static mp_type_ValueError: ObjType;
    safe static mp_type_TypeError: ObjType;
    safe static mp_type_NotImplementedError: ObjType;
    safe static mp_type_ImportError: ObjType;
    safe static mp_type_RuntimeError: ObjType;
    safe static mp_type_AttributeError: ObjType;
}

impl<'a> RomErrorText<'a> {
    pub const fn new(text: &'a CStr) -> Self {
        Self {
            ptr: text.as_ptr(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, T> From<&'a T> for RomErrorText<'a>
where
    T: AsRef<CStr>,
{
    fn from(value: &'a T) -> Self {
        Self::new(value.as_ref())
    }
}

#[repr(transparent)]
pub struct ExceptionType(ObjType);

impl ExceptionType {
    pub const fn new(name: Qstr, parent: &'static ExceptionType) -> Self {
        Self(
            unsafe {
                ObjFullType::new(TypeFlags::empty(), name)
                    .set_make_new_raw(mp_obj_exception_make_new)
                    .set_print_raw(mp_obj_exception_print)
                    .set_attr_raw(mp_obj_exception_attr)
                    .set_parent(&parent.0)
            }
            .into_obj_type(),
        )
    }

    const fn from_obj_type_ref(obj_type: &ObjType) -> &Self {
        unsafe { std::mem::transmute(obj_type) }
    }
}

pub const BASE_EXCEPTION_TYPE: &ExceptionType =
    ExceptionType::from_obj_type_ref(&mp_type_BaseException);
pub const EXCEPTION_TYPE: &ExceptionType = ExceptionType::from_obj_type_ref(&mp_type_Exception);
pub const VALUE_ERROR_TYPE: &ExceptionType = ExceptionType::from_obj_type_ref(&mp_type_ValueError);
pub const TYPE_ERROR_TYPE: &ExceptionType = ExceptionType::from_obj_type_ref(&mp_type_TypeError);
pub const NOT_IMPLEMENTED_ERROR_TYPE: &ExceptionType =
    ExceptionType::from_obj_type_ref(&mp_type_NotImplementedError);
pub const IMPORT_ERROR_TYPE: &ExceptionType =
    ExceptionType::from_obj_type_ref(&mp_type_ImportError);
pub const RUNTIME_ERROR_TYPE: &ExceptionType =
    ExceptionType::from_obj_type_ref(&mp_type_RuntimeError);
pub const ATTRIBUTE_ERROR_TYPE: &ExceptionType =
    ExceptionType::from_obj_type_ref(&mp_type_AttributeError);

pub fn raise_msg(_: InitToken, exc_type: &ExceptionType, msg: impl AsRef<CStr>) -> ! {
    unsafe { mp_raise_msg(&exc_type.0, RomErrorText::new(msg.as_ref())) };
}

pub fn raise_value_error(_: InitToken, msg: impl AsRef<CStr>) -> ! {
    unsafe { mp_raise_ValueError(RomErrorText::new(msg.as_ref())) };
}

pub fn raise_type_error(_: InitToken, msg: impl AsRef<CStr>) -> ! {
    unsafe { mp_raise_TypeError(RomErrorText::new(msg.as_ref())) };
}

pub fn raise_not_implemented_error(_: InitToken, msg: impl AsRef<CStr>) -> ! {
    unsafe { mp_raise_NotImplementedError(RomErrorText::new(msg.as_ref())) };
}

pub fn raise_stop_iteration(_: InitToken, arg: Obj) -> ! {
    unsafe { mp_raise_StopIteration(arg) };
}

pub fn raise_os_error(_: InitToken, errno: c_int) -> ! {
    unsafe { mp_raise_OSError(errno) };
}

pub fn raise_os_error_with_filename(_: InitToken, errno: c_int, filename: impl AsRef<CStr>) -> ! {
    unsafe { mp_raise_OSError_with_filename(errno, filename.as_ref().as_ptr()) };
}

#[derive(Clone, Copy)]
pub struct Exception<M> {
    pub ty: &'static ExceptionType,
    pub msg: M,
}

impl<M: AsRef<CStr>> Exception<M> {
    pub fn raise(&self, token: InitToken) -> ! {
        raise_msg(token, self.ty, self.msg.as_ref())
    }
}

impl<T, M> From<Result<T, Exception<M>>> for Obj
where
    T: Into<Obj>,
    M: AsRef<CStr>,
{
    fn from(value: Result<T, Exception<M>>) -> Self {
        match value {
            Ok(v) => v.into(),
            Err(e) => e.raise(token()),
        }
    }
}

pub fn value_error<M: AsRef<CStr>>(msg: impl Into<M>) -> Exception<M> {
    Exception {
        ty: VALUE_ERROR_TYPE,
        msg: msg.into(),
    }
}

pub fn type_error<M: AsRef<CStr>>(msg: impl Into<M>) -> Exception<M> {
    Exception {
        ty: TYPE_ERROR_TYPE,
        msg: msg.into(),
    }
}

pub fn not_implemented_error<M: AsRef<CStr>>(msg: impl Into<M>) -> Exception<M> {
    Exception {
        ty: NOT_IMPLEMENTED_ERROR_TYPE,
        msg: msg.into(),
    }
}

pub fn runtime_error<M: AsRef<CStr>>(msg: impl Into<M>) -> Exception<M> {
    Exception {
        ty: RUNTIME_ERROR_TYPE,
        msg: msg.into(),
    }
}

pub fn attribute_error<M: AsRef<CStr>>(msg: impl Into<M>) -> Exception<M> {
    Exception {
        ty: ATTRIBUTE_ERROR_TYPE,
        msg: msg.into(),
    }
}

pub fn import_error<M: AsRef<CStr>>(msg: impl Into<M>) -> Exception<M> {
    Exception {
        ty: IMPORT_ERROR_TYPE,
        msg: msg.into(),
    }
}
