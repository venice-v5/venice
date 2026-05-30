use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{Obj, ObjBase, ObjTrait},
    ops::UnaryOpCode,
    print::{Print, PrintKind},
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
    fn unary_op(op: UnaryOpCode, obj: &Self) -> Obj {
        match op {
            UnaryOpCode::Invert => match obj.direction() {
                Direction::Forward => Obj::from_static(Self::REVERSE),
                Direction::Reverse => Obj::from_static(Self::FORWARD),
            },
            _ => Obj::NULL,
        }
    }

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        print.print(match self.direction {
            Direction::Forward => "Direction.FORWARD",
            Direction::Reverse => "Direction.REVERSE",
        });
    }
}
