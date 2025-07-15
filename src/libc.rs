use core::ffi::{CStr, c_char, c_int};

use crate::serial::println;

#[unsafe(no_mangle)]
unsafe extern "C" fn __assert_func(
    file: *const c_char,
    line: c_int,
    func: *const c_char,
    failedexpr: *const c_char,
) {
    println!("__assert_func called");
    let file = unsafe { CStr::from_ptr(file) }
        .to_str()
        .unwrap_or("(invalid unicode)");
    let failedexpr = unsafe { CStr::from_ptr(failedexpr) }
        .to_str()
        .unwrap_or("(invalid unicode)");

    let func = if func.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(func) })
    }
    .unwrap_or(c"(n/a)")
    .to_str()
    .unwrap_or("(invalid unicode)");

    panic!(
        "[{}:{}, function: {}] micropython assertion \"{}\" failed",
        file, line, func, failedexpr,
    );
}

fn has_null(n: u32) -> bool {
    // detects NUL bytes
    // https://graphics.stanford.edu/~seander/bithacks.html#ZeroInWord
    ((n - 0x01010101) & !n & 0x80808080) != 0
}

// adapted from newlib implementation
#[unsafe(no_mangle)]
unsafe extern "C" fn strncmp(mut s1: *const c_char, mut s2: *const c_char, mut n: usize) -> c_int {
    if n == 0 {
        return 0;
    }

    // if s1 and s2 are aligned to word size
    if s1.align_offset(align_of::<usize>()) == 0 && s2.align_offset(align_of::<usize>()) == 0 {
        let mut a1 = s1 as *const usize;
        let mut a2 = s2 as *const usize;

        while n >= size_of::<usize>() && unsafe { *a1 == *a2 } {
            n -= size_of::<usize>();

            // if we've run out of bytes or hit a NUL, return zero since we already know *a1 == *a2
            if n == 0 || has_null(unsafe { *a1 } as u32) {
                return 0;
            }

            a1 = unsafe { a1.add(1) };
            a2 = unsafe { a2.add(1) };
        }

        // a difference was detected in the last few bytes of s1, so search bytewise
        s1 = a1 as *const u8;
        s2 = a2 as *const u8;
    }

    while n > 0 && unsafe { *s1 == *s2 } {
        n -= 1;

        // if we've run out of bytes or hit a NUL, return zero since we already know *s1 == *s2
        if n == 0 || unsafe { *s1 } == b'\0' {
            return 0;
        }

        s1 = unsafe { s1.add(1) };
        s2 = unsafe { s2.add(1) };
    }

    unsafe { (*s1) as c_int - (*s2) as c_int }
}

// adapted from newlib implementation
#[unsafe(no_mangle)]
unsafe extern "C" fn strchr(mut s1: *const c_char, i: c_int) -> *mut c_char {
    // TODO: improve performance by adapting speed optimized version
    let c = i as u8;

    while unsafe { *s1 != b'\0' && *s1 != c } {
        s1 = unsafe { s1.add(1) };
    }

    if unsafe { *s1 == c } {
        s1.cast_mut()
    } else {
        core::ptr::null_mut()
    }
}
