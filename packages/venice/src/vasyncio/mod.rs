pub mod event_loop;
pub mod sleep;
pub mod task;

use micropython_rs::{
    const_dict,
    fun::{Fun0, Fun1},
    map::Dict,
};

use crate::{qstrgen::qstr, vasyncio::sleep::sleep_ms};

#[unsafe(no_mangle)]
#[allow(non_upper_case_globals)]
static vasyncio_globals: Dict = const_dict![
    qstr!(__name__) => Obj::from_qstr(qstr!(vasyncio)),
    qstr!(new_event_loop) => {
        static F: Fun0 = Fun0::new(event_loop::new_event_loop);
        F.as_obj()
    },
    qstr!(sleep_ms) => Fun1::new(sleep_ms).as_obj(),
];
