pub mod motor;
pub mod registries;

use micropython_rs::{const_dict, map::Dict, obj::Obj};

use self::motor::MOTOR_OBJ_TYPE;
use crate::qstrgen::qstr;

#[unsafe(no_mangle)]
#[allow(non_upper_case_globals)]
static venice_globals: Dict = const_dict![
    qstr!(__name__) => Obj::from_qstr(qstr!(__name__)),
    qstr!(Motor) => Obj::from_static(&MOTOR_OBJ_TYPE),
];
