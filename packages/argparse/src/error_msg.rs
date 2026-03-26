use std::{ffi::CStr, fmt::Write};

use micropython_rs::{except::Message, init::token};

pub const STACK_BUF_SIZE: usize = 128;
pub const USABLE_STACK_BUF_SIZE: usize = STACK_BUF_SIZE - 1;

pub enum Buffer {
    Stack([u8; STACK_BUF_SIZE]),
    Heap(Vec<u8>),
}

pub struct MessageWriter {
    buf: Buffer,
    pos: usize,
}

impl MessageWriter {
    pub fn new() -> Self {
        Self {
            buf: Buffer::Stack([0; STACK_BUF_SIZE]),
            pos: 0,
        }
    }

    pub fn finish(self) -> Message {
        let size = self.pos + 1;

        let src = match &self.buf {
            Buffer::Heap(v) => &v[..size],
            Buffer::Stack(a) => &a[..size],
        };
        let cstr = unsafe { CStr::from_bytes_with_nul_unchecked(src) };

        Message::from_cstr(cstr, token()).unwrap()
    }
}

impl Write for MessageWriter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let start = self.pos;
        let end = self.pos + s.len();
        self.pos = end;

        let slice = match &mut self.buf {
            Buffer::Heap(v) => {
                v.resize(self.pos + 1, 0);
                &mut v[start..end]
            }
            Buffer::Stack(a) => {
                if a.len() < end + 1 {
                    let mut vec = Vec::new();
                    vec.extend_from_slice(a.as_slice());
                    vec.resize(end + 1, 0);

                    let _ = std::mem::replace(&mut self.buf, Buffer::Heap(vec));

                    let Buffer::Heap(vec) = &mut self.buf else {
                        unreachable!()
                    };

                    &mut vec[start..end]
                } else {
                    &mut a[start..end]
                }
            }
        };

        slice.copy_from_slice(s.as_bytes());
        Ok(())
    }
}

#[macro_export]
macro_rules! error_msg {
    ($($arg:tt)*) => {{
        use ::std::fmt::Write;
        let mut writer = $crate::MessageWriter::new();
        // Write implementation always returns Ok, no need to check
        let _ = write!(writer, $($arg)*);
        writer.finish()
    }};
}
