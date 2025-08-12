use core::ffi::{CStr, c_char, c_int};

use micropython_rs::vstr::Vstr;

use crate::serial::STDIN;

pub struct Readline {
    // TODO: expand with state fields like history, cursor position
}

impl Readline {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read(&mut self, vstr: &mut Vstr, _prompt: &[u8]) {
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

            vstr.add_byte(char);
        }
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn readline(line: *mut Vstr, prompt: *const c_char) -> c_int {
    let mut readline = Readline::new();
    let prompt = unsafe { CStr::from_ptr(prompt) };
    readline.read(unsafe { &mut *line }, prompt.to_bytes());
    0
}
