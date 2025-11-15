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
    fun::fun2_from_fn, modvenice::units::rotation::RotationUnitObj, obj::alloc_obj, qstrgen::qstr,
};

const AI_VISION_OBJECT_LOCAL_DICT: Dict = const_dict![
    qstr!(angle) => Obj::from_static(&fun2_from_fn!(ai_vision_object_angle, &AiVisionObjectObj, &RotationUnitObj)),
];

static AI_VISION_OBJECT_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(AiVisionObject))
        .set_make_new(make_new_from_fn!(ai_vision_object_make_new));

pub(crate) static AI_VISION_COLOR_OBJECT_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(AiVisionColorObject))
        .set_slot_parent(AI_VISION_OBJECT_OBJ_TYPE.as_obj_type())
        .set_slot_locals_dict_from_static(&AI_VISION_OBJECT_LOCAL_DICT)
        .set_attr(attr_from_fn!(ai_vision_object_state_attr));

pub(crate) static AI_VISION_CODE_OBJECT_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(AiVisionCodeObject))
        .set_slot_parent(AI_VISION_OBJECT_OBJ_TYPE.as_obj_type())
        .set_slot_locals_dict_from_static(&AI_VISION_OBJECT_LOCAL_DICT)
        .set_attr(attr_from_fn!(ai_vision_object_state_attr));

pub(crate) static AI_VISION_APRIL_TAG_OBJECT_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(AiVisionAprilTagObject))
        .set_slot_parent(AI_VISION_OBJECT_OBJ_TYPE.as_obj_type())
        .set_slot_locals_dict_from_static(&AI_VISION_OBJECT_LOCAL_DICT)
        .set_attr(attr_from_fn!(ai_vision_object_state_attr));

pub(crate) static AI_VISION_MODEL_OBJECT_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(AiVisionModelObject))
        .set_slot_parent(AI_VISION_OBJECT_OBJ_TYPE.as_obj_type())
        .set_slot_locals_dict_from_static(&AI_VISION_OBJECT_LOCAL_DICT)
        .set_attr(attr_from_fn!(ai_vision_object_state_attr));

#[repr(C)]
pub struct AiVisionObjectObj {
    base: ObjBase<'static>,
    object: AiVisionObject,
}
unsafe impl ObjTrait for AiVisionObjectObj {
    const OBJ_TYPE: &ObjType = AI_VISION_OBJECT_OBJ_TYPE.as_obj_type();
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

fn ai_vision_object_make_new(ty: &'static ObjType, n_pos: usize, n_kw: usize, args: &[Obj]) -> Obj {
    raise_not_implemented_error(token().unwrap(), "inaccessible initializer")
}

fn ai_vision_object_state_attr(this: &AiVisionObjectObj, attr: Qstr, op: AttrOp) {
    let AttrOp::Load { dest } = op else { return };
    *dest = match attr.bytes() {
        b"id" => {
            let id = match this.object {
                AiVisionObject::Color { id, .. } => id,
                AiVisionObject::Code { id, .. } => id,
                AiVisionObject::AprilTag { id, .. } => id,
                AiVisionObject::Model { id, .. } => id,
            };
            Obj::from_int(id as _)
        }
        b"position_x" => {
            let pos = match this.object {
                AiVisionObject::Color { position, .. } => Some(position.x),
                AiVisionObject::Code { position, .. } => Some(position.x),
                AiVisionObject::AprilTag { .. } => None,
                AiVisionObject::Model { position, .. } => Some(position.x),
            };
            if let Some(v) = pos {
                Obj::from_int(v as _)
            } else {
                Obj::NONE
            }
        }
        b"position_y" => {
            let pos = match this.object {
                AiVisionObject::Color { position, .. } => Some(position.y),
                AiVisionObject::Code { position, .. } => Some(position.y),
                AiVisionObject::AprilTag { .. } => None,
                AiVisionObject::Model { position, .. } => Some(position.y),
            };
            if let Some(v) = pos {
                Obj::from_int(v as _)
            } else {
                Obj::NONE
            }
        }
        b"width" => {
            let width = match this.object {
                AiVisionObject::Color { width, .. } => Some(width),
                AiVisionObject::Code { width, .. } => Some(width),
                AiVisionObject::Model { width, .. } => Some(width),
                _ => None,
            };
            if let Some(w) = width {
                Obj::from_int(w as _)
            } else {
                Obj::NONE
            }
        }
        b"height" => {
            let height = match this.object {
                AiVisionObject::Color { height, .. } => Some(height),
                AiVisionObject::Code { height, .. } => Some(height),
                AiVisionObject::Model { height, .. } => Some(height),
                _ => None,
            };
            if let Some(h) = height {
                Obj::from_int(h as _)
            } else {
                Obj::NONE
            }
        }
        b"classification" => {
            if let AiVisionObject::Model { classification, .. } = &this.object {
                Obj::from_qstr(Qstr::from_bytes(classification.as_bytes()))
            } else {
                Obj::NONE
            }
        }
        b"confidence" => {
            if let AiVisionObject::Model { confidence, .. } = this.object {
                Obj::from_int(confidence as _)
            } else {
                Obj::NONE
            }
        }
        // Code's angle is implemented as a method
        _ => {
            if let AiVisionObject::AprilTag {
                top_left,
                top_right,
                bottom_left,
                bottom_right,
                ..
            } = this.object
            {
                let value = match attr.bytes() {
                    b"top_left_x" => Some(top_left.x),
                    b"top_left_y" => Some(top_left.y),
                    b"top_right_x" => Some(top_right.x),
                    b"top_right_y" => Some(top_right.y),
                    b"bottom_left_x" => Some(bottom_left.x),
                    b"bottom_left_y" => Some(bottom_left.y),
                    b"bottom_right_x" => Some(bottom_right.x),
                    b"bottom_right_y" => Some(bottom_right.y),
                    _ => None,
                };

                if let Some(value) = value {
                    Obj::from_int(value as _)
                } else {
                    Obj::NONE
                }
            } else {
                Obj::NONE
            }
        } // _ => return
    }
}

fn ai_vision_object_angle(this: &AiVisionObjectObj, angle_unit: &RotationUnitObj) -> Obj {
    if let AiVisionObject::Code { angle, .. } = this.object {
        Obj::from_float(angle_unit.unit().in_angle(angle))
    } else {
        Obj::NONE
    }
}
