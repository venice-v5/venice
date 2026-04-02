use std::mem::MaybeUninit;

use bytemuck::PodCastError;
use thiserror::Error;

use crate::obj::{BufferFn, Obj, Slot};

pub struct Buffer<'a> {
    typecode: u8,
    buffer: &'a [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum BufferError {
    #[error("object is not a buffer")]
    NonBuffer,
    #[error("buffer could not be accessed for reading")]
    BufferUnavailable,
}

impl Obj {
    pub fn buffer<'a>(&'a self) -> Result<Buffer<'a>, BufferError> {
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

        Ok(Buffer {
            typecode: info.typecode as u8,
            buffer: unsafe { std::slice::from_raw_parts(info.buf as *const u8, info.len) },
        })
    }
}

impl<'a> Buffer<'a> {
    pub fn typecode(&self) -> u8 {
        self.typecode
    }

    pub fn buffer(&self) -> &'a [u8] {
        self.buffer
    }

    pub fn buffer_as<T>(&self) -> Result<&'a [T], PodCastError>
    where
        T: bytemuck::AnyBitPattern,
    {
        bytemuck::try_cast_slice(self.buffer)
    }
}
