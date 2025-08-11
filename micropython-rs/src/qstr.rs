use super::raw::{qstr_data, qstr_from_strn};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Qstr(usize);

impl Qstr {
    pub const unsafe fn from_index(index: usize) -> Self {
        Self(index)
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        unsafe { qstr_from_strn(bytes.as_ptr(), bytes.len()) }
    }

    pub const fn index(self) -> usize {
        self.0
    }

    pub fn bytes(self) -> &'static [u8] {
        let mut len = 0;
        unsafe {
            let ptr = qstr_data(self, &raw mut len);
            core::slice::from_raw_parts(ptr, len)
        }
    }
}
