use std::fmt::Write;

use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{AttrOp, Obj, ObjBase, ObjTrait},
    ops::BinaryOpCode,
    print::{Print, PrintKind},
    qstr::Qstr,
};
use vexide_devices::controller::{ButtonState, ControllerState, JoystickState};

use crate::{modvenice::read_only_attr::read_only_attr, obj::alloc_obj};

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
    #[stub(attrs = [
        "button_a: ButtonState",
        "button_b: ButtonState",
        "button_x: ButtonState",
        "button_y: ButtonState",
        "button_up: ButtonState",
        "button_down: ButtonState",
        "button_right: ButtonState",
        "button_left: ButtonState",
        "button_l1: ButtonState",
        "button_l2: ButtonState",
        "button_r1: ButtonState",
        "button_r2: ButtonState",
        "left_stick: JoystickState",
        "right_stick: JoystickState",
    ])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };
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

    #[binary_op]
    fn binary_op(op: BinaryOpCode, lhs: &Self, rhs: Obj) -> Obj {
        match op {
            BinaryOpCode::Equal => Obj::from_bool(lhs.state == rhs.as_obj::<Self>().state),
            _ => Obj::NULL,
        }
    }

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        print.print("ControllerState(");

        let mut print_button_state = |name, state| {
            print.print(name); // punctuation should be included in the name to reduce `print` calls
            ButtonStateObj::print_state(state, print);
        };

        print_button_state("button_a=", &self.state.button_a);
        print_button_state(", button_b=", &self.state.button_b);
        print_button_state(", button_x=", &self.state.button_x);
        print_button_state(", button_y=", &self.state.button_y);

        print_button_state(", button_up=", &self.state.button_up);
        print_button_state(", button_down=", &self.state.button_down);
        print_button_state(", button_left=", &self.state.button_left);
        print_button_state(", button_right=", &self.state.button_right);

        print_button_state(", button_l1=", &self.state.button_l1);
        print_button_state(", button_l2=", &self.state.button_l2);
        print_button_state(", button_r1=", &self.state.button_r1);
        print_button_state(", button_r2=", &self.state.button_r2);

        print.print(", left_stick=");
        JoystickStateObj::print_state(&self.state.left_stick, print);

        print.print(", right_stick=");
        JoystickStateObj::print_state(&self.state.right_stick, print);

        print.print(")");
    }
}

#[class_methods]
impl ButtonStateObj {
    #[attr]
    #[stub(attrs = [
        "is_pressed: bool",
        "is_released: bool",
        "is_now_pressed: bool",
        "is_now_released: bool",
    ])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };
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

    #[binary_op]
    fn binary_op(op: BinaryOpCode, lhs: &Self, rhs: Obj) -> Obj {
        match op {
            BinaryOpCode::Equal => Obj::from_bool(lhs.state == rhs.as_obj::<Self>().state),
            _ => Obj::NULL,
        }
    }

    fn print_state(state: &ButtonState, print: &mut Print) {
        let _ = write!(
            print,
            "ButtonState(is_pressed={}, is_released={}, is_now_pressed={}, is_now_released={})",
            state.is_pressed(),
            state.is_released(),
            state.is_now_pressed(),
            state.is_now_released()
        );
    }

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        Self::print_state(&self.state, print);
    }
}

#[class_methods]
impl JoystickStateObj {
    #[attr]
    #[stub(attrs = ["x: float", "y: float", "x_raw: int", "y_raw: int"])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };
        result.return_value(match attr.as_str() {
            "x" => Obj::from_float(self.state.x() as f32),
            "y" => Obj::from_float(self.state.y() as f32),
            "x_raw" => Obj::from_int(self.state.x_raw() as i32),
            "y_raw" => Obj::from_int(self.state.y_raw() as i32),
            _ => return,
        });
    }

    #[binary_op]
    fn binary_op(op: BinaryOpCode, lhs: &Self, rhs: Obj) -> Obj {
        match op {
            BinaryOpCode::Equal => Obj::from_bool(lhs.state == rhs.as_obj::<Self>().state),
            _ => Obj::NULL,
        }
    }

    fn print_state(state: &JoystickState, print: &mut Print) {
        let _ = write!(
            print,
            "JoystickState(x={}, y={}, x_raw={}, y_raw={})",
            state.x(),
            state.y(),
            state.x_raw(),
            state.y_raw()
        );
    }

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        Self::print_state(&self.state, print);
    }
}
