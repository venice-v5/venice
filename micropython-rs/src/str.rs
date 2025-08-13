use crate::{
    obj::ObjType,
    raw::{mp_obj_base_t, mp_obj_type_t},
};

unsafe extern "C" {
    static mp_type_str: mp_obj_type_t;
}

/// From: `py/objstr.h`
#[repr(C)]
pub struct Str {
    base: mp_obj_base_t,
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
    const TYPE_OBJ: *const crate::raw::mp_obj_type_t = &raw const mp_type_str;
}
