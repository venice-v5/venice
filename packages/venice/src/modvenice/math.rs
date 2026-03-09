use std::cell::Cell;

use argparse::{ArgType, error_msg};
use micropython_rs::{
    class, class_methods,
    except::raise_type_error,
    init::token,
    obj::{AttrOp, Obj, ObjBase, ObjTrait},
    qstr::Qstr,
};
use vexide_devices::math::Angle;

use crate::modvenice::units::rotation::RotationUnit;

#[class(qstr!(Vec3))]
#[repr(C)]
pub struct Vec3 {
    base: ObjBase<'static>,
    x: Cell<f32>,
    y: Cell<f32>,
    z: Cell<f32>,
}

#[class(qstr!(Quaternion))]
#[repr(C)]
pub struct Quaternion {
    base: ObjBase<'static>,
    // i
    x: Cell<f32>,
    // j
    y: Cell<f32>,
    // k
    z: Cell<f32>,
    // real
    w: Cell<f32>,
}

#[class(qstr!(EulerAngles))]
#[repr(C)]
pub struct EulerAngles {
    base: ObjBase<'static>,
    // z
    yaw: Cell<f32>,
    // y
    pitch: Cell<f32>,
    // x
    roll: Cell<f32>,
}

#[class_methods]
impl Vec3 {
    pub fn new(v: vexide_devices::math::Vector3<f64>) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            x: Cell::new(v.x as f32),
            y: Cell::new(v.y as f32),
            z: Cell::new(v.z as f32),
        }
    }

    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let coord = match attr.as_str() {
            "x" => &self.x,
            "y" => &self.y,
            "z" => &self.z,
            _ => return,
        };

        handle_op(op, coord);
    }
}

#[class_methods]
impl Quaternion {
    pub fn new(q: vexide_devices::math::Quaternion<f64>) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            x: Cell::new(q.v.x as f32),
            y: Cell::new(q.v.y as f32),
            z: Cell::new(q.v.z as f32),
            w: Cell::new(q.s as f32),
        }
    }

    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let val = match attr.as_str() {
            "x" => &self.x,
            "y" => &self.y,
            "z" => &self.z,
            "w" => &self.w,
            _ => return,
        };

        handle_op(op, val);
    }
}

#[class_methods]
impl EulerAngles {
    pub fn new<B>(e: vexide_devices::math::EulerAngles<Angle, B>, unit: RotationUnit) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            yaw: Cell::new(unit.angle_to_float(e.b)),
            pitch: Cell::new(unit.angle_to_float(e.a)),
            roll: Cell::new(unit.angle_to_float(e.c)),
        }
    }

    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let val = match attr.as_str() {
            "yaw" | "z" => &self.yaw,
            "pitch" | "y" => &self.pitch,
            "roll" | "x" => &self.roll,
            _ => return,
        };

        handle_op(op, val);
    }
}

fn handle_op(op: AttrOp, val: &Cell<f32>) {
    match op {
        AttrOp::Load { result } => result.return_value(Obj::from_float(val.get())),
        AttrOp::Store { src, result } => {
            if let Some(f) = src.try_to_int().map(|i| i as f32).or(src.try_to_float()) {
                val.set(f);
                result.success();
            } else {
                raise_type_error(
                    token(),
                    error_msg!("expected f32, got <{}>", ArgType::of(&src)),
                );
            }
        }
        AttrOp::Delete { result } => {
            val.set(0.0);
            // "sucess" bruh
            result.sucess();
        }
    }
}
