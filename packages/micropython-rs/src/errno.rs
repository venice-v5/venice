use std::ffi::c_int;

/// Operation not permitted
pub const MP_EPERM: c_int = 1;
/// No such file or directory
pub const MP_ENOENT: c_int = 2;
/// No such process
pub const MP_ESRCH: c_int = 3;
/// Interrupted system call
pub const MP_EINTR: c_int = 4;
/// I/O error
pub const MP_EIO: c_int = 5;
/// No such device or address
pub const MP_ENXIO: c_int = 6;
/// Argument list too long
pub const MP_E2BIG: c_int = 7;
/// Exec format error
pub const MP_ENOEXEC: c_int = 8;
/// Bad file number
pub const MP_EBADF: c_int = 9;
/// No child processes
pub const MP_ECHILD: c_int = 10;
/// Try again
pub const MP_EAGAIN: c_int = 11;
/// Out of memory
pub const MP_ENOMEM: c_int = 12;
/// Permission denied
pub const MP_EACCES: c_int = 13;
/// Bad address
pub const MP_EFAULT: c_int = 14;
/// Block device required
pub const MP_ENOTBLK: c_int = 15;
/// Device or resource busy
pub const MP_EBUSY: c_int = 16;
/// File exists
pub const MP_EEXIST: c_int = 17;
/// Cross-device link
pub const MP_EXDEV: c_int = 18;
/// No such device
pub const MP_ENODEV: c_int = 19;
/// Not a directory
pub const MP_ENOTDIR: c_int = 20;
/// Is a directory
pub const MP_EISDIR: c_int = 21;
/// Invalid argument
pub const MP_EINVAL: c_int = 22;
/// File table overflow
pub const MP_ENFILE: c_int = 23;
/// Too many open files
pub const MP_EMFILE: c_int = 24;
/// Not a typewriter
pub const MP_ENOTTY: c_int = 25;
/// Text file busy
pub const MP_ETXTBSY: c_int = 26;
/// File too large
pub const MP_EFBIG: c_int = 27;
/// No space left on device
pub const MP_ENOSPC: c_int = 28;
/// Illegal seek
pub const MP_ESPIPE: c_int = 29;
/// Read-only file system
pub const MP_EROFS: c_int = 30;
/// Too many links
pub const MP_EMLINK: c_int = 31;
/// Broken pipe
pub const MP_EPIPE: c_int = 32;
/// Math argument out of domain of func
pub const MP_EDOM: c_int = 33;
/// Math result not representable
pub const MP_ERANGE: c_int = 34;
/// Operation would block
pub const MP_EWOULDBLOCK: c_int = MP_EAGAIN;
/// Operation not supported on transport endpoint
pub const MP_EOPNOTSUPP: c_int = 95;
/// Address family not supported by protocol
pub const MP_EAFNOSUPPORT: c_int = 97;
/// Address already in use
pub const MP_EADDRINUSE: c_int = 98;
/// Software caused connection abort
pub const MP_ECONNABORTED: c_int = 103;
/// Connection reset by peer
pub const MP_ECONNRESET: c_int = 104;
/// No buffer space available
pub const MP_ENOBUFS: c_int = 105;
/// Transport endpoint is already connected
pub const MP_EISCONN: c_int = 106;
/// Transport endpoint is not connected
pub const MP_ENOTCONN: c_int = 107;
/// Connection timed out
pub const MP_ETIMEDOUT: c_int = 110;
/// Connection refused
pub const MP_ECONNREFUSED: c_int = 111;
/// No route to host
pub const MP_EHOSTUNREACH: c_int = 113;
/// Operation already in progress
pub const MP_EALREADY: c_int = 114;
/// Operation now in progress
pub const MP_EINPROGRESS: c_int = 115;
/// Operation canceled
pub const MP_ECANCELED: c_int = 125;
