use std::cell::Cell;

use argparse::{ArgType, Args, error_msg};
use micropython_rs::{
    class, class_methods,
    except::type_error,
    init::token,
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    qstr::Qstr,
};
use mint::{EulerAngles, IntraZYX};
use vexide_devices::math::Angle;

use crate::modvenice::{Exception, units::rotation::RotationUnit};

#[class(qstr!(Vec3))]
#[repr(C)]
pub struct Vec3 {
    base: ObjBase,
    x: Cell<f32>,
    y: Cell<f32>,
    z: Cell<f32>,
}

#[class(qstr!(Quaternion))]
#[repr(C)]
pub struct Quaternion {
    base: ObjBase,
    // i
    x: Cell<f32>,
    // j
    y: Cell<f32>,
    // k
    z: Cell<f32>,
    // real
    w: Cell<f32>,
}

#[class(qstr!(EulerZYX))]
#[repr(C)]
pub struct EulerZYX {
    base: ObjBase,
    yaw: Cell<f32>,
    pitch: Cell<f32>,
    roll: Cell<f32>,
}

#[class(qstr!(Point2))]
#[derive(Clone)]
#[repr(C)]
pub struct Point2 {
    base: ObjBase,
    x: Cell<f32>,
    y: Cell<f32>,
}

#[class_methods]
impl Vec3 {
    pub fn new(v: vexide_devices::math::Vector3<f64>) -> Self {
        Self {
            base: Self::OBJ_TYPE.into(),
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
            base: Self::OBJ_TYPE.into(),
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
impl EulerZYX {
    pub fn new(e: EulerAngles<Angle, IntraZYX>, unit: RotationUnit) -> Self {
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
            "yaw" => &self.yaw,
            "pitch" => &self.pitch,
            "roll" => &self.roll,
            _ => return,
        };

        handle_op(op, val);
    }
}

#[class_methods]
impl Point2 {
    #[make_new]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(2, 2).assert_nkw(0, 0);

        let x = reader.next_positional()?;
        let y = reader.next_positional()?;

        Ok(Self {
            base: ObjBase::new(ty),
            x: Cell::new(x),
            y: Cell::new(y),
        })
    }

    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let coord = match attr.as_str() {
            "x" => &self.x,
            "y" => &self.x,
            _ => return,
        };

        handle_op(op, coord);
    }

    pub fn from_vexide_point2(point2: vexide_devices::math::Point2<f64>) -> Self {
        Self {
            base: Self::OBJ_TYPE.into(),
            x: Cell::new(point2.x as f32),
            y: Cell::new(point2.y as f32),
        }
    }

    pub fn as_vexide_point2(&self) -> vexide_devices::math::Point2<f64> {
        vexide_devices::math::Point2 {
            x: self.x.get() as f64,
            y: self.y.get() as f64,
        }
    }
}

impl From<vexide_devices::math::Point2<f64>> for Point2 {
    fn from(value: vexide_devices::math::Point2<f64>) -> Self {
        Self::from_vexide_point2(value)
    }
}

pub fn handle_op(op: AttrOp, val: &Cell<f32>) {
    match op {
        AttrOp::Load { result } => result.return_value(Obj::from_float(val.get())),
        AttrOp::Store { src, result } => {
            if let Some(f) = src.try_to_int().map(|i| i as f32).or(src.try_to_float()) {
                val.set(f);
                result.success();
            } else {
                type_error(error_msg!("expected f32, found <{}>", ArgType::of(&src)))
                    .raise(token());
            }
        }
        AttrOp::Delete { result } => {
            val.set(0.0);
            // "sucess" bruh
            result.sucess();
        }
    }
}
