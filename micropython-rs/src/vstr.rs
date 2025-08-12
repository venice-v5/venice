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
            crate::raw::vstr_add_byte(self as *mut Self, byte);
        }
    }
}
