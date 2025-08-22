use micropython_rs::{const_dict, const_map, map::Dict};

use crate::qstrgen::qstr;

#[unsafe(no_mangle)]
#[allow(non_upper_case_globals)]
static mut vasyncio_globals: Dict = const_dict![];

/// Initialize vasyncio globals
///
/// MicroPython built-in function objects cannot be created at compile time because function
/// pointer addresses aren't available then. Therefore, the vasyncio functions must be inserted
/// into the globals dictionary at runtime, before code gets a change to use the module.
///
/// We also want the globals dictionary to be fixed, or read-only, so that it can neither be
/// modified by user code nor MicroPython. So, instead of just inserting the desired function
/// objects, we will initialize the entire dictionary at runtime.
///
/// # Safety
///
/// This function must be called once during program execution.
///
/// # Future internal note
///
/// This function should remain safe to call before MicroPython is initialized, or in other words,
/// it should work without a `MicroPython` struct and refrain from direct FFI calls to MicroPython.
pub unsafe fn init_vasyncio() {
    let globals = const_dict![
        qstr!(__name__) => Obj::from_qstr(qstr!(vasyncio)),
    ];

    unsafe {
        vasyncio_globals = globals;
    }
}
