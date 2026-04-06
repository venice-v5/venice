use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{AttrOp, Obj, ObjBase, ObjTrait},
    qstr::Qstr,
};
use vexide_devices::smart::optical::{Gesture, GestureDirection};

use crate::modvenice::read_only_attr::read_only_attr;

#[class(qstr!(Gesture))]
#[repr(C)]
pub struct GestureObj {
    base: ObjBase,
    direction: Obj,
    // TODO: how do we make this value accessible?
    // time: LowResolutionTime,
    count: u16,
    up: u8,
    down: u8,
    left: u8,
    right: u8,
    gesture_type: u8,
}

#[class(qstr!(GestureDirection))]
#[repr(C)]
pub struct GestureDirectionObj {
    base: ObjBase,
}

#[class_methods]
impl GestureDirectionObj {
    const fn new() -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
        }
    }

    #[constant]
    pub const UP: &Self = &Self::new();
    #[constant]
    pub const DOWN: &Self = &Self::new();
    #[constant]
    pub const LEFT: &Self = &Self::new();
    #[constant]
    pub const RIGHT: &Self = &Self::new();
}

impl GestureObj {
    pub fn new(gesture: Gesture) -> Self {
        Self {
            base: Self::OBJ_TYPE.into(),
            direction: Obj::from_static(match gesture.direction {
                GestureDirection::Up => GestureDirectionObj::UP,
                GestureDirection::Down => GestureDirectionObj::DOWN,
                GestureDirection::Left => GestureDirectionObj::LEFT,
                GestureDirection::Right => GestureDirectionObj::RIGHT,
            }),
            count: gesture.count,
            up: gesture.up,
            down: gesture.down,
            left: gesture.left,
            right: gesture.right,
            gesture_type: gesture.gesture_type,
        }
    }
}

#[class_methods]
impl GestureObj {
    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };

        result.return_value(match attr.as_str() {
            "direction" => self.direction,
            "count" => Obj::from_int(self.count as i32),
            "up" => Obj::from_int(self.up as i32),
            "down" => Obj::from_int(self.down as i32),
            "left" => Obj::from_int(self.left as i32),
            "right" => Obj::from_int(self.right as i32),
            "gesture_type" => Obj::from_int(self.gesture_type as i32),
            _ => return,
        })
    }
}
