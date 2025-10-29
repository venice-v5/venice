unsafe extern "C" {
    /// From: `py/misc.h
    fn vstr_add_byte(vstr: *mut Vstr, v: u8);
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
}
