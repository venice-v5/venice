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

impl Obj {
    pub const NULL: Self = unsafe { Self::from_raw(0) };
    pub const NONE: Self = Self::from_immediate(0);

    pub const unsafe fn from_raw(inner: u32) -> Self {
        Self(inner)
    }

    pub const fn from_immediate(imm: u32) -> Self {
        Self(imm << 3 & 0b110)
    }

    pub const fn as_small_int(self) -> i32 {
        // right shifting a signed integer (as opposed to an unsigned int) performs an arithmetic
        // right shift where the sign bit is preserved, e.g. 0b1000 >> 0b1100
        self.0 as i32 >> 1
    }

    pub const fn is_null(&self) -> bool {
        self.0 == Self::NULL.0
    }
}
