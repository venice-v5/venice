use super::raw::vstr;
use crate::{micropython::raw::vstr_add_byte, serial::STDIN};

pub struct Readline {
    // TODO: expand with state fields like history, cursor position
}

impl Readline {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read(&mut self, vstr: *mut vstr, _prompt: &[u8]) {
        crate::serial::println!("readline starting");
        let mut stdin = STDIN
            .try_lock()
            .expect("attempt to read while stdin was locked");
        loop {
            let char = stdin.read_char();
            if char == -1 {
                continue;
            }

            let char = char as u8;
            if char == b'\n' {
                break;
            }

            unsafe { vstr_add_byte(vstr, char) };
        }
    }
}
