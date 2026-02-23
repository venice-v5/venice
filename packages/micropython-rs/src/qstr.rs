pub type QstrShort = u16;
pub type QstrHash = u16;
pub type QstrLen = u8;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Qstr(usize);

/// From: `py/qstr.h`
#[repr(C)]
pub struct QstrPool {
    prev: *const Self,
    // originally bitfields
    total_prev_len: usize,
    alloc: usize,
    len: usize,
    hashes: *mut QstrHash,
    lengths: *mut QstrLen,
    // const char* qstrs[];
    qstrs: (),
}

unsafe extern "C" {
    /// From: `py/qstr.h`
    fn qstr_from_strn(str: *const u8, len: usize) -> Qstr;

    /// From: `py/qstr.h`
    fn qstr_data(q: Qstr, len: *mut usize) -> *const u8;
}

impl Qstr {
    pub const unsafe fn from_index(index: usize) -> Self {
        Self(index)
    }

    pub fn from_str(str: &str) -> Self {
        unsafe { qstr_from_strn(str.as_ptr(), str.len()) }
    }

    pub const fn index(self) -> usize {
        self.0
    }

    pub fn as_str(self) -> &'static str {
        let mut len = 0;
        unsafe {
            let ptr = qstr_data(self, &raw mut len);
            str::from_utf8_unchecked(core::slice::from_raw_parts(ptr, len))
        }
    }
}
