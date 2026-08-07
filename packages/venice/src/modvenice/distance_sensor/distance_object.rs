use std::fmt::Write;

use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{AttrOp, Obj, ObjBase, ObjTrait},
    print::{Print, PrintKind},
    qstr::Qstr,
};
use vexide_devices::smart::distance::DistanceObject;

use crate::modvenice::read_only_attr::read_only_attr;

/// Readings from a physical object detected by a Distance Sensor.
///
/// Instances are returned by `DistanceSensor.get_object` and can't be constructed directly. All
/// attributes are read-only.
///
/// - `distance` is the distance of the object from the sensor in millimeters.
/// - `relative_size` is a guess at the object's "relative size". This is a unitless value from 0 to
///   400. An 18" x 30" grey card returns approximately 75 in typical room lighting. If the sensor
///   isn't able to determine an object's size, `None` is returned. It's unknown what the sensor is
///   actually measuring here, so use this data with a grain of salt.
/// - `velocity` is the approach velocity of the object in m/s. This is calculated by the Brain by
///   differentiating `distance` with respect to time and applying a simple low-pass filter.
/// - `confidence` is the confidence in the distance measurement from 0.0 to 1.0.
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
    /// Loads the read-only `confidence`, `distance`, `velocity`, and `relative_size` readings
    /// described by `DistanceObject`.
    ///
    /// # Raises
    ///
    /// - `TypeError`: If an attribute is assigned or deleted.
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

    /// Formats all available readings as `DistanceObject(...)`.
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
