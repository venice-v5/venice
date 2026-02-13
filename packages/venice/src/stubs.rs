#![allow(non_camel_case_types)]

use std::io::{Read, Write, stdin, stdout};

use cty::{c_int, c_long, c_void, ptrdiff_t, size_t, ssize_t};

const STDIN_FILENO: c_int = 0;
const STDOUT_FILENO: c_int = 1;
const STDERR_FILENO: c_int = 2;

unsafe extern "C" {
    static mut errno: c_int;
}

const EIO: c_int = 5;
const EBADF: c_int = 9;
const ENOMEM: c_int = 12;
const ENOSYS: c_int = 88;

type off_t = c_long;

#[unsafe(no_mangle)]
extern "C" fn _init() {}

#[unsafe(no_mangle)]
extern "C" fn _sbrk(_incr: ptrdiff_t) -> *mut c_void {
    unsafe { errno = ENOMEM };
    core::ptr::null_mut()
}

#[unsafe(no_mangle)]
unsafe extern "C" fn _read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
    if fd != STDIN_FILENO {
        unsafe { errno = EBADF };
        return -1;
    }

    let buf = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, count) };

    let ret = stdin().read(buf);

    ret.map(|read| read as ssize_t).unwrap_or_else(|_| {
        unsafe { errno = EIO };
        -1
    })
}

#[unsafe(no_mangle)]
unsafe extern "C" fn _write(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t {
    if fd != STDOUT_FILENO && fd != STDERR_FILENO {
        unsafe { errno = EBADF };
        return -1;
    }

    let buf = unsafe { core::slice::from_raw_parts(buf as *const u8, count) };

    let ret = stdout().write(buf);

    ret.map(|written| written as ssize_t).unwrap_or_else(|_| {
        unsafe { errno = EIO };
        -1
    })
}

#[unsafe(no_mangle)]
unsafe extern "C" fn _close(_fd: c_int) -> c_int {
    unsafe { errno = ENOSYS };
    -1
}

#[unsafe(no_mangle)]
extern "C" fn _getpid() -> c_int {
    1
}

#[unsafe(no_mangle)]
unsafe extern "C" fn _kill(_pid: c_int, _signal: c_int) -> c_int {
    unsafe { errno = ENOSYS };
    -1
}

#[unsafe(no_mangle)]
unsafe extern "C" fn _lseek(_fd: c_int, _offset: off_t, _whence: c_int) -> off_t {
    unsafe { errno = ENOSYS };
    -1
}

type stat = c_void;

#[unsafe(no_mangle)]
unsafe extern "C" fn _fstat(_fd: c_int, _pstat: *mut stat) -> c_int {
    unsafe { errno = ENOSYS };
    -1
}

#[unsafe(no_mangle)]
unsafe extern "C" fn _exit(status: c_int) -> ! {
    std::process::exit(status);
}

#[unsafe(no_mangle)]
extern "C" fn _isatty(fd: c_int) -> c_int {
    (fd == STDIN_FILENO || fd == STDOUT_FILENO || fd == STDERR_FILENO) as c_int
}
