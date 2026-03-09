use micropython_rs::{
    class, class_methods,
    obj::{ObjBase, ObjTrait},
};
use vexide_devices::smart::motor::Gearset;

use crate::qstrgen::qstr;

#[class(qstr!(Gearset))]
#[repr(C)]
pub struct GearsetObj {
    base: ObjBase<'static>,
    gearset: Gearset,
}

#[class_methods]
impl GearsetObj {
    const fn new(gearset: Gearset) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            gearset,
        }
    }

    #[constant]
    pub const RED: &Self = &Self::new(Gearset::Red);
    #[constant]
    pub const GREEN: &Self = &Self::new(Gearset::Green);
    #[constant]
    pub const BLUE: &Self = &Self::new(Gearset::Blue);

    pub const fn gearset(&self) -> Gearset {
        self.gearset
    }
}
