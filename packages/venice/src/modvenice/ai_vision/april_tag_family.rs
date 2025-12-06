use micropython_rs::{
    const_dict,
    obj::{Obj, ObjBase, ObjFullType, ObjTrait, TypeFlags},
};
use vexide_devices::smart::ai_vision::AprilTagFamily;

use crate::qstrgen::qstr;

static APRIL_TAG_FAMILY_OBJ_TYPE: ObjFullType = ObjFullType::new(
    TypeFlags::empty(),
    qstr!(AprilTagFamily),
)
.set_slot_locals_dict_from_static(const_dict![
    qstr!(CIRCLE21H7) => Obj::from_static(&AprilTagFamilyObj::new(AprilTagFamily::Circle21h7)),
    qstr!(TAG16H5) => Obj::from_static(&AprilTagFamilyObj::new(AprilTagFamily::Tag16h5)),
    qstr!(TAG25H9) => Obj::from_static(&AprilTagFamilyObj::new(AprilTagFamily::Tag25h9)),
    qstr!(TAG36H11) => Obj::from_static(&AprilTagFamilyObj::new(AprilTagFamily::Tag36h11)),
]);

#[repr(C)]
pub struct AprilTagFamilyObj {
    base: ObjBase<'static>,
    family: AprilTagFamily,
}

unsafe impl ObjTrait for AprilTagFamilyObj {
    const OBJ_TYPE: &micropython_rs::obj::ObjType = APRIL_TAG_FAMILY_OBJ_TYPE.as_obj_type();
}

impl AprilTagFamilyObj {
    pub const fn new(family: AprilTagFamily) -> Self {
        Self {
            base: ObjBase::new(APRIL_TAG_FAMILY_OBJ_TYPE.as_obj_type()),
            family,
        }
    }

    pub const fn family(&self) -> AprilTagFamily {
        self.family
    }
}
