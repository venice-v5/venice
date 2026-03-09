use micropython_rs::{
    class, class_methods,
    obj::{ObjBase, ObjTrait},
};
use vexide_devices::math::Direction;

use crate::qstrgen::qstr;

#[class(qstr!(Direction))]
#[repr(C)]
pub struct DirectionObj {
    base: ObjBase<'static>,
    direction: Direction,
}

#[class_methods]
impl DirectionObj {
    const fn new(direction: Direction) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            direction,
        }
    }

    #[constant]
    pub const FORWARD: &Self = &Self::new(Direction::Forward);
    #[constant]
    pub const REVERSE: &Self = &Self::new(Direction::Reverse);

    pub const fn direction(&self) -> Direction {
        self.direction
    }
}
