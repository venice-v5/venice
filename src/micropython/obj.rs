use super::{qstr::Qstr, raw::m_malloc};
use crate::micropython::raw::{
    mp_module_context_t, mp_obj_base_t, mp_obj_str_t, mp_obj_type_t, mp_type_module, mp_type_str,
};

/// MicroPython object
///
/// # Representation
///
/// MicroPython has four object representations. This port uses representation A, whereby:
///
/// - `xxxx...xxx1` is a small int, and bits 1 and above are the value
/// - `xxxx...x010` is a qstr, and bits 3 and above are the value
/// - `xxxx...x110` is an immediate object, and bits 3 and abvoe are the value
/// - `xxxx...xx00` is a pointer to an [`ObjBase`]
///
/// [`ObjBase`]: super::raw::ObjBase
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Obj(u32);

/// # Safety
///
/// Object representation must begin with [`mp_obj_base_t`]
pub unsafe trait ObjType: Sized {
    const TYPE_OBJ: *const mp_obj_type_t;

    fn new() -> *mut Self {
        unsafe {
            let this = m_malloc(size_of::<Self>()) as *mut mp_obj_base_t;
            (*this).r#type = Self::TYPE_OBJ;
            this as *mut Self
        }
    }
}

impl Obj {
    pub const NULL: Self = unsafe { Self::from_raw(0) };
    pub const NONE: Self = Self::from_immediate(0);

    pub fn new<T: ObjType>() -> Self {
        Self(T::new() as u32)
    }

    pub const unsafe fn from_raw(inner: u32) -> Self {
        Self(inner)
    }

    pub const fn from_immediate(imm: u32) -> Self {
        Self(imm << 3 & 0b110)
    }

    pub fn from_qstr(qstr: Qstr) -> Self {
        Self((qstr.index() as u32) << 3 & 0b010)
    }

    pub const fn as_small_int(self) -> i32 {
        // right shifting a signed integer (as opposed to an unsigned int) performs an arithmetic
        // right shift where the sign bit is preserved, e.g. 0b1000 >> 1 = 0b1100
        self.0 as i32 >> 1
    }

    pub const fn is_null(&self) -> bool {
        self.0 == Self::NULL.0
    }

    pub const fn as_qstr(&self) -> Option<Qstr> {
        if self.0 & 0b111 == 0b10 {
            Some(unsafe { Qstr::from_index((self.0 >> 3) as usize) })
        } else {
            None
        }
    }

    pub fn get_str(&self) -> Option<&[u8]> {
        if let Some(qstr) = self.as_qstr() {
            return Some(qstr.bytes());
        }

        if let Some(str) = Self::as_obj::<mp_obj_str_t>(self) {
            return Some(unsafe { core::slice::from_raw_parts((*str).data, (*str).len) });
        }

        None
    }

    pub fn as_obj<T: ObjType>(&self) -> Option<*mut T> {
        if self.0 & 0b11 != 0 {
            return None;
        }

        let ptr = self.0 as *mut mp_obj_base_t;
        if unsafe { *ptr }.r#type != T::TYPE_OBJ {
            return None;
        }

        Some(ptr as *mut T)
    }
}

unsafe impl ObjType for mp_module_context_t {
    const TYPE_OBJ: *const mp_obj_type_t = &raw const mp_type_module;
}

unsafe impl ObjType for mp_obj_str_t {
    const TYPE_OBJ: *const mp_obj_type_t = &raw const mp_type_str;
}
