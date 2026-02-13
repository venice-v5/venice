use std::{
    ffi::{CStr, c_char, c_int},
    io::stdin,
};

use micropython_rs::vstr::Vstr;

pub struct Readline {
    // TODO: expand with state fields like history, cursor position
}

impl Readline {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read(&mut self, vstr: &mut Vstr, _prompt: &[u8]) {
        let mut buf = String::new();
        stdin().read_line(&mut buf).expect("couldn't read line");
        vstr.add_str(buf.as_bytes());
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn readline(line: *mut Vstr, prompt: *const c_char) -> c_int {
    let mut readline = Readline::new();
    let prompt = unsafe { CStr::from_ptr(prompt) };
    readline.read(unsafe { &mut *line }, prompt.to_bytes());
    0
}
