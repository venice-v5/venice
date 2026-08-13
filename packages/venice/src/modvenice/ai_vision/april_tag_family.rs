use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{ObjBase, ObjTrait},
    print::{Print, PrintKind},
};
use vexide_devices::smart::ai_vision::AprilTagFamily;

/// Possible AprilTag families to be detected by the sensor.
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

    /// Circle21h7 family.
    #[constant]
    pub const CIRCLE21H7: &Self = &Self::new(AprilTagFamily::Circle21h7);
    /// 16h5 family.
    #[constant]
    pub const TAG16H5: &Self = &Self::new(AprilTagFamily::Tag16h5);
    /// 25h9 family.
    #[constant]
    pub const TAG25H9: &Self = &Self::new(AprilTagFamily::Tag25h9);
    /// 36h11 family.
    #[constant]
    pub const TAG36H11: &Self = &Self::new(AprilTagFamily::Tag36h11);

    pub fn family(&self) -> AprilTagFamily {
        self.family
    }

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        print.print(match self.family {
            AprilTagFamily::Circle21h7 => "AprilTagFamily.CIRCLE21H7",
            AprilTagFamily::Tag16h5 => "AprilTagFamily.TAG16H5",
            AprilTagFamily::Tag25h9 => "AprilTagFamily.TAG25H9",
            AprilTagFamily::Tag36h11 => "AprilTagFamily.TAG36H11",
        });
    }
}
