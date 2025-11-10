use micropython_rs::{
    attr_from_fn, obj::{AttrOp, Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags}, qstr::Qstr
};
use vexide_devices::controller::{ButtonState, ControllerState, JoystickState};

use crate::{obj::alloc_obj, qstrgen::qstr};

#[repr(C)]
pub struct ControllerStateObj {
    base: ObjBase<'static>,
    state: ControllerState,
}

#[repr(C)]
pub struct ButtonStateObj {
    base: ObjBase<'static>,
    state: ButtonState,
}

#[repr(C)]
pub struct JoystickStateObj {
    base: ObjBase<'static>,
    state: JoystickState,
}

static CONTROLLER_STATE_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(ControllerState))
        .set_attr(attr_from_fn!(controller_state_attr));

static BUTTON_STATE_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(ButtonState)).set_attr(attr_from_fn!(button_state_attr));

static JOYSTICK_STATE_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(JoystickState)).set_attr(attr_from_fn!(joystick_state_attr));

unsafe impl ObjTrait for ControllerStateObj {
    const OBJ_TYPE: &ObjType = CONTROLLER_STATE_OBJ_TYPE.as_obj_type();
}

unsafe impl ObjTrait for ButtonStateObj {
    const OBJ_TYPE: &ObjType = BUTTON_STATE_OBJ_TYPE.as_obj_type();
}

unsafe impl ObjTrait for JoystickStateObj {
    const OBJ_TYPE: &ObjType = JOYSTICK_STATE_OBJ_TYPE.as_obj_type();
}

impl ControllerStateObj {
    pub fn new(state: ControllerState) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            state,
        }
    }
}

fn controller_state_attr(this: &ControllerStateObj, attr: Qstr, op: AttrOp) {
    match op {
        AttrOp::Load { dest } => {
            let state = this.state;
            let attr_bytes = attr.bytes();
            let button_state = match attr_bytes {
                b"button_a" => state.button_a,
                b"button_b" => state.button_b,
                b"button_x" => state.button_x,
                b"button_y" => state.button_y,

                b"button_up" => state.button_up,
                b"button_down" => state.button_down,
                b"button_right" => state.button_right,
                b"button_left" => state.button_left,

                b"button_l1" => state.button_l1,
                b"button_l2" => state.button_l2,
                b"button_r1" => state.button_r1,
                b"button_r2" => state.button_r2,
                _ => {
                    let joystick_state = match attr_bytes {
                        b"left_stick" => state.left_stick,
                        b"right_stick" => state.right_stick,
                        _ => return,
                    };

                    *dest = alloc_obj(JoystickStateObj {
                        base: ObjBase::new(JoystickStateObj::OBJ_TYPE),
                        state: joystick_state,
                    });

                    return;
                }
            };

            *dest = alloc_obj(ButtonStateObj {
                base: ObjBase::new(ButtonStateObj::OBJ_TYPE),
                state: button_state,
            });
        }
        _ => return
    }
}

fn button_state_attr(this: &ButtonStateObj, attr: Qstr, op: AttrOp) {
    match op {
        AttrOp::Load { dest } => {
            let state = &this.state;
            *dest = Obj::from_bool(match attr.bytes() {
                b"is_pressed" => state.is_pressed(),
                b"is_released" => state.is_released(),
                b"is_now_pressed" => state.is_now_pressed(),
                b"is_now_released" => state.is_now_released(),
                _ => return,
            });
        },
        _ => return
    }
}

fn joystick_state_attr(this: &JoystickStateObj, attr: Qstr, op: AttrOp) {
    match op {
        AttrOp::Load { dest } => {
            *dest = match attr.bytes() {
                b"x" => Obj::from_float(this.state.x() as f32),
                b"y" => Obj::from_float(this.state.y() as f32),
                b"x_raw" => Obj::from_int(this.state.x_raw() as i32),
                b"y_raw" => Obj::from_int(this.state.y_raw() as i32),
                _ => return,
            };
        },
        _ => return
    }
}
