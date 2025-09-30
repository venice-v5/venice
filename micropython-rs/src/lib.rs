#![no_std]

mod init;
mod nlr;
mod raw;
mod reentrancy;
mod state;

pub(crate) mod print;

pub mod fun;
pub mod gc;
pub mod map;
pub mod module;
pub mod obj;
pub mod qstr;
pub mod str;
pub mod vstr;

use hashbrown::HashMap;

pub struct MicroPython {
    module_map: HashMap<&'static [u8], module::VptModule<'static>>,
}
