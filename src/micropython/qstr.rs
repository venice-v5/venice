use super::raw::{qstr_data, qstr_from_strn};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Qstr(usize);

include!(env!("GENERATED_QSTRS_RS"));

impl GeneratedQstr {
    pub const fn as_qstr(self) -> Qstr {
        unsafe { Qstr::from_index(self as usize) }
    }
}

macro_rules! qstr {
    ($str:ident) => {
        ::paste::paste! {
            $crate::micropython::GeneratedQstr::[<MP_QSTR_ $str>].as_qstr()
        }
    };
}

pub(crate) use qstr;

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
