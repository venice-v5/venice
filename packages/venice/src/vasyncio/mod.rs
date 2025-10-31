pub mod event_loop;
pub mod sleep;
pub mod task;

use micropython_rs::{
    const_dict,
    fun::{Fun0, Fun1},
    map::Dict,
    obj::Obj,
};

use crate::{
    qstrgen::qstr,
    vasyncio::{
        event_loop::{EVENT_LOOP_OBJ_TYPE, get_running_loop, vasyncio_run},
        sleep::SLEEP_OBJ_TYPE,
    },
};

#[unsafe(no_mangle)]
#[allow(non_upper_case_globals)]
static vasyncio_globals: Dict = const_dict![
    qstr!(__name__) => Obj::from_qstr(qstr!(vasyncio)),
    qstr!(EventLoop) => Obj::from_static(&EVENT_LOOP_OBJ_TYPE),
    qstr!(Sleep) => Obj::from_static(&SLEEP_OBJ_TYPE),
    qstr!(get_running_loop) => Obj::from_static(&Fun0::new(get_running_loop)),
    qstr!(run) => Obj::from_static(&Fun1::new(vasyncio_run)),
];
