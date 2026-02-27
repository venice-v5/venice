use micropython_rs::{
    attr_from_fn, const_dict,
    except::{mp_type_AttributeError, raise_msg},
    init::token,
    obj::{AttrOp, Obj, ObjBase, ObjFullType, ObjTrait, TypeFlags},
    qstr::Qstr,
};
use vexide_devices::smart::optical::{Gesture, GestureDirection};

use crate::qstrgen::qstr;

#[repr(C)]
pub struct GestureObj {
    base: ObjBase<'static>,
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

#[repr(C)]
pub struct GestureDirectionObj {
    base: ObjBase<'static>,
}

impl GestureDirectionObj {
    const fn new() -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
        }
    }
}

mod direction_statics {
    use super::*;

    pub static UP: GestureDirectionObj = GestureDirectionObj::new();
    pub static DOWN: GestureDirectionObj = GestureDirectionObj::new();
    pub static LEFT: GestureDirectionObj = GestureDirectionObj::new();
    pub static RIGHT: GestureDirectionObj = GestureDirectionObj::new();
}

pub static GESTURE_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(Gesture)).set_attr(attr_from_fn!(gesture_attr));

unsafe impl ObjTrait for GestureObj {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = GESTURE_OBJ_TYPE.as_obj_type();
}

pub static GESTURE_DIRECTION_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(GestureDirection)).set_locals_dict(const_dict![
        qstr!(UP) => Obj::from_static(&direction_statics::UP),
        qstr!(DOWN) => Obj::from_static(&direction_statics::DOWN),
        qstr!(LEFT) => Obj::from_static(&direction_statics::LEFT),
        qstr!(RIGHT) => Obj::from_static(&direction_statics::RIGHT),
    ]);

unsafe impl ObjTrait for GestureDirectionObj {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = GESTURE_DIRECTION_OBJ_TYPE.as_obj_type();
}

impl GestureObj {
    pub fn new(gesture: Gesture) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            direction: Obj::from_static(match gesture.direction {
                GestureDirection::Up => &direction_statics::UP,
                GestureDirection::Down => &direction_statics::DOWN,
                GestureDirection::Left => &direction_statics::LEFT,
                GestureDirection::Right => &direction_statics::RIGHT,
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

fn gesture_attr(this: &GestureObj, attr: Qstr, op: AttrOp) {
    let AttrOp::Load { result } = op else {
        raise_msg(token(), &mp_type_AttributeError, c"cannot write to Gesture")
    };

    result.return_value(match attr.as_str() {
        "direction" => this.direction,
        "count" => Obj::from_int(this.count as i32),
        "up" => Obj::from_int(this.up as i32),
        "down" => Obj::from_int(this.down as i32),
        "left" => Obj::from_int(this.left as i32),
        "right" => Obj::from_int(this.right as i32),
        "gesture_type" => Obj::from_int(this.gesture_type as i32),
        _ => return,
    })
}
