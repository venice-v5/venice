use micropython_rs::{
    const_dict,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, TypeFlags},
};
use vexide_devices::{math::Direction, smart::ai_vision::AiVisionFlags};

use crate::qstrgen::qstr;

static AI_VISION_FLAGS_OBJ_TYPE: ObjFullType = ObjFullType::new(TypeFlags::empty(), qstr!(AiVisionFlags))
    .set_slot_locals_dict_from_static(&const_dict![
        qstr!(DISABLE_APRILTAG) => Obj::from_static(&AiVisionFlagsObj::DISABLE_APRILTAG),
        qstr!(DISABLE_COLOR) => Obj::from_static(&AiVisionFlagsObj::DISABLE_COLOR),
        qstr!(DISABLE_MODEL) => Obj::from_static(&AiVisionFlagsObj::DISABLE_MODEL),
        qstr!(COLOR_MERGE) => Obj::from_static(&AiVisionFlagsObj::COLOR_MERGE),
        qstr!(DISABLE_STATUS_OVERLAY) => Obj::from_static(&AiVisionFlagsObj::DISABLE_STATUS_OVERLAY),
        qstr!(DISABLE_USB_OVERLAY) => Obj::from_static(&AiVisionFlagsObj::DISABLE_USB_OVERLAY)
    ]);
    // TODO!: add binary or operator for unions

#[repr(C)]
pub struct AiVisionFlagsObj {
    base: ObjBase<'static>,
    flags: AiVisionFlags,
}

unsafe impl ObjTrait for AiVisionFlagsObj {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = AI_VISION_FLAGS_OBJ_TYPE.as_obj_type();
}

impl AiVisionFlagsObj {
    pub const DISABLE_APRILTAG: Self = Self::new(AiVisionFlags::DISABLE_APRILTAG);
    pub const DISABLE_COLOR: Self = Self::new(AiVisionFlags::DISABLE_COLOR);
    pub const DISABLE_MODEL: Self = Self::new(AiVisionFlags::DISABLE_MODEL);
    pub const COLOR_MERGE: Self = Self::new(AiVisionFlags::COLOR_MERGE);
    pub const DISABLE_STATUS_OVERLAY: Self = Self::new(AiVisionFlags::DISABLE_STATUS_OVERLAY);
    pub const DISABLE_USB_OVERLAY: Self = Self::new(AiVisionFlags::DISABLE_USB_OVERLAY);


    pub const fn new(flags: AiVisionFlags) -> Self {
        Self {
            base: ObjBase::new(AI_VISION_FLAGS_OBJ_TYPE.as_obj_type()),
            flags,
        }
    }

    pub const fn flags(&self) -> AiVisionFlags {
        self.flags
    }
}
