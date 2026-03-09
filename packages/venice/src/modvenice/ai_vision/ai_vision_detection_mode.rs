use argparse::{ArgType, error_msg};
use micropython_rs::{
    class, class_methods,
    except::raise_type_error,
    init::token,
    obj::{Obj, ObjBase, ObjTrait},
    ops::BinaryOp,
};
use vexide_devices::smart::ai_vision::AiVisionDetectionMode;

use crate::{obj::alloc_obj, qstrgen::qstr};

#[class(qstr!(AiVisionDetectionMode))]
#[repr(C)]
pub struct AiVisionDetectionModeObj {
    base: ObjBase<'static>,
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
    extern "C" fn binary_op(op: BinaryOp, obj_1: Obj, obj_2: Obj) -> Obj {
        if let BinaryOp::Or = op {
        } else if let BinaryOp::InplaceOr = op {
        } else {
            return Obj::NULL;
        }
        let lhs = obj_1
            .try_as_obj::<AiVisionDetectionModeObj>()
            .unwrap_or_else(|| {
                raise_type_error(
                    token(),
                    error_msg!(
                        "expected <AiVisionFlags> for argument #1, found <{}>",
                        ArgType::of(&obj_1)
                    ),
                )
            })
            .mode;
        let rhs = obj_2
            .try_as_obj::<AiVisionDetectionModeObj>()
            .unwrap_or_else(|| {
                raise_type_error(
                    token(),
                    error_msg!(
                        "expected <AiVisionFlags> for argument #2, found <{}>",
                        ArgType::of(&obj_2)
                    ),
                )
            })
            .mode;
        alloc_obj(AiVisionDetectionModeObj::new(lhs.union(rhs)))
    }
}
