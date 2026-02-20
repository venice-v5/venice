use std::{ffi::CStr, fmt::Write, ptr::copy_nonoverlapping};

use micropython_rs::{gc, init::token};

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

pub struct Message {
    ptr: *mut u8,
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
        let token = token();
        let mem = unsafe { gc::alloc(token, size, false) };

        let src = match &self.buf {
            Buffer::Heap(v) => &v[..size],
            Buffer::Stack(a) => &a[..size],
        };

        unsafe {
            copy_nonoverlapping(src.as_ptr(), mem, size);
        }

        Message { ptr: mem }
    }
}

impl AsRef<CStr> for Message {
    fn as_ref(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.ptr) }
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

macro_rules! error_msg {
    ($($arg:tt)*) => {{
        use ::std::fmt::Write;
        let mut writer = $crate::error_msg::MessageWriter::new();
        // Write implementation always returns Ok, no need to check
        let _ = write!(writer, $($arg)*);
        writer.finish()
    }};
}

pub(crate) use error_msg;
