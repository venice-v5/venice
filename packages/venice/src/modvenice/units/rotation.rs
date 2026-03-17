use micropython_rs::{
    class, class_methods,
    obj::{ObjBase, ObjTrait},
};
use vexide_devices::math::Angle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationUnit {
    Radians,
    Degrees,
    Turns,
}

#[class(qstr!(RotationUnit))]
#[repr(C)]
pub struct RotationUnitObj {
    base: ObjBase<'static>,
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

    #[constant]
    pub const RADIANS: &Self = &Self::new(RotationUnit::Radians);
    #[constant]
    pub const DEGREES: &Self = &Self::new(RotationUnit::Degrees);
    #[constant]
    pub const TURNS: &Self = &Self::new(RotationUnit::Turns);

    pub const fn unit(&self) -> RotationUnit {
        self.unit
    }
}
