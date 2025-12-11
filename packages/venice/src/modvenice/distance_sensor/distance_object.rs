use micropython_rs::{
    attr_from_fn,
    obj::{AttrOp, Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
    qstr::Qstr,
};
use vexide_devices::smart::distance::DistanceObject;

use crate::qstrgen::qstr;

#[repr(C)]
pub struct DistanceObjectObj {
    base: ObjBase<'static>,
    object: DistanceObject,
}

pub static DISTANCE_OBJECT_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(DistanceObject))
        .set_attr(attr_from_fn!(distance_object_attr));

unsafe impl ObjTrait for DistanceObjectObj {
    const OBJ_TYPE: &ObjType = DISTANCE_OBJECT_OBJ_TYPE.as_obj_type();
}

impl DistanceObjectObj {
    pub fn new(object: DistanceObject) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            object,
        }
    }
}

fn distance_object_attr(this: &DistanceObjectObj, attr: Qstr, op: AttrOp) {
    match op {
        AttrOp::Load { dest } => {
            let attr_bytes = attr.bytes();
            *dest = match attr_bytes {
                b"confidence" => Obj::from_float(this.object.confidence as _),
                b"distance" => Obj::from_int(this.object.distance as _),
                b"relative_size" => {
                    if let Some(v) = this.object.relative_size {
                        Obj::from_int(v as _)
                    } else {
                        Obj::NONE
                    }
                }
                b"velocity" => Obj::from_float(this.object.velocity as _),
                _ => return,
            };
        }
        _ => {}
    }
}
