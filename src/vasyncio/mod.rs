pub mod event_loop;

use alloc::boxed::Box;

use micropython_rs::{
    const_dict,
    fun::Fun0,
    map::{Dict, Map},
    map_table,
    obj::Obj,
};

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
    static NEW_EVENT_LOOP_OBJ: Fun0 = Fun0::new(event_loop::new_event_loop);

    let global_table = Box::leak(Box::new(map_table![
        qstr!(__name__) => Obj::from_qstr(qstr!(vasyncio)),
        qstr!(new_event_loop) => NEW_EVENT_LOOP_OBJ.as_obj(),
    ]));

    let global_map = unsafe {
        Map::from_raw_parts(
            global_table.as_mut_ptr(),
            global_table.len(),
            global_table.len(),
            true,
            true,
            true,
        )
    };

    let global_dict = Dict::new(global_map);

    unsafe {
        vasyncio_globals = global_dict;
    }
}
