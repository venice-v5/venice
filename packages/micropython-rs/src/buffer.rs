use std::mem::MaybeUninit;

use bytemuck::{AnyBitPattern, NoUninit, PodCastError};
use thiserror::Error;

use crate::obj::{BufferFn, Obj, Slot};

pub struct Buffer<'a, T> {
    typecode: u8,
    buffer: &'a [T],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum BufferError {
    #[error("object is not a buffer")]
    NonBuffer,
    #[error("buffer could not be accessed for reading")]
    BufferUnavailable,
}

impl Obj {
    pub fn buffer<'a>(&'a self) -> Result<Buffer<'a, u8>, BufferError> {
        const MP_BUFFER_READ: u32 = 1;
        // const MP_BUFFER_WRITE: u32 = 2;
        // const MP_BUFFER_RW: u32 = MP_BUFFER_READ | MP_BUFFER_WRITE;
        // const MP_BUFFER_RAISE_IF_UNSUPPORTED: u32 = 4;

        let buffer_fn = self
            .obj_type()
            .slot_value_raw(Slot::Buffer)
            .ok_or(BufferError::NonBuffer)?;

        let info = unsafe {
            let buffer_fn: BufferFn = std::mem::transmute(buffer_fn);
            let mut buffer_info = MaybeUninit::uninit();
            if buffer_fn(*self, buffer_info.as_mut_ptr(), MP_BUFFER_READ) != 0 {
                return Err(BufferError::BufferUnavailable);
            }
            buffer_info.assume_init()
        };

        // info.ptr might be null if info.len = 0, so avoid passing that pointer to from_raw_parts
        // and just use an empty slice
        let slice = if info.len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(info.buf as *const u8, info.len) }
        };

        Ok(Buffer {
            typecode: info.typecode as u8,
            buffer: slice,
        })
    }
}

impl<'a, T> Buffer<'a, T> {
    pub fn typecode(&self) -> u8 {
        self.typecode
    }

    pub fn buffer(&self) -> &'a [T] {
        self.buffer
    }
}

impl<'a, T> Buffer<'a, T>
where
    T: NoUninit,
{
    pub fn cast<U>(self) -> Result<Buffer<'a, U>, PodCastError>
    where
        U: AnyBitPattern,
    {
        Ok(Buffer {
            buffer: bytemuck::try_cast_slice(self.buffer)?,
            typecode: self.typecode,
        })
    }
}
