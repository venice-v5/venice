use crate::obj::{ObjBase, ObjFullType, ObjType};

unsafe extern "C" {
    static mp_type_str: ObjFullType;
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

unsafe impl ObjType for Str {
    const TYPE_OBJ: *const ObjFullType = &raw const mp_type_str;
}
