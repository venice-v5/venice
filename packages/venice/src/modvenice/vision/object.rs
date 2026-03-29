use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{AttrOp, Obj, ObjBase, ObjTrait},
    qstr::Qstr,
};
use vexide_devices::{
    math::{Angle, Point2},
    smart::vision::VisionObject,
};

use crate::modvenice::units::rotation::RotationUnitObj;

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
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else { return };
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

    #[method]
    fn get_angle(&self, unit: &RotationUnitObj) -> f32 {
        unit.unit()
            .angle_to_float(Angle::from_radians(self.angle_radians as f64))
    }
}
