use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{AttrOp, Obj, ObjBase, ObjTrait},
    qstr::Qstr,
};
use vexide_devices::controller::{ButtonState, ControllerState, JoystickState};

use crate::obj::alloc_obj;

#[class(qstr!(ControllerState))]
#[repr(C)]
pub struct ControllerStateObj {
    base: ObjBase,
    state: ControllerState,
}

#[class(qstr!(ButtonState))]
#[repr(C)]
pub struct ButtonStateObj {
    base: ObjBase,
    state: ButtonState,
}

#[class(qstr!(JoystickState))]
#[repr(C)]
pub struct JoystickStateObj {
    base: ObjBase,
    state: JoystickState,
}

impl ControllerStateObj {
    pub fn new(state: ControllerState) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            state,
        }
    }
}

#[class_methods]
impl ControllerStateObj {
    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else { return };
        let state = self.state;
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
}

#[class_methods]
impl ButtonStateObj {
    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else { return };
        let state = &self.state;
        let ret = Obj::from_bool(match attr.as_str() {
            "is_pressed" => state.is_pressed(),
            "is_released" => state.is_released(),
            "is_now_pressed" => state.is_now_pressed(),
            "is_now_released" => state.is_now_released(),
            _ => return,
        });

        result.return_value(ret);
    }
}

#[class_methods]
impl JoystickStateObj {
    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else { return };
        result.return_value(match attr.as_str() {
            "x" => Obj::from_float(self.state.x() as f32),
            "y" => Obj::from_float(self.state.y() as f32),
            "x_raw" => Obj::from_int(self.state.x_raw() as i32),
            "y_raw" => Obj::from_int(self.state.y_raw() as i32),
            _ => return,
        });
    }
}
