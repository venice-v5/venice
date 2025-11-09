use micropython_rs::{
    const_dict,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, TypeFlags},
};
use vexide_devices::math::Direction;

use crate::qstrgen::qstr;

static DIRECTION_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(Direction))
    .set_slot_locals_dict_from_static(&const_dict![
        qstr!(FORWARD) => Obj::from_static(&DirectionObj::FORWARD),
        qstr!(REVERSE) => Obj::from_static(&DirectionObj::REVERSE),
    ]);

#[repr(C)]
pub struct DirectionObj {
    base: ObjBase<'static>,
    direction: Direction,
}

unsafe impl ObjTrait for DirectionObj {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = DIRECTION_OBJ_TYPE.as_obj_type();
}

impl DirectionObj {
    pub const FORWARD: Self = Self::new(Direction::Forward);
    pub const REVERSE: Self = Self::new(Direction::Reverse);

    pub const fn new(direction: Direction) -> Self {
        Self {
            base: ObjBase::new(DIRECTION_OBJ_TYPE.as_obj_type()),
            direction,
        }
    }

    pub const fn direction(&self) -> Direction {
        self.direction
    }
}
