use std::ffi::{c_int, c_long};

use bitflags::bitflags;

use crate::{
    fun::{Fun1, Fun2, FunVarBetween},
    obj::{Obj, ObjFullType},
};

pub const STREAM_ERROR: usize = usize::MAX;

pub const SEEK_SET: c_int = 0;
pub const SEEK_CUR: c_int = 1;
pub const SEEK_END: c_int = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Seek {
    pub offset: c_long,
    pub whence: c_int,
}

pub const IOCTL_FLUSH: u32 = 1;
pub const IOCTL_SEEK: u32 = 2;
pub const IOCTL_POLL: u32 = 3;
pub const IOCTL_CLOSE: u32 = 4;
pub const IOCTL_TIMEOUT: u32 = 5;
pub const IOCTL_GET_OPTS: u32 = 6;
pub const IOCTL_SET_OPTS: u32 = 7;
pub const IOCTL_GET_DATA_OPTS: u32 = 8;
pub const IOCTL_SET_DATA_OPTS: u32 = 9;
pub const IOCTL_GET_FILENO: u32 = 10;
pub const IOCTL_GET_BUFFER_SIZE: u32 = 11;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Poll: usize {
        const RD = 0x1;
        const WR = 0x4;
        const ERR = 0x8;
        const HUP = 0x10;
        const NVAL = 0x20;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoctlReq<'a> {
    Flush,
    Seek(&'a Seek),
    Poll(Poll),
    Close,
    /// Get/set timeout (single op)
    Timeout,
    /// Get stream options
    GetOpts,
    /// Set stream options
    SetOpts,
    /// Get data/message options
    GetDataOpts,
    /// Set data/message options
    SetDataOpts,
    /// Get fileno of underlying file
    GetFileno,
    /// Get preferred buffer size for file
    GetBufferSize,
}

pub type ReadFn =
    unsafe extern "C" fn(obj: Obj, buf: *mut u8, size: usize, errcode: *mut c_int) -> usize;
pub type WriteFn =
    unsafe extern "C" fn(obj: Obj, buf: *const u8, size: usize, errcode: *mut c_int) -> usize;
pub type IoctlFn =
    unsafe extern "C" fn(obj: Obj, request: u32, arg: usize, errcode: *mut c_int) -> usize;

#[repr(C)]
pub struct Read(ReadFn);
#[repr(C)]
pub struct Write(WriteFn);
#[repr(C)]
pub struct Ioctl(IoctlFn);

impl Read {
    pub const unsafe fn new(f: ReadFn) -> Self {
        Self(f)
    }
}

impl Write {
    pub const unsafe fn new(f: WriteFn) -> Self {
        Self(f)
    }
}

impl Ioctl {
    pub const unsafe fn new(f: IoctlFn) -> Self {
        Self(f)
    }
}

#[macro_export]
macro_rules! read_from_fn {
    ($f:expr) => {{
        unsafe extern "C" fn trampoline<'a>(
            obj: $crate::obj::Obj,
            buf: *mut u8,
            size: usize,
            errcode: *mut ::std::ffi::c_int,
        ) -> usize {
            let f: fn($crate::obj::Obj, &'a mut [u8]) -> Result<usize, ::std::ffi::c_int> = $f;
            let buf_slice = unsafe { ::std::slice::from_raw_parts_mut(buf, size) };

            match f(obj, buf_slice) {
                Ok(v) => v,
                Err(e) => {
                    unsafe { *errcode = e };
                    $crate::stream::STREAM_ERROR
                }
            }
        }

        unsafe { $crate::stream::Read::new(trampoline) }
    }};
}

#[macro_export]
macro_rules! write_from_fn {
    ($f:expr) => {{
        unsafe extern "C" fn trampoline<'a>(
            obj: $crate::obj::Obj,
            buf: *const u8,
            size: usize,
            errcode: *mut ::std::ffi::c_int,
        ) -> usize {
            let f: fn($crate::obj::Obj, &'a [u8]) -> Result<usize, ::std::ffi::c_int> = $f;
            let buf_slice = unsafe { ::std::slice::from_raw_parts(buf, size) };

            match f(obj, buf_slice) {
                Ok(v) => v,
                Err(e) => {
                    unsafe { *errcode = e };
                    $crate::stream::STREAM_ERROR
                }
            }
        }

        unsafe { $crate::stream::Write::new(trampoline) }
    }};
}

#[macro_export]
macro_rules! ioctl_from_fn {
    ($f:expr) => {{
        unsafe extern "C" fn trampoline<'a>(
            obj: $crate::obj::Obj,
            request: u32,
            arg: usize,
            errcode: *mut ::std::ffi::c_int,
        ) -> usize {
            let f: fn(
                $crate::obj::Obj,
                $crate::stream::IoctlReq<'a>,
            ) -> Result<usize, ::std::ffi::c_int> = $f;

            let r = {
                use $crate::stream::*;
                match request {
                    IOCTL_FLUSH => IoctlReq::Flush,
                    IOCTL_SEEK => IoctlReq::Seek(unsafe { &*(arg as *const Seek) }),
                    IOCTL_POLL => IoctlReq::Poll(Poll::from_bits_retain(arg)),
                    IOCTL_CLOSE => IoctlReq::Close,
                    IOCTL_TIMEOUT => IoctlReq::Timeout,
                    IOCTL_GET_OPTS => IoctlReq::GetOpts,
                    IOCTL_SET_OPTS => IoctlReq::SetOpts,
                    IOCTL_GET_DATA_OPTS => IoctlReq::GetDataOpts,
                    IOCTL_SET_DATA_OPTS => IoctlReq::SetDataOpts,
                    IOCTL_GET_FILENO => IoctlReq::GetFileno,
                    IOCTL_GET_BUFFER_SIZE => IoctlReq::GetBufferSize,
                    _ => unreachable!(),
                }
            };

            match f(obj, r) {
                Ok(v) => v,
                Err(e) => {
                    unsafe { *errcode = e };
                    $crate::stream::STREAM_ERROR
                }
            }
        }

        unsafe { $crate::stream::Ioctl::new(trampoline) }
    }};
}

#[repr(C)]
pub struct Stream {
    pub read: Read,
    pub write: Write,
    pub ioctl: Ioctl,
    pub is_text: u32,
}

impl ObjFullType {
    pub const fn set_stream(self, stream: &'static Stream) -> Self {
        unsafe { self.set_protocol_raw(stream as *const _ as *const _) }
    }
}

unsafe extern "C" {
    pub safe static mp_stream_read_obj: FunVarBetween;
    pub safe static mp_stream_read1_obj: FunVarBetween;
    pub safe static mp_stream_readinto_obj: FunVarBetween;
    pub safe static mp_stream_unbuffered_readline_obj: FunVarBetween;
    pub safe static mp_stream_unbuffered_readlines_obj: Fun1;
    pub safe static mp_stream_write_obj: FunVarBetween;
    pub safe static mp_stream_write1_obj: Fun2;
    pub safe static mp_stream_close_obj: Fun1;
    pub safe static mp_stream___exit___obj: FunVarBetween;
    pub safe static mp_stream_seek_obj: FunVarBetween;
    pub safe static mp_stream_tell_obj: Fun1;
    pub safe static mp_stream_flush_obj: Fun1;
    pub safe static mp_stream_ioctl_obj: FunVarBetween;
}
