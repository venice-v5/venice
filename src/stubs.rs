// TODO: use the proper function signatures

macro_rules! stub {
    ($($name:ident),*) => {
        $(
            #[unsafe(no_mangle)]
            extern "C" fn $name() {}
        )*
    }
}

#[rustfmt::skip]
stub!(
    // called by __libc_init_array; unneeded
    _init,
    // called by __libc_fini_array; unneeded
    _fini,
    _sbrk,
    _write,
    _read,
    _lseek,
    _close,
    _fstat,
    _isatty,
    _exit,
    _kill,
    _getpid
);
