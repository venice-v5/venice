use std::cell::Cell;

use micropython_rs::{
    attr_from_fn,
    except::raise_type_error,
    init::token,
    obj::{AttrOp, Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
    qstr::Qstr,
};
use vexide_devices::math::Angle;

use crate::{
    args::ArgType, error_msg::error_msg, modvenice::units::rotation::RotationUnit, qstrgen::qstr,
};

#[repr(C)]
pub struct Vec3 {
    base: ObjBase<'static>,
    x: Cell<f32>,
    y: Cell<f32>,
    z: Cell<f32>,
}

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

pub static VEC3_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(Vec3)).set_attr(attr_from_fn!(vec3_attr));
pub static QUATERNION_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(Quaternion))
        .set_attr(attr_from_fn!(quaternion_attr));
pub static EULER_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(EulerAngles)).set_attr(attr_from_fn!(euler_attr));

unsafe impl ObjTrait for Vec3 {
    const OBJ_TYPE: &ObjType = VEC3_OBJ_TYPE.as_obj_type();
}

unsafe impl ObjTrait for Quaternion {
    const OBJ_TYPE: &ObjType = QUATERNION_OBJ_TYPE.as_obj_type();
}

unsafe impl ObjTrait for EulerAngles {
    const OBJ_TYPE: &ObjType = EULER_OBJ_TYPE.as_obj_type();
}

impl Vec3 {
    pub fn new(v: vexide_devices::math::Vector3<f64>) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            x: Cell::new(v.x as f32),
            y: Cell::new(v.y as f32),
            z: Cell::new(v.z as f32),
        }
    }
}

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
}

impl EulerAngles {
    pub fn new<B>(e: vexide_devices::math::EulerAngles<Angle, B>, unit: RotationUnit) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            yaw: Cell::new(unit.angle_to_float(e.b)),
            pitch: Cell::new(unit.angle_to_float(e.a)),
            roll: Cell::new(unit.angle_to_float(e.c)),
        }
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

fn vec3_attr(this: &Vec3, attr: Qstr, op: AttrOp) {
    let coord = match attr.as_str() {
        "x" => &this.x,
        "y" => &this.y,
        "z" => &this.z,
        _ => return,
    };

    handle_op(op, coord);
}

fn quaternion_attr(this: &Quaternion, attr: Qstr, op: AttrOp) {
    let val = match attr.as_str() {
        "x" => &this.x,
        "y" => &this.y,
        "z" => &this.z,
        "w" => &this.w,
        _ => return,
    };

    handle_op(op, val);
}

fn euler_attr(this: &EulerAngles, attr: Qstr, op: AttrOp) {
    let val = match attr.as_str() {
        "yaw" | "z" => &this.yaw,
        "pitch" | "y" => &this.pitch,
        "roll" | "x" => &this.roll,
        _ => return,
    };

    handle_op(op, val);
}
