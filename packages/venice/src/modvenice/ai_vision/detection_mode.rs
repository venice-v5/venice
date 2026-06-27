use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{Obj, ObjBase, ObjTrait},
    ops::BinaryOpCode,
};
use vexide_devices::smart::ai_vision::AiVisionDetectionMode;

use crate::obj::alloc_obj;

#[class(qstr!(AiVisionDetectionMode))]
#[repr(C)]
pub struct AiVisionDetectionModeObj {
    base: ObjBase,
    mode: AiVisionDetectionMode,
}

#[class_methods]
impl AiVisionDetectionModeObj {
    #[constant]
    pub const APRILTAG: &Self = &Self::new(AiVisionDetectionMode::APRILTAG);
    #[constant]
    pub const COLOR: &Self = &Self::new(AiVisionDetectionMode::COLOR);
    #[constant]
    pub const MODEL: &Self = &Self::new(AiVisionDetectionMode::MODEL);
    #[constant]
    pub const COLOR_MERGE: &Self = &Self::new(AiVisionDetectionMode::COLOR_MERGE);

    pub const fn new(mode: AiVisionDetectionMode) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            mode,
        }
    }

    pub fn mode(&self) -> AiVisionDetectionMode {
        self.mode
    }

    #[binary_op]
    fn binary_op(op: BinaryOpCode, lhs: &Self, rhs: Obj) -> Obj {
        match op {
            BinaryOpCode::Or | BinaryOpCode::InplaceOr => {
                let rhs = match rhs.try_as_obj::<Self>() {
                    Some(r) => r,
                    None => return Obj::NULL,
                };
                alloc_obj(Self::new(lhs.mode.union(rhs.mode)))
            }
            _ => Obj::NULL,
        }
    }

    // TODO
    // not sure how to print this object out
}
