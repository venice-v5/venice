use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{Obj, ObjBase, ObjTrait},
    ops::BinaryOpCode,
};
use vexide_devices::smart::ai_vision::AiVisionFlags;

use crate::obj::alloc_obj;

#[class(qstr!(AiVisionFlags))]
#[repr(C)]
pub struct AiVisionFlagsObj {
    base: ObjBase,
    flags: AiVisionFlags,
}

#[class_methods]
impl AiVisionFlagsObj {
    #[constant]
    pub const DISABLE_APRILTAG: &Self = &Self::new(AiVisionFlags::DISABLE_APRILTAG);
    #[constant]
    pub const DISABLE_COLOR: &Self = &Self::new(AiVisionFlags::DISABLE_COLOR);
    #[constant]
    pub const DISABLE_MODEL: &Self = &Self::new(AiVisionFlags::DISABLE_MODEL);
    #[constant]
    pub const COLOR_MERGE: &Self = &Self::new(AiVisionFlags::COLOR_MERGE);
    #[constant]
    pub const DISABLE_STATUS_OVERLAY: &Self = &Self::new(AiVisionFlags::DISABLE_STATUS_OVERLAY);
    #[constant]
    pub const DISABLE_USB_OVERLAY: &Self = &Self::new(AiVisionFlags::DISABLE_USB_OVERLAY);

    pub const fn new(flags: AiVisionFlags) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            flags,
        }
    }

    pub fn flags(&self) -> AiVisionFlags {
        self.flags
    }

    #[binary_op]
    fn binary_op(op: BinaryOpCode, lhs: &Self, rhs: Obj) -> Obj {
        match op {
            BinaryOpCode::Or | BinaryOpCode::InplaceOr => {
                let rhs = match rhs.try_as_obj::<Self>() {
                    Some(r) => r,
                    _ => return Obj::NULL,
                };
                alloc_obj(Self::new(lhs.flags.union(rhs.flags)))
            }
            _ => Obj::NULL,
        }
    }
}
