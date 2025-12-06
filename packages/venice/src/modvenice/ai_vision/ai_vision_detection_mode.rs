use micropython_rs::{
    const_dict,
    except::raise_type_error,
    init::token,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, TypeFlags},
    ops::BinaryOp,
};
use vexide_devices::smart::ai_vision::AiVisionDetectionMode;

use crate::{args::ArgType, obj::alloc_obj, qstrgen::qstr};

static AI_VISION_DETECTION_MODE_OBJ_TYPE: ObjFullType =
    ObjFullType::new(TypeFlags::empty(), qstr!(AiVisionDetectionMode))
        .set_slot_locals_dict_from_static(const_dict![
            qstr!(APRILTAG) => Obj::from_static(&AiVisionDetectionModeObj::APRILTAG),
            qstr!(COLOR) => Obj::from_static(&AiVisionDetectionModeObj::COLOR),
            qstr!(MODEL) => Obj::from_static(&AiVisionDetectionModeObj::MODEL),
            qstr!(COLOR_MERGE) => Obj::from_static(&AiVisionDetectionModeObj::COLOR_MERGE)
        ])
        .set_slot_binary_op(ai_vision_detection_mode_binary_op);

#[repr(C)]
pub struct AiVisionDetectionModeObj {
    base: ObjBase<'static>,
    mode: AiVisionDetectionMode,
}

unsafe impl ObjTrait for AiVisionDetectionModeObj {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = AI_VISION_DETECTION_MODE_OBJ_TYPE.as_obj_type();
}

impl AiVisionDetectionModeObj {
    pub const APRILTAG: Self = Self::new(AiVisionDetectionMode::APRILTAG);
    pub const COLOR: Self = Self::new(AiVisionDetectionMode::COLOR);
    pub const MODEL: Self = Self::new(AiVisionDetectionMode::MODEL);
    pub const COLOR_MERGE: Self = Self::new(AiVisionDetectionMode::COLOR_MERGE);

    pub const fn new(mode: AiVisionDetectionMode) -> Self {
        Self {
            base: ObjBase::new(AI_VISION_DETECTION_MODE_OBJ_TYPE.as_obj_type()),
            mode,
        }
    }

    pub fn mode(&self) -> AiVisionDetectionMode {
        self.mode
    }
}
extern "C" fn ai_vision_detection_mode_binary_op(op: BinaryOp, obj_1: Obj, obj_2: Obj) -> Obj {
    if let BinaryOp::Or = op {
    } else if let BinaryOp::InplaceOr = op {
    } else {
        return Obj::NULL;
    }
    let lhs = obj_1
        .try_to_obj::<AiVisionDetectionModeObj>()
        .unwrap_or_else(|| {
            raise_type_error(
                token().unwrap(),
                format!(
                    "expected <AiVisionFlags> for argument #1, found <{}>",
                    ArgType::of(&obj_1)
                ),
            )
        })
        .mode;
    let rhs = obj_2
        .try_to_obj::<AiVisionDetectionModeObj>()
        .unwrap_or_else(|| {
            raise_type_error(
                token().unwrap(),
                format!(
                    "expected <AiVisionFlags> for argument #2, found <{}>",
                    ArgType::of(&obj_2)
                ),
            )
        })
        .mode;
    alloc_obj(AiVisionDetectionModeObj::new(lhs.union(rhs)))
}
