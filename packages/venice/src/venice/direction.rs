use micropython_rs::{
    const_dict,
    obj::{Obj, ObjFullType, TypeFlags},
};

use crate::qstrgen::qstr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward = 0,
    Reverse = 1,
}

impl Direction {
    pub const fn from_int(int: i32) -> Option<Self> {
        match int {
            0 => Some(Self::Forward),
            1 => Some(Self::Reverse),
            _ => None,
        }
    }

    pub fn from_obj(obj: Obj) -> Option<Self> {
        obj.as_small_int().and_then(Self::from_int)
    }
}

pub static DIRECTION_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(Direction))
    .set_slot_locals_dict_from_static(&const_dict![
        qstr!(FORWARD) => Obj::from_small_int(Direction::Forward as i32),
        qstr!(REVERSE) => Obj::from_small_int(Direction::Reverse as i32),
    ]);
