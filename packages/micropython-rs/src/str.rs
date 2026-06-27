use std::{
    ffi::{CStr, FromBytesWithNulError},
    hint::unreachable_unchecked,
};

use crate::obj::{MpCStrError, Obj, ObjBase, ObjTrait, ObjType};

unsafe extern "C" {
    static mp_type_str: ObjType;
}

/// From: `py/objstr.h`
#[repr(C)]
pub struct Str {
    base: ObjBase,
    hash: usize,
    len: usize,
    data: *const u8,
}

impl Str {
    pub fn new(s: &str) -> Obj {
        unsafe extern "C" {
            /// From: `py/objstr.h`
            fn mp_obj_new_str_copy(ty: *const ObjType, data: *const u8, len: usize) -> Obj;
        }

        unsafe { mp_obj_new_str_copy(Self::OBJ_TYPE, s.as_ptr(), s.len()) }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn data(&self) -> &str {
        unsafe { str::from_utf8_unchecked(core::slice::from_raw_parts(self.data, self.len)) }
    }

    pub fn as_cstr(&self) -> Result<&CStr, MpCStrError> {
        let bytes = unsafe {
            // data always contains a nul byte at len+1
            std::slice::from_raw_parts(self.data, self.len + 1)
        };
        CStr::from_bytes_with_nul(bytes).map_err(|e| match e {
            FromBytesWithNulError::InteriorNul { position } => MpCStrError { position },
            FromBytesWithNulError::NotNulTerminated => unsafe { unreachable_unchecked() },
        })
    }
}

unsafe impl ObjTrait for Str {
    const OBJ_TYPE: &ObjType = unsafe { &mp_type_str };
}
