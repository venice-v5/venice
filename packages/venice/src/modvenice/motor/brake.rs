use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{ObjBase, ObjTrait},
    print::{Print, PrintKind},
};
use vexide_devices::smart::motor::BrakeMode;

#[class(qstr!(BrakeMode))]
#[repr(C)]
pub struct BrakeModeObj {
    base: ObjBase,
    mode: BrakeMode,
}

#[class_methods]
impl BrakeModeObj {
    const fn new(mode: BrakeMode) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            mode,
        }
    }

    #[constant]
    pub const BRAKE: &Self = &Self::new(BrakeMode::Brake);
    #[constant]
    pub const COAST: &Self = &Self::new(BrakeMode::Coast);
    #[constant]
    pub const HOLD: &Self = &Self::new(BrakeMode::Hold);

    pub const fn mode(&self) -> BrakeMode {
        self.mode
    }

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        print.print(match self.mode {
            BrakeMode::Brake => "BrakeMode.BRAKE",
            BrakeMode::Coast => "BrakeMode.COAST",
            BrakeMode::Hold => "BrakeMode.HOLD",
        });
    }
}
