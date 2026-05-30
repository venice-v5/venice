use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{ObjBase, ObjTrait},
    print::{Print, PrintKind},
};
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

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        print.print(match self.mode {
            VisionMode::ColorDetection => "VisionMode.COLOR_DETECTION",
            VisionMode::LineDetection => "VisionMode.LINE_DETECTION",
            VisionMode::MixedDetection => "VisionMode.MIXED_DETECTION",
            VisionMode::Wifi => "VisionMode.WIFI",
            VisionMode::Test => "VisionMode.TEST",
        });
    }
}
