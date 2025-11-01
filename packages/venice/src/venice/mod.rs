pub mod motor;
pub mod registries;

use micropython_rs::{const_dict, map::Dict, obj::Obj};

use crate::{qstrgen::qstr, venice::motor::MOTOR_OBJ_TYPE};

#[unsafe(no_mangle)]
#[allow(non_upper_case_globals)]
static venice_globals: Dict = const_dict![
    qstr!(__name__) => Obj::from_qstr(qstr!(__name__)),
    qstr!(Motor) => Obj::from_static(&MOTOR_OBJ_TYPE),
];
