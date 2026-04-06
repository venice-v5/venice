use std::{
    ffi::{c_char, c_void},
    marker::PhantomData,
};

use crate::obj::Obj;

/// From: `py/mpprint.h`
pub type PrintStrn = unsafe extern "C" fn(data: *mut c_void, str: *const c_char, len: usize);

/// From: `py/mpprint.h`
#[repr(C)]
pub struct Print {
    data: *mut c_void,
    print_strn: PrintStrn,
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
    pub(crate) static mp_plat_print: Print;

    /// From: `py/obj.h`
    pub(crate) fn mp_obj_print_exception(print: *const Print, exc: Obj);
}

pub struct StringPrint<'a> {
    print: Print,
    _phantom: PhantomData<&'a mut String>,
}

unsafe extern "C" fn string_print_print_strn(data: *mut c_void, str: *const c_char, len: usize) {
    unsafe {
        let string = &mut *(data as *mut String);
        let str = str::from_utf8_unchecked(std::slice::from_raw_parts(str, len));
        string.push_str(str);
    }
}

impl<'a> StringPrint<'a> {
    pub fn new(string: &'a mut String) -> Self {
        Self {
            print: Print {
                data: string as *mut String as *mut c_void,
                print_strn: string_print_print_strn,
            },
            _phantom: PhantomData,
        }
    }

    pub fn print(&mut self) -> &mut Print {
        &mut self.print
    }

    pub fn string(&mut self) -> &mut String {
        unsafe { &mut *(self.print.data as *mut String) }
    }
}
