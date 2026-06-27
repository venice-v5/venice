use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{ObjBase, ObjTrait},
    print::{Print, PrintKind},
};
use vexide_devices::smart::motor::Gearset;

#[class(qstr!(Gearset))]
#[repr(C)]
pub struct GearsetObj {
    base: ObjBase,
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

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        print.print(match self.gearset {
            Gearset::Red => "Gearset.RED",
            Gearset::Green => "Gearset.GREEN",
            Gearset::Blue => "Gearset.BLUE",
        });
    }
}
