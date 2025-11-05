pub mod motor;

use micropython_rs::{
    const_dict,
    map::Dict,
    obj::{Obj, ObjTrait},
};

use self::motor::{MotorObj, direction::DirectionObj};
use crate::qstrgen::qstr;

#[unsafe(no_mangle)]
#[allow(non_upper_case_globals)]
static venice_globals: Dict = const_dict![
    qstr!(__name__) => Obj::from_qstr(qstr!(__name__)),
    qstr!(Motor) => Obj::from_static(MotorObj::OBJ_TYPE),
    qstr!(Direction) => Obj::from_static(DirectionObj::OBJ_TYPE),
];
