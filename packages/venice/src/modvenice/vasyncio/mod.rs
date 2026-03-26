use micropython_rs::{
    const_dict,
    fun::{Fun0, Fun1},
    map::Dict,
    obj::{Obj, ObjTrait},
};

use crate::modvenice::vasyncio::{
    event_loop::{EventLoop, vasyncio_get_running_loop, vasyncio_run, vasyncio_spawn},
    sleep::Sleep,
};

pub mod event_loop;
pub mod sleep;
pub mod task;
pub mod time32;

pub const VASYNCIO_DICT: &Dict = const_dict![
    qstr!(__name__) => Obj::from_qstr(qstr!(vasyncio)),
    qstr!(EventLoop) => Obj::from_static(EventLoop::OBJ_TYPE),
    qstr!(Sleep) => Obj::from_static(Sleep::OBJ_TYPE),
    qstr!(get_running_loop) => Obj::from_static(&Fun0::new(vasyncio_get_running_loop)),
    qstr!(run) => Obj::from_static(&Fun1::new(vasyncio_run)),
    qstr!(spawn) => Obj::from_static(&Fun1::new(vasyncio_spawn)),
];
