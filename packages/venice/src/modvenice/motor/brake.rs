use micropython_rs::{
    class, class_methods,
    obj::{ObjBase, ObjTrait},
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
}
