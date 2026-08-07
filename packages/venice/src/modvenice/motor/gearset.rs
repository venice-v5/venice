use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{ObjBase, ObjTrait},
    print::{Print, PrintKind},
};
use vexide_devices::smart::motor::Gearset;

/// Internal gearset used by VEX Smart motors.
///
/// The selected value must match an 11W motor's physical cartridge so position and velocity are
/// scaled correctly. EXP motors have a fixed 200 RPM gearset reported as `Gearset.GREEN`.
/// `Gearset` is root-importable and isn't constructed directly; use one of its singleton constants.
/// Values have readable representations such as `Gearset.GREEN`.
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

    /// 36:1 gear ratio with a rated maximum speed of 100 RPM.
    #[constant]
    pub const RED: &Self = &Self::new(Gearset::Red);
    /// 18:1 gear ratio with a rated maximum speed of 200 RPM.
    #[constant]
    pub const GREEN: &Self = &Self::new(Gearset::Green);
    /// 6:1 gear ratio with a rated maximum speed of 600 RPM.
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
