#[cfg(not(target_pointer_width = "32"))]
compile_error!(
    "micropython-rs only supports 32-bit object representation and must be compiled on a 32-bit target"
);

mod raw;

pub(crate) mod print;

pub mod buffer;
pub mod class;
pub mod errno;
pub mod except;
pub mod fun;
pub mod gc;
pub mod generator;
pub mod init;
pub mod list;
pub mod map;
pub mod module;
pub mod nlr;
pub mod obj;
pub mod ops;
pub mod qstr;
pub mod state;
pub mod str;
pub mod stream;
pub mod tuple;
pub mod vstr;
