use argparse::{ArgType, error_msg};
use micropython_rs::{
    class, class_methods,
    except::type_error,
    init::token,
    obj::{Obj, ObjBase, ObjTrait},
    ops::BinaryOp,
};
use vexide_devices::smart::ai_vision::AiVisionFlags;

use crate::obj::alloc_obj;

#[class(qstr!(AiVisionFlags))]
#[repr(C)]
pub struct AiVisionFlagsObj {
    base: ObjBase<'static>,
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
    extern "C" fn binary_op(op: BinaryOp, obj_1: Obj, obj_2: Obj) -> Obj {
        if let BinaryOp::Or = op {
        } else if let BinaryOp::InplaceOr = op {
        } else {
            return Obj::NULL;
        }
        let lhs = obj_1
            .try_as_obj::<AiVisionFlagsObj>()
            .unwrap_or_else(|| {
                type_error(error_msg!(
                    "expected <AiVisionFlags> for argument #1, found <{}>",
                    ArgType::of(&obj_1)
                ))
                .raise(token())
            })
            .flags;
        let rhs = obj_2
            .try_as_obj::<AiVisionFlagsObj>()
            .unwrap_or_else(|| {
                type_error(error_msg!(
                    "expected <AiVisionFlags> for argument #2, found <{}>",
                    ArgType::of(&obj_2)
                ))
                .raise(token())
            })
            .flags;
        alloc_obj(AiVisionFlagsObj::new(lhs.union(rhs)))
    }
}
