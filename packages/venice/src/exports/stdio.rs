use std::io::{Read, Write, stdin, stdout};

use micropython_rs::{
    const_dict,
    errno::{MP_EINVAL, MP_EIO},
    ioctl_from_fn,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, TypeFlags},
    read_from_fn,
    stream::{
        IoctlReq, Stream, mp_stream_flush_obj, mp_stream_read_obj, mp_stream_readinto_obj,
        mp_stream_write_obj,
    },
    write_from_fn,
};

use crate::qstrgen::qstr;

const STDIN_STREAM: Stream = Stream {
    read: read_from_fn!(|_, buf| {
        stdin()
            .read(buf)
            .map_err(|e| e.raw_os_error().unwrap_or(MP_EIO))
    }),
    write: write_from_fn!(|_, _| { Err(MP_EINVAL) }),
    ioctl: ioctl_from_fn!(|_, _| { Err(MP_EINVAL) }),
    is_text: 1,
};

const STDOUT_STREAM: Stream = Stream {
    read: read_from_fn!(|_, _| { Err(MP_EINVAL) }),
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

pub static STDIN_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(Stdin))
    .set_stream(&STDIN_STREAM)
    .set_locals_dict(const_dict![
        qstr!(read) => Obj::from_static(&mp_stream_read_obj),
        qstr!(readinto) => Obj::from_static(&mp_stream_readinto_obj),
        //qstr!(readline) => Obj::from_static(&mp_stream_unbuffered_readline_obj),
    ]);

pub static STDOUT_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(Stdout))
    .set_stream(&STDOUT_STREAM)
    .set_locals_dict(const_dict![
        qstr!(write) => Obj::from_static(&mp_stream_write_obj),
        qstr!(flush) => Obj::from_static(&mp_stream_flush_obj),
    ]);

#[repr(C)]
pub struct Stdin {
    base: ObjBase<'static>,
}

#[repr(C)]
pub struct Stdout {
    base: ObjBase<'static>,
}

unsafe impl ObjTrait for Stdin {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = STDIN_OBJ_TYPE.as_obj_type();
}

unsafe impl ObjTrait for Stdout {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = STDOUT_OBJ_TYPE.as_obj_type();
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
