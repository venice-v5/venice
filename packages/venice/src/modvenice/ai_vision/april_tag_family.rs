use micropython_rs::{
    class, class_methods,
    obj::{ObjBase, ObjTrait},
};
use vexide_devices::smart::ai_vision::AprilTagFamily;

#[class(qstr!(AprilTagFamily))]
#[repr(C)]
pub struct AprilTagFamilyObj {
    base: ObjBase,
    family: AprilTagFamily,
}

#[class_methods]
impl AprilTagFamilyObj {
    const fn new(family: AprilTagFamily) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            family,
        }
    }

    #[constant]
    pub const CIRCLE21H7: &Self = &Self::new(AprilTagFamily::Circle21h7);
    #[constant]
    pub const TAG16H5: &Self = &Self::new(AprilTagFamily::Tag16h5);
    #[constant]
    pub const TAG25H9: &Self = &Self::new(AprilTagFamily::Tag25h9);
    #[constant]
    pub const TAG36H11: &Self = &Self::new(AprilTagFamily::Tag36h11);

    pub fn family(&self) -> AprilTagFamily {
        self.family
    }
}
