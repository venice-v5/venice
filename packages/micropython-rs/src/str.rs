use crate::obj::{ObjBase, ObjTrait, ObjType};

unsafe extern "C" {
    pub static mp_type_str: ObjType;
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
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn data(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.data, self.len) }
    }
}

unsafe impl ObjTrait for Str {
    const OBJ_TYPE: *const ObjType = &raw const mp_type_str;
}
