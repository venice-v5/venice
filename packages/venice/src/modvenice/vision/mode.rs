use micropython_macros::{class, class_methods};
use micropython_rs::obj::{ObjBase, ObjTrait};
use vexide_devices::smart::vision::VisionMode;

#[class(qstr!(VisionMode))]
#[repr(C)]
pub struct VisionModeObj {
    base: ObjBase,
    mode: VisionMode,
}

#[class_methods]
impl VisionModeObj {
    const fn new(mode: VisionMode) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            mode,
        }
    }

    #[constant]
    pub const COLOR_DETECTION: &Self = &Self::new(VisionMode::ColorDetection);
    #[constant]
    pub const LINE_DETECTION: &Self = &Self::new(VisionMode::LineDetection);
    #[constant]
    pub const MIXED_DETECTION: &Self = &Self::new(VisionMode::MixedDetection);
    #[constant]
    pub const WIFI: &Self = &Self::new(VisionMode::Wifi);
    #[constant]
    pub const TEST: &Self = &Self::new(VisionMode::Test);

    pub fn mode(&self) -> VisionMode {
        self.mode
    }
}
