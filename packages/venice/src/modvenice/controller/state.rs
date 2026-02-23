use micropython_rs::{
    attr_from_fn,
    obj::{AttrOp, Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
    qstr::Qstr,
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

static CONTROLLER_STATE_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(ControllerState))
        .set_attr(attr_from_fn!(controller_state_attr));

static BUTTON_STATE_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(ButtonState))
        .set_attr(attr_from_fn!(button_state_attr));

static JOYSTICK_STATE_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(JoystickState))
        .set_attr(attr_from_fn!(joystick_state_attr));

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
    let AttrOp::Load { result } = op else { return };
    let state = this.state;
    let attr_bytes = attr.as_str();
    // Even though we can compare qstrs cheaply by their indices, that would mean losing the
    // ability to match them. So, we just match the underlying strings.
    let button_state = match attr_bytes {
        "button_a" => state.button_a,
        "button_b" => state.button_b,
        "button_x" => state.button_x,
        "button_y" => state.button_y,

        "button_up" => state.button_up,
        "button_down" => state.button_down,
        "button_right" => state.button_right,
        "button_left" => state.button_left,

        "button_l1" => state.button_l1,
        "button_l2" => state.button_l2,
        "button_r1" => state.button_r1,
        "button_r2" => state.button_r2,
        _ => {
            let joystick_state = match attr_bytes {
                "left_stick" => state.left_stick,
                "right_stick" => state.right_stick,
                _ => return,
            };

            result.return_value(alloc_obj(JoystickStateObj {
                base: ObjBase::new(JoystickStateObj::OBJ_TYPE),
                state: joystick_state,
            }));

            return;
        }
    };

    result.return_value(alloc_obj(ButtonStateObj {
        base: ObjBase::new(ButtonStateObj::OBJ_TYPE),
        state: button_state,
    }));
}

fn button_state_attr(this: &ButtonStateObj, attr: Qstr, op: AttrOp) {
    let AttrOp::Load { result } = op else { return };
    let state = &this.state;
    let ret = Obj::from_bool(match attr.as_str() {
        "is_pressed" => state.is_pressed(),
        "is_released" => state.is_released(),
        "is_now_pressed" => state.is_now_pressed(),
        "is_now_released" => state.is_now_released(),
        _ => return,
    });

    result.return_value(ret);
}

fn joystick_state_attr(this: &JoystickStateObj, attr: Qstr, op: AttrOp) {
    let AttrOp::Load { result } = op else { return };
    result.return_value(match attr.as_str() {
        "x" => Obj::from_float(this.state.x() as f32),
        "y" => Obj::from_float(this.state.y() as f32),
        "x_raw" => Obj::from_int(this.state.x_raw() as i32),
        "y_raw" => Obj::from_int(this.state.y_raw() as i32),
        _ => return,
    });
}
