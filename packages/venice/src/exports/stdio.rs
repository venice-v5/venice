use std::{
    ffi::{CStr, c_int},
    io::{Read, Write, stdin, stdout},
    os::raw::c_char,
};

use micropython_rs::{
    class, class_methods,
    errno::{MP_EINVAL, MP_EIO},
    fun::{Fun1, FunVarBetween},
    ioctl_from_fn,
    obj::{ObjBase, ObjTrait},
    read_from_fn,
    stream::{
        IoctlReq, Stream, mp_stream_flush_obj, mp_stream_read_obj, mp_stream_readinto_obj,
        mp_stream_write_obj,
    },
    write_from_fn,
};
use vex_sdk_jumptable::vexSerialReadChar;

#[class(qstr!(Stdin))]
#[repr(C)]
pub struct Stdin {
    base: ObjBase,
}

#[class(qstr!(Stdout))]
#[repr(C)]
pub struct Stdout {
    base: ObjBase,
}

#[class_methods]
impl Stdin {
    #[stream]
    const STREAM: Stream = Stream {
        read: read_from_fn!(|_, buf| {
            stdin()
                .read(buf)
                .map_err(|e| e.raw_os_error().unwrap_or(MP_EIO))
        }),
        write: write_from_fn!(|_, _| Err(MP_EINVAL)),
        ioctl: ioctl_from_fn!(|_, _| Err(MP_EINVAL)),
        is_text: 1,
    };

    #[constant(qstr!(read))]
    const READ: &FunVarBetween = &mp_stream_read_obj;
    #[constant(qstr!(readinto))]
    const READINTO: &FunVarBetween = &mp_stream_readinto_obj;
}

#[class_methods]
impl Stdout {
    #[stream]
    const STDOUT_STREAM: Stream = Stream {
        read: read_from_fn!(|_, _| Err(MP_EINVAL)),
        write: write_from_fn!(|_, buf| {
            stdout()
                .write(buf)
                .map_err(|e| e.raw_os_error().unwrap_or_default())
        }),
        ioctl: ioctl_from_fn!(|_, req| {
            match req {
                IoctlReq::Flush => stdout()
                    .flush()
                    .map(|_| 0)
                    .map_err(|e| e.raw_os_error().unwrap_or(MP_EIO)),
                _ => Err(MP_EINVAL),
            }
        }),
        is_text: 1,
    };

    #[constant(qstr!(write))]
    const WRITE: &FunVarBetween = &mp_stream_write_obj;

    #[constant(qstr!(flush))]
    const FLUSH: &Fun1 = &mp_stream_flush_obj;
}

#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut mp_sys_stdin_obj: Stdin = Stdin {
    base: ObjBase::new(Stdin::OBJ_TYPE),
};

#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut mp_sys_stdout_obj: Stdout = Stdout {
    base: ObjBase::new(Stdout::OBJ_TYPE),
};

#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut mp_sys_stderr_obj: Stdout = Stdout {
    base: ObjBase::new(Stdout::OBJ_TYPE),
};

#[unsafe(no_mangle)]
unsafe extern "C" fn mp_hal_stdout_tx_strn_cooked(str: *const c_char, len: u32) {
    let slice = unsafe { core::slice::from_raw_parts(str, len as usize) };
    stdout().write_all(slice).expect("couldn't write to stdout");
}

#[unsafe(no_mangle)]
unsafe extern "C" fn mp_hal_stdout_tx_strn(str: *const c_char, len: u32) -> usize {
    let slice = unsafe { core::slice::from_raw_parts(str, len as usize) };
    stdout().write(slice).expect("couldn't write to stdout")
}

#[unsafe(no_mangle)]
unsafe extern "C" fn mp_hal_stdout_tx_str(str: *const c_char) {
    let cstr = unsafe { CStr::from_ptr(str) };
    stdout()
        .write_all(cstr.to_bytes())
        .expect("couldn't write to stdout");
}

#[unsafe(no_mangle)]
unsafe extern "C" fn mp_hal_stdin_rx_chr() -> c_int {
    unsafe { vexSerialReadChar(1) }
}
