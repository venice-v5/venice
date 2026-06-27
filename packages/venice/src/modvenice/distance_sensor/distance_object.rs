use std::fmt::Write;

use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{AttrOp, Obj, ObjBase, ObjTrait},
    print::{Print, PrintKind},
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
    #[stub(attrs = [
        "confidence: float",
        "distance: int",
        "velocity: float",
        "relative_size: int | None",
    ])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };
        result.return_value(match attr.as_str() {
            "confidence" => Obj::from_float(self.object.confidence as _),
            "distance" => Obj::from_int(self.object.distance as _),
            "velocity" => Obj::from_float(self.object.velocity as _),
            "relative_size" => self.object.relative_size.map(|v| v as i32).into(),
            _ => return,
        });
    }

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        let _ = write!(
            print,
            "DistanceObject(confidence={}, distance={}, velocity={}",
            self.object.confidence, self.object.distance, self.object.velocity
        );
        if let Some(relative_size) = self.object.relative_size {
            let _ = write!(print, ", relative_size={}", relative_size);
        }
        print.print(")");
    }
}
