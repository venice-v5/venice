use crate::obj::{ObjBase, ObjTrait, ObjType};

unsafe extern "C" {
    static mp_type_str: ObjType;
}

/// From: `py/objstr.h`
#[repr(C)]
pub struct Str {
    base: ObjBase<'static>,
    hash: usize,
    len: usize,
    data: *const u8,
}

impl Str {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn data(&self) -> &str {
        unsafe { str::from_utf8_unchecked(core::slice::from_raw_parts(self.data, self.len)) }
    }
}

unsafe impl ObjTrait for Str {
    const OBJ_TYPE: &ObjType = unsafe { &mp_type_str };
}
