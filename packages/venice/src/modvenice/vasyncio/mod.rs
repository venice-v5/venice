use micropython_rs::{
    const_dict,
    map::Dict,
    obj::{Obj, ObjTrait},
};

use crate::modvenice::vasyncio::{
    event_loop::{EventLoop, get_running_loop_obj, run_obj, spawn_obj},
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
    qstr!(get_running_loop) => get_running_loop_obj,
    qstr!(run) => run_obj,
    qstr!(spawn) => spawn_obj,
];
