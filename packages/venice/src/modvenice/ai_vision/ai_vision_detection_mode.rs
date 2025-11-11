use micropython_rs::{
    const_dict,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, TypeFlags},
};
use crate::qstrgen::qstr;

static AI_VISION_DETECTION_MODE_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(AiVisionDetectionMode))
    .set_slot_locals_dict_from_static(&const_dict![
        qstr!(APRILTAG) => Obj::from_static(&AiVisionDetectionModeObj::APRILTAG),
        qstr!(COLOR) => Obj::from_static(&AiVisionDetectionModeObj::COLOR),
        qstr!(MODEL) => Obj::from_static(&AiVisionDetectionModeObj::MODEL),
        qstr!(COLOR_MERGE) => Obj::from_static(&AiVisionDetectionModeObj::COLOR_MERGE)
    ]);
    // TODO!: add binary or operator for unions

#[repr(C)]
pub struct AiVisionDetectionModeObj {
    base: ObjBase<'static>,
    mode: AiVisionDetectionMode,
}

unsafe impl ObjTrait for AiVisionDetectionModeObj {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = AI_VISION_DETECTION_MODE_OBJ_TYPE.as_obj_type();
}

// The enum as specified by the user.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct AiVisionDetectionMode(pub u8);

impl AiVisionDetectionMode {
    pub const APRILTAG: Self = Self(1 << 0);
    pub const COLOR: Self = Self(1 << 1);
    pub const MODEL: Self = Self(1 << 2);
    pub const COLOR_MERGE: Self = Self(1 << 4);
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

    pub const fn mode(&self) -> AiVisionDetectionMode {
        self.mode
    }
}
