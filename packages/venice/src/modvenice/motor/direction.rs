use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{Obj, ObjBase, ObjTrait},
    ops::UnaryOp,
};
use vexide_devices::math::Direction;

#[class(qstr!(Direction))]
#[repr(C)]
pub struct DirectionObj {
    base: ObjBase,
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

    #[unary_op]
    extern "C" fn unary_op(op: UnaryOp, obj: Obj) -> Obj {
        match op {
            UnaryOp::Invert => match obj.try_as_obj::<Self>().unwrap().direction() {
                Direction::Forward => Obj::from_static(Self::REVERSE),
                Direction::Reverse => Obj::from_static(Self::FORWARD),
            },
            _ => Obj::NULL,
        }
    }
}
