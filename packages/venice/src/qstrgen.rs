use micropython_rs::qstr::Qstr;

include!(env!("GENERATED_QSTRS_RS"));

impl GeneratedQstr {
    pub const fn as_qstr(self) -> Qstr {
        unsafe { Qstr::from_index(self as usize) }
    }
}

macro_rules! qstr {
    ($qstr:ident) => {
        ::paste::paste! {
            $crate::qstrgen::GeneratedQstr::[<MP_QSTR_ $qstr>]
        }
        .as_qstr()
    };
}

pub(crate) use qstr;
