use std::fmt::Write;

use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{AttrOp, Obj, ObjBase, ObjTrait},
    print::{Print, PrintKind},
    qstr::Qstr,
};
use vexide_devices::{
    math::{Angle, Point2},
    smart::vision::VisionObject,
};

use crate::modvenice::{read_only_attr::read_only_attr, units::rotation::RotationUnitObj};

/// A detected vision object.
///
/// This root-importable class contains metadata about objects detected by the Vision Sensor.
/// Instances are returned by `VisionSensor.get_objects` after adding signatures and color codes to
/// the sensor, and cannot be constructed directly. All attributes are read-only.
///
/// - `source` is the signature or color code used to detect this object.
/// - `width` is the width of the detected object's bounding box in pixels.
/// - `height` is the height of the detected object's bounding box in pixels.
/// - `offset_x` and `offset_y` are the top-left coordinate of the detected object relative to the
///   top-left of the camera's field of view.
/// - `center_x` and `center_y` are the center coordinate of the detected object relative to the
///   top-left of the camera's field of view.
///
/// The readable representation summarizes the source and all six pixel measurements.
#[class(qstr!(VisionObject))]
#[repr(C)]
pub struct VisionObjectObj {
    base: ObjBase,
    source: Obj,
    width: u16,
    height: u16,
    offset: Point2<u16>,
    center: Point2<u16>,
    angle_radians: f32,
}

#[class_methods]
impl VisionObjectObj {
    pub fn new(object: VisionObject) -> Self {
        Self {
            base: Self::OBJ_TYPE.into(),
            source: super::source::new(object.source),
            width: object.width,
            height: object.height,
            offset: object.offset,
            center: object.center,
            angle_radians: object.angle.as_radians() as f32,
        }
    }

    #[attr]
    #[stub(attrs = [
        "source: DetectionSource",
        "width: int",
        "height: int",
        "offset_x: int",
        "offset_y: int",
        "center_x: int",
        "center_y: int",
    ])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };
        result.return_value(match attr.as_str() {
            "source" => self.source,
            "width" => (self.width as i32).into(),
            "height" => (self.height as i32).into(),
            "offset_x" => (self.offset.x as i32).into(),
            "offset_y" => (self.offset.y as i32).into(),
            "center_x" => (self.center.x as i32).into(),
            "center_y" => (self.center.y as i32).into(),
            _ => return,
        })
    }

    /// Returns the approximate rotation of the detected object's bounding box in `unit`.
    ///
    /// The underlying sensor reports tenths of a degree, which Venice converts to `DEGREES`, `RADIANS`, or
    /// `TURNS`.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sensor = VisionSensor(1)
    /// for object in sensor.get_objects():
    ///     print(object.get_angle(DEGREES))
    /// ```
    #[method]
    fn get_angle(&self, unit: &RotationUnitObj) -> f32 {
        unit.unit()
            .angle_to_float(Angle::from_radians(self.angle_radians as f64))
    }

    #[printer]
    fn printer(&self, print: &mut Print, kind: PrintKind) {
        print.print("VisionObject(source=");
        let _ = self.source.print(print, kind);
        let _ = write!(
            print,
            ", width={}, height={}, offset_x={}, offset_y={}, center_x={}, center_y={})",
            self.width, self.height, self.offset.x, self.offset.y, self.center.x, self.center.y
        );
    }
}
