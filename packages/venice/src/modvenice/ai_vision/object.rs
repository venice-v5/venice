use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{AttrOp, Obj, ObjBase, ObjTrait},
    qstr::Qstr,
    str::Str,
};
use vexide_devices::{
    math::{Angle, Point2},
    smart::ai_vision::AiVisionObject,
};

use crate::{
    modvenice::{read_only_attr::read_only_attr, units::rotation::RotationUnitObj},
    obj::alloc_obj,
};

#[class(qstr!(AiVisionColorObject))]
#[repr(C)]
pub struct Color {
    base: ObjBase,
    position: Point2<u16>,
    width: u16,
    height: u16,
    id: u8,
}

#[class_methods]
impl Color {
    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };
        result.return_value(match attr.as_str() {
            "id" => self.id as i32,
            "x" => self.position.x as i32,
            "y" => self.position.y as i32,
            "width" => self.width as i32,
            "height" => self.height as i32,
            _ => return,
        })
    }
}

#[class(qstr!(AiVisionCodeObject))]
#[repr(C)]
pub struct Code {
    base: ObjBase,
    position: Point2<u16>,
    width: u16,
    height: u16,
    angle_radians: f32,
    id: u8,
}

#[class_methods]
impl Code {
    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };
        result.return_value(match attr.as_str() {
            "id" => self.id as i32,
            "x" => self.position.x as i32,
            "y" => self.position.y as i32,
            "width" => self.width as i32,
            "height" => self.height as i32,
            _ => return,
        })
    }

    #[method]
    fn get_angle(&self, unit: &RotationUnitObj) -> f32 {
        unit.unit()
            .angle_to_float(Angle::from_radians(self.angle_radians as f64))
    }
}

#[class(qstr!(AiVisionAprilTagObject))]
#[repr(C)]
pub struct AprilTag {
    base: ObjBase,
    top_left: Point2<i16>,
    top_right: Point2<i16>,
    bottom_right: Point2<i16>,
    bottom_left: Point2<i16>,
    id: u8,
}

#[class_methods]
impl AprilTag {
    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };
        result.return_value(match attr.as_str() {
            "top_left_x" => self.top_left.x.into(),
            "top_left_y" => self.top_left.y.into(),
            "top_right_x" => self.top_right.x.into(),
            "top_right_y" => self.top_right.y.into(),
            "bottom_left_x" => self.bottom_left.x.into(),
            "bottom_left_y" => self.bottom_left.y.into(),
            "bottom_right_x" => self.bottom_right.x.into(),
            "bottom_right_y" => self.bottom_right.y.into(),
            "id" => self.id as i32,
            _ => return,
        })
    }
}

#[class(qstr!(AiVisionModelObject))]
#[repr(C)]
pub struct Model {
    base: ObjBase,
    classification: Obj,
    position: Point2<u16>,
    width: u16,
    height: u16,
    confidence: u16,
    id: u8,
}

#[class_methods]
impl Model {
    #[attr]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };
        result.return_value(match attr.as_str() {
            "id" => (self.id as i32).into(),
            "classification" => self.classification,
            "x" => (self.position.x as i32).into(),
            "y" => (self.position.y as i32).into(),
            "width" => (self.width as i32).into(),
            "height" => (self.height as i32).into(),
            "confidence" => (self.confidence as i32).into(),
            _ => return,
        })
    }
}

pub fn create_obj(object: AiVisionObject) -> Obj {
    match object {
        AiVisionObject::Color {
            id,
            position,
            width,
            height,
        } => alloc_obj(Color {
            base: ObjBase::new(Color::OBJ_TYPE),
            position,
            width,
            height,
            id,
        }),
        AiVisionObject::Code {
            id,
            position,
            width,
            height,
            angle,
        } => alloc_obj(Code {
            base: ObjBase::new(Code::OBJ_TYPE),
            position,
            width,
            height,
            angle_radians: angle.as_radians() as f32,
            id,
        }),
        AiVisionObject::AprilTag {
            id,
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        } => alloc_obj(AprilTag {
            base: ObjBase::new(AprilTag::OBJ_TYPE),
            top_left,
            top_right,
            bottom_left,
            bottom_right,
            id,
        }),
        AiVisionObject::Model {
            id,
            classification,
            position,
            width,
            height,
            confidence,
        } => alloc_obj(Model {
            base: ObjBase::new(Model::OBJ_TYPE),
            classification: Str::new(&classification),
            position,
            width,
            height,
            confidence,
            id,
        }),
    }
}
