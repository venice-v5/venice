use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{AttrOp, Obj, ObjBase, ObjTrait},
    qstr::Qstr,
};
use vexide_devices::smart::distance::DistanceObject;

use crate::modvenice::read_only_attr::read_only_attr;

#[class(qstr!(DistanceObject))]
#[repr(C)]
pub struct DistanceObjectObj {
    base: ObjBase,
    object: DistanceObject,
}

impl DistanceObjectObj {
    pub fn new(object: DistanceObject) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            object,
        }
    }
}

#[class_methods]
impl DistanceObjectObj {
    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };
        result.return_value(match attr.as_str() {
            "confidence" => Obj::from_float(self.object.confidence as _),
            "distance" => Obj::from_int(self.object.distance as _),
            "relative_size" => self.object.relative_size.map(|v| v as i32).into(),
            "velocity" => Obj::from_float(self.object.velocity as _),
            _ => return,
        });
    }
}
