use core::ffi::c_void;

use crate::{gc::Gc, qstr::Qstr, str::Str};

/// From: `py/obj.h`
#[repr(C)]
pub struct ObjFullType {
    base: ObjBase,

    flags: u16,
    name: u16,

    slot_index_make_new: u8,
    slot_index_print: u8,
    slot_index_call: u8,
    slot_index_unary_op: u8,
    slot_index_binary_op: u8,
    slot_index_attr: u8,
    slot_index_subscr: u8,
    slot_index_iter: u8,
    slot_index_buffer: u8,
    slot_index_protocol: u8,
    slot_index_parent: u8,
    slot_index_locals_dict: u8,

    slots: [*const c_void; 12],
}

/// From: `py/obj.h`
#[derive(Clone, Copy)]
#[repr(C)]
pub struct ObjBase {
    r#type: *const ObjFullType,
}

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
/// Object representation must begin with an [`mp_obj_base_t`], always initialized to `TYPE_OBJ`
pub unsafe trait ObjType: Sized {
    const TYPE_OBJ: *const ObjFullType;
}

unsafe impl Sync for ObjBase {}

impl ObjBase {
    pub const fn new(r#type: &'static ObjFullType) -> Self {
        Self {
            r#type: r#type as *const ObjFullType,
        }
    }
}

impl Obj {
    pub const NULL: Self = unsafe { Self::from_raw(0) };
    pub const NONE: Self = Self::from_immediate(0);

    pub fn new<T: ObjType>(&mut self, o: T, alloc: &mut Gc) -> Option<Obj> {
        unsafe {
            let mem = alloc.alloc(size_of::<T>());
            if mem.is_null() {
                return None;
            }
            (mem as *mut T).write(o);
            Some(Obj(mem as u32))
        }
    }

    pub const unsafe fn from_raw(inner: u32) -> Self {
        Self(inner)
    }

    pub const fn from_immediate(imm: u32) -> Self {
        Self(imm << 3 | 0b110)
    }

    pub const fn from_qstr(qstr: Qstr) -> Self {
        Self((qstr.index() as u32) << 3 | 0b010)
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

        if let Some(str) = Self::as_obj::<Str>(self) {
            return Some(str.data());
        }

        None
    }

    pub fn as_obj_raw<T: ObjType>(&self) -> Option<*mut T> {
        if self.0 & 0b11 != 0 {
            return None;
        }

        let ptr = self.0 as *mut ObjBase;
        if unsafe { *ptr }.r#type != T::TYPE_OBJ {
            return None;
        }

        Some(ptr as *mut T)
    }

    pub fn as_obj<T: ObjType>(&self) -> Option<&T> {
        self.as_obj_raw().map(|ptr| unsafe { &*ptr })
    }
}

// for potential future use
//
// unsafe extern "C" {
//     fn mp_obj_print_helper(print: *const Print, o_in: Obj, kind: PrintKind);
// }
//
// pub fn print(&mut self, obj: Obj, kind: PrintKind) {
//     unsafe {
//         mp_obj_print_helper(&raw const mp_plat_print, obj, kind);
//     }
// }
