use micropython_rs::{
    class, class_methods,
    obj::{AttrOp, Obj, ObjBase, ObjTrait},
    qstr::Qstr,
};
use vexide_devices::smart::distance::DistanceObject;

use crate::qstrgen::qstr;

#[class(qstr!(DistanceObject))]
#[repr(C)]
pub struct DistanceObjectObj {
    base: ObjBase<'static>,
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
        let AttrOp::Load { result } = op else { return };
        result.return_value(match attr.as_str() {
            "confidence" => Obj::from_float(self.object.confidence as _),
            "distance" => Obj::from_int(self.object.distance as _),
            "relative_size" => Obj::from_int(self.object.relative_size as _),
            "velocity" => Obj::from_float(self.object.velocity as _),
            _ => return,
        });
    }
}
