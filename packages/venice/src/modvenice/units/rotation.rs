use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{ObjBase, ObjTrait},
    print::{Print, PrintKind},
};
use vexide_devices::math::Angle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationUnit {
    Radians,
    Degrees,
    Turns,
}

/// A unit selector for angular values.
///
/// Venice APIs accept a numeric angle together with one of these singleton values and return numeric
/// angles in the selected unit. Angles are signed displacements from some rotation representing zero;
/// they are unbounded and are not automatically made modular. One turn equals 360 degrees or `2 * pi`
/// radians.
///
/// This class is not constructed directly. Use `RotationUnit.RADIANS`, `RotationUnit.DEGREES`, or
/// `RotationUnit.TURNS`, which are also exported at the package root as `RADIANS`, `DEGREES`, and
/// `TURNS`. Values have readable representations such as `RotationUnit.DEGREES`.
#[class(qstr!(RotationUnit))]
#[repr(C)]
pub struct RotationUnitObj {
    base: ObjBase,
    unit: RotationUnit,
}

impl RotationUnit {
    pub fn angle_to_float(self, angle: Angle) -> f32 {
        (match self {
            Self::Radians => angle.as_radians(),
            Self::Degrees => angle.as_degrees(),
            Self::Turns => angle.as_turns(),
        }) as f32
    }

    pub fn float_to_angle(self, value: f32) -> Angle {
        let value = value as f64;
        match self {
            Self::Radians => Angle::from_radians(value),
            Self::Degrees => Angle::from_degrees(value),
            Self::Turns => Angle::from_turns(value),
        }
    }
}

#[class_methods]
impl RotationUnitObj {
    const fn new(unit: RotationUnit) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            unit,
        }
    }

    /// Selects the number of radians rotated; also root-importable as `RADIANS`.
    #[constant]
    pub const RADIANS: &Self = &Self::new(RotationUnit::Radians);
    /// Selects the number of degrees rotated; also root-importable as `DEGREES`.
    #[constant]
    pub const DEGREES: &Self = &Self::new(RotationUnit::Degrees);
    /// Selects the number of turns (revolutions) rotated; also root-importable as `TURNS`.
    #[constant]
    pub const TURNS: &Self = &Self::new(RotationUnit::Turns);

    pub const fn unit(&self) -> RotationUnit {
        self.unit
    }

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        print.print(match self.unit {
            RotationUnit::Radians => "RotationUnit.RADIANS",
            RotationUnit::Degrees => "RotationUnit.DEGREES",
            RotationUnit::Turns => "RotationUnit.TURNS",
        })
    }
}
