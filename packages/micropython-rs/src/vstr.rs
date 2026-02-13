unsafe extern "C" {
    /// From: `py/misc.h
    fn vstr_add_byte(vstr: *mut Vstr, v: u8);

    /// From: `py/misc.h
    fn vstr_add_strn(vstr: *mut Vstr, str: *const u8, len: usize);
}

#[repr(C)]
pub struct Vstr {
    alloc: usize,
    len: usize,
    buf: *mut u8,
    fixed_buf: bool,
}

impl Vstr {
    pub fn add_byte(&mut self, byte: u8) {
        unsafe {
            vstr_add_byte(self as *mut Self, byte);
        }
    }

    pub fn add_str(&mut self, str: &[u8]) {
        unsafe {
            vstr_add_strn(self as *mut Self, str.as_ptr(), str.len());
        }
    }
}
