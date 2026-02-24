use micropython_rs::{
    attr_from_fn, const_dict,
    except::raise_not_implemented_error,
    init::token,
    make_new_from_fn,
    map::Dict,
    obj::{AttrOp, Obj, ObjBase, ObjFullType, ObjTrait, ObjType, TypeFlags},
    qstr::Qstr,
};
use vexide_devices::smart::ai_vision::AiVisionObject;

use crate::{
    fun::fun2, modvenice::units::rotation::RotationUnitObj, obj::alloc_obj, qstrgen::qstr,
};

const AI_VISION_OBJECT_LOCAL_DICT: &Dict = const_dict![
    qstr!(angle) => Obj::from_static(&fun2!(ai_vision_object_angle, &AiVisionObjectObj, &RotationUnitObj)),
];

static AI_VISION_OBJECT_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(AiVisionObject))
        .set_make_new(make_new_from_fn!(ai_vision_object_make_new));

pub(crate) static AI_VISION_COLOR_OBJECT_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(AiVisionColorObject))
        .set_parent(AI_VISION_OBJECT_OBJ_TYPE.as_obj_type())
        .set_locals_dict(AI_VISION_OBJECT_LOCAL_DICT)
        .set_attr(attr_from_fn!(ai_vision_object_state_attr));

pub(crate) static AI_VISION_CODE_OBJECT_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(AiVisionCodeObject))
        .set_parent(AI_VISION_OBJECT_OBJ_TYPE.as_obj_type())
        .set_locals_dict(AI_VISION_OBJECT_LOCAL_DICT)
        .set_attr(attr_from_fn!(ai_vision_object_state_attr));

pub(crate) static AI_VISION_APRIL_TAG_OBJECT_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(AiVisionAprilTagObject))
        .set_parent(AI_VISION_OBJECT_OBJ_TYPE.as_obj_type())
        .set_locals_dict(AI_VISION_OBJECT_LOCAL_DICT)
        .set_attr(attr_from_fn!(ai_vision_object_state_attr));

pub(crate) static AI_VISION_MODEL_OBJECT_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(AiVisionModelObject))
        .set_parent(AI_VISION_OBJECT_OBJ_TYPE.as_obj_type())
        .set_locals_dict(AI_VISION_OBJECT_LOCAL_DICT)
        .set_attr(attr_from_fn!(ai_vision_object_state_attr));

#[repr(C)]
pub struct AiVisionObjectObj {
    base: ObjBase<'static>,
    object: AiVisionObject,
}
unsafe impl ObjTrait for AiVisionObjectObj {
    const OBJ_TYPE: &ObjType = AI_VISION_OBJECT_OBJ_TYPE.as_obj_type();

    fn coercable(ty: &ObjType) -> bool {
        ty == AI_VISION_COLOR_OBJECT_OBJ_TYPE.as_obj_type()
            || ty == AI_VISION_CODE_OBJECT_OBJ_TYPE.as_obj_type()
            || ty == AI_VISION_APRIL_TAG_OBJECT_OBJ_TYPE.as_obj_type()
            || ty == AI_VISION_MODEL_OBJECT_OBJ_TYPE.as_obj_type()
    }
}
impl AiVisionObjectObj {
    pub fn create_obj(object: AiVisionObject) -> Obj {
        let ty = match object {
            AiVisionObject::AprilTag { .. } => AI_VISION_APRIL_TAG_OBJECT_OBJ_TYPE.as_obj_type(),
            AiVisionObject::Model { .. } => AI_VISION_MODEL_OBJECT_OBJ_TYPE.as_obj_type(),
            AiVisionObject::Code { .. } => AI_VISION_CODE_OBJECT_OBJ_TYPE.as_obj_type(),
            AiVisionObject::Color { .. } => AI_VISION_COLOR_OBJECT_OBJ_TYPE.as_obj_type(),
        };
        alloc_obj(AiVisionObjectObj {
            base: ObjBase::new(ty),
            object,
        })
    }
}

fn ai_vision_object_make_new(
    _ty: &'static ObjType,
    _n_pos: usize,
    _n_kw: usize,
    _args: &[Obj],
) -> Obj {
    raise_not_implemented_error(token(), c"inaccessible initializer")
}

fn ai_vision_object_state_attr(this: &AiVisionObjectObj, attr: Qstr, op: AttrOp) {
    let AttrOp::Load { result } = op else { return };
    result.return_value(match attr.as_str() {
        "id" => {
            let id = match this.object {
                AiVisionObject::Color { id, .. } => id,
                AiVisionObject::Code { id, .. } => id,
                AiVisionObject::AprilTag { id, .. } => id,
                AiVisionObject::Model { id, .. } => id,
            };
            Obj::from_int(id as _)
        }
        "position_x" => {
            let pos = match this.object {
                AiVisionObject::Color { position, .. } => position.x,
                AiVisionObject::Code { position, .. } => position.x,
                AiVisionObject::Model { position, .. } => position.x,
                AiVisionObject::AprilTag { .. } => return,
            };
            Obj::from_int(pos as _)
        }
        "position_y" => {
            let pos = match this.object {
                AiVisionObject::Color { position, .. } => position.y,
                AiVisionObject::Code { position, .. } => position.y,
                AiVisionObject::Model { position, .. } => position.y,
                AiVisionObject::AprilTag { .. } => return,
            };
            Obj::from_int(pos as _)
        }
        "width" => {
            let width = match this.object {
                AiVisionObject::Color { width, .. } => width,
                AiVisionObject::Code { width, .. } => width,
                AiVisionObject::Model { width, .. } => width,
                _ => return,
            };
            Obj::from_int(width as _)
        }
        "height" => {
            let height = match this.object {
                AiVisionObject::Color { height, .. } => height,
                AiVisionObject::Code { height, .. } => height,
                AiVisionObject::Model { height, .. } => height,
                _ => return,
            };
            Obj::from_int(height as _)
        }
        "classification" => {
            let AiVisionObject::Model { classification, .. } = &this.object else {
                return;
            };
            Obj::from_qstr(Qstr::from_str(classification.as_str()))
        }
        "confidence" => {
            let AiVisionObject::Model { confidence, .. } = this.object else {
                return;
            };
            Obj::from_int(confidence as _)
        }
        // Code's angle is implemented as a method
        _ => {
            let AiVisionObject::AprilTag {
                top_left,
                top_right,
                bottom_left,
                bottom_right,
                ..
            } = this.object
            else {
                return;
            };

            let value = match attr.as_str() {
                "top_left_x" => top_left.x,
                "top_left_y" => top_left.y,
                "top_right_x" => top_right.x,
                "top_right_y" => top_right.y,
                "bottom_left_x" => bottom_left.x,
                "bottom_left_y" => bottom_left.y,
                "bottom_right_x" => bottom_right.x,
                "bottom_right_y" => bottom_right.y,
                _ => return,
            };

            Obj::from_int(value as _)
        }
    });
}

fn ai_vision_object_angle(this: &AiVisionObjectObj, angle_unit: &RotationUnitObj) -> Obj {
    if let AiVisionObject::Code { angle, .. } = this.object {
        Obj::from_float(angle_unit.unit().angle_to_float(angle))
    } else {
        Obj::NONE
    }
}
